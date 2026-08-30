//! Catch and quality. Setnets fish without crew; picking recovers what's in the web.

use crate::data::species::SpeciesId;
use crate::sim::models::{CrewMember, CrewStatus, Net, Weather};
use crate::sim::rng::Rng;

/// Game selectivity. Mesh is a player knob — Kodiak regs do not specify gillnet mesh.
fn mesh_sel(mesh: &str, sp: SpeciesId) -> f64 {
    match (mesh, sp) {
        ("pink", SpeciesId::King) => 0.25,
        ("pink", SpeciesId::Red) => 0.55,
        ("pink", SpeciesId::Pink) => 1.15,
        ("pink", SpeciesId::Chum) => 0.50,
        ("pink", SpeciesId::Silver) => 0.45,
        ("red", SpeciesId::King) => 0.70,
        ("red", SpeciesId::Red) => 1.15,
        ("red", SpeciesId::Pink) => 0.40,
        ("red", SpeciesId::Chum) => 1.05,
        ("red", SpeciesId::Silver) => 1.00,
        (_, SpeciesId::King) => 0.55,
        (_, SpeciesId::Red) => 1.00,
        (_, SpeciesId::Pink) => 0.85,
        (_, SpeciesId::Chum) => 0.90,
        (_, SpeciesId::Silver) => 0.85,
    }
}

pub const NET_CAP_LBS: f64 = 520.0;

pub fn soak_penalty(soak_tides: i32) -> f64 {
    match soak_tides {
        n if n <= 1 => 1.0,
        2 => 0.88,
        3 => 0.68,
        _ => 0.48,
    }
}

pub fn quality_decay(net: &Net, weather: &Weather, daytime: bool) -> f64 {
    let mut q = net.quality;
    if net.soak_tides >= 2 {
        q -= 0.08;
    }
    if net.soak_tides >= 4 {
        q -= 0.12;
    }
    if daytime && weather.fog < 0.25 && net.soak_tides >= 3 {
        q -= 0.10;
    }
    if net.fish.total() > NET_CAP_LBS * 0.75 {
        q -= 0.06;
    }
    q.max(0.35)
}

pub fn weather_catch_mod(weather: &Weather) -> f64 {
    let mut mod_ = 1.0;
    if weather.seas_ft >= 6.0 {
        mod_ *= 0.45;
    } else if weather.seas_ft >= 4.0 {
        mod_ *= 0.72;
    }
    if weather.fog >= 0.6 {
        mod_ *= 0.90;
    }
    if weather.williwaw {
        mod_ *= 0.40;
    }
    mod_
}

pub fn crew_efficiency(members: &[CrewMember]) -> f64 {
    if members.is_empty() {
        return 0.35;
    }
    let mut parts = Vec::new();
    for c in members {
        if c.status != CrewStatus::Working {
            continue;
        }
        let skill = 0.45 + 0.07 * f64::from(c.skill);
        let energy = 0.55 + 0.45 * (c.energy / 100.0);
        let hunger = if c.hunger < 55.0 {
            1.0
        } else {
            f64::max(1.0 - (c.hunger - 55.0) / 80.0, 0.45)
        };
        let morale = 0.60 + 0.40 * (c.morale / 100.0);
        parts.push(skill * energy * hunger * morale);
    }
    if parts.is_empty() {
        return 0.30;
    }
    let mean = parts.iter().sum::<f64>() / parts.len() as f64;
    (mean * (1.0 + 0.08 * (parts.len() as f64 - 1.0))).min(1.35)
}

pub fn soak_net(
    net: &mut Net,
    available: &[(SpeciesId, f64)],
    weather: &Weather,
    mammal: f64,
    orca_scatter: f64,
    rng: &mut Rng,
    fathom_scale: f64,
    accumulate_pressure: bool,
) {
    if !net.in_water || net.condition <= 8.0 {
        return;
    }
    let room = (NET_CAP_LBS - net.fish.total()).max(0.0);
    if room <= 1.0 {
        net.quality = (net.quality - 0.05).max(0.35);
        return;
    }
    let soak = soak_penalty(net.soak_tides);
    let cond = (net.condition / 100.0).max(0.15);
    let wx = weather_catch_mod(weather);
    let mut gained = 0.0;
    for &(sp, base) in available {
        if base <= 0.0 {
            continue;
        }
        let raw = base
            * mesh_sel(&net.mesh, sp)
            * soak
            * cond
            * wx
            * mammal
            * orca_scatter
            * fathom_scale
            * rng.uniform(0.80, 1.20);
        let take = raw.min(room - gained);
        if take > 0.05 {
            net.fish.add(sp, take);
            gained += take;
        }
    }
    net.soak_tides += 1;
    net.quality = quality_decay(net, weather, true);
    if accumulate_pressure && net.fish.total() > NET_CAP_LBS * 0.55 {
        net.sea_lion_pressure = (net.sea_lion_pressure + 0.12).min(1.0);
    }
}

/// Return recovered lbs by species and quality. Leftovers stay in the web.
pub fn pick_net(net: &mut Net, crew: &[CrewMember], rng: &mut Rng) -> ([(SpeciesId, f64); 5], f64) {
    let eff = crew_efficiency(crew);
    let mut recovered = [(SpeciesId::King, 0.0); 5];
    let q = net.quality * (0.85 + 0.15 * eff);
    for (i, &sp) in SpeciesId::ALL.iter().enumerate() {
        let amt = net.fish.get(sp);
        if amt <= 0.0 {
            recovered[i] = (sp, 0.0);
            continue;
        }
        let frac = (0.72 + 0.22 * eff).min(0.98);
        let take = amt * frac;
        let drop = amt - take;
        recovered[i] = (sp, take);
        net.fish.add(sp, -take);
        if drop < 0.8 {
            net.fish.add(sp, -drop);
        }
    }
    net.soak_tides = 0;
    net.sea_lion_pressure *= 0.4;
    net.quality = (net.quality + 0.12).min(1.0);
    if rng.random() < 0.08 {
        net.condition = (net.condition - rng.uniform(1.0, 4.0)).max(5.0);
    }
    (recovered, q.max(0.40).min(1.12))
}
