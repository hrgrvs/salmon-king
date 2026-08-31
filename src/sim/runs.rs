//! Run curves by species and site. Timing is real; amplitudes are game numbers.

use crate::data::sites::site;
use crate::data::species::{species as spec_of, SpeciesId};
use crate::sim::clock::{doy, GameDate};
use crate::sim::models::RunMods;

/// Game-number daily available pounds at a 1.0 affinity, 1.0 run, peak day.
const PEAK_LBS: [f64; 5] = [18.0, 880.0, 2600.0, 440.0, 280.0];

const LATE_RED_PEAK_DOY: i32 = 235;
const LATE_RED_WIDTH: f64 = 16.0;

fn gauss_curve(n: i32, peak: i32, width: f64) -> f64 {
    let x = (n - peak) as f64 / width;
    (-0.5 * x * x).exp()
}

/// Pounds available to a full-condition net this tide, before gear/weather/crew.
pub fn site_availability(site_id: &str, day: GameDate, mods: &RunMods) -> [(SpeciesId, f64); 5] {
    let s = site(site_id).expect("unknown site");
    let n = doy(day);
    let mut out = [(SpeciesId::King, 0.0); 5];
    for (i, &sp) in SpeciesId::ALL.iter().enumerate() {
        let spec = spec_of(sp);
        let aff = s.affinity_of(sp);
        if aff <= 0.0 {
            out[i] = (sp, 0.0);
            continue;
        }
        let mut curve = gauss_curve(n, spec.peak_doy, spec.width_days);
        let run = match sp {
            SpeciesId::Pink => mods.pink,
            SpeciesId::King => mods.king,
            SpeciesId::Chum => mods.chum,
            SpeciesId::Silver => mods.silver,
            SpeciesId::Red => {
                let early = if s.district == "central" {
                    mods.karluk_early
                } else {
                    mods.frazer
                };
                let late = if s.district == "central" {
                    mods.karluk_late
                } else {
                    mods.upper_station
                };
                let early_c = gauss_curve(n, spec.peak_doy, spec.width_days);
                let late_c = gauss_curve(n, LATE_RED_PEAK_DOY, LATE_RED_WIDTH);
                if matches!(s.id, "upper_station_outer" | "akalura_outer" | "dog_salmon") {
                    curve = 0.35 * early_c + 0.90 * late_c;
                    0.4 * early + 0.8 * late
                } else {
                    curve = 0.75 * early_c + 0.55 * late_c;
                    0.65 * early + 0.50 * late
                }
            }
        };
        let lbs = PEAK_LBS[sp.idx()] * aff * run * curve;
        out[i] = (sp, (lbs * 0.86).max(0.0));
    }
    out
}
