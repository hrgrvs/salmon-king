"""Deterministic season engine. Pure Python. TUI is a client."""

from __future__ import annotations

from dataclasses import dataclass, field

from salmon_king.data.camps import CAMPS, CampDef
from salmon_king.data.crew_pool import CANDIDATES, OWNER_NAME, STARTING_HIRE
from salmon_king.data.sites import PULSE_ALITAK, PLAYABLE, SITES, sites_for_camp, travel_tides
from salmon_king.data.species import SPECIES, SpeciesId, prices_for_year
from salmon_king.sim.catch import NET_CAP_LBS, pick_net, soak_net
from salmon_king.sim.clock import (
    Tide,
    advance,
    phase_label,
    season_end,
    season_start,
    tides_in_season,
)
from salmon_king.sim.events import maybe_fire
from salmon_king.sim.mammals import (
    new_wildlife,
    pinnipeds_present,
    resident_scatter,
    status_lines,
    tick_wildlife,
)
from salmon_king.sim.models import (
    CrewMember,
    CrewStatus,
    GameEnd,
    Lbs,
    Ledger,
    LogLine,
    Net,
    OpeningWindow,
    RunMods,
    Skiff,
    SkiffJob,
    Tender,
    Weather,
)
from salmon_king.sim.openings import current_window, generate_openings, generate_run_mods, is_open
from salmon_king.sim.rng import Rng
from salmon_king.sim.runs import site_availability
from salmon_king.sim.weather import roll_weather, skiffs_grounded

FOOD_PER_PERSON_DAY = 1
FUEL_PER_HOP = 5.5
HOLDING_CAP = 1400.0
PICKER_CAP = 520.0
HIRE_COST = 0  # share / wage, no signing bonus
# Supply prices are game numbers (tender store).
BUY = {
    "food": (18, 160),  # person-days, $
    "fuel": (40, 170),
    "ice": (20, 55),
    "twine": (1, 90),  # +12 net condition spread
    "prop": (1, 220),
}


@dataclass
class Recap:
    survived: bool
    end: GameEnd
    year: int
    camp: str
    lbs: Lbs
    gross: float
    expenses: float
    net: float
    settle: float
    crew_stayed: list[str]
    notable: list[str]
    grade: str
    nickname: str
    cash: float
    tickets: int

    def as_text(self) -> str:
        lines = [
            f"SEASON {self.year}  {self.camp}",
            f"End: {self.end or 'season'}",
            f"Kings {self.lbs.king:.0f} lb   Reds {self.lbs.red:.0f} lb   Pinks {self.lbs.pink:.0f} lb",
            f"Chums {self.lbs.chum:.0f} lb   Silvers {self.lbs.silver:.0f} lb",
            f"Gross ${self.gross:,.0f}   Expenses ${self.expenses:,.0f}   Net ${self.net:,.0f}",
            f"Postseason settle {self.settle:+.0f}   Cash ${self.cash:,.0f}   Tickets {self.tickets}",
            f"Stayed: {', '.join(self.crew_stayed) or 'nobody'}",
            f"Grade {self.grade} — {self.nickname}",
        ]
        if self.notable:
            lines.append("Notable:")
            lines.extend(f"  · {n}" for n in self.notable[:8])
        return "\n".join(lines)


