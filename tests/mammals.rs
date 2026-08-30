//! Transient orcas empty the haulout. Residents scatter fish. Neither raids the web.

use salmon_king::sim::catch::soak_net;
use salmon_king::sim::clock::GameDate;
use salmon_king::sim::engine::new_game;
use salmon_king::sim::events::{apply_harbor_seal_raid, apply_sea_lion_raid};
use salmon_king::sim::mammals::{
    arrive_residents, arrive_transients, pinnipeds_present, resident_scatter, tick_wildlife,
    transients_present, PRESSURE_FLOOR,
};
use salmon_king::sim::models::{Lbs, Net, Weather};
use salmon_king::sim::rng::Rng;
use salmon_king::sim::runs::site_availability;

fn make_game(camp_id: &str, seed: u64, year: i32) -> salmon_king::sim::engine::Game {
    new_game(seed, camp_id, year).unwrap()
}

fn uyak_nets(g: &mut salmon_king::sim::engine::Game) -> usize {
    g.nets[0].in_water = true;
    g.nets[0].site_id = Some("harvester".into());
    g.nets[0].fish = Lbs {
        red: 90.0,
        pink: 40.0,
        ..Lbs::default()
    };
    g.nets[0].sea_lion_pressure = 0.8;
    g.nets[0].condition = 90.0;
    0
}

#[test]
fn transients_clear_lions_and_do_not_take_fish() {
    let mut g = make_game("larsen", 3, 2025);
    let i = uyak_nets(&mut g);
    let before = g.nets[i].fish.total();
    let msg = arrive_transients(&mut g, Some("harvester"));
    assert!(msg.contains("Transients in Uyak"));
    assert!(msg.to_lowercase().contains("lions gone"));
    assert!(msg.to_lowercase().contains("do not pick"));
    assert!(transients_present(&g.wildlife, "harvester"));
    assert!(transients_present(&g.wildlife, "cape_uyak"));
    assert!(!pinnipeds_present(&g.wildlife, "harvester"));
    assert!(g.nets[i].sea_lion_pressure <= PRESSURE_FLOOR + 0.001);
    assert_eq!(g.nets[i].fish.total(), before);
}

#[test]
fn sea_lion_raid_blocked_while_transients_present() {
    let mut g = make_game("larsen", 4, 2025);
    let i = uyak_nets(&mut g);
    arrive_transients(&mut g, Some("harvester"));
    let before_lbs = g.nets[i].fish.total();
    let before_cond = g.nets[i].condition;
    let msg = apply_sea_lion_raid(&mut g, Some("harvester"));
    assert_eq!(g.nets[i].fish.total(), before_lbs);
    assert_eq!(g.nets[i].condition, before_cond);
    let low = msg.to_lowercase();
    assert!(low.contains("gone") || low.contains("empty"));
    assert!(low.contains("not picking") || low.contains("did not touch") || low.contains("haulout"));
}

#[test]
fn harbor_seal_raid_blocked_in_same_bay() {
    let mut g = make_game("larsen", 5, 2025);
    let i = uyak_nets(&mut g);
    arrive_transients(&mut g, Some("cape_uyak"));
    let before = g.nets[i].fish.total();
    let msg = apply_harbor_seal_raid(&mut g, Some("harvester"));
    assert_eq!(g.nets[i].fish.total(), before);
    let low = msg.to_lowercase();
    assert!(low.contains("seal") || low.contains("transients"));
}

#[test]
fn raid_still_works_on_another_bay() {
    let mut g = make_game("larsen", 6, 2025);
    uyak_nets(&mut g);
    g.nets[1].in_water = true;
    g.nets[1].site_id = Some("spiridon_outer".into());
    g.nets[1].fish = Lbs {
        red: 50.0,
        ..Lbs::default()
    };
    g.nets[1].condition = 88.0;
    arrive_transients(&mut g, Some("harvester"));
    assert!(pinnipeds_present(&g.wildlife, "spiridon_outer"));
    let msg = apply_sea_lion_raid(&mut g, Some("spiridon_outer"));
    assert!(g.nets[1].fish.total() < 50.0);
    assert!(msg.contains("Steller"));
}

