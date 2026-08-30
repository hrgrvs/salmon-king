use std::collections::{HashMap, HashSet};

use salmon_king::data::maps::render_map;
use salmon_king::data::sites::site;
use salmon_king::data::species::{short_code, SpeciesId};
use salmon_king::sim::engine::Game;
use salmon_king::sim::mammals::{empty_haulout_sites, resident_sites, transient_sites};
use salmon_king::sim::models::CrewStatus;

pub fn bar(value: f64, width: usize) -> String {
    let v = value.clamp(0.0, 100.0);
    let fill = ((v / 100.0) * width as f64).round() as usize;
    let fill = fill.min(width);
    format!("{}{}", "█".repeat(fill), "░".repeat(width - fill))
}

pub fn render_map_panel(game: &Game, cursor: Option<&str>) -> String {
    let deployed: HashMap<String, String> = game
        .nets
        .iter()
        .filter(|n| n.in_water)
        .filter_map(|n| n.site_id.clone().map(|s| (n.id.clone(), s)))
        .collect();
    let mut skiffs = HashMap::new();
    for s in &game.skiffs {
        if !s.wrecked {
            skiffs.insert(s.id.clone(), s.location.clone());
        }
    }
    let mut closed = HashSet::new();
    for s in game.playable_sites() {
        if !game.site_is_open(s.id).0 {
            closed.insert(s.id.to_string());
        }
    }
    let text = render_map(
        game.camp.map_id,
        game.camp.id,
        &deployed,
        &skiffs,
        game.tender.present,
        &closed,
        cursor,
        &transient_sites(&game.wildlife),
        &resident_sites(&game.wildlife),
    );
    let mut extra = Vec::new();
    let empty = empty_haulout_sites(&game.wildlife);
    for n in &game.nets {
        if n.in_water {
            if let Some(sid) = &n.site_id {
                if let Some(sdef) = site(sid) {
                    let lions = if empty.contains(sid) { "  lions gone" } else { "" };
                    extra.push(format!(
                        "  {} ╫ {}  {:.0} lb  soak {}  cond {:.0}%  {}{lions}",
                        n.id, sdef.short, n.fish.total(), n.soak_tides, n.condition, n.mesh
                    ));
                }
            }
        } else {
            extra.push(format!(
                "  {} on the beach  cond {:.0}%  {}",
                n.id, n.condition, n.mesh
            ));
        }
    }
    for s in &game.skiffs {
        let flag = if s.wrecked {
            "wrecked".into()
        } else {
            format!(
                "{} @ {}  {:.0} lb  fuel {:.0}",
                s.job.as_str(),
                s.location,
                s.cargo.total(),
                s.fuel
            )
        };
        extra.push(format!("  › {}: {flag}", s.name));
    }
    extra.extend(game.mammal_status().into_iter().map(|l| format!("  {l}")));
    let legend = "▲ camp   ╫ net   › skiff   ■ tender   · open   x dark   ω transients   r residents";
    format!("{text}\n{legend}\n{}", extra.join("\n"))
}

pub fn render_crew(game: &Game) -> String {
    let mut lines = vec!["CREW".to_string()];
    for c in &game.crew {
        if c.status == CrewStatus::Quit {
            lines.push(format!("  {}  QUIT", c.name));
            continue;
        }
        let job = c.assigned.as_deref().unwrap_or(c.role.as_str());
        let own = if c.is_owner { " *" } else { "" };
        lines.push(format!(
            "  {}{own}  {}/{job}  {}  {}\n    E {}  H {}  M {}",
            c.name,
            c.role,
            c.status.as_str(),
            c.tag,
            bar(c.energy, 8),
            bar(100.0 - c.hunger, 8),
            bar(c.morale, 8)
        ));
    }
    lines.join("\n")
}

pub fn render_tender(game: &Game) -> String {
    let p = &game.tender.prices;
    let eta = if game.tender.present {
        "IN THE HOLE".into()
    } else {
        format!("ETA {} tides", game.tender.eta_tides)
    };
    let prices = SpeciesId::ALL
        .iter()
        .map(|&sp| format!("{} ${:.2}", short_code(sp), p.get(sp)))
        .collect::<Vec<_>>()
        .join("  ");
    format!(
        "TENDER  {}  {eta}\n  {prices}\n  last: {}",
        game.tender.name, game.tender.last_note
    )
}

pub fn render_camp(game: &Game) -> String {
    let people = game
        .crew
        .iter()
        .filter(|c| c.status != CrewStatus::Quit)
        .count()
        .max(1);
    let days = game.food / people as f64;
    let jv = if game.joint_venture { "  JV 3rd net" } else { "" };
    format!(
        "CAMP  {}{jv}\n  cash ${:.0}   food {days:.1} d   fuel {:.0} gal   ice {:.0}   twine {}   prop {}\n  cookshack {}  bunk {}  loft {}  stalls {}\n  gross ${:.0}  exp ${:.0}  tickets {}",
        game.camp.name,
        game.ledger.cash,
        game.fuel_cache,
        game.ice_cache,
        game.twine,
        game.spare_prop,
        game.cookshack,
        game.bunkhouse,
        game.net_loft,
        game.skiff_stalls,
        game.ledger.gross,
        game.ledger.expenses(),
        game.ledger.tickets
    )
}

pub fn render_clock(game: &Game) -> String {
    let tag = if game.any_open() { "OPEN" } else { "DARK" };
    let skip = if game.skipper_in_town {
        "  PERMIT OFF-SITE"
    } else {
        ""
    };
    let mammal = game.mammal_status();
    let extra = if mammal.is_empty() {
        String::new()
    } else {
        format!("\n  {}", mammal.iter().take(2).cloned().collect::<Vec<_>>().join("\n  "))
    };
    format!(
        "CLOCK  {}  {} tide  {tag}{skip}\n  {}\n  {}\n  {}{extra}",
        game.day.fmt_long(),
        game.tide.as_str(),
        game.phase_label(),
        game.open_reason(),
        game.weather.label
    )
}

pub fn render_log(game: &Game, n: usize) -> String {
    let lines: Vec<_> = game.log.iter().rev().take(n).rev().collect();
    if lines.is_empty() {
        return "EVENT LOG".into();
    }
    let body = lines
        .iter()
        .map(|ln| format!("  {} {}", ln.day.fmt_md(), ln.text))
        .collect::<Vec<_>>()
        .join("\n");
    format!("EVENT LOG\n{body}")
}