@dataclass
class Game:
    rng: Rng
    year: int
    camp: CampDef
    day: object
    tide: Tide
    tick: int = 0
    weather: Weather = field(default_factory=Weather)
    nets: list[Net] = field(default_factory=list)
    skiffs: list[Skiff] = field(default_factory=list)
    crew: list[CrewMember] = field(default_factory=list)
    hire_pool: list[str] = field(default_factory=list)
    tender: Tender = field(default_factory=Tender)
    ledger: Ledger = field(default_factory=Ledger)
    log: list[LogLine] = field(default_factory=list)
    openings: list[OpeningWindow] = field(default_factory=list)
    mods: RunMods = field(default_factory=RunMods)
    food: float = 36.0
    fuel_cache: float = 90.0
    twine: int = 3
    spare_prop: int = 1
    ice_cache: float = 12.0
    bunkhouse: int = 1
    cookshack: int = 1
    net_loft: int = 1
    skiff_stalls: int = 2
    joint_venture: bool = False
    skipper_in_town: bool = False
    skipper_town_eta: int = 0
    wildlife: dict = field(default_factory=new_wildlife)
    bonus_school: int = 0
    extend_open_tides: int = 0
    landed: Lbs = field(default_factory=Lbs)
    end: GameEnd = GameEnd.NONE
    notable: list[str] = field(default_factory=list)
    midseason_cut: bool = False
    last_open: bool = False
    payday_counter: int = 0

    @property
    def odd_year(self) -> bool:
        return self.year % 2 == 1

    @property
    def district(self) -> str:
        return self.camp.district

    @property
    def open_reason(self) -> str:
        win = current_window(self.openings, self.day, self.district)
        if self.extend_open_tides > 0:
            return "extended period (ADF&G radio, last 24 hr)"
        if win:
            return win.reason
        return "closed — no emergency order"

    def site_is_open(self, site_id: str) -> tuple[bool, str]:
        if self.extend_open_tides > 0:
            site = SITES[site_id]
            if site.district == self.district:
                if site_id in PULSE_ALITAK:
                    return is_open(self.openings, self.day, self.district, site_id, PULSE_ALITAK)
                return True, self.open_reason
        return is_open(self.openings, self.day, self.district, site_id, PULSE_ALITAK)

    def playable_sites(self):
        return sites_for_camp(self.camp.id)

    def mammal_status(self) -> list[str]:
        return status_lines(self.wildlife, self.camp.id)

    def note(self, text: str, kind: str = "info") -> None:
        self.log.append(LogLine(self.tick, self.day, text, kind))
        if len(self.log) > 400:
            self.log = self.log[-300:]

    def scare_crew(self, amt: float) -> None:
        for c in self.crew:
            if c.status is not CrewStatus.QUIT:
                c.morale = max(0, c.morale - amt)

    def owner(self) -> CrewMember:
        return next(c for c in self.crew if c.is_owner)

    def working_deck(self) -> list[CrewMember]:
        return [
            c
            for c in self.crew
            if c.status is CrewStatus.WORKING and c.role != "cook" and not (c.is_owner and self.skipper_in_town)
        ]

    def cook(self) -> CrewMember | None:
        for c in self.crew:
            if c.role == "cook" and c.status is not CrewStatus.QUIT:
                return c
        return None

    # ----- tick ----------------------------------------------------------
    def step(self) -> list[LogLine]:
        start = len(self.log)
        if self.end is not GameEnd.NONE:
            return []
        self.tick += 1
        self.day, self.tide = advance(self.day, self.tide)
        if self.day > season_end(self.year):
            self._finish(GameEnd.SEASON)
            return self.log[start:]

        self.weather = roll_weather(self.day, self.camp, self.rng)
        for line in tick_wildlife(self.wildlife):
            self.note(line, "mammal")
        if self.bonus_school > 0:
            self.bonus_school -= 1
        if self.extend_open_tides > 0:
            self.extend_open_tides -= 1

        self._town_clock()
        self._tender_clock()
        self._maybe_midseason_cut()
        ev = maybe_fire(self)
        if ev:
            self.note(ev, "event")
            self.notable.append(ev)

        self._soak_all()
        self._skiffs_act()
        self._crew_metabolism()
        self._payroll()
        self._repair_idle()
        self._check_fail()
        return self.log[start:]

    def _town_clock(self) -> None:
        if self.skipper_in_town:
            self.skipper_town_eta -= 1
            if self.skipper_town_eta <= 0:
                self.skipper_in_town = False
                self.owner().status = CrewStatus.WORKING
                self.note("Permit holder back on the site. Nets can fish again.", "adfg")

    def _tender_clock(self) -> None:
        if self.tender.present:
            self.tender.stay -= 1
            if self.tender.stay <= 0:
                self.tender.present = False
                gap = self.rng.randint(6, 12) if self.district == "alitak" else self.rng.randint(5, 10)
                if self.tender.late:
                    gap += 2
                self.tender.eta_tides = gap
                self.tender.late = False
                self.note(f"{self.tender.name} pulled the hook. Next look in {gap} tides.", "market")
        else:
            self.tender.eta_tides -= 1
            if self.tender.eta_tides <= 0:
                self.tender.present = True
                self.tender.stay = self.rng.randint(2, 4)
                self.note(f"{self.tender.name} in the hole. Weigh and buy. Ice if you need it.", "market")

    def _maybe_midseason_cut(self) -> None:
        if self.midseason_cut or not self.mods.pink_flood:
            return
        if self.day.month == 8 and self.day.day >= 1:
            floor = prices_for_year(self.year)[SpeciesId.PINK] * 0.50
            self.tender.prices[SpeciesId.PINK] = max(floor, self.tender.prices[SpeciesId.PINK] * 0.82)
            self.midseason_cut = True
            self.note(
                f"Midseason pink cut — flood year. ${self.tender.prices[SpeciesId.PINK]:.2f}/lb. "
                "Opening estimate is paper.",
                "market",
            )

    def _soak_all(self) -> None:
        legal_skipper = not self.skipper_in_town
        if not legal_skipper and any(n.in_water for n in self.nets):
            self.note("Permit holder is off-site. Sets cannot legally fish.", "adfg")
        any_open = False
        for net in self.nets:
            if not net.in_water or not net.site_id:
                continue
            open_, reason = self.site_is_open(net.site_id)
            if not open_:
                if self.last_open:
                    self.note(f"Closer. {reason}. Gear out of the water.", "adfg")
                # illegal to fish; soak stops. Fine if they leave it in more than a tide after close.
                if net.soak_tides > 0 and not open_:
                    net.soak_tides += 1
                    if net.soak_tides > 2:
                        fine = 250
                        self.ledger.cash -= fine
                        self.ledger.fines += fine
                        net.in_water = False
                        site = net.site_id
                        net.site_id = None
                        self.note(
                            f"Gear still fishing a closer at {site}. ADF&G would have the card. "
                            f"${fine} (game fine) and the net is on the beach.",
                            "adfg",
                        )
                continue
            if not legal_skipper:
                continue
            any_open = True
            avail = site_availability(net.site_id, self.day, self.mods)
            if self.bonus_school:
                for sp in avail:
                    avail[sp] *= 1.55
            site = SITES[net.site_id]
            lions = pinnipeds_present(self.wildlife, net.site_id)
            mammal = 1.0
            if lions:
                mammal -= 0.15 * net.sea_lion_pressure * site.sea_lion
                if net.soak_tides >= 3:
                    mammal -= 0.08 * site.harbor_seal
            else:
                net.sea_lion_pressure = min(net.sea_lion_pressure, 0.04)
            mammal = max(0.25, mammal)
            scale = net.fathoms / 75.0
            scatter = resident_scatter(self.wildlife, net.site_id)
            soak_net(
                net,
                avail,
                self.weather,
                mammal,
                scatter,
                self.rng,
                scale,
                accumulate_pressure=lions,
            )
        self.last_open = any_open or (current_window(self.openings, self.day, self.district) is not None)

    def _skiffs_act(self) -> None:
        grounded = skiffs_grounded(self.weather)
        for skiff in self.skiffs:
            if skiff.wrecked:
                continue
            if skiff.eta > 0:
                skiff.eta -= 1
                if skiff.eta == 0 and skiff.dest:
                    skiff.location = skiff.dest
                    skiff.dest = None
                    self.note(f"{skiff.name} on {skiff.location}.", "skiff")
                continue
            if grounded and skiff.job is not SkiffJob.IDLE:
                self.note(f"{skiff.name} stays on the running line — weather.", "weather")
                continue
            if skiff.job is SkiffJob.REPAIR:
                skiff.condition = min(92, skiff.condition + 8 + 2 * self.net_loft)
                if skiff.condition >= 70:
                    skiff.job = SkiffJob.IDLE
                    self.note(f"{skiff.name} will start. Don't ask how.", "skiff")
                continue
            if skiff.job is SkiffJob.TOWN:
                self._run_town(skiff)
                continue
            if skiff.job is SkiffJob.TENDER:
                self._run_tender(skiff)
                continue
            if skiff.job is SkiffJob.PICK:
                self._run_pick(skiff)

    def _crew_on(self, skiff: Skiff) -> list[CrewMember]:
        out = []
        for cid in skiff.crew_ids:
            c = self._crew_by_id(cid)
            if c and c.status is CrewStatus.WORKING:
                out.append(c)
        # owner counts if not in town and assigned here or unassigned on pick
        own = self.owner()
        if not self.skipper_in_town and own.status is CrewStatus.WORKING:
            if own.assigned == skiff.id or (own.assigned is None and skiff.job is SkiffJob.PICK):
                if own not in out:
                    out.append(own)
        return out

    def _crew_by_id(self, cid: str) -> CrewMember | None:
        for c in self.crew:
            if c.id == cid:
                return c
        return None

    def _burn_fuel(self, skiff: Skiff, hops: int = 1) -> bool:
        need = FUEL_PER_HOP * hops
        if skiff.fuel >= need:
            skiff.fuel -= need
            return True
        take = min(self.fuel_cache, need - skiff.fuel)
        self.fuel_cache -= take
        skiff.fuel += take
        if skiff.fuel >= need:
            skiff.fuel -= need
            return True
        return False

    def _goto(self, skiff: Skiff, dest: str) -> None:
        if skiff.location == dest:
            return
        hops = 1
        if dest in PLAYABLE and self.camp.id in PLAYABLE[dest].travel_from_camp:
            hops = max(1, PLAYABLE[dest].travel_from_camp[self.camp.id])
        if dest == "tender":
            hops = 1
        if dest == "town":
            hops = self.camp.town_tides
        if not self._burn_fuel(skiff, hops):
            self.note(f"{skiff.name} is dry. No fuel, no go.", "skiff")
            skiff.job = SkiffJob.IDLE
            return
        if hops <= 1:
            skiff.location = dest
        else:
            skiff.location = "transit"
            skiff.dest = dest
            skiff.eta = hops - 1

    def _run_pick(self, skiff: Skiff) -> None:
        site = skiff.job_site
        if not site:
            # pick any deployed net that's open
            deployed = [n.site_id for n in self.nets if n.in_water and n.site_id]
            site = deployed[0] if deployed else None
            skiff.job_site = site
        if not site:
            return
        if skiff.location != site:
            self._goto(skiff, site)
            if skiff.location != site:
                return
        nets = [n for n in self.nets if n.in_water and n.site_id == site]
        if not nets:
            return
        crew = self._crew_on(skiff)
        for c in crew:
            c.energy = max(0, c.energy - 9)
            c.hunger = min(100, c.hunger + 4)
        cap = HOLDING_CAP if skiff.kind == "holding" else PICKER_CAP
        for net in nets:
            rec, q = pick_net(net, crew, self.rng)
            room = cap - skiff.cargo.total()
            for sp, amt in rec.items():
                take = min(amt, max(0.0, room))
                skiff.cargo.add(sp, take)
                room -= take
            # ice helps quality
            ice_q = 1.08 if skiff.ice > 0 else 0.82
            skiff.ice = max(0, skiff.ice - 1.2)
            w = (skiff.cargo.total() or 1)
            skiff.cargo_quality = (skiff.cargo_quality * (w - sum(rec.values()) + 1) + q * ice_q * sum(rec.values())) / (w + 1)
        picked = sum(n.fish.total() == 0 or True for n in nets)
        lbs = skiff.cargo.total()
        self.note(f"{skiff.name} picked {site} ({len(nets)} set). Hold {lbs:.0f} lb.", "skiff")
        if lbs >= cap * 0.85:
            self.note(f"{skiff.name} is in the marks. Run the tender or you stop picking.", "skiff")

    def _run_tender(self, skiff: Skiff) -> None:
        if not self.tender.present:
            self.note(f"{skiff.name} waiting on a tender that isn't here.", "market")
            return
        if skiff.location != "tender" and skiff.location != self.camp.tender_anchorage:
            self._goto(skiff, self.camp.tender_anchorage)
            if skiff.location not in {self.camp.tender_anchorage, "tender"}:
                return
            skiff.location = "tender"
        if skiff.cargo.total() <= 0.5:
            skiff.job = SkiffJob.IDLE
            skiff.location = "camp"
            return
        self.settle(skiff)

    def _run_town(self, skiff: Skiff) -> None:
        if skiff.location != "town":
            self._goto(skiff, "town")
            if self.owner().assigned == skiff.id:
                self.skipper_in_town = True
                self.skipper_town_eta = max(self.skipper_town_eta, self.camp.town_tides * 2)
                self.owner().status = CrewStatus.TOWN
                self.note("Permit holder ran town. Sets go dark until you're back on the site.", "adfg")
            return
        # buy a prop automatically if we came for parts
        if self.spare_prop <= 0:
            self.spare_prop = 1
            self.ledger.cash -= 220
            self.ledger.repairs += 220
        skiff.condition = min(95, skiff.condition + 20)
        skiff.job = SkiffJob.IDLE
        self._goto(skiff, "camp")

    def settle(self, skiff: Skiff) -> float:
        if skiff.cargo.total() <= 0:
            return 0.0
        q = max(0.50, min(1.12, skiff.cargo_quality))
        claim = ""
        if q < 0.70:
            claim = "quality claim — soft / warm"
        elif q < 0.85:
            claim = "docked a bit — not enough ice"
        elif q >= 1.05:
            claim = "chilled bonus"
        pay = 0.0
        bits = []
        for sp, amt in skiff.cargo.as_dict().items():
            if amt <= 0:
                continue
            px = self.tender.prices[sp] * q
            dol = amt * px
            pay += dol
            self.landed.add(sp, amt)
            bits.append(f"{amt:.0f} {sp.value}")
        self.ledger.cash += pay
        self.ledger.gross += pay
        self.ledger.tickets += 1
        self.tender.last_ticket = pay
        self.tender.last_lbs = skiff.cargo.total()
        self.tender.last_note = f"${pay:,.0f}  {', '.join(bits)}  q={q:.2f} {claim}".strip()
        # accrue shares on this ticket
        for c in self.crew:
            if c.is_owner or c.status is CrewStatus.QUIT or c.share <= 0:
                continue
            c.accrued_share += pay * c.share
        skiff.cargo = Lbs()
        skiff.cargo_quality = 1.0
        skiff.location = "camp"
        skiff.job = SkiffJob.IDLE
        self.note(f"Fish ticket: {self.tender.last_note}", "market")
        return pay

    def _crew_metabolism(self) -> None:
        # food once per day (on ebb → next flood already consumed? eat on flood)
        eat = self.tide is Tide.FLOOD
        n_people = len([c for c in self.crew if c.status is not CrewStatus.QUIT])
        fed = True
        if eat:
            need = n_people * FOOD_PER_PERSON_DAY
            if self.food >= need:
                self.food -= need
            else:
                self.food = 0
                fed = False
                self.note("Cookshack's empty. People get mean on coffee.", "camp")
        cook = self.cook()
        cook_mod = 1.0 + 0.12 * (self.cookshack - 1)
        if cook:
            cook_mod += 0.08 * (cook.skill / 10)
        else:
            cook_mod *= 0.55
        rest_mod = 1.0 + 0.10 * (self.bunkhouse - 1)
        for c in self.crew:
            if c.status is CrewStatus.QUIT:
                continue
            if c.status is CrewStatus.SICK:
                c.energy = min(70, c.energy + 6 * rest_mod)
                if c.energy > 45 and c.hunger < 70:
                    c.status = CrewStatus.RESTING
                continue
            if eat:
                if fed:
                    c.hunger = max(0, c.hunger - 28 * cook_mod)
                    c.morale = min(100, c.morale + 1.2)
                else:
                    c.hunger = min(100, c.hunger + 22)
            if c.status is CrewStatus.RESTING:
                c.energy = min(100, c.energy + 16 * rest_mod)
                if c.energy > 70:
                    c.status = CrewStatus.WORKING
            elif c.assigned is None and c.role != "cook" and not c.is_owner:
                # idle
                c.energy = min(100, c.energy + 6 * rest_mod)
                if not self.last_open:
                    c.morale = max(0, c.morale - 0.4)  # bored in a closer
                else:
                    c.morale = max(0, c.morale - 1.1)  # idle while it's open
            if c.hunger > 70:
                c.morale = max(0, c.morale - 6)
            if c.hunger > 90:
                c.morale = max(0, c.morale - 8)
                c.energy = max(0, c.energy - 5)
            if c.energy < 15:
                c.status = CrewStatus.RESTING
                self.note(f"{c.name} is cooked. Bunkhouse.", "crew")
            if c.morale < 8 and not c.is_owner and self.rng.random() < 0.10:
                c.status = CrewStatus.QUIT
                c.assigned = None
                self.note(f"{c.name} quit. Caught the next ride.", "crew")
                self.notable.append(f"{c.name} quit")
            if c.role == "cook" and c.status is CrewStatus.WORKING:
                c.energy = max(20, c.energy - 4)

    def _payroll(self) -> None:
        self.payday_counter += 1
        if self.payday_counter < 14:
            return
        self.payday_counter = 0
        bill = 0.0
        for c in self.crew:
            if c.status is CrewStatus.QUIT:
                continue
            if c.daily_wage:
                bill += c.daily_wage * 7
            if c.accrued_share > 0:
                bill += c.accrued_share
                c.accrued_share = 0
        if bill <= 0:
            return
        if self.ledger.cash >= bill:
            self.ledger.cash -= bill
            self.ledger.wages += bill
            self.note(f"Payday. Crew ${bill:,.0f}. Cash ${self.ledger.cash:,.0f}.", "ledger")
            for c in self.crew:
                if c.status is not CrewStatus.QUIT:
                    c.morale = min(100, c.morale + 6)
        else:
            self.ledger.cash -= bill
            self.ledger.wages += bill
            self.note(f"Payday short. Still owe the crew ${bill:,.0f}. Books in the hole.", "ledger")
            self.scare_crew(15)

    def _repair_idle(self) -> None:
        if self.tide is not Tide.EBB:
            return
        loft = 2.0 * self.net_loft
        for net in self.nets:
            if not net.in_water and net.condition < 100 and self.twine > 0:
                net.condition = min(100, net.condition + 6 + loft)
        for s in self.skiffs:
            if s.location == "camp" and s.job is SkiffJob.IDLE and not s.wrecked:
                s.condition = min(96, s.condition + 1)

    def _check_fail(self) -> None:
        if self.ledger.cash < -400 and self.food <= 0:
            self._finish(GameEnd.BANKRUPT)
            return
        if self.ledger.cash < -2500:
            self._finish(GameEnd.BANKRUPT)
            return
        live = [s for s in self.skiffs if not s.wrecked]
        if not live:
            self._finish(GameEnd.NO_SKIFFS)
            return
        deck = [c for c in self.crew if c.status in {CrewStatus.WORKING, CrewStatus.RESTING, CrewStatus.SICK} and c.role != "cook"]
        if not deck:
            self._finish(GameEnd.NO_CREW)

    def _finish(self, end: GameEnd) -> None:
        self.end = end
        if end is GameEnd.BANKRUPT:
            self.note("Broke. Can't buy fuel, food, or a payday. That's the season.", "end")
        elif end is GameEnd.NO_SKIFFS:
            self.note("No skiff will start. You watch the corks from the beach.", "end")
        elif end is GameEnd.NO_CREW:
            self.note("Nobody left who will pick. Permit card doesn't lift fish.", "end")
        else:
            self.note("September. Camps pulling. You weigh the books.", "end")

    # ----- actions -------------------------------------------------------
    def deploy_net(self, net_id: str, site_id: str) -> str:
        if site_id not in PLAYABLE or self.camp.id not in PLAYABLE[site_id].travel_from_camp:
            return "That's not your water. Seiners, or someone else's lease."
        site = PLAYABLE[site_id]
        if not site.legal_setnet:
            return "Seine water. Your S04K doesn't set here."
        net = self._net(net_id)
        if not net:
            return "No such net."
        # 900 ft minimum — game: one net per site
        for other in self.nets:
            if other is not net and other.in_water and other.site_id == site_id:
                return "900 feet between sets (game: one net to a site). Waived in some inner Alitak — not here."
        open_, reason = self.site_is_open(site_id)
        if not open_:
            return f"Dark. {reason}. Nets stay on the beach."
        if self.skipper_in_town:
            return "Permit holder has to be on the site."
        net.site_id = site_id
        net.in_water = True
        net.soak_tides = 0
        lease = "Your DNR lease." if site.lease == "lease" else "Unleased — scramble with the neighbors."
        self.note(f"{net.id} in at {site.short}. {lease}", "gear")
        return f"{net.id} fishing {site.name}."

    def pull_net(self, net_id: str) -> str:
        net = self._net(net_id)
        if not net or not net.in_water:
            return "That net is already on the beach."
        where = net.site_id
        net.in_water = False
        net.site_id = None
        leftover = net.fish.total()
        self.note(f"Pulled {net.id} off {where}. {leftover:.0f} lb still in the web." if leftover else f"Pulled {net.id}.", "gear")
        return f"{net.id} on the beach."

    def set_mesh(self, net_id: str, mesh: str) -> str:
        if mesh not in {"pink", "mixed", "red"}:
            return "Mesh knob: pink / mixed / red. Not a legal stretched-mesh — Kodiak doesn't spec the gillnet."
        net = self._net(net_id)
        if not net:
            return "No such net."
        if net.in_water:
            return "Pull it first."
        net.mesh = mesh
        return f"{net.id} hung {mesh} (game selectivity)."

    def assign_skiff(self, skiff_id: str, job: str, site_id: str | None = None) -> str:
        skiff = self._skiff(skiff_id)
        if not skiff or skiff.wrecked:
            return "Skiff's done."
        try:
            skiff.job = SkiffJob(job)
        except ValueError:
            return "Jobs: pick, tender, idle, town, repair."
        skiff.job_site = site_id
        if job == "pick" and site_id and site_id not in PLAYABLE:
            return "Not a setnet site."
        self.note(f"{skiff.name} → {job}" + (f" @ {site_id}" if site_id else ""), "skiff")
        return f"{skiff.name} assigned {job}."

    def assign_crew(self, crew_id: str, where: str) -> str:
        c = self._crew_by_id(crew_id)
        if not c or c.status is CrewStatus.QUIT:
            return "They're gone."
        if where == "cook":
            c.assigned = "cook"
            c.role = "cook"
            return f"{c.name} in the cookshack."
        if where == "camp":
            c.assigned = None
            c.status = CrewStatus.RESTING
            return f"{c.name} in the bunkhouse."
        skiff = self._skiff(where)
        if not skiff:
            return "No such skiff."
        # unassign from others
        for s in self.skiffs:
            if c.id in s.crew_ids:
                s.crew_ids.remove(c.id)
        skiff.crew_ids.append(c.id)
        c.assigned = skiff.id
        c.status = CrewStatus.WORKING
        return f"{c.name} on {skiff.name}."

    def hire(self, cand_id: str) -> str:
        if cand_id not in self.hire_pool:
            return "They're not on the beach."
        if not self.tender.present and self.tick > 2:
            return "Hands come out on the tender."
        cand = next(c for c in CANDIDATES if c.id == cand_id)
        self.crew.append(
            CrewMember(
                id=cand.id,
                name=cand.name,
                role=cand.role,
                skill=cand.skill,
                share=cand.wage_share,
                daily_wage=cand.daily_wage,
                trait=cand.trait,
                energy=cand.energy,
                hunger=cand.hunger,
                morale=cand.morale,
                assigned="cook" if cand.role == "cook" else None,
            )
        )
        self.hire_pool.remove(cand_id)
        self.note(f"Hired {cand.name} ({cand.trait}).", "crew")
        return f"Hired {cand.name}."

    def buy(self, item: str) -> str:
        if not self.tender.present:
            return "Tender's not here. That's the store."
        if item not in BUY:
            return "Buy: food, fuel, ice, twine, prop."
        qty, cost = BUY[item]
        if self.ledger.cash < cost:
            return "Not enough cash on the ticket book."
        self.ledger.cash -= cost
        if item == "food":
            self.food += qty
            self.ledger.food += cost
        elif item == "fuel":
            self.fuel_cache += qty
            self.ledger.fuel += cost
        elif item == "ice":
            self.ice_cache += qty
            # load onto holding skiff if present
            for s in self.skiffs:
                if s.kind == "holding":
                    s.ice += qty * 0.6
            self.ledger.other += cost
        elif item == "twine":
            self.twine += 2
            for n in self.nets:
                n.condition = min(100, n.condition + 12)
            self.ledger.gear += cost
        elif item == "prop":
            self.spare_prop += 1
            self.ledger.repairs += cost
        self.note(f"Bought {item} for ${cost}.", "ledger")
        return f"Bought {item} (${cost})."

    def upgrade(self, what: str) -> str:
        if not self.tender.present:
            return "Freight comes on the tender."
        prices = {"cookshack": 1800, "bunkhouse": 1600, "loft": 1400, "stall": 2200}
        if what not in prices:
            return "Upgrade: cookshack, bunkhouse, loft, stall."
        cost = prices[what]
        if self.ledger.cash < cost:
            return "Can't cover the freight."
        self.ledger.cash -= cost
        self.ledger.other += cost
        if what == "cookshack":
            self.cookshack += 1
        elif what == "bunkhouse":
            self.bunkhouse += 1
        elif what == "loft":
            self.net_loft += 1
        elif what == "stall":
            self.skiff_stalls += 1
        self.note(f"Upgraded {what} (${cost}).", "camp")
        return f"{what} is better. ${cost}."

    def form_joint_venture(self) -> str:
        if self.joint_venture:
            return "Already doubled up."
        if not self.tender.present:
            return "The other card comes out on a tender."
        cost = 3500  # game: you carry some of the second operation
        if self.ledger.cash < cost:
            return "Need cash on hand to make a joint venture work."
        self.ledger.cash -= cost
        self.ledger.other += cost
        self.joint_venture = True
        fm = 100 if self.district == "central" else 75
        # 175*2=350 central / 150*2=300 alitak — third net
        self.nets.append(Net(id="net-3", site_id=None, fathoms=fm, mesh="mixed", in_water=False))
        self.note(
            "Joint venture with a second S04K. Three nets, 300 fm (350 Central). They're on the beach.",
            "gear",
        )
        return "Joint venture is in. Third net in the loft."

    def _net(self, net_id: str) -> Net | None:
        for n in self.nets:
            if n.id == net_id:
                return n
        return None

    def _skiff(self, skiff_id: str) -> Skiff | None:
        for s in self.skiffs:
            if s.id == skiff_id:
                return s
        return None

    def recap(self) -> Recap:
        # COAR-style postseason settle: ticket prelim ≠ final. Game ±3–8%.
        drift = self.rng.uniform(-0.04, 0.08)
        settle = self.ledger.gross * drift
        cash = self.ledger.cash + settle
        stayed = [c.name for c in self.crew if c.status is not CrewStatus.QUIT]
        net = self.ledger.net() + settle
        grade, nick = _grade(net, self.end, len(stayed), self.landed.total())
        return Recap(
            survived=self.end in {GameEnd.NONE, GameEnd.SEASON},
            end=self.end if self.end is not GameEnd.NONE else GameEnd.SEASON,
            year=self.year,
            camp=self.camp.name,
            lbs=self.landed,
            gross=self.ledger.gross,
            expenses=self.ledger.expenses(),
            net=net,
            settle=settle,
            crew_stayed=stayed,
            notable=self.notable[-10:],
            grade=grade,
            nickname=nick,
            cash=cash,
            tickets=self.ledger.tickets,
        )


