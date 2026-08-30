"""Site-level killer whale / pinniped chain. Transients empty the haulout; residents scatter fish."""

from __future__ import annotations

from dataclasses import dataclass

from salmon_king.data.sites import BAY_LABEL, BAYS, SITES, bay_id_for_site, sites_for_camp, sites_in_bay

# Game-number tide counts (two tides / day).
TRANSIENT_STAY = (4, 7)
PINNIPED_RETURN_DELAY = (3, 5)  # after orcas leave, not instant
RESIDENT_STAY = (3, 5)
RESIDENT_SCATTER = 0.48  # catch multiplier while residents are in that bay
PRESSURE_FLOOR = 0.04


@dataclass
class BayWildlife:
    bay_id: str
    label: str
    transient_tides: int = 0
    resident_tides: int = 0
    pinniped_absent_tides: int = 0

    def transients_here(self) -> bool:
        return self.transient_tides > 0

    def residents_here(self) -> bool:
        return self.resident_tides > 0

    def pinnipeds_present(self) -> bool:
        return self.pinniped_absent_tides <= 0


def new_wildlife() -> dict[str, BayWildlife]:
    return {
        bid: BayWildlife(bay_id=bid, label=BAY_LABEL[bid])
        for bid in BAYS
    }


def bay_of(site_id: str, wildlife: dict[str, BayWildlife]) -> BayWildlife | None:
    bid = bay_id_for_site(site_id)
    if not bid:
        return None
    return wildlife.get(bid)


def pinnipeds_present(wildlife: dict[str, BayWildlife], site_id: str) -> bool:
    bay = bay_of(site_id, wildlife)
    if bay is None:
        return True
    return bay.pinnipeds_present()


def transients_present(wildlife: dict[str, BayWildlife], site_id: str) -> bool:
    bay = bay_of(site_id, wildlife)
    return bool(bay and bay.transients_here())


def resident_scatter(wildlife: dict[str, BayWildlife], site_id: str) -> float:
    bay = bay_of(site_id, wildlife)
    if bay and bay.residents_here():
        return RESIDENT_SCATTER
    return 1.0


def transient_sites(wildlife: dict[str, BayWildlife]) -> set[str]:
    out: set[str] = set()
    for bay in wildlife.values():
        if bay.transients_here():
            out.update(sites_in_bay(bay.bay_id))
    return out


def resident_sites(wildlife: dict[str, BayWildlife]) -> set[str]:
    out: set[str] = set()
    for bay in wildlife.values():
        if bay.residents_here():
            out.update(sites_in_bay(bay.bay_id))
    return out


def empty_haulout_sites(wildlife: dict[str, BayWildlife]) -> set[str]:
    out: set[str] = set()
    for bay in wildlife.values():
        if not bay.pinnipeds_present():
            out.update(sites_in_bay(bay.bay_id))
    return out


def status_lines(wildlife: dict[str, BayWildlife], camp_id: str) -> list[str]:
    """Player-facing mixed-blessing lines for the map / clock."""
    playable = {s.id for s in sites_for_camp(camp_id)}
    lines = []
    for bay in wildlife.values():
        if not playable.intersection(sites_in_bay(bay.bay_id)):
            continue
        if bay.transients_here():
            lines.append(
                f"ω transients in {bay.label} — lions gone ({bay.transient_tides} tides). "
                f"They do not pick the web."
            )
        elif bay.pinniped_absent_tides > 0:
            lines.append(
                f"haulout empty at {bay.label} — transients moved on, lions not back "
                f"({bay.pinniped_absent_tides} tides)"
            )
        if bay.residents_here():
            lines.append(
                f"residents in {bay.label} — fish acting weird, catch off "
                f"({bay.resident_tides} tides). Lions still on the rocks."
            )
    return lines


def tick_wildlife(wildlife: dict[str, BayWildlife]) -> list[str]:
    logs: list[str] = []
    for bay in wildlife.values():
        if bay.transient_tides > 0:
            bay.transient_tides -= 1
            if bay.transient_tides == 0:
                left = (
                    f"Transients left {bay.label}. Haulout still empty — "
                    f"Stellers and seals not back for {bay.pinniped_absent_tides} tides."
                )
                logs.append(left)
        if bay.resident_tides > 0:
            bay.resident_tides -= 1
            if bay.resident_tides == 0:
                logs.append(f"Residents left {bay.label}. Fish settling.")
        if bay.pinniped_absent_tides > 0:
            bay.pinniped_absent_tides -= 1
            if bay.pinniped_absent_tides == 0:
                logs.append(f"Stellers back on the rocks at {bay.label}. Seals too.")
    return logs


def arrive_transients(game, site_id: str | None = None) -> str:
    """Transient killer whales: hunt pinnipeds. Empty haulout. Do not take salmon from gear."""
    wildlife: dict[str, BayWildlife] = game.wildlife
    site_id = site_id or _pick_site(game, prefer_nets=True)
    bay = bay_of(site_id, wildlife)
    if bay is None:
        return "Transient orcas offshore. Not on your beaches."
    stay = game.rng.randint(*TRANSIENT_STAY)
    delay = game.rng.randint(*PINNIPED_RETURN_DELAY)
    bay.transient_tides = stay
    bay.pinniped_absent_tides = stay + delay
    cleared = 0.0
    for net in game.nets:
        if net.site_id in sites_in_bay(bay.bay_id):
            cleared += net.sea_lion_pressure
            net.sea_lion_pressure = PRESSURE_FLOOR
    names = " / ".join(SITES[s].short for s in sites_in_bay(bay.bay_id) if s in SITES)
    return (
        f"Transients in {bay.label} ({names}). Lions gone. Seals gone. "
        f"They do not pick salmon out of the gear — empty haulout is the gift. "
        f"{stay} tides here, then a few more before the rocks fill again."
    )


def arrive_residents(game, site_id: str | None = None) -> str:
    """Resident killer whales: fish-eaters. Scatter salmon. Do not clear lions. Do not raid nets."""
    wildlife: dict[str, BayWildlife] = game.wildlife
    site_id = site_id or _pick_site(game, prefer_nets=True)
    bay = bay_of(site_id, wildlife)
    if bay is None:
        return "Residents passing offshore. Fish still on your beach."
    bay.resident_tides = game.rng.randint(*RESIDENT_STAY)
    names = " / ".join(SITES[s].short for s in sites_in_bay(bay.bay_id) if s in SITES)
    return (
        f"Residents in {bay.label} ({names}). Fish acting weird — diving, sliding wide. "
        f"Catch is off. They are not raiding the nets, and the Stellers are still here."
    )


def _pick_site(game, prefer_nets: bool) -> str:
    deployed = [n.site_id for n in game.nets if n.in_water and n.site_id]
    if prefer_nets and deployed:
        return game.rng.choice(deployed)
    playable = [s.id for s in game.playable_sites()]
    return game.rng.choice(playable) if playable else "uganik_pass"
