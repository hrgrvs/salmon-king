from salmon_king.sim.catch import crew_efficiency
from salmon_king.sim.clock import Tide
from salmon_king.sim.models import CrewStatus
from tests.conftest import make_game


def test_hungry_crew_lose_morale_and_efficiency():
    g = make_game()
    picker = next(c for c in g.crew if not c.is_owner and c.role != "cook")
    picker.hunger = 20
    picker.morale = 70
    picker.energy = 80
    picker.status = CrewStatus.WORKING
    fed_eff = crew_efficiency([picker])

    picker.hunger = 88
    hungry_eff = crew_efficiency([picker])
    assert hungry_eff < fed_eff

    g.food = 0
    g.tide = Tide.EBB  # next step advances to flood, which triggers eating
    before = picker.morale
    g.step()
    # metabolism runs; food empty → hunger up, morale down
    assert picker.hunger >= 88
    assert picker.morale <= before
