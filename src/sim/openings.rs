//! ADF&G-style emergency-order calendar. Generated from weir strength + even/odd.
//!
//! Does not hard-code a 2026 opening board. Windows follow 5 AAC 18.362 / 18.361.

use crate::data::sites::is_pulse_alitak;
use crate::sim::clock::{doy, GameDate};
use crate::sim::models::{OpeningWindow, RunMods};
use crate::sim::rng::Rng;

fn add_window(
    windows: &mut Vec<OpeningWindow>,
    start: GameDate,
    hours: i32,
    district: &str,
    reason: impl Into<String>,
    pulse: bool,
) {
    let days = ((hours as f64 / 24.0).round() as i32).max(1);
    let end = if hours <= 36 {
        start.add_days(1)
    } else if hours <= 60 {
        start.add_days(2)
    } else {
        start.add_days(days - 1)
    };
    windows.push(OpeningWindow {
        start_doy: doy(start),
        end_doy: doy(end),
        hours,
        district: district.into(),
        reason: reason.into(),
        pulse_sites_only: pulse,
    });
}

pub fn generate_run_mods(year: i32, rng: &mut Rng) -> RunMods {
    let odd = year % 2 == 1;
    let karluk_early = rng.uniform(0.12, 1.20);
    let frazer = rng.uniform(0.40, 1.20);
    let upper_station = rng.uniform(0.50, 1.20);
    RunMods {
        karluk_early,
        karluk_late: rng.uniform(0.55, 1.25),
        frazer,
        upper_station,
        pink: if odd {
            rng.uniform(1.35, 2.10)
        } else {
            rng.uniform(0.35, 0.75)
        },
        chum: rng.uniform(0.70, 1.15),
        silver: rng.uniform(0.45, 1.10),
        king: rng.uniform(0.12, 0.45),
        pink_flood: odd,
        karluk_early_fail: karluk_early < 0.40,
        frazer_goals_ok: frazer >= 0.85 && upper_station >= 0.80,
    }
}

pub fn generate_openings(year: i32, district: &str, mods: &RunMods, rng: &mut Rng) -> Vec<OpeningWindow> {
    if district == "alitak" {
        alitak(year, mods, rng)
    } else {
        central(year, mods, rng)
    }
}

fn central(year: i32, mods: &RunMods, rng: &mut Rng) -> Vec<OpeningWindow> {
    let mut w = Vec::new();
    let first = GameDate::new(year, 6, rng.randint(1, 4) as u8);
    let second = GameDate::new(year, 6, rng.randint(9, 13) as u8);
    add_window(&mut w, first, 33, "central", "33-hr test · mixed-stock early sockeye", false);
    add_window(&mut w, second, 33, "central", "33-hr test · mixed-stock early sockeye", false);

    if !mods.karluk_early_fail {
        let mut d = GameDate::new(year, 6, 16);
        let last = GameDate::new(year, 7, 5);
        while d <= last {
            if rng.random() < 0.50 + 0.25 * mods.karluk_early.min(1.0) {
                let hrs = if rng.random() < 0.55 { 33 } else { 57 };
                add_window(
                    &mut w,
                    d,
                    hrs,
                    "central",
                    format!("{hrs}-hr period · Karluk early sockeye"),
                    false,
                );
                d = d.add_days(if hrs == 33 { 3 } else { 4 });
            } else {
                d = d.add_days(rng.randint(2, 4));
            }
        }
    }

    let mut d = GameDate::new(year, 7, 6);
    let last = GameDate::new(year, 8, 15);
    let p = if mods.pink_flood { 0.92 } else { 0.50 };
    while d <= last {
        if rng.random() < p {
            let hrs = if mods.pink_flood || rng.random() < 0.40 {
                57
            } else {
                33
            };
            add_window(
                &mut w,
                d,
                hrs,
                "central",
                format!("{hrs}-hr period · pink salmon"),
                false,
            );
            d = d.add_days(if mods.pink_flood { 3 } else { 4 });
        } else {
            d = d.add_days(rng.randint(2, 3));
        }
    }

    let mut d = GameDate::new(year, 8, 16);
    let last = GameDate::new(year, 8, 24);
    while d <= last {
        if rng.random() < 0.60 {
            add_window(
                &mut w,
                d,
                33,
                "central",
                "33-hr period · pinks + Karluk late sockeye",
                false,
            );
            d = d.add_days(3);
        } else {
            d = d.add_days(2);
        }
    }

    let mut d = GameDate::new(year, 8, 25);
    let last = GameDate::new(year, 9, 5);
    while d <= last {
        if rng.random() < 0.45 + 0.2 * mods.karluk_late {
            add_window(&mut w, d, 33, "central", "33-hr period · Karluk late sockeye", false);
            d = d.add_days(3);
        } else {
            d = d.add_days(2);
        }
    }

    let mut d = GameDate::new(year, 9, 6);
    let last = GameDate::new(year, 9, 17);
    while d <= last {
        if rng.random() < 0.35 {
            add_window(&mut w, d, 33, "central", "33-hr period · late sockeye + coho", false);
            d = d.add_days(4);
        } else {
            d = d.add_days(3);
        }
    }
    w
}

