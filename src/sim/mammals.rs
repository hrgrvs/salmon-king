//! Site-level killer whale / pinniped chain.
//! Transients empty the haulout; residents scatter fish. Neither raids the web.

use std::collections::{BTreeMap, HashSet};

use crate::data::sites::{bay_id_for_site, site, sites_for_camp, sites_in_bay, BAYS};
use crate::sim::engine::Game;

pub const TRANSIENT_STAY: (i32, i32) = (4, 7);
pub const PINNIPED_RETURN_DELAY: (i32, i32) = (3, 5);
pub const RESIDENT_STAY: (i32, i32) = (3, 5);
pub const RESIDENT_SCATTER: f64 = 0.48;
pub const PRESSURE_FLOOR: f64 = 0.04;

#[derive(Clone, Debug)]
pub struct BayWildlife {
    pub bay_id: String,
    pub label: String,
    pub transient_tides: i32,
    pub resident_tides: i32,
    pub pinniped_absent_tides: i32,
}

impl BayWildlife {
    pub fn transients_here(&self) -> bool {
        self.transient_tides > 0
    }

    pub fn residents_here(&self) -> bool {
        self.resident_tides > 0
    }

    pub fn pinnipeds_present(&self) -> bool {
        self.pinniped_absent_tides <= 0
    }
}

pub type Wildlife = BTreeMap<String, BayWildlife>;

pub fn new_wildlife() -> Wildlife {
    BAYS.iter()
        .map(|b| {
            (
                b.id.to_string(),
                BayWildlife {
                    bay_id: b.id.into(),
                    label: b.label.into(),
                    transient_tides: 0,
                    resident_tides: 0,
                    pinniped_absent_tides: 0,
                },
            )
        })
        .collect()
}

pub fn bay_of<'a>(site_id: &str, wildlife: &'a Wildlife) -> Option<&'a BayWildlife> {
    bay_id_for_site(site_id).and_then(|bid| wildlife.get(bid))
}

pub fn bay_of_mut<'a>(site_id: &str, wildlife: &'a mut Wildlife) -> Option<&'a mut BayWildlife> {
    bay_id_for_site(site_id).and_then(|bid| wildlife.get_mut(bid))
}

pub fn pinnipeds_present(wildlife: &Wildlife, site_id: &str) -> bool {
    bay_of(site_id, wildlife)
        .map(|b| b.pinnipeds_present())
        .unwrap_or(true)
}

pub fn transients_present(wildlife: &Wildlife, site_id: &str) -> bool {
    bay_of(site_id, wildlife)
        .map(|b| b.transients_here())
        .unwrap_or(false)
}

pub fn resident_scatter(wildlife: &Wildlife, site_id: &str) -> f64 {
    if bay_of(site_id, wildlife).is_some_and(|b| b.residents_here()) {
        RESIDENT_SCATTER
    } else {
        1.0
    }
}

pub fn transient_sites(wildlife: &Wildlife) -> HashSet<String> {
    let mut out = HashSet::new();
    for bay in wildlife.values() {
        if bay.transients_here() {
            for s in sites_in_bay(&bay.bay_id) {
                out.insert((*s).to_string());
            }
        }
    }
    out
}

pub fn resident_sites(wildlife: &Wildlife) -> HashSet<String> {
    let mut out = HashSet::new();
    for bay in wildlife.values() {
        if bay.residents_here() {
            for s in sites_in_bay(&bay.bay_id) {
                out.insert((*s).to_string());
            }
        }
    }
    out
}

pub fn empty_haulout_sites(wildlife: &Wildlife) -> HashSet<String> {
    let mut out = HashSet::new();
    for bay in wildlife.values() {
        if !bay.pinnipeds_present() {
            for s in sites_in_bay(&bay.bay_id) {
                out.insert((*s).to_string());
            }
        }
    }
    out
}

pub fn status_lines(wildlife: &Wildlife, camp_id: &str) -> Vec<String> {
    let playable: HashSet<&str> = sites_for_camp(camp_id).into_iter().map(|s| s.id).collect();
    let mut lines = Vec::new();
    for bay in wildlife.values() {
        let members = sites_in_bay(&bay.bay_id);
        if !members.iter().any(|s| playable.contains(s)) {
            continue;
        }
        if bay.transients_here() {
            lines.push(format!(
                "ω transients in {} — lions gone ({} tides). They do not pick the web.",
                bay.label, bay.transient_tides
            ));
        } else if bay.pinniped_absent_tides > 0 {
            lines.push(format!(
                "haulout empty at {} — transients moved on, lions not back ({} tides)",
                bay.label, bay.pinniped_absent_tides
            ));
        }
        if bay.residents_here() {
            lines.push(format!(
                "residents in {} — fish acting weird, catch off ({} tides). Lions still on the rocks.",
                bay.label, bay.resident_tides
            ));
        }
    }
    lines
}

