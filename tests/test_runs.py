"""Run curves produce fish in the right months; odd-year pinks dwarf even-year."""

from datetime import date

from salmon_king.data.species import SpeciesId
from salmon_king.sim.openings import generate_run_mods
from salmon_king.sim.rng import Rng
from salmon_king.sim.runs import site_availability


def _sum_reds(site: str, year: int, mods, month: int) -> float:
    total = 0.0
    for day in range(1, 28):
        avail = site_availability(site, date(year, month, day), mods)
        total += avail[SpeciesId.RED]
    return total


def _sum_pinks(site: str, year: int, mods, month: int) -> float:
    total = 0.0
    for day in range(1, 28):
        avail = site_availability(site, date(year, month, day), mods)
        total += avail[SpeciesId.PINK]
    return total


def test_westside_reds_stronger_in_june_july_than_september():
    mods = generate_run_mods(2025, Rng(1))
    mods.karluk_early = 1.0
    mods.karluk_late = 1.0
    early = _sum_reds("cape_uyak", 2025, mods, 6) + _sum_reds("cape_uyak", 2025, mods, 7)
    late = _sum_reds("cape_uyak", 2025, mods, 9)
    assert early > late * 1.3


def test_pinks_peak_july_august_not_june():
    mods = generate_run_mods(2025, Rng(2))
    mods.pink = 1.8
    july = _sum_pinks("uganik_pass", 2025, mods, 7)
    august = _sum_pinks("uganik_pass", 2025, mods, 8)
    june = _sum_pinks("uganik_pass", 2025, mods, 6)
    assert july + august > june * 4


def test_odd_year_pinks_exceed_even_year():
    odd = generate_run_mods(2025, Rng(3))
    even = generate_run_mods(2024, Rng(3))
    # same seed, different year line
    odd.pink = 1.8
    even.pink = 0.5
    odd_lbs = _sum_pinks("uganik_pass", 2025, odd, 7)
    even_lbs = _sum_pinks("uganik_pass", 2024, even, 7)
    assert odd_lbs > even_lbs * 2


def test_kings_are_incidental():
    mods = generate_run_mods(2025, Rng(4))
    mods.king = 0.3
    mods.karluk_early = 1.0
    kings = 0.0
    reds = 0.0
    for day in range(1, 28):
        a = site_availability("cape_uyak", date(2025, 6, day), mods)
        kings += a[SpeciesId.KING]
        reds += a[SpeciesId.RED]
    assert kings < reds * 0.25


def test_silvers_show_in_september():
    mods = generate_run_mods(2025, Rng(5))
    mods.silver = 1.0
    sept = 0.0
    june = 0.0
    for d in range(1, 18):
        sept += site_availability("olga_narrows", date(2025, 9, d), mods)[SpeciesId.SILVER]
        june += site_availability("olga_narrows", date(2025, 6, d), mods)[SpeciesId.SILVER]
    assert sept > june * 2
