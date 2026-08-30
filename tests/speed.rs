//! Pause / speed is a TUI concern. Same seed + N ticks = same state.

use salmon_king::sim::engine::new_game;
use salmon_king::sim::models::GameEnd;

fn snapshot(g: &salmon_king::sim::engine::Game) -> String {
    format!(
        "{:?}|{:?}|{:?}|{:.2}|{:.2}|{:.2}|{:?}|{:?}",
        g.tick,
        (g.day.year, g.day.month, g.day.day),
        g.tide.as_str(),
        (g.ledger.cash * 100.0).round() / 100.0,
        (g.nets.iter().map(|n| n.fish.total()).sum::<f64>() * 100.0).round() / 100.0,
        (g.food * 100.0).round() / 100.0,
        g.crew
            .iter()
            .map(|c| format!(
                "{}:{:.1}:{:.1}:{}",
                c.id,
                (c.energy * 10.0).round() / 10.0,
                (c.hunger * 10.0).round() / 10.0,
                c.status.as_str()
            ))
            .collect::<Vec<_>>(),
        g.nets
            .iter()
            .map(|n| format!(
                "{}:{:?}:{}:{:.1}",
                n.id,
                n.site_id,
                n.in_water,
                (n.condition * 10.0).round() / 10.0
            ))
            .collect::<Vec<_>>()
    )
}

#[test]
fn same_seed_same_ticks_same_state() {
    let mut a = new_game(99, "larsen", 2025).unwrap();
    let mut b = new_game(99, "larsen", 2025).unwrap();
    for _ in 0..24 {
        a.step();
        b.step();
    }
    assert_eq!(snapshot(&a), snapshot(&b));
}

#[test]
fn interleaved_inspection_does_not_desync() {
    let mut g = new_game(42, "bailey", 2024).unwrap();
    for _ in 0..10 {
        g.step();
    }
    let mid = snapshot(&g);
    let _ = g.open_reason();
    let _: Vec<f64> = g.nets.iter().map(|n| n.fish.total()).collect();
    assert_eq!(snapshot(&g), mid);
    g.step();
    assert_eq!(g.tick, 11);
}

#[test]
fn closed_period_does_not_add_fish() {
    let mut g = new_game(1, "olga", 2024).unwrap();
    g.nets[0].in_water = true;
    g.nets[0].site_id = Some("olga_narrows".into());
    for _ in 0..80 {
        let (open, _) = g.site_is_open("olga_narrows");
        if !open && g.extend_open_tides <= 0 {
            let before = g.nets[0].fish.total();
            g.step();
            let (open2, _) = g.site_is_open("olga_narrows");
            if !open2 {
                assert!(g.nets[0].fish.total() <= before + 0.01);
                return;
            }
        }
        g.step();
        if g.end != GameEnd::None {
            break;
        }
    }
}
