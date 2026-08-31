//! Skipper asides. One at a time. Never contradict ADF&G. Never treat tender gossip as fact.

use crate::data::camps::camp;
use crate::sim::clock::doy;
use crate::sim::engine::Game;
use crate::sim::mammals::{pinnipeds_present, transients_present};
use crate::sim::models::{CrewStatus, SkiffJob};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hint {
    pub id: String,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Urgency {
    Wait = 1,
    Soon = 2,
    Now = 3,
    Danger = 4,
}

struct Cand {
    id: &'static str,
    urgency: Urgency,
    once: bool,
    snooze: i32,
    text: String,
}

pub fn dismiss_hint(game: &mut Game) {
    game.hint = None;
    game.hint_snooze_tick = game.tick + 2;
}

pub fn set_hints(game: &mut Game, on: bool) {
    game.hints_on = on;
    if !on {
        game.hint = None;
    }
}

/// Pick at most one aside. No hint when off, or if the player already did the thing.
pub fn tick_hints(game: &mut Game) {
    if !game.hints_on {
        game.hint = None;
        return;
    }
    if game.end != crate::sim::models::GameEnd::None {
        game.hint = None;
        return;
    }
    let mut cands: Vec<Cand> = Vec::new();
    let open = game.any_open();
    let people = game
        .crew
        .iter()
        .filter(|c| c.status != CrewStatus::Quit)
        .count()
        .max(1) as f64;
    let food_days = game.food / people;
    let nets_in = game.nets.iter().any(|n| n.in_water);
    let holding = game.skiffs.iter().find(|s| s.kind == "holding");
    let picker = game.skiffs.iter().find(|s| s.kind == "picker");
    let hold_lbs = holding.map(|s| s.cargo.total()).unwrap_or(0.0);
    let pick_lbs = picker.map(|s| s.cargo.total()).unwrap_or(0.0);
    let hold_ice = holding.map(|s| s.ice).unwrap_or(0.0);
    let cargo_out = game.skiffs.iter().any(|s| s.cargo.total() > 40.0);

    if !open && nets_in {
        cands.push(Cand {
            id: "opener_dark_pull",
            urgency: Urgency::Danger,
            once: false,
            snooze: 4,
            text:
                "Closer. Pull the gear (p) or ADF&G has the card — fine and the net on the beach."
                    .into(),
        });
    }
    if game.ledger.cash < 700.0 && game.ledger.cash > -400.0 && (food_days < 5.0 || cargo_out) {
        cands.push(Cand {
            id: "cash_low",
            urgency: Urgency::Danger,
            once: false,
            snooze: 10,
            text: "Books are thin. Miss this ticket and you're talking bankruptcy.".into(),
        });
    }
    if food_days < 3.5 {
        cands.push(Cand {
            id: "food_low",
            urgency: Urgency::Danger,
            once: false,
            snooze: 8,
            text: if game.tender.present {
                "Cookshack's down to a few days. Call the tender (r) and buy food (b).".into()
            } else {
                "Food measured in days. Tender's not in the hole — you wait, or people get mean."
                    .into()
            },
        });
    }
    if game.fuel_cache < 18.0 {
        cands.push(Cand {
            id: "fuel_low",
            urgency: Urgency::Now,
            once: false,
            snooze: 8,
            text: if game.tender.present {
                "Fuel cache is low. Buy off the tender before a skiff goes dry.".into()
            } else {
                "Fuel's short. No hops until the tender hooks up.".into()
            },
        });
    }
    if game
        .crew
        .iter()
        .any(|c| c.status != CrewStatus::Quit && c.hunger > 68.0)
    {
        cands.push(Cand {
            id: "crew_hungry",
            urgency: Urgency::Now,
            once: false,
            snooze: 6,
            text: "Somebody's running on coffee. Feed them or they walk.".into(),
        });
    }
    if game
        .crew
        .iter()
        .any(|c| !c.is_owner && c.status != CrewStatus::Quit && c.morale < 22.0)
    {
        cands.push(Cand {
            id: "crew_morale",
            urgency: Urgency::Now,
            once: false,
            snooze: 8,
            text: "Morale's in the webbing. Rest (c → bunkhouse), feed, or they'll catch the next ride."
                .into(),
        });
    }
    if game.skipper_in_town && nets_in {
        cands.push(Cand {
            id: "skipper_town",
            urgency: Urgency::Now,
            once: false,
            snooze: 6,
            text: "You're in town. Nets stay dark until the permit's back on the site. Radio still works."
                .into(),
        });
    }
    if game.weather.williwaw
        && game
            .skiffs
            .iter()
            .any(|s| !s.wrecked && s.job != SkiffJob::Idle && s.job != SkiffJob::Repair)
    {
        cands.push(Cand {
            id: "williwaw",
            urgency: Urgency::Now,
            once: false,
            snooze: 4,
            text:
                "Williwaw. Stay on the beach (s → idle). Katabatic does not care about your opener."
                    .into(),
        });
    }
    if hold_lbs >= 1400.0 * 0.82 || pick_lbs >= 520.0 * 0.85 {
        cands.push(Cand {
            id: "holding_full_tender",
            urgency: Urgency::Now,
            once: false,
            snooze: 5,
            text: if game.tender.present {
                "Skiff's in the marks. Run the tender (r / t) or you stop picking and quality dies."
                    .into()
            } else {
                "Hold's filling and the tender's offshore. Ice what you have. Don't let it go soft."
                    .into()
            },
        });
    }
    if game.tender.present && cargo_out {
        let text = if game.last_treat.is_none() {
            "Hit the tender, they have garlic bread. Weigh it (t) before they pull the hook.".into()
        } else {
            "Tender's in the hole with fish on the skiff. Weigh it (t) before they pull the hook."
                .into()
        };
        cands.push(Cand {
            id: "tender_in_hole",
            urgency: Urgency::Soon,
            once: false,
            snooze: 6,
            text,
        });
    }
    if game.tender.present && !game.tender.late && game.last_treat.is_none() {
        cands.push(Cand {
            id: "tender_treat",
            urgency: Urgency::Wait,
            once: true,
            snooze: 0,
            text: "Hit the tender, they have garlic bread. Snack, not supper — cook still matters."
                .into(),
        });
    }
    if open && !nets_in && !game.skipper_in_town {
        let ice_bit = if hold_ice < 1.0 {
            " Ice the holding skiff (b)."
        } else {
            ""
        };
        cands.push(Cand {
            id: "opener_set",
            urgency: Urgency::Soon,
            once: false,
            snooze: 5,
            text: format!(
                "Opener's up. ADF&G said so on 16. Set the nets (enter), someone on the pick boat (r).{ice_bit}"
            ),
        });
    }
    if hold_ice < 0.5 && hold_lbs > 15.0 {
        cands.push(Cand {
            id: "ice_holding",
            urgency: Urgency::Soon,
            once: false,
            snooze: 8,
            text: "Holding skiff is out of ice. Tender docks warm fish and they will be right."
                .into(),
        });
    }

    let home = camp(game.camp.id).map(|c| c.home_sites).unwrap_or(&[]);
    if home.iter().any(|s| transients_present(&game.wildlife, s)) && nets_in {
        cands.push(Cand {
            id: "transients_window",
            urgency: Urgency::Soon,
            once: true,
            snooze: 0,
            text: "Transients in the bay — lions are off. Take the pick window. They do not pick your salmon."
                .into(),
        });
    }
    if game.skiffs.iter().any(|s| {
        s.job == SkiffJob::Pick
            && !matches!(s.location.as_str(), "camp" | "town" | "tender" | "transit")
    }) && home.iter().any(|s| pinnipeds_present(&game.wildlife, s))
    {
        cands.push(Cand {
            id: "seals_in_pick",
            urgency: Urgency::Wait,
            once: true,
            snooze: 0,
            text: "Lions and seals show on the bay chart only while you're picking. In the water at the web — you notice them in the pick."
                .into(),
        });
    }

    let n = doy(game.day);
    if game.odd_year() && (187..228).contains(&n) {
        cands.push(Cand {
            id: "pink_flood",
            urgency: Urgency::Wait,
            once: true,
            snooze: 0,
            text: "Pink push. Volume is the job: pick, ice, hit the tender, hang pink mesh if you want them."
                .into(),
        });
    }
    if !game.odd_year() && (187..228).contains(&n) {
        cands.push(Cand {
            id: "even_thin",
            urgency: Urgency::Wait,
            once: true,
            snooze: 0,
            text: "Even-year pinks are thin. Work the reds, don't hire a crowd, survive the lean."
                .into(),
        });
    }
    if game.mods.karluk_early_fail && game.district() == "central" && n < 187 {
        cands.push(Cand {
            id: "june_tests",
            urgency: Urgency::Wait,
            once: true,
            snooze: 0,
            text: "Karluk early looks weak. These June 33-hr tests may be all you get until July 6. Don't miss them."
                .into(),
        });
    }
    if !open && !nets_in {
        cands.push(Cand {
            id: "dark_wait",
            urgency: Urgency::Wait,
            once: false,
            snooze: 12,
            text: "Dark. Mend, rest, listen to 16. ADF&G reports once a day. x skips the wait without getting lost."
                .into(),
        });
    }

    cands.sort_by(|a, b| b.urgency.cmp(&a.urgency));
    for c in cands {
        if c.once && game.hint_seen.contains(c.id) {
            continue;
        }
        if !c.once
            && game.hint.as_ref().is_some_and(|h| h.id == c.id)
            && game.tick < game.hint_snooze_tick
        {
            return;
        }
        if let Some(cur) = &game.hint {
            if cur.id == c.id {
                return;
            }
        }
        if c.once {
            game.hint_seen.insert(c.id.to_string());
        } else {
            game.hint_seen.insert(c.id.to_string());
            game.hint_snooze_tick = game.tick + c.snooze;
        }
        game.hint = Some(Hint {
            id: c.id.into(),
            text: c.text,
        });
        return;
    }
    game.hint = None;
}
