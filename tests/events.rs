use salmon_king::data::species::SpeciesId;
use salmon_king::sim::engine::new_game;
use salmon_king::sim::events::apply_sea_lion_raid;
use salmon_king::sim::models::Lbs;

fn make_game(seed: u64, camp_id: &str, year: i32) -> salmon_king::sim::engine::Game {
    new_game(seed, camp_id, year).unwrap()
}

#[test]
fn sea_lion_raid_reduces_catch_and_damages_net() {
    let mut g = make_game(11, "uganik", 2025);
    g.nets[0].in_water = true;
    g.nets[0].site_id = Some("uganik_pass".into());
    g.nets[0].fish = Lbs {
        red: 80.0,
        pink: 40.0,
        ..Lbs::default()
    };
    g.nets[0].condition = 90.0;
    let before_lbs = g.nets[0].fish.total();
    let msg = apply_sea_lion_raid(&mut g, Some("uganik_pass"));
    assert!(g.nets[0].fish.total() < before_lbs);
    assert!(g.nets[0].condition < 90.0);
    assert!(msg.contains("Steller"));
    assert!(msg.to_lowercase().contains("do not shoot") || msg.contains("You do not shoot"));
}

#[test]
fn sea_lion_does_not_require_a_kill() {
    let mut g = make_game(7, "uganik", 2025);
    g.nets[0].in_water = true;
    g.nets[0].site_id = Some("uganik_pass".into());
    g.nets[0].fish.add(SpeciesId::Pink, 20.0);
    let msg = apply_sea_lion_raid(&mut g, None);
    assert!(!msg.to_lowercase().contains("killed"));
}

#[test]
fn cannot_set_on_karluk_lagoon() {
    let mut g = make_game(7, "larsen", 2025);
    let msg = g.deploy_net("net-1", "karluk_lagoon");
    let low = msg.to_lowercase();
    assert!(low.contains("not your water") || low.contains("seine"));
    assert!(!(g.nets[0].in_water && g.nets[0].site_id.as_deref() == Some("karluk_lagoon")));
}