pub fn tick_wildlife(wildlife: &mut Wildlife) -> Vec<String> {
    let mut logs = Vec::new();
    for bay in wildlife.values_mut() {
        if bay.transient_tides > 0 {
            bay.transient_tides -= 1;
            if bay.transient_tides == 0 {
                logs.push(format!(
                    "Transients left {}. Haulout still empty — Stellers and seals not back for {} tides.",
                    bay.label, bay.pinniped_absent_tides
                ));
            }
        }
        if bay.resident_tides > 0 {
            bay.resident_tides -= 1;
            if bay.resident_tides == 0 {
                logs.push(format!("Residents left {}. Fish settling.", bay.label));
            }
        }
        if bay.pinniped_absent_tides > 0 {
            bay.pinniped_absent_tides -= 1;
            if bay.pinniped_absent_tides == 0 {
                logs.push(format!(
                    "Stellers back on the rocks at {}. Seals too.",
                    bay.label
                ));
            }
        }
    }
    logs
}

/// Transient killer whales: hunt pinnipeds. Empty haulout. Do not take salmon from gear.
pub fn arrive_transients(game: &mut Game, site_id: Option<&str>) -> String {
    let site_id = site_id
        .map(|s| s.to_string())
        .unwrap_or_else(|| pick_site(game, true));
    let Some(bid) = bay_id_for_site(&site_id) else {
        return "Transient orcas offshore. Not on your beaches.".into();
    };
    let stay = game.rng.randint(TRANSIENT_STAY.0, TRANSIENT_STAY.1);
    let delay = game.rng.randint(PINNIPED_RETURN_DELAY.0, PINNIPED_RETURN_DELAY.1);
    let label;
    {
        let bay = game.wildlife.get_mut(bid).unwrap();
        bay.transient_tides = stay;
        bay.pinniped_absent_tides = stay + delay;
        label = bay.label.clone();
    }
    let members = sites_in_bay(bid);
    for net in &mut game.nets {
        if net.site_id.as_deref().is_some_and(|s| members.contains(&s)) {
            net.sea_lion_pressure = PRESSURE_FLOOR;
        }
    }
    let names: Vec<&str> = members
        .iter()
        .filter_map(|s| site(s).map(|sd| sd.short))
        .collect();
    format!(
        "Transients in {label} ({}). Lions gone. Seals gone. \
They do not pick salmon out of the gear — empty haulout is the gift. \
{stay} tides here, then a few more before the rocks fill again.",
        names.join(" / ")
    )
}

/// Resident killer whales: fish-eaters. Scatter salmon. Do not clear lions. Do not raid nets.
pub fn arrive_residents(game: &mut Game, site_id: Option<&str>) -> String {
    let site_id = site_id
        .map(|s| s.to_string())
        .unwrap_or_else(|| pick_site(game, true));
    let Some(bid) = bay_id_for_site(&site_id) else {
        return "Residents passing offshore. Fish still on your beach.".into();
    };
    let stay = game.rng.randint(RESIDENT_STAY.0, RESIDENT_STAY.1);
    let label;
    {
        let bay = game.wildlife.get_mut(bid).unwrap();
        bay.resident_tides = stay;
        label = bay.label.clone();
    }
    let members = sites_in_bay(bid);
    let names: Vec<&str> = members
        .iter()
        .filter_map(|s| site(s).map(|sd| sd.short))
        .collect();
    format!(
        "Residents in {label} ({}). Fish acting weird — diving, sliding wide. \
Catch is off. They are not raiding the nets, and the Stellers are still here.",
        names.join(" / ")
    )
}

fn pick_site(game: &mut Game, prefer_nets: bool) -> String {
    let deployed: Vec<String> = game
        .nets
        .iter()
        .filter(|n| n.in_water)
        .filter_map(|n| n.site_id.clone())
        .collect();
    if prefer_nets && !deployed.is_empty() {
        return game.rng.choice(&deployed).clone();
    }
    let playable: Vec<String> = game.playable_sites().into_iter().map(|s| s.id.to_string()).collect();
    if playable.is_empty() {
        "uganik_pass".into()
    } else {
        game.rng.choice(&playable).clone()
    }
}
