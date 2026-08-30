from __future__ import annotations

from textual.widgets import Static

from salmon_king.data.maps import render_map
from salmon_king.data.sites import SITES
from salmon_king.sim.mammals import empty_haulout_sites, resident_sites, transient_sites
from salmon_king.data.species import SHORT, SpeciesId
from salmon_king.sim.clock import phase_label
from salmon_king.sim.engine import Game
from salmon_king.sim.models import CrewStatus


def bar(value: float, width: int = 8) -> str:
    v = max(0.0, min(100.0, value))
    fill = int(round(v / 100 * width))
    return "█" * fill + "░" * (width - fill)


def render_map_panel(game: Game, cursor: str | None) -> str:
    deployed = {n.id: n.site_id for n in game.nets if n.in_water and n.site_id}
    skiffs = {}
    for s in game.skiffs:
        if s.wrecked:
            continue
        skiffs[s.id] = s.location
    closed = set()
    for site in game.playable_sites():
        open_, _ = game.site_is_open(site.id)
        if not open_:
            closed.add(site.id)
    text = render_map(
        game.camp.map_id,
        game.camp.id,
        deployed,
        skiffs,
        game.tender.present,
        closed,
        cursor,
        transient_sites=transient_sites(game.wildlife),
        resident_sites=resident_sites(game.wildlife),
    )
    extra = []
    empty = empty_haulout_sites(game.wildlife)
    for n in game.nets:
        if n.in_water and n.site_id:
            site = SITES[n.site_id]
            lions = "  lions gone" if n.site_id in empty else ""
            extra.append(
                f"  {n.id} ╫ {site.short}  {n.fish.total():.0f} lb  soak {n.soak_tides}  "
                f"cond {n.condition:.0f}%  {n.mesh}{lions}"
            )
        else:
            extra.append(f"  {n.id} on the beach  cond {n.condition:.0f}%  {n.mesh}")
    for s in game.skiffs:
        flag = "wrecked" if s.wrecked else f"{s.job.value} @ {s.location}  {s.cargo.total():.0f} lb  fuel {s.fuel:.0f}"
        extra.append(f"  › {s.name}: {flag}")
    extra.extend(f"  {line}" for line in game.mammal_status())
    legend = "▲ camp   ╫ net   › skiff   ■ tender   · open   x dark   ω transients   r residents"
    return text + "\n" + legend + "\n" + "\n".join(extra)


def render_crew(game: Game) -> str:
    lines = ["CREW"]
    for c in game.crew:
        if c.status is CrewStatus.QUIT:
            lines.append(f"  {c.name}  QUIT")
            continue
        job = c.assigned or c.role
        own = " *" if c.is_owner else ""
        lines.append(
            f"  {c.name}{own}  {c.role}/{job}  {c.status.value}  {c.trait}\n"
            f"    E {bar(c.energy)}  H {bar(100 - c.hunger)}  M {bar(c.morale)}"
        )
    return "\n".join(lines)


def render_tender(game: Game) -> str:
    p = game.tender.prices
    eta = "IN THE HOLE" if game.tender.present else f"ETA {game.tender.eta_tides} tides"
    lines = [
        f"TENDER  {game.tender.name}  {eta}",
        "  "
        + "  ".join(f"{SHORT[sp]} ${p[sp]:.2f}" for sp in SpeciesId),
        f"  last: {game.tender.last_note}",
    ]
    return "\n".join(lines)


def render_camp(game: Game) -> str:
    people = len([c for c in game.crew if c.status is not CrewStatus.QUIT])
    days = game.food / max(1, people)
    jv = "  JV 3rd net" if game.joint_venture else ""
    return (
        f"CAMP  {game.camp.name}{jv}\n"
        f"  cash ${game.ledger.cash:,.0f}   food {days:.1f} d   fuel {game.fuel_cache:.0f} gal   "
        f"ice {game.ice_cache:.0f}   twine {game.twine}   prop {game.spare_prop}\n"
        f"  cookshack {game.cookshack}  bunk {game.bunkhouse}  loft {game.net_loft}  stalls {game.skiff_stalls}\n"
        f"  gross ${game.ledger.gross:,.0f}  exp ${game.ledger.expenses():.0f}  tickets {game.ledger.tickets}"
    )


def render_clock(game: Game) -> str:
    open_ = current_open(game)
    tag = "OPEN" if open_ else "DARK"
    skip = "  PERMIT OFF-SITE" if game.skipper_in_town else ""
    mammal = game.mammal_status()
    extra = ("\n  " + "\n  ".join(mammal[:2])) if mammal else ""
    return (
        f"CLOCK  {game.day.strftime('%a %d %b %Y')}  {game.tide.value} tide  {tag}{skip}\n"
        f"  {phase_label(game.day, game.odd_year, game.district)}\n"
        f"  {game.open_reason}\n"
        f"  {game.weather.label}{extra}"
    )


def current_open(game: Game) -> bool:
    for site in game.playable_sites():
        ok, _ = game.site_is_open(site.id)
        if ok:
            return True
    return False


def render_log(game: Game, n: int = 6) -> str:
    lines = game.log[-n:]
    if not lines:
        return "EVENT LOG"
    return "EVENT LOG\n" + "\n".join(f"  {ln.day:%m/%d} {ln.text}" for ln in lines)


class LiveMap(Static):
    pass