fn alitak(year: i32, mods: &RunMods, rng: &mut Rng) -> Vec<OpeningWindow> {
    let mut w = Vec::new();
    add_window(
        &mut w,
        GameDate::new(year, 6, rng.randint(5, 10) as u8),
        33,
        "alitak",
        "33-hr period · Frazer / early Upper Station",
        false,
    );

    let mut block = GameDate::new(year, 6, 13);
    let last = GameDate::new(year, 9, 15);
    while block <= last {
        let open_days = rng.randint(5, 7);
        let reason = if mods.pink_flood && block.month >= 7 {
            "Alitak pulse · odd-year pinks + sockeye"
        } else if !mods.pink_flood && block.month >= 7 {
            "Alitak pulse · even year, sockeye-weighted"
        } else {
            "Alitak pulse · Frazer / Upper Station / pinks"
        };
        add_window(&mut w, block, open_days * 24, "alitak", reason, false);
        if mods.frazer >= 1.0 && block.month >= 7 {
            add_window(
                &mut w,
                block.add_days(2),
                33,
                "alitak",
                "pulse · Dog Salmon Flats / Upper Station / Akalura",
                true,
            );
        }
        block = block.add_days(10);
    }
    w
}

pub fn is_open(
    windows: &[OpeningWindow],
    day: GameDate,
    district: &str,
    site_id: &str,
) -> (bool, String) {
    let n = doy(day);
    let mut reason = String::from("closed — no emergency order");
    let mut traditional_open = false;
    let mut pulse_open = false;
    let mut pulse_reason = String::new();
    for w in windows {
        if w.district != district {
            continue;
        }
        if w.start_doy <= n && n <= w.end_doy {
            if w.pulse_sites_only {
                pulse_open = true;
                pulse_reason = w.reason.clone();
            } else {
                traditional_open = true;
                reason = w.reason.clone();
            }
        }
    }
    if is_pulse_alitak(site_id) {
        if pulse_open {
            return (true, pulse_reason);
        }
        if traditional_open {
            return (
                false,
                "pulse site dark — traditional water is fishing".into(),
            );
        }
        return (false, reason);
    }
    if traditional_open {
        (true, reason)
    } else {
        (false, reason)
    }
}

pub fn current_window<'a>(
    windows: &'a [OpeningWindow],
    day: GameDate,
    district: &str,
) -> Option<&'a OpeningWindow> {
    let n = doy(day);
    windows.iter().find(|w| {
        w.district == district && !w.pulse_sites_only && w.start_doy <= n && n <= w.end_doy
    })
}
