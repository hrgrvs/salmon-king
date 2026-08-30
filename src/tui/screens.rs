use salmon_king::data::crew_pool::candidate;
use salmon_king::sim::engine::Game;
use salmon_king::sim::models::CrewStatus;

#[derive(Clone, Debug)]
pub struct ActionItem {
    pub id: String,
    pub label: String,
}

pub fn skiff_items(game: &Game) -> Vec<ActionItem> {
    let mut out = Vec::new();
    for s in &game.skiffs {
        out.push(item(format!("{}|pick", s.id), format!("{}: pick nets", s.name)));
        out.push(item(format!("{}|tender", s.id), format!("{}: run tender", s.name)));
        out.push(item(format!("{}|idle", s.id), format!("{}: idle / beach", s.name)));
        out.push(item(format!("{}|repair", s.id), format!("{}: repair", s.name)));
        out.push(item(
            format!("{}|town", s.id),
            format!("{}: town run (permit leaves → nets dark)", s.name),
        ));
    }
    out
}

pub fn crew_items(game: &Game) -> Vec<ActionItem> {
    let mut out = Vec::new();
    for c in &game.crew {
        if c.status == CrewStatus::Quit {
            continue;
        }
        for s in &game.skiffs {
            out.push(item(
                format!("{}|{}", c.id, s.id),
                format!("{} → {}", c.name, s.name),
            ));
        }
        out.push(item(format!("{}|camp", c.id), format!("{} → bunkhouse", c.name)));
        if c.role == "cook" || c.id.ends_with("cook") {
            out.push(item(format!("{}|cook", c.id), format!("{} → cookshack", c.name)));
        }
    }
    out
}

pub fn hire_items(game: &Game) -> Vec<ActionItem> {
    let mut out = Vec::new();
    for cid in &game.hire_pool {
        if let Some(cand) = candidate(cid) {
            let label = if cand.wage_share > 0.0 {
                format!(
                    "{}  {}  {}  share {:.0}%",
                    cand.name,
                    cand.role,
                    cand.tag,
                    cand.wage_share * 100.0
                )
            } else {
                format!("{}  cook  ${}/d", cand.name, cand.daily_wage)
            };
            out.push(item(cid.clone(), label));
        }
    }
    if out.is_empty() {
        out.push(item("none", "Nobody on the beach."));
    }
    out
}

pub fn buy_items() -> Vec<ActionItem> {
    vec![
        item("food", "Food — 18 person-days  $160"),
        item("fuel", "Fuel — 40 gal  $170"),
        item("ice", "Slush ice  $55"),
        item("twine", "Needles / twine / corks  $90"),
        item("prop", "Spare prop  $220"),
    ]
}

pub fn upgrade_items() -> Vec<ActionItem> {
    vec![
        item("cookshack", "Cookshack — feeds better  $1800"),
        item("bunkhouse", "Bunkhouse — rest  $1600"),
        item("loft", "Net loft — faster mend  $1400"),
        item("stall", "Skiff stall  $2200"),
    ]
}

pub fn mesh_items(game: &Game) -> Vec<ActionItem> {
    let mut out = Vec::new();
    for n in &game.nets {
        for m in ["pink", "mixed", "red"] {
            out.push(item(
                format!("{}|{m}", n.id),
                format!("{} hang {m} (game knob, not a legal mesh spec)", n.id),
            ));
        }
    }
    out
}

pub fn pull_items(game: &Game) -> Vec<ActionItem> {
    game.nets
        .iter()
        .map(|n| {
            item(
                n.id.clone(),
                format!("Pull {}  ({})", n.id, n.site_id.as_deref().unwrap_or("beach")),
            )
        })
        .collect()
}

fn item(id: impl Into<String>, label: impl Into<String>) -> ActionItem {
    ActionItem {
        id: id.into(),
        label: label.into(),
    }
}
