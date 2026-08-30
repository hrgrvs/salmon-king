use crate::data::camps::CampDef;
use crate::sim::clock::{doy, GameDate};
use crate::sim::models::Weather;
use crate::sim::rng::Rng;

pub fn roll_weather(day: GameDate, camp: &CampDef, rng: &mut Rng) -> Weather {
    let n = doy(day);
    let fog_p = camp.fog + if (152..=212).contains(&n) { 0.18 } else { 0.0 };
    let will_p = camp.williwaw + if n >= 244 { 0.08 } else { 0.0 };
    let mut wind = rng.gauss(14.0, 6.0).max(4.0);
    if n >= 244 {
        wind += rng.uniform(2.0, 10.0);
    }
    let mut seas = (wind / 6.0 + rng.uniform(-0.4, 1.2)).max(0.5);
    if camp.district == "central" {
        seas += 0.4;
    }
    let fog = rng.gauss(fog_p, 0.15).clamp(0.0, 1.0);
    let williwaw = rng.random() < will_p * 0.08;
    let (precip, label) = if williwaw {
        wind = wind.max(rng.uniform(42.0, 68.0));
        seas = seas.max(rng.uniform(4.5, 8.0));
        (
            "williwaw".to_string(),
            format!("WILLIWAW {} kt, {:.0} ft", wind as i32, seas),
        )
    } else if fog > 0.55 {
        (
            "fog".to_string(),
            format!("advection fog, {} kt", wind as i32),
        )
    } else if rng.random() < 0.45 {
        (
            "rain".to_string(),
            format!("rain {} kt, {:.0} ft", wind as i32, seas),
        )
    } else {
        (
            "overcast".to_string(),
            format!("overcast {} kt, {:.0} ft", wind as i32, seas),
        )
    };
    Weather {
        wind_kt: (wind * 10.0).round() / 10.0,
        seas_ft: (seas * 10.0).round() / 10.0,
        fog: (fog * 100.0).round() / 100.0,
        precip,
        williwaw,
        label,
    }
}

pub fn skiffs_grounded(weather: &Weather) -> bool {
    weather.williwaw || weather.seas_ft >= 7.5 || weather.wind_kt >= 45.0
}
