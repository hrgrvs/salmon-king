"""One tick = one tide. Two tides per calendar day (flood, then ebb)."""

from __future__ import annotations

from datetime import date, timedelta

from salmon_king.sim.models import Tide

SEASON_START_MD = (6, 1)
SEASON_END_MD = (9, 18)  # game: most westside camps are tearing down; legal season runs to Oct 31


def season_start(year: int) -> date:
    return date(year, *SEASON_START_MD)


def season_end(year: int) -> date:
    return date(year, *SEASON_END_MD)


def doy(d: date) -> int:
    return d.timetuple().tm_yday


def tides_in_season(year: int) -> int:
    return (season_end(year) - season_start(year)).days * 2


def advance(day: date, tide: Tide) -> tuple[date, Tide]:
    if tide is Tide.FLOOD:
        return day, Tide.EBB
    return day + timedelta(days=1), Tide.FLOOD


def phase_label(d: date, odd_year: bool, district: str) -> str:
    n = doy(d)
    if n < 167:
        return "mixed-stock early sockeye · 33-hr tests"
    if n < 187:
        return "Karluk early reds passing the beaches" if district == "central" else "Frazer / early Upper Station reds"
    if n < 228:
        return "pink push (odd-year flood)" if odd_year else "pink push (even-year thin)"
    if n < 237:
        return "pinks + Karluk late reds" if district == "central" else "pinks + Upper Station"
    if n < 249:
        return "Karluk late sockeye · camps pulling" if district == "central" else "Upper Station late · camps pulling"
    return "late sockeye + coho · skeleton crew"
