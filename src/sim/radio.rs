//! VHF traffic. Skipper orders, tender gossip, and ADF&G dry reports.
//!
//! Rumors are tagged true / false / half-true for tests. The spoken line is
//! always chatter — never a fact stamp.

use crate::data::camps::camp;
use crate::data::sites::site;
use crate::data::species::{short_code, SpeciesId};
use crate::sim::clock::GameDate;
use crate::sim::engine::Game;
use crate::sim::mammals::pinnipeds_present;
use crate::sim::models::Skiff;
use crate::sim::openings::current_window;
use crate::sim::runs::site_availability;
use crate::sim::status::place_name;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadioVoice {
    Skipper,
    Crew,
    Tender,
    Adfg,
}

impl RadioVoice {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Skipper => "CAMP",
            Self::Crew => "SKIFF",
            Self::Tender => "TENDER",
            Self::Adfg => "ADF&G",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadioKind {
    Order,
    Reply,
    TenderQuote,
    Rumor,
    Daily,
    Opener,
    Closer,
    Listen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RumorTruth {
    True,
    False,
    Half,
}

#[derive(Clone, Debug)]
pub struct RadioLine {
    pub tick: i32,
    pub voice: RadioVoice,
    pub kind: RadioKind,
    pub channel: &'static str,
    pub text: String,
    pub rumor_truth: Option<RumorTruth>,
}

#[derive(Clone, Debug)]
pub struct Rumor {
    pub chatter: String,
    pub truth: RumorTruth,
}

pub fn tender_on_the_grounds(game: &Game) -> bool {
    game.tender.present && !game.tender.late
}

pub fn tender_price_board(game: &Game) -> String {
    let p = &game.tender.prices;
    SpeciesId::ALL
        .iter()
        .map(|&sp| format!("{} ${:.2}", short_code(sp), p.get(sp)))
        .collect::<Vec<_>>()
        .join("  ")
}

pub fn radio_skiff_handle(_game: &Game, skiff: &Skiff) -> String {
    if let Some(sid) = skiff.job_site.as_deref().or_else(|| {
        let loc = skiff.location.as_str();
        if matches!(loc, "camp" | "town" | "tender" | "transit") {
            None
        } else {
            Some(loc)
        }
    }) {
        if !matches!(sid, "camp" | "town" | "tender" | "transit") {
            return place_name(sid);
        }
    }
    if skiff.kind == "holding" {
        "Holding".into()
    } else {
        "Skiff".into()
    }
}

pub fn first_name(name: &str) -> &str {
    name.split_whitespace().next().unwrap_or(name)
}

pub fn official_daily_text(game: &Game) -> String {
    let district = if game.district() == "alitak" {
        "Alitak"
    } else {
        "Central Section"
    };
    let pink = if game.odd_year() {
        "Odd-year pink line — flood on the books."
    } else {
        "Even-year pink line — thin."
    };
    let weir = weir_color(game);
    let plan = game.phase_label();
    let window = window_recap(game);
    format!("ADF&G {district}. {pink} {weir} Plan: {plan}. {window}")
}

pub fn official_opener_text(game: &Game) -> String {
    let district = if game.district() == "alitak" {
        "Alitak"
    } else {
        "Central Section"
    };
    let reason = game.open_reason();
    let until = current_window(&game.openings, game.day, game.district())
        .map(|w| format!(" Open through {}.", fmt_doy(game.year, w.end_doy)))
        .unwrap_or_default();
    if game.extend_open_tides > 0 {
        format!("ADF&G {district} OPEN. {reason}.{until} Extension on the air.")
    } else {
        format!("ADF&G {district} OPEN. {reason}.{until}")
    }
}

pub fn official_closer_text(game: &Game) -> String {
    let district = if game.district() == "alitak" {
        "Alitak"
    } else {
        "Central Section"
    };
    let reason = game.open_reason();
    let next = next_window(game)
        .map(|w| format!(" Dark until {}.", fmt_doy(game.year, w.start_doy)))
        .unwrap_or_else(|| " No emergency order on the board.".into());
    let alitak_dark = if game.district() == "alitak" {
        " 63-hr dark clock unless the weirs move."
    } else {
        ""
    };
    format!("ADF&G {district} DARK. {reason}.{next}{alitak_dark} Gear off the water.")
}

fn weir_color(game: &Game) -> String {
    if game.district() == "alitak" {
        if game.mods.frazer_goals_ok {
            "Frazer / Upper Station: weir color good.".into()
        } else if game.mods.frazer < 0.55 {
            "Frazer weir: weak. Watch the pulse clock.".into()
        } else {
            "Frazer / Upper Station: fair, not made.".into()
        }
    } else if game.mods.karluk_early_fail {
        "Karluk early weir: weak / concern.".into()
    } else if game.mods.karluk_early >= 0.95 {
        "Karluk early weir: strong.".into()
    } else {
        "Karluk early weir: fair.".into()
    }
}

fn window_recap(game: &Game) -> String {
    if let Some(w) = current_window(&game.openings, game.day, game.district()) {
        format!(
            "Open until {} ({}-hr).",
            fmt_doy(game.year, w.end_doy),
            w.hours
        )
    } else if let Some(w) = next_window(game) {
        format!("Dark until {}.", fmt_doy(game.year, w.start_doy))
    } else {
        "Dark. No further period on the board.".into()
    }
}

fn next_window(game: &Game) -> Option<&crate::sim::models::OpeningWindow> {
    let n = game.day.doy();
    let district = game.district();
    game.openings
        .iter()
        .filter(|w| w.district == district && !w.pulse_sites_only && w.start_doy > n)
        .min_by_key(|w| w.start_doy)
}

fn fmt_doy(year: i32, n: i32) -> String {
    GameDate::new(year, 1, 1).add_days(n - 1).fmt_short()
}

/// Tender gossip from live estimates. Some days they are full of it.
pub fn spin_tender_rumor(game: &mut Game) -> Rumor {
    let mut truths: Vec<String> = Vec::new();
    let mut lies: Vec<String> = Vec::new();
    let mut halves: Vec<(String, RumorTruth)> = Vec::new();

    if game.bonus_school > 0 {
        truths.push("they're seeing a school on the cape".into());
    } else {
        lies.push("heard they're thick on the cape".into());
    }

    let home = camp(game.camp.id).map(|c| c.home_sites).unwrap_or(&[]);
    for &sid in home {
        let name = place_name(sid);
        let lions = pinnipeds_present(&game.wildlife, sid);
        let sdef = site(sid);
        let thick = sdef.is_some_and(|s| s.sea_lion >= 0.40)
            || game
                .nets
                .iter()
                .any(|n| n.site_id.as_deref() == Some(sid) && n.sea_lion_pressure > 0.2);
        if lions && thick {
            truths.push(format!("lions thick at {name}"));
        } else if !lions {
            truths.push(format!("lions gone off {name}"));
            lies.push(format!("lions thick at {name}"));
        } else {
            lies.push(format!("lions thick at {name}"));
        }

        let avail = site_availability(sid, game.day, &game.mods);
        let pinks = avail
            .iter()
            .find(|(sp, _)| *sp == SpeciesId::Pink)
            .map(|(_, lbs)| *lbs)
            .unwrap_or(0.0);
        if pinks > 80.0 && game.mods.pink_flood {
            truths.push(format!("pinks hitting {name}"));
        } else if pinks > 40.0 {
            halves.push((format!("pinks maybe starting at {name}"), RumorTruth::Half));
            lies.push(format!("pinks hitting {name}"));
        } else {
            lies.push(format!("pinks hitting {name}"));
        }
    }

    if game.mods.pink_flood && !game.midseason_cut && game.day.month >= 7 {
        truths.push("town's talking a pink cut".into());
        lies.push("board's holding, they say".into());
    } else if game.midseason_cut {
        truths.push("pink's already been cut — you saw the board".into());
    } else {
        lies.push("price cut coming, they say".into());
    }

    let full_of_it = game.rng.random() < 0.38;
    if full_of_it && !lies.is_empty() {
        let chatter = game.rng.choice(&lies).clone();
        Rumor {
            chatter,
            truth: RumorTruth::False,
        }
    } else if !full_of_it && game.rng.random() < 0.22 && !halves.is_empty() {
        let (chatter, truth) = game.rng.choice(&halves).clone();
        Rumor { chatter, truth }
    } else if !truths.is_empty() {
        let chatter = game.rng.choice(&truths).clone();
        Rumor {
            chatter,
            truth: RumorTruth::True,
        }
    } else if !lies.is_empty() {
        let chatter = game.rng.choice(&lies).clone();
        Rumor {
            chatter,
            truth: RumorTruth::False,
        }
    } else {
        Rumor {
            chatter: "quiet on the grounds, they say".into(),
            truth: RumorTruth::Half,
        }
    }
}

pub fn order_copy(handle: &str, job: &str) -> String {
    match job {
        "pick" => format!("{handle} pick, copy"),
        "tender" => format!("{handle} copy, running fish"),
        "idle" => format!("{handle} copy, coming home"),
        "repair" => format!("{handle} copy, on the beach"),
        "town" => format!("{handle} copy, town run"),
        _ => format!("{handle} copy"),
    }
}

pub fn order_tx(handle: &str, job: &str) -> String {
    match job {
        "pick" => format!("{handle}, come pick the web"),
        "tender" => format!("{handle}, run the tender"),
        "idle" => format!("{handle}, come home"),
        "repair" => format!("{handle}, stay on the beach, mend"),
        "town" => format!("{handle}, town run"),
        _ => format!("{handle}, {job}"),
    }
}
