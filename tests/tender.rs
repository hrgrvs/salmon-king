use salmon_king::sim::engine::new_game;
use salmon_king::sim::hints::{set_hints, tick_hints};
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

#[test]
fn tender_visit_feeds_the_skiff_crew() {
    let mut visit = make_game();
    let mut control = make_game();
    visit.tender.present = true;
    visit.tender.late = false;
    control.tender.present = true;
    control.tender.late = false;
    visit.skiffs[0].cargo = Lbs {
        red: 80.0,
        ..Lbs::default()
    };
    visit.skiffs[0].cargo_quality = 1.0;
    let rider = visit
        .crew
        .iter()
        .position(|c| c.assigned.as_deref() == Some(visit.skiffs[0].id.as_str()) || c.is_owner)
        .unwrap();
    let h0 = visit.crew[rider].hunger;
    let m0 = visit.crew[rider].morale;
    visit.settle(0);
    assert!(
        visit.crew[rider].hunger < h0,
        "treat should cut hunger: {} -> {}",
        h0,
        visit.crew[rider].hunger
    );
    assert!(
        visit.crew[rider].morale > m0,
        "treat should lift morale: {} -> {}",
        m0,
        visit.crew[rider].morale
    );
    assert!(visit.last_treat.is_some());
    assert!(visit
        .log
        .iter()
        .any(|l| l.text.starts_with("tender treat:")));
    assert!(visit
        .radio
        .iter()
        .any(|l| l.text.starts_with("tender treat:")));
    assert_eq!(control.crew[rider].hunger, h0);
    assert_eq!(control.crew[rider].morale, m0);
    assert!(control.last_treat.is_none());
}

#[test]
fn no_treat_when_tender_is_offshore() {
    let mut g = make_game();
    g.tender.present = false;
    g.skiffs[0].cargo = Lbs {
        red: 80.0,
        ..Lbs::default()
    };
    g.skiffs[0].cargo_quality = 1.0;
    let rider = 0;
    let h0 = g.crew[rider].hunger;
    g.settle(0);
    assert!(g.last_treat.is_none());
    assert_eq!(g.crew[rider].hunger, h0);
    assert!(!g.log.iter().any(|l| l.text.starts_with("tender treat:")));

    g.tender.present = true;
    g.tender.late = true;
    g.skiffs[0].cargo = Lbs {
        red: 80.0,
        ..Lbs::default()
    };
    g.settle(0);
    assert!(g.last_treat.is_none(), "late in the hole is not a treat");
}

#[test]
fn same_day_treat_shrinks_and_cannot_be_farmed() {
    let mut g = make_game();
    g.tender.present = true;
    g.tender.late = false;
    let rider = g.crew.iter().position(|c| c.is_owner).unwrap();
    g.crew[rider].hunger = 40.0;
    g.crew[rider].morale = 50.0;
    let mut cuts = Vec::new();
    for _ in 0..4 {
        let h = g.crew[rider].hunger;
        g.skiffs[0].cargo = Lbs {
            red: 40.0,
            ..Lbs::default()
        };
        g.skiffs[0].cargo_quality = 1.0;
        g.settle(0);
        cuts.push(h - g.crew[rider].hunger);
    }
    assert!(cuts[0] > cuts[1] && cuts[1] > cuts[2]);
    assert_eq!(cuts[3], 0.0, "fourth visit same day is dry");
}

#[test]
fn tender_treat_hint_once_and_respects_off() {
    let mut g = make_game();
    g.tender.present = true;
    g.tender.late = false;
    g.last_treat = None;
    g.extend_open_tides = 4;
    g.weather.williwaw = false;
    for n in &mut g.nets {
        n.in_water = true;
        n.site_id = Some(g.camp.home_sites[0].into());
    }
    for s in &mut g.skiffs {
        s.cargo = Lbs::default();
    }
    for c in &mut g.crew {
        c.hunger = 20.0;
        c.morale = 70.0;
    }
    g.food = 200.0;
    g.fuel_cache = 80.0;
    g.ledger.cash = 8000.0;
    set_hints(&mut g, false);
    tick_hints(&mut g);
    assert!(g.hint.is_none());
    set_hints(&mut g, true);
    g.hint_seen.clear();
    g.hint_seen.insert("transients_window".into());
    g.hint = None;
    g.hint_snooze_tick = 0;
    tick_hints(&mut g);
    let h = g.hint.expect("treat aside when hints on");
    assert_eq!(h.id, "tender_treat");
    assert!(h.text.to_ascii_lowercase().contains("garlic bread"));
}
