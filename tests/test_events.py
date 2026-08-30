from salmon_king.data.species import SpeciesId
from salmon_king.sim.events import apply_sea_lion_raid
from salmon_king.sim.models import Lbs
from tests.conftest import make_game


def test_sea_lion_raid_reduces_catch_and_damages_net():
    g = make_game(seed=11, camp_id="uganik", year=2025)
    net = g.nets[0]
    net.in_water = True
    net.site_id = "uganik_pass"
    net.fish = Lbs(red=80, pink=40)
    net.condition = 90
    before_lbs = net.fish.total()
    msg = apply_sea_lion_raid(g, "uganik_pass")
    assert net.fish.total() < before_lbs
    assert net.condition < 90
    assert "Steller" in msg
    assert "do not shoot" in msg.lower() or "You do not shoot" in msg


def test_sea_lion_does_not_require_a_kill():
    g = make_game()
    g.nets[0].in_water = True
    g.nets[0].site_id = "uganik_pass"
    g.nets[0].fish.add(SpeciesId.PINK, 20)
    msg = apply_sea_lion_raid(g)
    assert "killed" not in msg.lower()


def test_cannot_set_on_karluk_lagoon():
    g = make_game(camp_id="larsen")
    msg = g.deploy_net("net-1", "karluk_lagoon")
    assert "not your water" in msg.lower() or "seine" in msg.lower()
    assert not g.nets[0].in_water or g.nets[0].site_id != "karluk_lagoon"
