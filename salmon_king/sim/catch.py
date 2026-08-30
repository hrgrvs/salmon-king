"""Catch and quality. Setnets fish without crew; picking recovers what's in the web."""

from __future__ import annotations

from salmon_king.data.species import SpeciesId
from salmon_king.data.sites import SITES
from salmon_king.sim.models import CrewMember, Net, Weather
from salmon_king.sim.rng import Rng

# Game selectivity. Mesh is a player knob — Kodiak regs do not specify gillnet mesh.
MESH_SEL: dict[str, dict[SpeciesId, float]] = {
    "pink": {
        SpeciesId.KING: 0.25,
        SpeciesId.RED: 0.55,
        SpeciesId.PINK: 1.15,
        SpeciesId.CHUM: 0.50,
        SpeciesId.SILVER: 0.45,
    },
    "mixed": {
        SpeciesId.KING: 0.55,
        SpeciesId.RED: 1.00,
        SpeciesId.PINK: 0.85,
        SpeciesId.CHUM: 0.90,
        SpeciesId.SILVER: 0.85,
    },
    "red": {
        SpeciesId.KING: 0.70,
        SpeciesId.RED: 1.15,
        SpeciesId.PINK: 0.40,
        SpeciesId.CHUM: 1.05,
        SpeciesId.SILVER: 1.00,
    },
}

NET_CAP_LBS = 520.0  # game number, one 75–87 fm set


def soak_penalty(soak_tides: int) -> float:
    """Diminishing returns: dropouts, seals, fish rolling out."""
    if soak_tides <= 1:
        return 1.0
    if soak_tides == 2:
        return 0.88
    if soak_tides == 3:
        return 0.68
    return 0.48


def quality_decay(net: Net, weather: Weather, daytime: bool) -> float:
    q = net.quality
    if net.soak_tides >= 2:
        q -= 0.08
    if net.soak_tides >= 4:
        q -= 0.12
    if daytime and weather.fog < 0.25 and net.soak_tides >= 3:
        q -= 0.10  # sunburn
    if net.fish.total() > NET_CAP_LBS * 0.75:
        q -= 0.06
    return max(0.35, q)


def weather_catch_mod(weather: Weather) -> float:
    mod = 1.0
    if weather.seas_ft >= 6:
        mod *= 0.45
    elif weather.seas_ft >= 4:
        mod *= 0.72
    if weather.fog >= 0.6:
        mod *= 0.90
    if weather.williwaw:
        mod *= 0.40
    return mod


def crew_efficiency(members: list[CrewMember]) -> float:
    if not members:
        return 0.35  # you can kind of pick alone, badly
    parts = []
    for c in members:
        if c.status.value not in {"working"}:
            continue
        skill = 0.45 + 0.07 * c.skill
        energy = 0.55 + 0.45 * (c.energy / 100.0)
        hunger = 1.0 if c.hunger < 55 else max(0.45, 1.0 - (c.hunger - 55) / 80.0)
        morale = 0.60 + 0.40 * (c.morale / 100.0)
        parts.append(skill * energy * hunger * morale)
    if not parts:
        return 0.30
    return min(1.35, sum(parts) / len(parts) * (1.0 + 0.08 * (len(parts) - 1)))


def soak_net(
    net: Net,
    available: dict[SpeciesId, float],
    weather: Weather,
    mammal: float,
    orca_scatter: float,
    rng: Rng,
    fathom_scale: float,
    accumulate_pressure: bool = True,
) -> None:
    if not net.in_water or net.condition <= 8:
        return
    room = max(0.0, NET_CAP_LBS - net.fish.total())
    if room <= 1:
        net.quality = max(0.35, net.quality - 0.05)
        return
    sel = MESH_SEL[net.mesh]
    soak = soak_penalty(net.soak_tides)
    cond = max(0.15, net.condition / 100.0)
    wx = weather_catch_mod(weather)
    gained = 0.0
    for sp, base in available.items():
        if base <= 0:
            continue
        raw = (
            base
            * sel[sp]
            * soak
            * cond
            * wx
            * mammal
            * orca_scatter
            * fathom_scale
            * rng.uniform(0.80, 1.20)
        )
        take = min(raw, room - gained)
        if take > 0.05:
            net.fish.add(sp, take)
            gained += take
    net.soak_tides += 1
    net.quality = quality_decay(net, weather, daytime=True)
    # Full bag invites lions — not while transients emptied the haulout
    if accumulate_pressure and net.fish.total() > NET_CAP_LBS * 0.55:
        net.sea_lion_pressure = min(1.0, net.sea_lion_pressure + 0.12)


def pick_net(net: Net, crew: list[CrewMember], rng: Rng) -> tuple[dict[SpeciesId, float], float]:
    """Return recovered lbs by species and quality. Leftovers stay in the web."""
    eff = crew_efficiency(crew)
    recovered: dict[SpeciesId, float] = {}
    q = net.quality * (0.85 + 0.15 * eff)
    for sp, amt in net.fish.as_dict().items():
        if amt <= 0:
            recovered[sp] = 0.0
            continue
        frac = min(0.98, 0.72 + 0.22 * eff)
        take = amt * frac
        drop = amt - take
        recovered[sp] = take
        net.fish.add(sp, -take)
        if drop < 0.8:
            net.fish.add(sp, -drop)
    net.soak_tides = 0
    net.sea_lion_pressure *= 0.4
    net.quality = min(1.0, net.quality + 0.12)
    if rng.random() < 0.08:
        net.condition = max(5.0, net.condition - rng.uniform(1, 4))
    return recovered, max(0.40, min(1.12, q))
