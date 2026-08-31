use salmon_king::data::bay_charts::{render_bay_chart, BayMarks, BaySkiff};
use salmon_king::sim::engine::new_game;
use salmon_king::sim::mammals::{arrive_transients, transient_sites};
use salmon_king::sim::models::SkiffJob;
use salmon_king::sim::status::seal_sites_visible;
use std::collections::HashSet;

#[test]
fn bay_shows_camp_and_live_boats() {
    let g = new_game(1701, "uganik", 2025).unwrap();
    let skiffs: Vec<BaySkiff> = g
        .skiffs
        .iter()
        .map(|s| BaySkiff {
            location: s.location.clone(),
            dest: s.dest.clone(),
            from: s.from.clone(),
            eta: s.eta,
            kind: s.kind.clone(),
        })
        .collect();
    let empty = HashSet::new();
    let text = render_bay_chart(
        g.camp.id,
        &BayMarks {
            skiffs: &skiffs,
            tender_here: g.tender.present,
            transient_sites: &empty,
            resident_sites: &empty,
            seal_sites: &empty,
        },
    );
    assert!(text.contains("CAMP") || text.contains("Uganik"), "{text}");
    assert!(text.contains('▲'), "{text}");
    assert!(text.contains('›') || text.contains('»'), "{text}");
}

#[test]
fn transit_moves_the_glyph_off_camp() {
    let empty = HashSet::new();
    let parked = [BaySkiff {
        location: "camp".into(),
        dest: None,
        from: None,
        eta: 0,
        kind: "picker".into(),
    }];
    let moving = [BaySkiff {
        location: "transit".into(),
        dest: Some("uganik_pass".into()),
        from: Some("camp".into()),
        eta: 2,
        kind: "picker".into(),
    }];
    let a = render_bay_chart(
        "uganik",
        &BayMarks {
            skiffs: &parked,
            tender_here: false,
            transient_sites: &empty,
            resident_sites: &empty,
            seal_sites: &empty,
        },
    );
    let b = render_bay_chart(
        "uganik",
        &BayMarks {
            skiffs: &moving,
            tender_here: false,
            transient_sites: &empty,
            resident_sites: &empty,
            seal_sites: &empty,
        },
    );
    assert_ne!(a, b, "transit should move the boat\n{a}\n{b}");
}

#[test]
fn whales_always_seals_only_while_picking() {
    let mut g = new_game(3, "larsen", 2025).unwrap();
    arrive_transients(&mut g, Some("harvester"));
    let trans = transient_sites(&g.wildlife);
    assert!(!trans.is_empty());

    g.skiffs[0].job = SkiffJob::Idle;
    g.skiffs[0].location = "camp".into();
    assert!(seal_sites_visible(&g).is_empty());

    g.skiffs[0].job = SkiffJob::Pick;
    g.skiffs[0].location = "harvester".into();
    // Transients emptied the haulout — still no seals even while picking.
    assert!(seal_sites_visible(&g).is_empty());

    // Restore lions, pick: seals appear.
    if let Some(bay) = g.wildlife.get_mut("uyak") {
        bay.transient_tides = 0;
        bay.pinniped_absent_tides = 0;
    }
    let seals = seal_sites_visible(&g);
    assert!(seals.contains("harvester"), "{seals:?}");

    g.skiffs[0].job = SkiffJob::Idle;
    assert!(seal_sites_visible(&g).is_empty());

    let empty = HashSet::new();
    let text = render_bay_chart(
        "larsen",
        &BayMarks {
            skiffs: &[],
            tender_here: false,
            transient_sites: &trans,
            resident_sites: &empty,
            seal_sites: &empty,
        },
    );
    assert!(text.contains('ω'), "whales always when present\n{text}");
}