def _grade(net: float, end: GameEnd, stayed: int, lbs: float) -> tuple[str, str]:
    if end is GameEnd.BANKRUPT:
        return "F", "Sold the permit talk"
    if end is GameEnd.NO_SKIFFS:
        return "F", "Walked home"
    if end is GameEnd.NO_CREW:
        return "D", "Cooked it alone"
    if net >= 18000 and stayed >= 3:
        return "A", "Salmon King"
    if net >= 8000:
        return "B", "Made the nut"
    if net >= 0:
        return "C", "Winter in the black"
    if net >= -5000:
        return "D", "Winter in the red"
    return "F", "Books in the stove"


def new_game(seed: int, camp_id: str, year: int) -> Game:
    if camp_id not in CAMPS:
        raise ValueError(f"Unknown camp {camp_id}")
    camp = CAMPS[camp_id]
    rng = Rng(seed)
    mods = generate_run_mods(year, rng)
    openings = generate_openings(year, camp.district, mods, rng)
    prices = prices_for_year(year)
    fm = 87 if camp.district == "central" else 75  # 175 / 150 aggregate, two nets
    home = camp.home_sites
    g = Game(
        rng=rng,
        year=year,
        camp=camp,
        day=season_start(year),
        tide=Tide.FLOOD,
        mods=mods,
        openings=openings,
        ledger=Ledger(cash=float(camp.starting_cash)),
        tender=Tender(
            present=True,
            stay=3,
            eta_tides=0,
            prices=prices,
            name=rng.choice(["M/V Kodiak Star", "M/V Pacific Pearl", "M/V Uyak Provider"]),
        ),
        food=55,
        fuel_cache=110,
    )
    g.nets = [
        Net(id="net-1", site_id=home[0], fathoms=fm, mesh="mixed", in_water=False),
        Net(id="net-2", site_id=home[1] if len(home) > 1 else home[0], fathoms=fm, mesh="mixed", in_water=False),
    ]
    g.skiffs = [
        Skiff(id="skiff-1", name="Picking skiff", kind="picker", location="camp", fuel=30, ice=0),
        Skiff(id="skiff-2", name="Holding skiff", kind="holding", location="camp", fuel=32, ice=14),
    ]
    owner = CrewMember(
        id="owner",
        name=OWNER_NAME,
        role="operator",
        skill=7,
        share=0.0,
        daily_wage=0,
        trait="S04K permit holder",
        energy=80,
        hunger=15,
        morale=75,
        is_owner=True,
        assigned="skiff-1",
    )
    g.crew = [owner]
    g.skiffs[0].crew_ids.append("owner")
    for cid in STARTING_HIRE[camp_id]:
        cand = next(c for c in CANDIDATES if c.id == cid)
        cm = CrewMember(
            id=cand.id,
            name=cand.name,
            role=cand.role,
            skill=cand.skill,
            share=cand.wage_share,
            daily_wage=cand.daily_wage,
            trait=cand.trait,
            energy=cand.energy,
            hunger=cand.hunger,
            morale=cand.morale,
            assigned="cook" if cand.role == "cook" else "skiff-1",
        )
        g.crew.append(cm)
        if cand.role != "cook":
            g.skiffs[0].crew_ids.append(cm.id)
    g.hire_pool = [c.id for c in CANDIDATES if c.id not in STARTING_HIRE[camp_id]]
    g.skiffs[0].job = SkiffJob.PICK
    g.skiffs[0].job_site = home[0]
    # Nets start on the beach; player (or headless) sets them when it's open
    g.note(
        f"{camp.name}. S04K. Two nets, {camp.max_fathoms} fm aggregate. "
        f"{'Odd-year pink flood.' if g.odd_year else 'Even-year pinks are thin.'}",
        "info",
    )
    if mods.karluk_early_fail and camp.district == "central":
        g.note(
            "Karluk early looks weak. After the June tests the Central Section may stay dark until July 6.",
            "adfg",
        )
    g.note(f"Opening estimate: red ${prices[SpeciesId.RED]:.2f}  pink ${prices[SpeciesId.PINK]:.2f}/lb.", "market")
    return g


