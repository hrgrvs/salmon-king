from salmon_king.sim.models import Lbs
from tests.conftest import make_game


def test_tender_settlement_credits_cash():
    g = make_game()
    g.tender.present = True
    skiff = g.skiffs[0]
    skiff.cargo = Lbs(red=100, pink=200)
    skiff.cargo_quality = 1.0
    before = g.ledger.cash
    pay = g.settle(skiff)
    assert pay > 0
    assert g.ledger.cash == before + pay
    assert g.ledger.gross == pay
    assert g.ledger.tickets == 1
    assert skiff.cargo.total() == 0
    assert g.landed.red == 100
    assert g.landed.pink == 200


def test_soft_fish_docks_the_ticket():
    g = make_game()
    g.tender.present = True
    good = g.skiffs[0]
    good.cargo = Lbs(red=100)
    good.cargo_quality = 1.1
    hi = g.settle(good)

    g.skiffs[1].cargo = Lbs(red=100)
    g.skiffs[1].cargo_quality = 0.55
    lo = g.settle(g.skiffs[1])
    assert lo < hi
