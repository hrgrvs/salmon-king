"""Resolve events against live sim state. No 'you killed a sea lion' default."""

from __future__ import annotations

from salmon_king.data.events import EVENT_TABLE
from salmon_king.data.sites import SITES
from salmon_king.sim.catch import NET_CAP_LBS
from salmon_king.sim.clock import doy
from salmon_king.sim.mammals import arrive_residents, arrive_transients, pinnipeds_present, transients_present
from salmon_king.sim.models import CrewStatus, GameEnd, Lbs, SkiffJob
from salmon_king.sim.rng import Rng


def apply_sea_lion_raid(game, site_id: str | None = None) -> str:
    """Steller sea lions take fish and puncture gear. NOAA 2002: near gear, zero observed SSL deaths."""
    if site_id and not pinnipeds_present(game.wildlife, site_id):
        where = SITES[site_id].short if site_id in SITES else site_id
        if transients_present(game.wildlife, site_id):
            return (
                f"Transients in {where}. Stellers are gone. "
                f"The orcas are not picking your salmon — haulout is empty."
            )
        return f"Haulout empty at {where}. Lions haven't come back yet."
    nets = [n for n in game.nets if n.in_water and n.fish.total() > 0]
    if site_id:
        nets = [n for n in nets if n.site_id == site_id]
    nets = [n for n in nets if n.site_id and pinnipeds_present(game.wildlife, n.site_id)]
    if not nets:
        nets = [
            n
            for n in game.nets
            if n.in_water and n.site_id and pinnipeds_present(game.wildlife, n.site_id)
        ]
    if not nets:
        return _no_pinniped_raid(game, "Stellers")
    net = max(nets, key=lambda n: n.fish.total() + n.sea_lion_pressure)
    lost = 0.0
    for sp, amt in net.fish.as_dict().items():
        take = amt * game.rng.uniform(0.25, 0.55)
        net.fish.add(sp, -take)
        lost += take
    dmg = game.rng.uniform(8, 22)
    net.condition = max(4.0, net.condition - dmg)
    net.sea_lion_pressure = min(1.0, net.sea_lion_pressure + 0.3)
    site = SITES.get(net.site_id or "", None)
    where = site.short if site else "the set"
    game.scare_crew(6)
    return (
        f"Stellers on the {where} lead. They took {lost:.0f} lb and punched the web "
        f"(net {net.condition:.0f}%). You do not shoot them."
    )


def apply_harbor_seal_raid(game, site_id: str | None = None) -> str:
    if site_id and not pinnipeds_present(game.wildlife, site_id):
        where = SITES[site_id].short if site_id in SITES else site_id
        if transients_present(game.wildlife, site_id):
            return f"Transients in {where}. Harbor seals left with the lions."
        return f"Seals still off {where}. Haulout hasn't filled."
    nets = [
        n
        for n in game.nets
        if n.in_water and n.site_id and pinnipeds_present(game.wildlife, n.site_id)
    ]
    if site_id:
        nets = [n for n in nets if n.site_id == site_id]
    if not nets:
        return _no_pinniped_raid(game, "Harbor seals")
    net = game.rng.choice(nets)
    site = SITES.get(net.site_id or "")
    quiet = site and site.harbor_seal > 0.25
    frac = game.rng.uniform(0.10, 0.28 if quiet else 0.16)
    lost = 0.0
    for sp, amt in net.fish.as_dict().items():
        take = amt * frac
        net.fish.add(sp, -take)
        lost += take
    net.condition = max(8.0, net.condition - game.rng.uniform(2, 8))
    where = site.short if site else "a set"
    return f"Harbor seals working {where}. Smaller mouths, same hole. {lost:.0f} lb gone."


def maybe_fire(game) -> str | None:
    if game.end is not GameEnd.NONE:
        return None
    if game.rng.random() > 0.16:
        return None
    n = doy(game.day)
    weights = []
    for ev in EVENT_TABLE:
        w = ev.weight
        if ev.id == "september_winter" and n < 244:
            w *= 0.05
        if ev.id == "whale_pass_rip" and game.camp.id != "bailey":
            w *= 0.15
        if ev.id == "advection_fog" and not (152 <= n <= 212):
            w *= 0.4
        if ev.id == "price_cut" and not game.mods.pink_flood:
            w *= 0.45
        weights.append((ev.id, w))
    eid = game.rng.weighted(weights)
    return resolve(game, eid)