def run_headless(game: Game, ticks: int | None = None, quiet: bool = False) -> Recap:
    """Simple skipper AI so a season can finish without a TUI."""
    limit = ticks if ticks is not None else tides_in_season(game.year) + 4
    for _ in range(limit):
        if game.end is not GameEnd.NONE:
            break
        _ai(game)
        game.step()
    if game.end is GameEnd.NONE:
        game._finish(GameEnd.SEASON)
    return game.recap()


def _ai(game: Game) -> None:
    # Set nets on home water when open
    for net in game.nets:
        if net.in_water and net.site_id:
            open_, _ = game.site_is_open(net.site_id)
            if not open_:
                game.pull_net(net.id)
        if not net.in_water:
            candidates = list(game.camp.home_sites) + [s.id for s in game.playable_sites()]
            for site in candidates:
                open_, _ = game.site_is_open(site)
                taken = any(n.in_water and n.site_id == site for n in game.nets)
                if open_ and not taken:
                    game.deploy_net(net.id, site)
                    break
    # Skiff 1 picks, skiff 2 runs tender when fat or tender is in and we have fish
    s1, s2 = game.skiffs[0], game.skiffs[1]
    if not s1.wrecked:
        if game.tender.present and s1.cargo.total() > 40:
            game.assign_skiff(s1.id, "tender")
        else:
            site = next((n.site_id for n in game.nets if n.in_water), game.camp.home_sites[0])
            game.assign_skiff(s1.id, "pick", site)
    if not s2.wrecked:
        if game.tender.present and s2.cargo.total() > 40:
            game.assign_skiff(s2.id, "tender")
        elif game.tender.present and s1.cargo.total() > 40 and s1.job is not SkiffJob.TENDER:
            game.assign_skiff(s2.id, "tender")
        else:
            site = next((n.site_id for n in game.nets if n.in_water), None)
            if site:
                game.assign_skiff(s2.id, "pick", site)
    if game.tender.present:
        if game.food < 12:
            game.buy("food")
        if game.fuel_cache < 25:
            game.buy("fuel")
        if game.ice_cache < 6:
            game.buy("ice")
        if any(n.condition < 55 for n in game.nets):
            game.buy("twine")
        # hire a picker if short
        deck = [c for c in game.working_deck() if not c.is_owner]
        if not deck:
            for cid in list(game.hire_pool):
                cand = next(c for c in CANDIDATES if c.id == cid)
                if cand.role in {"picker", "operator"}:
                    game.hire(cid)
                    game.assign_crew(cid, s1.id)
                    break
