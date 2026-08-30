use salmon_king::sim::engine::new_game;
use salmon_king::sim::models::{CrewStatus, SkiffJob};
use salmon_king::sim::status::{
    crew_activity, crew_glance, skiff_job_words, skiff_status, skiff_where,
};

#[test]
fn picking_skiff_job_and_site_are_words() {
    let g = new_game(1701, "uganik", 2025).unwrap();
    let pick = g.skiffs.iter().find(|s| s.kind == "picker").unwrap();
    assert_eq!(pick.job, SkiffJob::Pick);
    assert_eq!(skiff_job_words(&g, pick), "picking");
    let st = skiff_status(&g, pick);
    assert_eq!(st.job_words, "picking");
    assert!(
        st.crew_names
            .iter()
            .any(|n| n.contains("permit") || n.contains("You")),
        "skipper should show on the picking skiff: {:?}",
        st.crew_names
    );
}

#[test]
fn holding_skiff_idle_in_the_hole() {
    let g = new_game(1701, "uganik", 2025).unwrap();
    let hold = g.skiffs.iter().find(|s| s.kind == "holding").unwrap();
    assert_eq!(hold.job, SkiffJob::Idle);
    assert_eq!(hold.location, "camp");
    assert_eq!(skiff_job_words(&g, hold), "idle in the hole");
    assert_eq!(skiff_where(hold), "at camp");
}

#[test]
fn transit_uses_named_destination() {
    let mut g = new_game(1701, "larsen", 2025).unwrap();
    g.skiffs[0].location = "transit".into();
    g.skiffs[0].dest = Some("harvester".into());
    g.skiffs[0].job = SkiffJob::Pick;
    assert_eq!(skiff_where(&g.skiffs[0]), "in transit to Harvester");
    assert_eq!(skiff_job_words(&g, &g.skiffs[0]), "picking");
}

#[test]
fn tender_run_and_town_jobs() {
    let mut g = new_game(1701, "uganik", 2025).unwrap();
    g.skiffs[1].job = SkiffJob::Tender;
    g.skiffs[1].location = "tender".into();
    g.tender.present = true;
    assert_eq!(skiff_job_words(&g, &g.skiffs[1]), "running fish to tender");
    assert_eq!(skiff_where(&g.skiffs[1]), "at the tender");

    g.tender.present = false;
    assert_eq!(skiff_job_words(&g, &g.skiffs[1]), "waiting on the tender");

    g.skiffs[1].job = SkiffJob::Town;
    g.skiffs[1].location = "town".into();
    assert_eq!(skiff_job_words(&g, &g.skiffs[1]), "town");
    assert_eq!(skiff_where(&g.skiffs[1]), "in town");

    g.skiffs[1].job = SkiffJob::Repair;
    g.skiffs[1].location = "camp".into();
    assert_eq!(skiff_job_words(&g, &g.skiffs[1]), "repair");
}

#[test]
fn cook_is_cooking_owner_is_picking_at_named_site() {
    let g = new_game(1701, "uganik", 2025).unwrap();
    let cook = g.crew.iter().find(|c| c.role == "cook").unwrap();
    assert_eq!(crew_activity(&g, cook), "cooking");

    let owner = g.crew.iter().find(|c| c.is_owner).unwrap();
    let act = crew_activity(&g, owner);
    assert!(
        act.starts_with("picking net at "),
        "owner should be picking a named site, got {act}"
    );
    assert!(act.contains("Picking skiff"), "{act}");
    let glance = crew_glance(&g, owner);
    assert_eq!(glance.skiff_name.as_deref(), Some("Picking skiff"));
}

#[test]
fn sleeping_sick_quit_and_idle_read_in_words() {
    let mut g = new_game(1701, "olga", 2025).unwrap();
    g.crew[0].status = CrewStatus::Resting;
    g.crew[0].assigned = None;
    assert_eq!(crew_activity(&g, &g.crew[0]), "sleeping");

    g.crew[0].status = CrewStatus::Sick;
    assert_eq!(crew_activity(&g, &g.crew[0]), "sick");

    g.crew[0].status = CrewStatus::Quit;
    assert_eq!(crew_activity(&g, &g.crew[0]), "quit");

    let picker = g
        .crew
        .iter_mut()
        .find(|c| !c.is_owner && c.role != "cook")
        .unwrap();
    picker.status = CrewStatus::Working;
    picker.assigned = None;
    // After the mut borrow ends:
    let picker = g
        .crew
        .iter()
        .find(|c| !c.is_owner && c.role != "cook")
        .unwrap();
    assert_eq!(crew_activity(&g, picker), "idle");
}

#[test]
fn crew_on_tender_run_names_the_skiff() {
    let mut g = new_game(1701, "bailey", 2025).unwrap();
    let sid = g.skiffs[0].id.clone();
    g.skiffs[0].job = SkiffJob::Tender;
    let ci = g.crew.iter().position(|c| c.is_owner).unwrap();
    g.crew[ci].assigned = Some(sid);
    g.crew[ci].status = CrewStatus::Working;
    let act = crew_activity(&g, &g.crew[ci]);
    assert_eq!(act, "on the tender run (Picking skiff)");
}
