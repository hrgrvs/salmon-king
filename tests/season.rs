use salmon_king::sim::clock::tides_in_season;
use salmon_king::sim::engine::{new_game, run_headless};
use salmon_king::sim::models::GameEnd;

#[test]
fn full_odd_year_season_completes() {
    let mut g = new_game(2025, "uganik", 2025).unwrap();
    let recap = run_headless(&mut g, None);
    assert!(matches!(
        recap.end,
        GameEnd::Season | GameEnd::Bankrupt | GameEnd::NoSkiffs | GameEnd::NoCrew
    ));
    assert!(g.tick >= 20);
    assert!(["A", "B", "C", "D", "F"].contains(&recap.grade.as_str()));
}

#[test]
fn alitak_season_does_not_crash() {
    let mut g = new_game(8, "olga", 2023).unwrap();
    let recap = run_headless(&mut g, Some(80));
    assert!(g.tick == 80 || g.end != GameEnd::None);
    let _ = recap;
}

#[test]
fn even_year_port_bailey_headless() {
    let mut g = new_game(4, "bailey", 2024).unwrap();
    let recap = run_headless(&mut g, Some(tides_in_season(2024)));
    let _ = recap.gross;
}

#[test]
fn four_camps_construct() {
    for camp in ["larsen", "uganik", "olga", "bailey"] {
        let g = new_game(1, camp, 2025).unwrap();
        assert_eq!(g.camp.id, camp);
        assert_eq!(g.nets.len(), 2);
        assert!(g
            .playable_sites()
            .iter()
            .all(|s| s.travel_from(g.camp.id).is_some()));
    }
}

#[test]
fn pause_and_speed_are_tui_only() {
    // Sim has no clock of its own: N steps is N tides regardless of "speed".
    let mut g = new_game(11, "uganik", 2025).unwrap();
    g.step();
    g.step();
    g.step();
    assert_eq!(g.tick, 3);
}
