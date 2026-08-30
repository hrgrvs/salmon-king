"""Run curves by species and site. Timing is real; amplitudes are game numbers."""

from __future__ import annotations

import math

from salmon_king.data.species import SPECIES, SpeciesId
from salmon_king.data.sites import SITES
from salmon_king.sim.clock import doy
from salmon_king.sim.models import RunMods

# Game-number daily available pounds at a 1.0 affinity, 1.0 run, peak day, one net-length of beach.
# Tuned so a competent two-net season lands roughly $25–45k gross in an odd year
# (Kodiak setnet permit reality: ~$20k–$64k).
PEAK_LBS = {
    SpeciesId.KING: 18.0,  # incidental
    SpeciesId.RED: 780.0,
    SpeciesId.PINK: 2400.0,
    SpeciesId.CHUM: 400.0,
    SpeciesId.SILVER: 260.0,
}

# Second sockeye pulse (Karluk late / Upper Station late)
LATE_RED_PEAK_DOY = 235
LATE_RED_WIDTH = 16.0


def _gauss_curve(n: int, peak: int, width: float) -> float:
    return math.exp(-0.5 * ((n - peak) / width) ** 2)


def site_availability(site_id: str, day, mods: RunMods) -> dict[SpeciesId, float]:
    """Pounds available to a full-condition net this tide, before gear/weather/crew."""
    site = SITES[site_id]
    n = doy(day)
    odd_scale = mods.pink
    out: dict[SpeciesId, float] = {}
    for sp, spec in SPECIES.items():
        aff = site.affinity.get(sp, 0.0)
        if aff <= 0:
            out[sp] = 0.0
            continue
        curve = _gauss_curve(n, spec.peak_doy, spec.width_days)
        run = 1.0
        if sp is SpeciesId.PINK:
            run = odd_scale
        elif sp is SpeciesId.KING:
            run = mods.king
        elif sp is SpeciesId.CHUM:
            run = mods.chum
        elif sp is SpeciesId.SILVER:
            run = mods.silver
        elif sp is SpeciesId.RED:
            early = mods.karluk_early if site.district == "central" else mods.frazer
            late = mods.karluk_late if site.district == "central" else mods.upper_station
            early_c = _gauss_curve(n, spec.peak_doy, spec.width_days)
            late_c = _gauss_curve(n, LATE_RED_PEAK_DOY, LATE_RED_WIDTH)
            # Alitak Upper Station / Akalura lean late
            if site.id in {"upper_station_outer", "akalura_outer", "dog_salmon"}:
                curve = 0.35 * early_c + 0.90 * late_c
                run = 0.4 * early + 0.8 * late
            else:
                curve = 0.75 * early_c + 0.55 * late_c
                run = 0.65 * early + 0.50 * late
        lbs = PEAK_LBS[sp] * aff * run * curve
        # Per-tide (half-day) — peak table is daily, split across two tides
        out[sp] = max(0.0, lbs * 0.5)
    return out
