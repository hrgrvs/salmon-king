"""ADF&G-style emergency-order calendar. Generated from weir strength + even/odd.

Does not hard-code a 2026 opening board. Windows follow 5 AAC 18.362 / 18.361.
"""

from __future__ import annotations

from datetime import date, timedelta

from salmon_king.sim.clock import doy
from salmon_king.sim.models import OpeningWindow, RunMods
from salmon_king.sim.rng import Rng


def _add(windows: list[OpeningWindow], start: date, hours: int, district: str, reason: str, pulse: bool = False) -> None:
    days = max(1, round(hours / 24))
    # 33 hr ≈ 1 day + 9 hr → treat as start day + next morning (2 calendar days, ~3 tides)
    if hours <= 36:
        end = start
        # keep a 33-hr feel: open that calendar day (both tides) and next flood
        end = start + timedelta(days=1)
    elif hours <= 60:
        end = start + timedelta(days=2)
    else:
        end = start + timedelta(days=days - 1)
    windows.append(
        OpeningWindow(
            start_doy=doy(start),
            end_doy=doy(end),
            hours=hours,
            district=district,
            reason=reason,
            pulse_sites_only=pulse,
        )
    )


def generate_run_mods(year: int, rng: Rng) -> RunMods:
    odd = year % 2 == 1
    karluk_early = rng.uniform(0.12, 1.20)
    mods = RunMods(
        karluk_early=karluk_early,
        karluk_late=rng.uniform(0.55, 1.25),
        frazer=rng.uniform(0.40, 1.20),
        upper_station=rng.uniform(0.50, 1.20),
        pink=(rng.uniform(1.35, 2.10) if odd else rng.uniform(0.35, 0.75)),
        chum=rng.uniform(0.70, 1.15),
        silver=rng.uniform(0.45, 1.10),
        king=rng.uniform(0.12, 0.45),
        pink_flood=odd,
        karluk_early_fail=karluk_early < 0.40,
        frazer_goals_ok=False,
    )
    mods.frazer_goals_ok = mods.frazer >= 0.85 and mods.upper_station >= 0.80
    return mods


def generate_openings(year: int, district: str, mods: RunMods, rng: Rng) -> list[OpeningWindow]:
    if district == "alitak":
        return _alitak(year, mods, rng)
    return _central(year, mods, rng)


def _central(year: int, mods: RunMods, rng: Rng) -> list[OpeningWindow]:
    w: list[OpeningWindow] = []
    # Jun 1–15: at least two 33-hour test openers (5 AAC 18.362)
    first = date(year, 6, rng.randint(1, 4))
    second = date(year, 6, rng.randint(9, 13))
    _add(w, first, 33, "central", "33-hr test · mixed-stock early sockeye")
    _add(w, second, 33, "central", "33-hr test · mixed-stock early sockeye")

    # Jun 16–Jul 5: Karluk early. Weak early = those tests may be all you get.
    if not mods.karluk_early_fail:
        d = date(year, 6, 16)
        while d <= date(year, 7, 5):
            if rng.random() < 0.50 + 0.25 * min(1.0, mods.karluk_early):
                hrs = 33 if rng.random() < 0.55 else 57
                _add(w, d, hrs, "central", f"{hrs}-hr period · Karluk early sockeye")
                d += timedelta(days=3 if hrs == 33 else 4)
            else:
                d += timedelta(days=rng.randint(2, 4))
    # Jul 6–Aug 15: pinks. Odd flood / even thin.
    d = date(year, 7, 6)
    p = 0.92 if mods.pink_flood else 0.50
    while d <= date(year, 8, 15):
        if rng.random() < p:
            hrs = 57 if mods.pink_flood or rng.random() < 0.40 else 33
            _add(w, d, hrs, "central", f"{hrs}-hr period · pink salmon")
            d += timedelta(days=3 if mods.pink_flood else 4)
        else:
            d += timedelta(days=rng.randint(2, 3))
    # Aug 16–24: pinks + Karluk late
    d = date(year, 8, 16)
    while d <= date(year, 8, 24):
        if rng.random() < 0.60:
            _add(w, d, 33, "central", "33-hr period · pinks + Karluk late sockeye")
            d += timedelta(days=3)
        else:
            d += timedelta(days=2)
    # Aug 25–Sep 5: Karluk late
    d = date(year, 8, 25)
    while d <= date(year, 9, 5):
        if rng.random() < 0.45 + 0.2 * mods.karluk_late:
            _add(w, d, 33, "central", "33-hr period · Karluk late sockeye")
            d += timedelta(days=3)
        else:
            d += timedelta(days=2)
    # After ~Sep 5: late sockeye + coho, skeleton
    d = date(year, 9, 6)
    while d <= date(year, 9, 17):
        if rng.random() < 0.35:
            _add(w, d, 33, "central", "33-hr period · late sockeye + coho")
            d += timedelta(days=4)
        else:
            d += timedelta(days=3)
    return w


def _alitak(year: int, mods: RunMods, rng: Rng) -> list[OpeningWindow]:
    w: list[OpeningWindow] = []
    # Early June: a couple of sockeye looks
    _add(w, date(year, 6, rng.randint(5, 10)), 33, "alitak", "33-hr period · Frazer / early Upper Station")

    # 10-day blocks 13 Jun–15 Sep: 5–7 days open, 63 consecutive hours closed
    # unless Frazer + Upper Station goals will be met (5 AAC 18.361)
    block = date(year, 6, 13)
    last = date(year, 9, 15)
    while block <= last:
        open_days = rng.randint(5, 7)
        reason = "Alitak pulse · Frazer / Upper Station / pinks"
        if mods.pink_flood and block.month >= 7:
            reason = "Alitak pulse · odd-year pinks + sockeye"
        elif not mods.pink_flood and block.month >= 7:
            reason = "Alitak pulse · even year, sockeye-weighted"
        _add(w, block, open_days * 24, "alitak", reason)
        # Pulse inner sites if weirs look good
        if mods.frazer >= 1.0 and block.month >= 7:
            _add(
                w,
                block + timedelta(days=2),
                33,
                "alitak",
                "pulse · Dog Salmon Flats / Upper Station / Akalura",
                pulse=True,
            )
        if mods.frazer_goals_ok and block >= date(year, 7, 20):
            # skip the mandatory 63-hr dark this block
            block += timedelta(days=10)
        else:
            # 63 hr closed is the gap before next 10-day clock
            block += timedelta(days=10)
    return w


def is_open(
    windows: list[OpeningWindow],
    day: date,
    district: str,
    site_id: str,
    pulse_sites: set[str],
) -> tuple[bool, str]:
    n = doy(day)
    reason = "closed — no emergency order"
    traditional_open = False
    pulse_open = False
    pulse_reason = ""
    for w in windows:
        if w.district != district:
            continue
        if w.start_doy <= n <= w.end_doy:
            if w.pulse_sites_only:
                pulse_open = True
                pulse_reason = w.reason
            else:
                traditional_open = True
                reason = w.reason
    if site_id in pulse_sites:
        if pulse_open:
            return True, pulse_reason
        if traditional_open:
            # traditional water is open; pulse sites stay dark unless named
            return False, "pulse site dark — traditional water is fishing"
        return False, reason
    if traditional_open:
        return True, reason
    return False, reason


def current_window(windows: list[OpeningWindow], day: date, district: str) -> OpeningWindow | None:
    n = doy(day)
    for w in windows:
        if w.district == district and not w.pulse_sites_only and w.start_doy <= n <= w.end_doy:
            return w
    return None
