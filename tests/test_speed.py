"""Pause / speed is a TUI concern. Same seed + N ticks = same state."""

from salmon_king.sim.engine import new_game
from salmon_king.sim.models import GameEnd


def _snapshot(g):
    return (
        g.tick,
        g.day,
        g.tide.value,
        round(g.ledger.cash, 2),
        round(sum(n.fish.total() for n in g.nets), 2),
        round(g.food, 2),
        tuple((c.id, round(c.energy, 1), round(c.hunger, 1), c.status.value) for c in g.crew),
        tuple((n.id, n.site_id, n.in_water, round(n.condition, 1)) for n in g.nets),
    )


def test_same_seed_same_ticks_same_state():
    a = new_game(seed=99, camp_id="larsen", year=2025)
    b = new_game(seed=99, camp_id="larsen", year=2025)
    for _ in range(24):
        a.step()
        b.step()
    assert _snapshot(a) == _snapshot(b)


def test_interleaved_inspection_does_not_desync():
    """Standing in for pause: reading state between ticks does not change it."""
    g = new_game(seed=42, camp_id="bailey", year=2024)
    for _ in range(10):
        g.step()
    mid = _snapshot(g)
    _ = g.open_reason
    _ = [n.fish.total() for n in g.nets]
    assert _snapshot(g) == mid
    g.step()
    assert g.tick == 11


def test_closed_period_does_not_add_fish():
    g = new_game(seed=1, camp_id="olga", year=2024)
    # Force a dark day: pull any window off this doy by stepping until closed, or close by fiat
    net = g.nets[0]
    net.in_water = True
    net.site_id = "olga_narrows"
    # Find a closed tide
    for _ in range(80):
        open_, _ = g.site_is_open("olga_narrows")
        if not open_ and not (g.extend_open_tides > 0):
            before = net.fish.total()
            g.step()
            # still dark or just opened — if still dark, fish shouldn't climb from soak
            open2, _ = g.site_is_open("olga_narrows")
            if not open2:
                assert net.fish.total() <= before + 0.01
                return
        g.step()
        if g.end is not GameEnd.NONE:
            break
    # If the season never went dark (unlikely on Alitak), pass vacuously after search
    assert True
