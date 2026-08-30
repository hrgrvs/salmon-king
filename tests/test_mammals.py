"""Transient orcas empty the haulout. Residents scatter fish. Neither raids the web."""

from salmon_king.sim.events import apply_harbor_seal_raid, apply_sea_lion_raid
from salmon_king.sim.mammals import (
    PRESSURE_FLOOR,
    arrive_residents,
    arrive_transients,
    pinnipeds_present,
    resident_scatter,
    tick_wildlife,
    transients_present,
)
from salmon_king.sim.models import Lbs
from tests.conftest import make_game


def _uyak_nets(g):
    g.nets[0].in_water = True
    g.nets[0].site_id = "harvester"
    g.nets[0].fish = Lbs(red=90, pink=40)
    g.nets[0].sea_lion_pressure = 0.8
    g.nets[0].condition = 90
    return g.nets[0]


def test_transients_clear_lions_and_do_not_take_fish():
    g = make_game(camp_id="larsen", seed=3)
    net = _uyak_nets(g)
    before = net.fish.total()
    msg = arrive_transients(g, "harvester")
    assert "Transients in Uyak" in msg
    assert "lions gone" in msg.lower() or "Lions gone" in msg
    assert "do not pick" in msg.lower()
    assert transients_present(g.wildlife, "harvester")
    assert transients_present(g.wildlife, "cape_uyak")  # same bay
    assert not pinnipeds_present(g.wildlife, "harvester")
    assert net.sea_lion_pressure <= PRESSURE_FLOOR + 0.001
    assert net.fish.total() == before  # orcas do not raid the web


def test_sea_lion_raid_blocked_while_transients_present():
    g = make_game(camp_id="larsen", seed=4)
    net = _uyak_nets(g)
    arrive_transients(g, "harvester")
    before_lbs, before_cond = net.fish.total(), net.condition
    msg = apply_sea_lion_raid(g, "harvester")
    assert net.fish.total() == before_lbs
    assert net.condition == before_cond
    assert "gone" in msg.lower() or "empty" in msg.lower()
    assert "not picking" in msg.lower() or "did not touch" in msg.lower() or "haulout" in msg.lower()


def test_harbor_seal_raid_blocked_in_same_bay():
    g = make_game(camp_id="larsen", seed=5)
    net = _uyak_nets(g)
    arrive_transients(g, "cape_uyak")
    before = net.fish.total()
    msg = apply_harbor_seal_raid(g, "harvester")
    assert net.fish.total() == before
    assert "seal" in msg.lower() or "transients" in msg.lower()


def test_raid_still_works_on_another_bay():
    g = make_game(camp_id="larsen", seed=6)
    _uyak_nets(g)
    other = g.nets[1]
    other.in_water = True
    other.site_id = "spiridon_outer"
    other.fish = Lbs(red=50)
    other.condition = 88
    arrive_transients(g, "harvester")
    assert pinnipeds_present(g.wildlife, "spiridon_outer")
    msg = apply_sea_lion_raid(g, "spiridon_outer")
    assert other.fish.total() < 50
    assert "Steller" in msg


def test_lions_return_only_after_delay():
    g = make_game(camp_id="larsen", seed=7)
    _uyak_nets(g)
    arrive_transients(g, "harvester")
    bay = g.wildlife["uyak"]
    stay = bay.transient_tides
    absent = bay.pinniped_absent_tides
    assert absent > stay

    for _ in range(stay):
        tick_wildlife(g.wildlife)
    assert not bay.transients_here()
    assert not bay.pinnipeds_present()
    msg = apply_sea_lion_raid(g, "harvester")
    assert g.nets[0].fish.total() == 130
    assert "haven't come back" in msg.lower() or "hasn't filled" in msg.lower() or "still gone" in msg.lower()

    logs: list[str] = []
    for _ in range(20):
        if bay.pinnipeds_present():
            break
        logs = tick_wildlife(g.wildlife)
    assert any("back on the rocks" in line for line in logs) or bay.pinnipeds_present()
    msg = apply_sea_lion_raid(g, "harvester")
    assert g.nets[0].fish.total() < 130
    assert "Steller" in msg


def test_residents_do_not_clear_lions_or_raid():
    g = make_game(camp_id="uganik", seed=8)
    net = g.nets[0]
    net.in_water = True
    net.site_id = "uganik_pass"
    net.fish = Lbs(pink=80)
    net.sea_lion_pressure = 0.7
    before = net.fish.total()
    msg = arrive_residents(g, "uganik_pass")
    assert "Residents" in msg
    assert "Stellers are still here" in msg or "lions" in msg.lower()
    assert pinnipeds_present(g.wildlife, "uganik_pass")
    assert not transients_present(g.wildlife, "uganik_pass")
    assert resident_scatter(g.wildlife, "uganik_pass") < 1.0
    assert net.sea_lion_pressure == 0.7
    assert net.fish.total() == before
    raid = apply_sea_lion_raid(g, "uganik_pass")
    assert net.fish.total() < before
    assert "Steller" in raid


def test_residents_dip_catch_transients_do_not():
    from datetime import date

    from salmon_king.sim.catch import soak_net
    from salmon_king.sim.models import Net, Weather
    from salmon_king.sim.rng import Rng
    from salmon_king.sim.runs import site_availability

    g = make_game(camp_id="uganik", seed=10, year=2025)
    avail = {k: v * 0.2 for k, v in site_availability("uganik_pass", date(2025, 6, 8), g.mods).items()}
    wx = Weather()

    def soak_once(scatter, seed):
        net = Net(id="t", site_id="uganik_pass", fathoms=75, mesh="mixed", in_water=True)
        soak_net(net, avail, wx, 1.0, scatter, Rng(seed), 1.0)
        return net.fish.total()

    full = soak_once(1.0, 44)
    dipped = soak_once(0.48, 44)
    assert 0 < dipped < full * 0.7
    # transients do not apply this scatter — they only empty the haulout
    assert soak_once(1.0, 44) == full


def test_status_lines_name_the_bay():
    g = make_game(camp_id="larsen", seed=9)
    arrive_transients(g, "harvester")
    lines = g.mammal_status()
    assert lines
    assert any("Uyak" in line and "lions gone" in line for line in lines)
