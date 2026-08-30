//! Glanceable boat / crew copy derived from live sim fields.
//!
//! No extra state. A missing "current job" string is assembled here from job,
//! location, dest, assignment, and crew status the engine already keeps.

use crate::data::sites::site;
use crate::sim::engine::Game;
use crate::sim::models::{CrewMember, CrewStatus, Skiff, SkiffJob};

/// Human place name for a skiff location or destination id.
pub fn place_name(id: &str) -> String {
    match id {
        "camp" => "camp".into(),
        "town" => "town".into(),
        "tender" => "the tender".into(),
        "transit" => "transit".into(),
        other => site(other)
            .map(|s| s.short.to_string())
            .unwrap_or_else(|| other.replace('_', " ")),
    }
}

/// Where the skiff is, in words: camp, named site, in transit to X, at tender.
pub fn skiff_where(skiff: &Skiff) -> String {
    if skiff.wrecked {
        return "wrecked".into();
    }
    if skiff.location == "transit" {
        let dest = skiff
            .dest
            .as_deref()
            .map(place_name)
            .unwrap_or_else(|| "somewhere".into());
        return format!("in transit to {dest}");
    }
    match skiff.location.as_str() {
        "camp" => "at camp".into(),
        "town" => "in town".into(),
        "tender" => "at the tender".into(),
        other => format!("at {}", place_name(other)),
    }
}

/// Current job in words. Idle at camp reads as idle in the hole.
pub fn skiff_job_words(game: &Game, skiff: &Skiff) -> String {
    if skiff.wrecked {
        return "wrecked".into();
    }
    match skiff.job {
        SkiffJob::Pick => "picking".into(),
        SkiffJob::Tender => {
            if !game.tender.present {
                "waiting on the tender".into()
            } else {
                "running fish to tender".into()
            }
        }
        SkiffJob::Town => "town".into(),
        SkiffJob::Repair => "repair".into(),
        SkiffJob::Idle => {
            if skiff.location == "camp" || skiff.location == "tender" {
                "idle in the hole".into()
            } else {
                "idle".into()
            }
        }
    }
}

/// Crew actually riding a given skiff (assignment, plus the skipper's implicit pick).
pub fn crew_names_on_skiff(game: &Game, skiff: &Skiff) -> Vec<String> {
    let mut names = Vec::new();
    for c in &game.crew {
        if c.status == CrewStatus::Quit {
            continue;
        }
        if crew_skiff(game, c).is_some_and(|s| s.id == skiff.id) {
            names.push(c.name.clone());
        }
    }
    names
}

/// Which skiff this person is on right now, if any.
pub fn crew_skiff<'a>(game: &'a Game, crew: &CrewMember) -> Option<&'a Skiff> {
    if crew.status == CrewStatus::Quit || crew.status == CrewStatus::Sick {
        return None;
    }
    if let Some(aid) = crew.assigned.as_deref() {
        if aid == "cook" || aid == "camp" {
            return None;
        }
        return game.skiffs.iter().find(|s| s.id == aid);
    }
    // Engine: unassigned working skipper rides a picking skiff.
    if crew.is_owner && !game.skipper_in_town && crew.status == CrewStatus::Working {
        return game
            .skiffs
            .iter()
            .find(|s| s.job == SkiffJob::Pick && !s.wrecked);
    }
    None
}

/// Activity in words: picking at a named site, cooking, mending, sleeping, tender run, town, idle, sick, quit.
pub fn crew_activity(game: &Game, crew: &CrewMember) -> String {
    match crew.status {
        CrewStatus::Quit => return "quit".into(),
        CrewStatus::Sick => return "sick".into(),
        CrewStatus::Town => return "town".into(),
        CrewStatus::Resting => return "sleeping".into(),
        CrewStatus::Working => {}
    }

    if crew.assigned.as_deref() == Some("cook") || (crew.role == "cook" && crew.assigned.is_none())
    {
        return "cooking".into();
    }

    if let Some(skiff) = crew_skiff(game, crew) {
        let boat = skiff.name.as_str();
        return match skiff.job {
            SkiffJob::Pick => {
                let loc = skiff.location.as_str();
                let site_id = skiff.job_site.as_deref().or({
                    if matches!(loc, "camp" | "town" | "tender" | "transit") {
                        None
                    } else {
                        Some(loc)
                    }
                });
                match site_id.filter(|id| !matches!(*id, "camp" | "town" | "tender" | "transit")) {
                    Some(id) => format!("picking net at {} ({boat})", place_name(id)),
                    None => format!("picking ({boat})"),
                }
            }
            SkiffJob::Tender => format!("on the tender run ({boat})"),
            SkiffJob::Town => format!("on the town run ({boat})"),
            SkiffJob::Repair => format!("mending ({boat})"),
            SkiffJob::Idle => format!("idle on {boat}"),
        };
    }

    if crew.role == "cook" {
        "cooking".into()
    } else {
        "idle".into()
    }
}

pub struct SkiffStatus {
    pub name: String,
    pub kind: String,
    pub wrecked: bool,
    pub where_words: String,
    pub job_words: String,
    pub crew_names: Vec<String>,
    pub cargo_lb: f64,
    pub fuel: f64,
    pub condition: f64,
}

pub fn skiff_status(game: &Game, skiff: &Skiff) -> SkiffStatus {
    SkiffStatus {
        name: skiff.name.clone(),
        kind: skiff.kind.clone(),
        wrecked: skiff.wrecked,
        where_words: skiff_where(skiff),
        job_words: skiff_job_words(game, skiff),
        crew_names: crew_names_on_skiff(game, skiff),
        cargo_lb: skiff.cargo.total(),
        fuel: skiff.fuel,
        condition: skiff.condition,
    }
}

pub struct CrewGlance {
    pub name: String,
    pub role: String,
    pub is_owner: bool,
    pub status: CrewStatus,
    pub activity: String,
    pub skiff_name: Option<String>,
    pub energy: f64,
    pub hunger: f64,
    pub morale: f64,
}

pub fn crew_glance(game: &Game, crew: &CrewMember) -> CrewGlance {
    CrewGlance {
        name: crew.name.clone(),
        role: crew.role.clone(),
        is_owner: crew.is_owner,
        status: crew.status,
        activity: crew_activity(game, crew),
        skiff_name: crew_skiff(game, crew).map(|s| s.name.clone()),
        energy: crew.energy,
        hunger: crew.hunger,
        morale: crew.morale,
    }
}
