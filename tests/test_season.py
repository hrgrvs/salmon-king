from salmon_king.sim.clock import tides_in_season
from salmon_king.sim.engine import new_game, run_headless
from salmon_king.sim.models import GameEnd


def test_full_odd_year_season_completes():
    g = new_game(seed=2025, camp_id="uganik", year=2025)
    recap = run_headless(g, quiet=True)
    assert recap.end in {GameEnd.SEASON, GameEnd.BANKRUPT, GameEnd.NO_SKIFFS, GameEnd.NO_CREW}
    assert g.tick >= 20
    # A finished season should have a grade
    assert recap.grade in {"A", "B", "C", "D", "F"}


def test_alitak_season_does_not_crash():
    g = new_game(seed=8, camp_id="olga", year=2023)
    recap = run_headless(g, ticks=80, quiet=True)
    assert recap is not None
    assert g.tick == 80 or g.end is not GameEnd.NONE


def test_even_year_port_bailey_headless():
    g = new_game(seed=4, camp_id="bailey", year=2024)
    recap = run_headless(g, ticks=tides_in_season(2024), quiet=True)
    assert isinstance(recap.gross, float)


def test_four_camps_construct():
    for camp in ("larsen", "uganik", "olga", "bailey"):
        g = new_game(seed=1, camp_id=camp, year=2025)
        assert g.camp.id == camp
        assert len(g.nets) == 2
        assert all(g.camp.id in site.travel_from_camp for site in g.playable_sites())
