//! Resolve events against live sim state. No "you killed a sea lion" default.

use crate::data::events::EVENT_TABLE;
use crate::data::sites::site;
use crate::data::species::{prices_for_year, SpeciesId};
use crate::sim::clock::doy;
use crate::sim::engine::Game;
use crate::sim::mammals::{
    arrive_residents, arrive_transients, pinnipeds_present, transients_present,
};
use crate::sim::models::{CrewStatus, GameEnd, Lbs, SkiffJob};

pub fn apply_sea_lion_raid(game: &mut Game, site_id: Option<&str>) -> String {
    if let Some(sid) = site_id {
        if !pinnipeds_present(&game.wildlife, sid) {
            let where_ = site(sid).map(|s| s.short).unwrap_or(sid);
            if transients_present(&game.wildlife, sid) {
                return format!(
                    "Transients in {where_}. Stellers are gone. \
The orcas are not picking your salmon — haulout is empty."
                );
            }
            return format!("Haulout empty at {where_}. Lions haven't come back yet.");
        }
    }

    let mut idxs: Vec<usize> = game
        .nets
        .iter()
        .enumerate()
        .filter(|(_, n)| n.in_water && n.fish.total() > 0.0)
        .filter(|(_, n)| site_id.map(|s| n.site_id.as_deref() == Some(s)).unwrap_or(true))
        .filter(|(_, n)| {
            n.site_id
                .as_deref()
                .is_some_and(|s| pinnipeds_present(&game.wildlife, s))
        })
        .map(|(i, _)| i)
        .collect();

    if idxs.is_empty() {
        idxs = game
            .nets
            .iter()
            .enumerate()
            .filter(|(_, n)| n.in_water)
            .filter(|(_, n)| {
                n.site_id
                    .as_deref()
                    .is_some_and(|s| pinnipeds_present(&game.wildlife, s))
            })
            .map(|(i, _)| i)
            .collect();
    }

    if idxs.is_empty() {
        return no_pinniped_raid(game, "Stellers");
    }

    let net_i = *idxs
        .iter()
        .max_by(|a, b| {
            let sa = game.nets[**a].fish.total() + game.nets[**a].sea_lion_pressure;
            let sb = game.nets[**b].fish.total() + game.nets[**b].sea_lion_pressure;
            sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap();

    let mut lost = 0.0;
    let takes: Vec<(SpeciesId, f64)> = SpeciesId::ALL
        .iter()
        .map(|&sp| {
            let amt = game.nets[net_i].fish.get(sp);
            let take = amt * game.rng.uniform(0.25, 0.55);
            (sp, take)
        })
        .collect();
    for (sp, take) in takes {
        game.nets[net_i].fish.add(sp, -take);
        lost += take;
    }
    let dmg = game.rng.uniform(8.0, 22.0);
    let net = &mut game.nets[net_i];
    net.condition = (net.condition - dmg).max(4.0);
    net.sea_lion_pressure = (net.sea_lion_pressure + 0.3).min(1.0);
    let where_ = net
        .site_id
        .as_deref()
        .and_then(site)
        .map(|s| s.short)
        .unwrap_or("the set");
    let cond = net.condition;
    game.scare_crew(6.0);
    format!(
        "Stellers on the {where_} lead. They took {lost:.0} lb and punched the web \
(net {cond:.0}%). You do not shoot them."
    )
}

pub fn apply_harbor_seal_raid(game: &mut Game, site_id: Option<&str>) -> String {
    if let Some(sid) = site_id {
        if !pinnipeds_present(&game.wildlife, sid) {
            let where_ = site(sid).map(|s| s.short).unwrap_or(sid);
            if transients_present(&game.wildlife, sid) {
                return format!("Transients in {where_}. Harbor seals left with the lions.");
            }
            return format!("Seals still off {where_}. Haulout hasn't filled.");
        }
    }

    let idxs: Vec<usize> = game
        .nets
        .iter()
        .enumerate()
        .filter(|(_, n)| n.in_water)
        .filter(|(_, n)| {
            n.site_id
                .as_deref()
                .is_some_and(|s| pinnipeds_present(&game.wildlife, s))
        })
        .filter(|(_, n)| site_id.map(|s| n.site_id.as_deref() == Some(s)).unwrap_or(true))
        .map(|(i, _)| i)
        .collect();

    if idxs.is_empty() {
        return no_pinniped_raid(game, "Harbor seals");
    }
    let net_i = *game.rng.choice(&idxs);
    let site_ref = game.nets[net_i].site_id.as_deref().and_then(site);
    let quiet = site_ref.is_some_and(|s| s.harbor_seal > 0.25);
    let hi = if quiet { 0.28 } else { 0.16 };
    let frac = game.rng.uniform(0.10, hi);
    let mut lost = 0.0;
    let takes: Vec<(SpeciesId, f64)> = SpeciesId::ALL
        .iter()
        .map(|&sp| (sp, game.nets[net_i].fish.get(sp) * frac))
        .collect();
    for (sp, take) in takes {
        game.nets[net_i].fish.add(sp, -take);
        lost += take;
    }
    let dmg = game.rng.uniform(2.0, 8.0);
    game.nets[net_i].condition = (game.nets[net_i].condition - dmg).max(8.0);
    let where_ = site_ref.map(|s| s.short).unwrap_or("a set");
    format!("Harbor seals working {where_}. Smaller mouths, same hole. {lost:.0} lb gone.")
}

pub fn maybe_fire(game: &mut Game) -> Option<String> {
    if game.end != GameEnd::None {
        return None;
    }
    if game.rng.random() > 0.16 {
        return None;
    }
    let n = doy(game.day);
    let mut weights = Vec::new();
    for ev in EVENT_TABLE {
        let mut w = ev.weight;
        if ev.id == "september_winter" && n < 244 {
            w *= 0.05;
        }
        if ev.id == "whale_pass_rip" && game.camp.id != "bailey" {
            w *= 0.15;
        }
        if ev.id == "advection_fog" && !(152..=212).contains(&n) {
            w *= 0.4;
        }
        if ev.id == "price_cut" && !game.mods.pink_flood {
            w *= 0.45;
        }
        weights.push((ev.id.to_string(), w));
    }
    let eid = game.rng.weighted(&weights);
    Some(resolve(game, &eid))
}

pub fn resolve(game: &mut Game, eid: &str) -> String {
    match eid {
        "williwaw" => williwaw(game),
        "advection_fog" => fog(game),
        "shelikof_seas" => seas(game),
        "whale_pass_rip" => rip(game),
        "september_winter" => sept(game),
        "sea_lion_raid" => apply_sea_lion_raid(game, None),
        "harbor_seal_raid" => apply_harbor_seal_raid(game, None),
        "otter_foul" => otter(game),
        "orca_residents" => arrive_residents(game, None),
        "orca_transients" => arrive_transients(game, None),
        "porpoise_bycatch" => porpoise(game),
        "humpback_wrap" => humpback(game),
        "outboard_down" => outboard(game),
        "net_tear" => tear(game),
        "skiff_swamped" => swamp(game),
        "ice_failure" => ice(game),
        "tender_late" => tender_late(game),
        "price_cut" => price(game),
        "cannery_dark" => cannery(game),
        "adfg_extension" => extension(game),
        "adfg_pulse" => pulse(game),
        "food_spoil" => spoil(game),
        "crew_fight" => fight(game),
        "bonus_school" => school(game),
        _ => "VHF static.".into(),
    }
}

fn williwaw(game: &mut Game) -> String {
    game.weather.williwaw = true;
    game.weather.wind_kt = game.weather.wind_kt.max(game.rng.uniform(48.0, 70.0));
    game.weather.seas_ft = game.weather.seas_ft.max(6.0);
    game.weather.label = format!("WILLIWAW {} kt", game.weather.wind_kt as i32);
    game.weather.precip = "williwaw".into();
    game.scare_crew(10.0);
    let mut wreck_name = None;
    for s in &mut game.skiffs {
        if s.location != "camp" && s.location != "town" && !s.wrecked {
            s.condition -= game.rng.uniform(4.0, 15.0);
            if s.condition < 12.0 && game.rng.random() < 0.12 {
                s.wrecked = true;
                wreck_name = Some(s.name.clone());
                break;
            }
        }
    }
    if let Some(name) = wreck_name {
        format!("Williwaw off the mountain. {name} swamped and she's done.")
    } else {
        "Williwaw. Katabatic, no warning. Skiffs stay on the running line if you like them.".into()
    }
}

fn fog(game: &mut Game) -> String {
    game.weather.fog = game.weather.fog.max(0.75);
    game.weather.precip = "fog".into();
    game.weather.label = "advection fog, thick".into();
    "June/July advection fog. You pick by the corks you can see.".into()
}

fn seas(game: &mut Game) -> String {
    game.weather.seas_ft = game.weather.seas_ft.max(game.rng.uniform(5.0, 8.5));
    game.weather.wind_kt = game.weather.wind_kt.max(28.0);
    game.weather.label = format!(
        "Shelikof {:.0} ft, short and steep",
        game.weather.seas_ft
    );
    "Shelikof stacking up. Short, steep, mean. Holding skiff takes water.".into()
}

fn rip(game: &mut Game) -> String {
    for s in &mut game.skiffs {
        if s.location == "town" || s.location == "transit" || s.job == SkiffJob::Town {
            s.eta += 1;
            s.condition -= 6.0;
            return format!("Whale Passage 6 knots against you. {} loses a tide.", s.name);
        }
    }
    "Whale Passage ripping. Stay off it unless you're going to town.".into()
}

fn sept(game: &mut Game) -> String {
    game.weather.wind_kt += 12.0;
    game.weather.seas_ft += 2.0;
    game.weather.label = "September — winter arriving".into();
    game.scare_crew(4.0);
    "September weather. The gulf is done being polite. Camps start talking about pulling.".into()
}

fn otter(game: &mut Game) -> String {
    let idxs: Vec<usize> = game
        .nets
        .iter()
        .enumerate()
        .filter(|(_, n)| n.in_water)
        .map(|(i, _)| i)
        .collect();
    if idxs.is_empty() {
        return "Otters in the kelp. They are not here for your salmon.".into();
    }
    let i = *game.rng.choice(&idxs);
    let dmg = game.rng.uniform(6.0, 16.0);
    let net = &mut game.nets[i];
    net.condition = (net.condition - dmg).max(10.0);
    net.soak_tides = (net.soak_tides - 1).max(0);
    format!(
        "Sea otter in the lead — entanglement / kelp raft, not a raid. \
They eat invertebrates. You lost soak time. Net {:.0}%.",
        net.condition
    )
}

fn no_pinniped_raid(game: &Game, who: &str) -> String {
    let transient_bays: Vec<&str> = game
        .wildlife
        .values()
        .filter(|b| b.transients_here())
        .map(|b| b.label.as_str())
        .collect();
    if !transient_bays.is_empty() {
        return format!(
            "{who} chased off — transients in {}. Empty haulout. They did not touch the fish.",
            transient_bays.join(", ")
        );
    }
    let empty: Vec<&str> = game
        .wildlife
        .values()
        .filter(|b| !b.pinnipeds_present())
        .map(|b| b.label.as_str())
        .collect();
    if !empty.is_empty() {
        return format!("{who} still gone from {}. Haulout hasn't filled.", empty.join(", "));
    }
    format!("{who} on the rocks. Nothing in the water for them.")
}

fn porpoise(game: &mut Game) -> String {
    game.scare_crew(14.0);
    for c in &mut game.crew {
        if !c.is_owner {
            c.morale = (c.morale - 12.0).max(5.0);
        }
    }
    "Harbor porpoise in the web. Rare and grim — this is the MMPA Category II driver, \
not a raid. You clear it. Nobody jokes."
        .into()
}

fn humpback(game: &mut Game) -> String {
    let idxs: Vec<usize> = game
        .nets
        .iter()
        .enumerate()
        .filter(|(_, n)| n.in_water)
        .map(|(i, _)| i)
        .collect();
    if idxs.is_empty() {
        return "Humpbacks working bait offshore. Keep the skiff's eyes up.".into();
    }
    let i = *game.rng.choice(&idxs);
    let dmg = game.rng.uniform(15.0, 35.0);
    let net = &mut game.nets[i];
    net.condition = (net.condition - dmg).max(5.0);
    net.in_water = false;
    let id = net.id.clone();
    net.site_id = None;
    format!(
        "Humpback rolled the {id} set. Gear wrap. Net's on the beach if you still have one. \
ADF&G will want a call if this gets worse."
    )
}

fn outboard(game: &mut Game) -> String {
    let idxs: Vec<usize> = game
        .skiffs
        .iter()
        .enumerate()
        .filter(|(_, s)| !s.wrecked && s.condition > 10.0)
        .map(|(i, _)| i)
        .collect();
    if idxs.is_empty() {
        return "Nothing left that would start anyway.".into();
    }
    let i = *game.rng.choice(&idxs);
    let s = &mut game.skiffs[i];
    s.condition = s.condition.min(18.0);
    s.job = SkiffJob::Repair;
    s.location = "camp".into();
    s.dest = None;
    format!(
        "{}: lower unit. She's on the beach until you throw a spare prop and a day at it.",
        s.name
    )
}

fn tear(game: &mut Game) -> String {
    let idxs: Vec<usize> = game
        .nets
        .iter()
        .enumerate()
        .filter(|(_, n)| n.in_water)
        .map(|(i, _)| i)
        .collect();
    if idxs.is_empty() {
        return "Loft's quiet. Nothing to tear.".into();
    }
    let i = *game.rng.choice(&idxs);
    let dmg = game.rng.uniform(18.0, 40.0);
    let net = &mut game.nets[i];
    net.condition = (net.condition - dmg).max(8.0);
    format!(
        "Torn web on {}. Needles and twine. Condition {:.0}%.",
        net.id, net.condition
    )
}

fn swamp(game: &mut Game) -> String {
    let idxs: Vec<usize> = game
        .skiffs
        .iter()
        .enumerate()
        .filter(|(_, s)| !s.wrecked && s.location != "camp" && s.location != "town")
        .map(|(i, _)| i)
        .collect();
    if idxs.is_empty() {
        return "Skiffs are on the beach. The tide still wants them.".into();
    }
    let i = *game.rng.choice(&idxs);
    let lost = game.skiffs[i].cargo.total();
    game.skiffs[i].cargo = Lbs::default();
    game.skiffs[i].condition -= game.rng.uniform(20.0, 40.0);
    game.skiffs[i].ice = 0.0;
    let live_else = game
        .skiffs
        .iter()
        .enumerate()
        .any(|(j, s)| j != i && !s.wrecked);
    let name = game.skiffs[i].name.clone();
    if game.skiffs[i].condition < 12.0 && live_else {
        game.skiffs[i].wrecked = true;
        return format!("{name} swamped and wrecked. {lost:.0} lb gone with her.");
    }
    if game.skiffs[i].condition < 12.0 {
        game.skiffs[i].condition = 18.0;
        game.skiffs[i].job = SkiffJob::Repair;
        game.skiffs[i].location = "camp".into();
        return format!("{name} swamped. Last skiff — you drag her home and start on the lower unit.");
    }
    game.skiffs[i].location = "camp".into();
    game.scare_crew(8.0);
    format!("{name} swamped. You got her home. {lost:.0} lb to the crabs.")
}

fn ice(game: &mut Game) -> String {
    for s in &mut game.skiffs {
        s.ice = 0.0;
        s.cargo_quality = s.cargo_quality.min(0.62);
    }
    for n in &mut game.nets {
        n.quality = n.quality.min(0.70);
    }
    "Ice / RSW down. Fish will go soft. Tender will dock you and they will be right.".into()
}

fn tender_late(game: &mut Game) -> String {
    game.tender.eta_tides += game.rng.randint(2, 5);
    game.tender.late = true;
    game.tender.present = false;
    format!(
        "{} is late. Holding skiff better have ice. Miss them and it rots.",
        game.tender.name
    )
}

fn price(game: &mut Game) -> String {
    let floor = prices_for_year(game.year).pink * 0.50;
    if game.tender.prices.pink <= floor + 0.01 {
        return "Town already cut the pink. Nothing left to give.".into();
    }
    let cut = game.rng.uniform(0.10, 0.18);
    game.tender.prices.pink = (game.tender.prices.pink * (1.0 - cut)).max(floor);
    if game.mods.pink_flood {
        let chum_floor = prices_for_year(game.year).chum * 0.50;
        game.tender.prices.chum = (game.tender.prices.chum * 0.94).max(chum_floor);
    }
    format!(
        "Town cut the pink. Opening estimate is dead. New pink ${:.2}/lb. Flood years do this.",
        game.tender.prices.pink
    )
}

fn cannery(game: &mut Game) -> String {
    game.tender.eta_tides += game.rng.randint(3, 6);
    game.tender.present = false;
    game.tender.prices.scale_all(0.94);
    "Cannery dark. Tenders farther, less often. You will feel it in the tote.".into()
}

fn extension(game: &mut Game) -> String {
    if game.open_reason().starts_with("closed") {
        return "VHF: no extension. Still dark.".into();
    }
    game.extend_open_tides = game.extend_open_tides.max(2);
    "ADF&G radio: period extended. Last twenty-four hours they changed their mind. Stay in it."
        .into()
}

fn pulse(game: &mut Game) -> String {
    game.bonus_school = 3;
    "ADF&G radio: extra pulse on the traditional water. Weir counts finally moved.".into()
}

fn spoil(game: &mut Game) -> String {
    let lost = game.food.min(f64::from(game.rng.randint(3, 8)));
    game.food -= lost;
    format!("Meat went off in the cache. Lost {lost:.0} person-days of food. Cook is not surprised.")
}

fn fight(game: &mut Game) -> String {
    let hired: Vec<usize> = game
        .crew
        .iter()
        .enumerate()
        .filter(|(_, c)| !c.is_owner && c.status != CrewStatus::Quit)
        .map(|(i, _)| i)
        .collect();
    if hired.len() < 2 {
        return "Bunkhouse is quiet. Not enough people left to have a fight.".into();
    }
    let (a, b) = if hired.len() == 2 {
        (hired[0], hired[1])
    } else {
        let a = *game.rng.choice(&hired);
        let rest: Vec<usize> = hired.iter().copied().filter(|&i| i != a).collect();
        let b = *game.rng.choice(&rest);
        (a, b)
    };
    game.crew[a].morale = (game.crew[a].morale - 18.0).max(5.0);
    game.crew[b].morale = (game.crew[b].morale - 18.0).max(5.0);
    format!(
        "{} and {} in the bunkhouse. Morale takes the webbing.",
        game.crew[a].name, game.crew[b].name
    )
}

fn school(game: &mut Game) -> String {
    game.bonus_school = game.bonus_school.max(2);
    "Fish showing on the lead. Thick. Get a skiff on it before the lions do.".into()
}