#[test]
fn lions_return_only_after_delay() {
    let mut g = make_game("larsen", 7, 2025);
    uyak_nets(&mut g);
    arrive_transients(&mut g, Some("harvester"));
    let stay = g.wildlife["uyak"].transient_tides;
    let absent = g.wildlife["uyak"].pinniped_absent_tides;
    assert!(absent > stay);

    for _ in 0..stay {
        tick_wildlife(&mut g.wildlife);
    }
    assert!(!g.wildlife["uyak"].transients_here());
    assert!(!g.wildlife["uyak"].pinnipeds_present());
    let msg = apply_sea_lion_raid(&mut g, Some("harvester"));
    assert_eq!(g.nets[0].fish.total(), 130.0);
    let low = msg.to_lowercase();
    assert!(
        low.contains("haven't come back")
            || low.contains("hasn't filled")
            || low.contains("still gone")
    );

    let mut logs = Vec::new();
    for _ in 0..20 {
        if g.wildlife["uyak"].pinnipeds_present() {
            break;
        }
        logs = tick_wildlife(&mut g.wildlife);
    }
    assert!(
        logs.iter().any(|l| l.contains("back on the rocks"))
            || g.wildlife["uyak"].pinnipeds_present()
    );
    let msg = apply_sea_lion_raid(&mut g, Some("harvester"));
    assert!(g.nets[0].fish.total() < 130.0);
    assert!(msg.contains("Steller"));
}

#[test]
fn residents_do_not_clear_lions_or_raid() {
    let mut g = make_game("uganik", 8, 2025);
    g.nets[0].in_water = true;
    g.nets[0].site_id = Some("uganik_pass".into());
    g.nets[0].fish = Lbs {
        pink: 80.0,
        ..Lbs::default()
    };
    g.nets[0].sea_lion_pressure = 0.7;
    let before = g.nets[0].fish.total();
    let msg = arrive_residents(&mut g, Some("uganik_pass"));
    assert!(msg.contains("Residents"));
    assert!(msg.contains("Stellers are still here") || msg.to_lowercase().contains("lions"));
    assert!(pinnipeds_present(&g.wildlife, "uganik_pass"));
    assert!(!transients_present(&g.wildlife, "uganik_pass"));
    assert!(resident_scatter(&g.wildlife, "uganik_pass") < 1.0);
    assert_eq!(g.nets[0].sea_lion_pressure, 0.7);
    assert_eq!(g.nets[0].fish.total(), before);
    let raid = apply_sea_lion_raid(&mut g, Some("uganik_pass"));
    assert!(g.nets[0].fish.total() < before);
    assert!(raid.contains("Steller"));
}

#[test]
fn residents_dip_catch_transients_do_not() {
    let g = make_game("uganik", 10, 2025);
    let avail: Vec<_> = site_availability("uganik_pass", GameDate::new(2025, 6, 8), &g.mods)
        .into_iter()
        .map(|(k, v)| (k, v * 0.2))
        .collect();
    let wx = Weather::default();

    let soak_once = |scatter: f64, seed: u64| {
        let mut net = Net::new("t", Some("uganik_pass".into()), 75, "mixed");
        net.in_water = true;
        soak_net(
            &mut net,
            &avail,
            &wx,
            1.0,
            scatter,
            &mut Rng::new(seed),
            1.0,
            true,
        );
        net.fish.total()
    };

    let full = soak_once(1.0, 44);
    let dipped = soak_once(0.48, 44);
    assert!(dipped > 0.0 && dipped < full * 0.7);
    assert_eq!(soak_once(1.0, 44), full);
}

#[test]
fn status_lines_name_the_bay() {
    let mut g = make_game("larsen", 9, 2025);
    arrive_transients(&mut g, Some("harvester"));
    let lines = g.mammal_status();
    assert!(!lines.is_empty());
    assert!(lines
        .iter()
        .any(|l| l.contains("Uyak") && l.contains("lions gone")));
}
