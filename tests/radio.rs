use salmon_king::sim::engine::new_game;
use salmon_king::sim::models::SkiffJob;
use salmon_king::sim::radio::{RadioKind, RadioVoice};

#[test]
fn radio_skiff_call_issues_a_real_order() {
    let mut g = new_game(1701, "uganik", 2025).unwrap();
    let hold = g
        .skiffs
        .iter()
        .find(|s| s.kind == "holding")
        .unwrap()
        .id
        .clone();
    let msg = g.radio_call(&format!("skiff|{hold}|pick"), Some("uganik_pass"));
    assert!(msg.to_lowercase().contains("assigned") || msg.to_lowercase().contains("pick"));
    let hold = g.skiffs.iter().find(|s| s.kind == "holding").unwrap();
    assert_eq!(hold.job, SkiffJob::Pick);
    assert!(g
        .radio
        .iter()
        .any(|l| l.voice == RadioVoice::Skipper && l.text.to_lowercase().contains("pick")));
    assert!(g
        .radio
        .iter()
        .any(|l| l.voice == RadioVoice::Crew && l.text.to_lowercase().contains("copy")));
}

#[test]
fn tender_on_grounds_gives_live_board_and_chatter() {
    let mut g = new_game(1701, "uganik", 2025).unwrap();
    g.tender.present = true;
    g.tender.late = false;
    let msg = g.radio_call("tender", None);
    assert!(msg.contains("RED") || msg.contains("$"), "{msg}");
    assert!(g
        .radio
        .iter()
        .any(|l| l.kind == RadioKind::TenderQuote && l.text.contains('$')));
    let rumor = g.radio.iter().find(|l| l.kind == RadioKind::Rumor);
    assert!(rumor.is_some(), "expected a rumor line");
    let r = rumor.unwrap();
    assert!(r.rumor_truth.is_some());
    let low = r.text.to_ascii_lowercase();
    assert!(!low.contains("official"));
    assert!(!low.contains("adf&g"));
    assert!(!low.contains("fact"));
    assert!(!low.contains("gospel"));
}

#[test]
fn tender_offshore_no_board_no_gossip() {
    let mut g = new_game(1701, "uganik", 2025).unwrap();
    g.tender.present = false;
    g.tender.late = false;
    g.tender.eta_tides = 5;
    let msg = g.radio_call("tender", None);
    assert!(
        msg.to_lowercase().contains("no answer") || msg.to_lowercase().contains("offshore"),
        "{msg}"
    );
    assert!(!g.radio.iter().any(|l| l.kind == RadioKind::TenderQuote));
    assert!(!g.radio.iter().any(|l| l.kind == RadioKind::Rumor));
}

#[test]
fn official_daily_once_per_calendar_day() {
    let mut g = new_game(1701, "uganik", 2025).unwrap();
    let day0 = g
        .radio
        .iter()
        .filter(|l| l.kind == RadioKind::Daily)
        .count();
    assert_eq!(day0, 1, "new season airs one daily");
    g.step(); // same calendar day (flood -> ebb)
    let day_same = g
        .radio
        .iter()
        .filter(|l| l.kind == RadioKind::Daily)
        .count();
    assert_eq!(day_same, 1, "do not spam daily every tide");
    g.step(); // next calendar day
    let day2 = g
        .radio
        .iter()
        .filter(|l| l.kind == RadioKind::Daily)
        .count();
    assert_eq!(day2, 2);
    assert!(g.radio.iter().any(|l| l.voice == RadioVoice::Adfg));
}

#[test]
fn rumor_is_never_marked_official() {
    let mut g = new_game(9, "larsen", 2025).unwrap();
    g.tender.present = true;
    g.tender.late = false;
    g.radio_call("tender", None);
    for line in &g.radio {
        if line.kind == RadioKind::Rumor {
            assert_ne!(line.voice, RadioVoice::Adfg);
            assert_eq!(line.channel, "68");
            assert!(line.rumor_truth.is_some());
        }
        if line.voice == RadioVoice::Adfg {
            assert_ne!(line.kind, RadioKind::Rumor);
        }
    }
}
