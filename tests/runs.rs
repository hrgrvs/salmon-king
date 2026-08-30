//! Run curves produce fish in the right months; odd-year pinks dwarf even-year.

use salmon_king::data::species::SpeciesId;
use salmon_king::sim::clock::GameDate;
use salmon_king::sim::openings::generate_run_mods;
use salmon_king::sim::rng::Rng;
use salmon_king::sim::runs::site_availability;

fn sum_reds(site: &str, year: i32, mods: &salmon_king::sim::models::RunMods, month: u8) -> f64 {
    let mut total = 0.0;
    for day in 1..28 {
        let avail = site_availability(site, GameDate::new(year, month, day), mods);
        total += avail[SpeciesId::Red.idx()].1;
    }
    total
}

fn sum_pinks(site: &str, year: i32, mods: &salmon_king::sim::models::RunMods, month: u8) -> f64 {
    let mut total = 0.0;
    for day in 1..28 {
        let avail = site_availability(site, GameDate::new(year, month, day), mods);
        total += avail[SpeciesId::Pink.idx()].1;
    }
    total
}

#[test]
fn westside_reds_stronger_in_june_july_than_september() {
    let mut mods = generate_run_mods(2025, &mut Rng::new(1));
    mods.karluk_early = 1.0;
    mods.karluk_late = 1.0;
    let early = sum_reds("cape_uyak", 2025, &mods, 6) + sum_reds("cape_uyak", 2025, &mods, 7);
    let late = sum_reds("cape_uyak", 2025, &mods, 9);
    assert!(early > late * 1.3);
}

#[test]
fn pinks_peak_july_august_not_june() {
    let mut mods = generate_run_mods(2025, &mut Rng::new(2));
    mods.pink = 1.8;
    let july = sum_pinks("uganik_pass", 2025, &mods, 7);
    let august = sum_pinks("uganik_pass", 2025, &mods, 8);
    let june = sum_pinks("uganik_pass", 2025, &mods, 6);
    assert!(july + august > june * 4.0);
}

#[test]
fn odd_year_pinks_exceed_even_year() {
    let mut odd = generate_run_mods(2025, &mut Rng::new(3));
    let mut even = generate_run_mods(2024, &mut Rng::new(3));
    odd.pink = 1.8;
    even.pink = 0.5;
    let odd_lbs = sum_pinks("uganik_pass", 2025, &odd, 7);
    let even_lbs = sum_pinks("uganik_pass", 2024, &even, 7);
    assert!(odd_lbs > even_lbs * 2.0);
}

#[test]
fn kings_are_incidental() {
    let mut mods = generate_run_mods(2025, &mut Rng::new(4));
    mods.king = 0.3;
    mods.karluk_early = 1.0;
    let mut kings = 0.0;
    let mut reds = 0.0;
    for day in 1..28 {
        let a = site_availability("cape_uyak", GameDate::new(2025, 6, day), &mods);
        kings += a[SpeciesId::King.idx()].1;
        reds += a[SpeciesId::Red.idx()].1;
    }
    assert!(kings < reds * 0.25);
}

#[test]
fn silvers_show_in_september() {
    let mut mods = generate_run_mods(2025, &mut Rng::new(5));
    mods.silver = 1.0;
    let mut sept = 0.0;
    let mut june = 0.0;
    for d in 1..18 {
        sept += site_availability("olga_narrows", GameDate::new(2025, 9, d), &mods)[SpeciesId::Silver.idx()].1;
        june += site_availability("olga_narrows", GameDate::new(2025, 6, d), &mods)[SpeciesId::Silver.idx()].1;
    }
    assert!(sept > june * 2.0);
}
