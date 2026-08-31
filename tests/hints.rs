use salmon_king::sim::engine::new_game;
use salmon_king::sim::hints::{set_hints, tick_hints};

#[test]
fn hints_default_on_and_toggle_persists() {
    let mut g = new_game(1701, "uganik", 2025).unwrap();
    assert!(g.hints_on);
    set_hints(&mut g, false);
    assert!(!g.hints_on);
    assert!(g.hint.is_none());
    tick_hints(&mut g);
    assert!(g.hint.is_none());
    set_hints(&mut g, true);
    assert!(g.hints_on);
}

#[test]
fn hint_on_opener_start_when_nets_still_on_the_beach() {
    let mut g = new_game(1701, "uganik", 2025).unwrap();
    assert!(g.hints_on);
    for n in &mut g.nets {
        n.in_water = false;
        n.site_id = None;
    }
    // Force an open window if this seed starts dark.
    if !g.any_open() {
        g.extend_open_tides = 4;
    }
    tick_hints(&mut g);
    let h = g.hint.expect("opener should prompt a set");
    assert_eq!(h.id, "opener_set");
    assert!(h.text.to_ascii_lowercase().contains("set"));
}

#[test]
fn no_hint_when_off() {
    let mut g = new_game(1701, "uganik", 2025).unwrap();
    for n in &mut g.nets {
        n.in_water = false;
    }
    if !g.any_open() {
        g.extend_open_tides = 4;
    }
    set_hints(&mut g, false);
    tick_hints(&mut g);
    assert!(g.hint.is_none());
}

#[test]
fn no_opener_hint_if_already_set() {
    let mut g = new_game(1701, "uganik", 2025).unwrap();
    if !g.any_open() {
        g.extend_open_tides = 4;
    }
    let site = g.camp.home_sites[0];
    let nid = g.nets[0].id.clone();
    g.deploy_net(&nid, site);
    tick_hints(&mut g);
    if let Some(h) = &g.hint {
        assert_ne!(h.id, "opener_set", "already set: {h:?}");
    }
}