def resolve(game, eid: str) -> str:
    fn = {
        "williwaw": _williwaw,
        "advection_fog": _fog,
        "shelikof_seas": _seas,
        "whale_pass_rip": _rip,
        "september_winter": _sept,
        "sea_lion_raid": lambda g: apply_sea_lion_raid(g),
        "harbor_seal_raid": apply_harbor_seal_raid,
        "otter_foul": _otter,
        "orca_residents": arrive_residents,
        "orca_transients": arrive_transients,
        "porpoise_bycatch": _porpoise,
        "humpback_wrap": _humpback,
        "outboard_down": _outboard,
        "net_tear": _tear,
        "skiff_swamped": _swamp,
        "ice_failure": _ice,
        "tender_late": _tender_late,
        "price_cut": _price,
        "cannery_dark": _cannery,
        "adfg_extension": _extension,
        "adfg_pulse": _pulse,
        "food_spoil": _spoil,
        "crew_fight": _fight,
        "bonus_school": _school,
    }.get(eid)
    if not fn:
        return "VHF static."
    return fn(game)


def _williwaw(game) -> str:
    game.weather.williwaw = True
    game.weather.wind_kt = max(game.weather.wind_kt, game.rng.uniform(48, 70))
    game.weather.seas_ft = max(game.weather.seas_ft, 6.0)
    game.weather.label = f"WILLIWAW {int(game.weather.wind_kt)} kt"
    game.weather.precip = "williwaw"
    game.scare_crew(10)
    for s in game.skiffs:
        if s.location not in {"camp", "town"} and not s.wrecked:
            s.condition -= game.rng.uniform(4, 15)
            if s.condition < 12 and game.rng.random() < 0.12:
                s.wrecked = True
                return f"Williwaw off the mountain. {s.name} swamped and she's done."
    return "Williwaw. Katabatic, no warning. Skiffs stay on the running line if you like them."


def _fog(game) -> str:
    game.weather.fog = max(game.weather.fog, 0.75)
    game.weather.precip = "fog"
    game.weather.label = "advection fog, thick"
    return "June/July advection fog. You pick by the corks you can see."


def _seas(game) -> str:
    game.weather.seas_ft = max(game.weather.seas_ft, game.rng.uniform(5.0, 8.5))
    game.weather.wind_kt = max(game.weather.wind_kt, 28)
    game.weather.label = f"Shelikof {game.weather.seas_ft:.0f} ft, short and steep"
    return "Shelikof stacking up. Short, steep, mean. Holding skiff takes water."


def _rip(game) -> str:
    for s in game.skiffs:
        if s.location in {"town", "transit"} or s.job.value == "town":
            s.eta += 1
            s.condition -= 6
            return f"Whale Passage 6 knots against you. {s.name} loses a tide."
    return "Whale Passage ripping. Stay off it unless you're going to town."


def _sept(game) -> str:
    game.weather.wind_kt += 12
    game.weather.seas_ft += 2
    game.weather.label = "September — winter arriving"
    game.scare_crew(4)
    return "September weather. The gulf is done being polite. Camps start talking about pulling."


def _otter(game) -> str:
    nets = [n for n in game.nets if n.in_water]
    if not nets:
        return "Otters in the kelp. They are not here for your salmon."
    net = game.rng.choice(nets)
    net.condition = max(10.0, net.condition - game.rng.uniform(6, 16))
    net.soak_tides = max(0, net.soak_tides - 1)
    return (
        f"Sea otter in the lead — entanglement / kelp raft, not a raid. "
        f"They eat invertebrates. You lost soak time. Net {net.condition:.0f}%."
    )


def _no_pinniped_raid(game, who: str) -> str:
    if any(b.transients_here() for b in game.wildlife.values()):
        bays = ", ".join(b.label for b in game.wildlife.values() if b.transients_here())
        return f"{who} chased off — transients in {bays}. Empty haulout. They did not touch the fish."
    if any(not b.pinnipeds_present() for b in game.wildlife.values()):
        bays = ", ".join(b.label for b in game.wildlife.values() if not b.pinnipeds_present())
        return f"{who} still gone from {bays}. Haulout hasn't filled."
    return f"{who} on the rocks. Nothing in the water for them."


def _porpoise(game) -> str:
    game.scare_crew(14)
    for c in game.crew:
        if not c.is_owner:
            c.morale = max(5, c.morale - 12)
    return (
        "Harbor porpoise in the web. Rare and grim — this is the MMPA Category II driver, "
        "not a raid. You clear it. Nobody jokes."
    )


def _humpback(game) -> str:
    nets = [n for n in game.nets if n.in_water]
    if nets:
        net = game.rng.choice(nets)
        net.condition = max(5.0, net.condition - game.rng.uniform(15, 35))
        net.in_water = False
        net.site_id = None
        return (
            f"Humpback rolled the {net.id} set. Gear wrap. Net's on the beach if you still have one. "
            "ADF&G will want a call if this gets worse."
        )
    return "Humpbacks working bait offshore. Keep the skiff's eyes up."


def _outboard(game) -> str:
    live = [s for s in game.skiffs if not s.wrecked and s.condition > 10]
    if not live:
        return "Nothing left that would start anyway."
    s = game.rng.choice(live)
    s.condition = min(s.condition, 18)
    s.job = SkiffJob.REPAIR
    s.location = "camp"
    s.dest = None
    return f"{s.name}: lower unit. She's on the beach until you throw a spare prop and a day at it."


