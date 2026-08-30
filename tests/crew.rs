use salmon_king::sim::catch::crew_efficiency;
use salmon_king::sim::clock::Tide;
use salmon_king::sim::engine::new_game;
use salmon_king::sim::models::CrewStatus;

#[test]
fn hungry_crew_lose_morale_and_efficiency() {
    let mut g = new_game(7, "uganik", 2025).unwrap();
    let i = g
        .crew
        .iter()
        .position(|c| !c.is_owner && c.role != "cook")
        .unwrap();
    g.crew[i].hunger = 20.0;
    g.crew[i].morale = 70.0;
    g.crew[i].energy = 80.0;
    g.crew[i].status = CrewStatus::Working;
    let fed_eff = crew_efficiency(std::slice::from_ref(&g.crew[i]));

    g.crew[i].hunger = 88.0;
    let hungry_eff = crew_efficiency(std::slice::from_ref(&g.crew[i]));
    assert!(hungry_eff < fed_eff);

    g.food = 0.0;
    g.tide = Tide::Ebb;
    let before = g.crew[i].morale;
    g.step();
    assert!(g.crew[i].hunger >= 88.0);
    assert!(g.crew[i].morale <= before);
}
