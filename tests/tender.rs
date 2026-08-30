use salmon_king::sim::engine::new_game;
use salmon_king::sim::models::Lbs;

fn make_game() -> salmon_king::sim::engine::Game {
    new_game(7, "uganik", 2025).unwrap()
}

#[test]
fn tender_settlement_credits_cash() {
    let mut g = make_game();
    g.tender.present = true;
    g.skiffs[0].cargo = Lbs {
        red: 100.0,
        pink: 200.0,
        ..Lbs::default()
    };
    g.skiffs[0].cargo_quality = 1.0;
    let before = g.ledger.cash;
    let pay = g.settle(0);
    assert!(pay > 0.0);
    assert_eq!(g.ledger.cash, before + pay);
    assert_eq!(g.ledger.gross, pay);
    assert_eq!(g.ledger.tickets, 1);
    assert_eq!(g.skiffs[0].cargo.total(), 0.0);
    assert_eq!(g.landed.red, 100.0);
    assert_eq!(g.landed.pink, 200.0);
}

#[test]
fn soft_fish_docks_the_ticket() {
    let mut g = make_game();
    g.tender.present = true;
    g.skiffs[0].cargo = Lbs {
        red: 100.0,
        ..Lbs::default()
    };
    g.skiffs[0].cargo_quality = 1.1;
    let hi = g.settle(0);

    g.skiffs[1].cargo = Lbs {
        red: 100.0,
        ..Lbs::default()
    };
    g.skiffs[1].cargo_quality = 0.55;
    let lo = g.settle(1);
    assert!(lo < hi);
}