def _tear(game) -> str:
    nets = [n for n in game.nets if n.in_water]
    if not nets:
        return "Loft's quiet. Nothing to tear."
    net = game.rng.choice(nets)
    net.condition = max(8.0, net.condition - game.rng.uniform(18, 40))
    return f"Torn web on {net.id}. Needles and twine. Condition {net.condition:.0f}%."


def _swamp(game) -> str:
    live = [s for s in game.skiffs if not s.wrecked and s.location not in {"camp", "town"}]
    if not live:
        return "Skiffs are on the beach. The tide still wants them."
    s = game.rng.choice(live)
    lost = s.cargo.total()
    s.cargo = Lbs()
    s.condition -= game.rng.uniform(20, 40)
    s.ice = 0
    live_else = [x for x in game.skiffs if x is not s and not x.wrecked]
    if s.condition < 12 and live_else:
        s.wrecked = True
        return f"{s.name} swamped and wrecked. {lost:.0f} lb gone with her."
    if s.condition < 12:
        s.condition = 18
        s.job = SkiffJob.REPAIR
        s.location = "camp"
        return f"{s.name} swamped. Last skiff — you drag her home and start on the lower unit."
    s.location = "camp"
    game.scare_crew(8)
    return f"{s.name} swamped. You got her home. {lost:.0f} lb to the crabs."


def _ice(game) -> str:
    for s in game.skiffs:
        s.ice = 0
        s.cargo_quality = min(s.cargo_quality, 0.62)
    for n in game.nets:
        n.quality = min(n.quality, 0.70)
    return "Ice / RSW down. Fish will go soft. Tender will dock you and they will be right."


def _tender_late(game) -> str:
    game.tender.eta_tides += game.rng.randint(2, 5)
    game.tender.late = True
    game.tender.present = False
    return f"{game.tender.name} is late. Holding skiff better have ice. Miss them and it rots."


def _price(game) -> str:
    from salmon_king.data.species import SpeciesId, prices_for_year

    floor = prices_for_year(game.year)[SpeciesId.PINK] * 0.50
    if game.tender.prices[SpeciesId.PINK] <= floor + 0.01:
        return "Town already cut the pink. Nothing left to give."
    cut = game.rng.uniform(0.10, 0.18)
    game.tender.prices[SpeciesId.PINK] = max(floor, game.tender.prices[SpeciesId.PINK] * (1.0 - cut))
    if game.mods.pink_flood:
        game.tender.prices[SpeciesId.CHUM] = max(
            prices_for_year(game.year)[SpeciesId.CHUM] * 0.50,
            game.tender.prices[SpeciesId.CHUM] * 0.94,
        )
    return (
        f"Town cut the pink. Opening estimate is dead. "
        f"New pink ${game.tender.prices[SpeciesId.PINK]:.2f}/lb. Flood years do this."
    )


def _cannery(game) -> str:
    game.tender.eta_tides += game.rng.randint(3, 6)
    game.tender.present = False
    from salmon_king.data.species import SpeciesId

    for sp in list(game.tender.prices):
        game.tender.prices[sp] *= 0.94
    return "Cannery dark. Tenders farther, less often. You will feel it in the tote."


def _extension(game) -> str:
    # last-24-hour extension flavor: stretch current window
    if game.open_reason.startswith("closed"):
        return "VHF: no extension. Still dark."
    game.extend_open_tides = max(game.extend_open_tides, 2)
    return "ADF&G radio: period extended. Last twenty-four hours they changed their mind. Stay in it."


def _pulse(game) -> str:
    game.bonus_school = 3
    return "ADF&G radio: extra pulse on the traditional water. Weir counts finally moved."


def _spoil(game) -> str:
    lost = min(game.food, game.rng.randint(3, 8))
    game.food -= lost
    return f"Meat went off in the cache. Lost {lost} person-days of food. Cook is not surprised."


def _fight(game) -> str:
    hired = [c for c in game.crew if not c.is_owner and c.status is not CrewStatus.QUIT]
    if len(hired) < 2:
        return "Bunkhouse is quiet. Not enough people left to have a fight."
    if len(hired) == 2:
        a, b = hired[0], hired[1]
    else:
        a = game.rng.choice(hired)
        b = game.rng.choice([c for c in hired if c is not a] or hired)
    a.morale = max(5, a.morale - 18)
    b.morale = max(5, b.morale - 18)
    return f"{a.name} and {b.name} in the bunkhouse. Morale takes the webbing."


def _school(game) -> str:
    game.bonus_school = max(game.bonus_school, 2)
    return "Fish showing on the lead. Thick. Get a skiff on it before the lions do."
