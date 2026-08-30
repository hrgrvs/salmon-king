//! Deterministic season engine. Pure sim. TUI is a client.

use crate::data::camps::{camp as camp_by_id, CampDef};
use crate::data::crew_pool::{candidate, starting_hire, CANDIDATES, OWNER_NAME};
use crate::data::sites::{is_playable, is_pulse_alitak, site, sites_for_camp, SiteDef};
use crate::data::species::{prices_for_year, SpeciesId};
use crate::sim::catch::{pick_net, soak_net};
use crate::sim::clock::{
    advance, phase_label, season_end, season_start, tides_in_season, GameDate, Tide,
};
use crate::sim::events::maybe_fire;
use crate::sim::mammals::{
    new_wildlife, pinnipeds_present, resident_scatter, status_lines, tick_wildlife, Wildlife,
};
use crate::sim::models::{
    CrewMember, CrewStatus, GameEnd, Lbs, Ledger, LogLine, Net, OpeningWindow, RunMods, Skiff,
    SkiffJob, Tender, Weather,
};
use crate::sim::openings::{current_window, generate_openings, generate_run_mods, is_open};
use crate::sim::rng::Rng;
use crate::sim::runs::site_availability;
use crate::sim::weather::{roll_weather, skiffs_grounded};

const FOOD_PER_PERSON_DAY: f64 = 1.0;
const FUEL_PER_HOP: f64 = 5.5;
const HOLDING_CAP: f64 = 1400.0;
const PICKER_CAP: f64 = 520.0;

fn buy_table(item: &str) -> Option<(f64, f64)> {
    match item {
        "food" => Some((18.0, 160.0)),
        "fuel" => Some((40.0, 170.0)),
        "ice" => Some((20.0, 55.0)),
        "twine" => Some((1.0, 90.0)),
        "prop" => Some((1.0, 220.0)),
        _ => None,
    }
}

#[derive(Clone, Debug)]
pub struct Recap {
    pub survived: bool,
    pub end: GameEnd,
    pub year: i32,
    pub camp: String,
    pub lbs: Lbs,
    pub gross: f64,
    pub expenses: f64,
    pub net: f64,
    pub settle: f64,
    pub crew_stayed: Vec<String>,
    pub notable: Vec<String>,
    pub grade: String,
    pub nickname: String,
    pub cash: f64,
    pub tickets: i32,
}

impl Recap {
    pub fn as_text(&self) -> String {
        let mut lines = vec![
            format!("SEASON {}  {}", self.year, self.camp),
            format!(
                "End: {}",
                if self.end == GameEnd::None {
                    "season"
                } else {
                    self.end.as_str()
                }
            ),
            format!(
                "Kings {:.0} lb   Reds {:.0} lb   Pinks {:.0} lb",
                self.lbs.king, self.lbs.red, self.lbs.pink
            ),
            format!(
                "Chums {:.0} lb   Silvers {:.0} lb",
                self.lbs.chum, self.lbs.silver
            ),
            format!(
                "Gross ${:.0}   Expenses ${:.0}   Net ${:.0}",
                self.gross, self.expenses, self.net
            ),
            format!(
                "Postseason settle {:+.0}   Cash ${:.0}   Tickets {}",
                self.settle, self.cash, self.tickets
            ),
            format!(
                "Stayed: {}",
                if self.crew_stayed.is_empty() {
                    "nobody".into()
                } else {
                    self.crew_stayed.join(", ")
                }
            ),
            format!("Grade {} — {}", self.grade, self.nickname),
        ];
        if !self.notable.is_empty() {
            lines.push("Notable:".into());
            for n in self.notable.iter().take(8) {
                lines.push(format!("  · {n}"));
            }
        }
        lines.join("\n")
    }
}

pub struct Game {
    pub rng: Rng,
    pub year: i32,
    pub camp: CampDef,
    pub day: GameDate,
    pub tide: Tide,
    pub tick: i32,
    pub weather: Weather,
    pub nets: Vec<Net>,
    pub skiffs: Vec<Skiff>,
    pub crew: Vec<CrewMember>,
    pub hire_pool: Vec<String>,
    pub tender: Tender,
    pub ledger: Ledger,
    pub log: Vec<LogLine>,
    pub openings: Vec<OpeningWindow>,
    pub mods: RunMods,
    pub food: f64,
    pub fuel_cache: f64,
    pub twine: i32,
    pub spare_prop: i32,
    pub ice_cache: f64,
    pub bunkhouse: i32,
    pub cookshack: i32,
    pub net_loft: i32,
    pub skiff_stalls: i32,
    pub joint_venture: bool,
    pub skipper_in_town: bool,
    pub skipper_town_eta: i32,
    pub wildlife: Wildlife,
    pub bonus_school: i32,
    pub extend_open_tides: i32,
    pub landed: Lbs,
    pub end: GameEnd,
    pub notable: Vec<String>,
    pub midseason_cut: bool,
    pub last_open: bool,
    pub payday_counter: i32,
}

impl Game {
    pub fn odd_year(&self) -> bool {
        self.year % 2 == 1
    }

    pub fn district(&self) -> &str {
        self.camp.district
    }

    pub fn open_reason(&self) -> String {
        if self.extend_open_tides > 0 {
            return "extended period (ADF&G radio, last 24 hr)".into();
        }
        if let Some(win) = current_window(&self.openings, self.day, self.district()) {
            win.reason.clone()
        } else {
            "closed — no emergency order".into()
        }
    }

    pub fn site_is_open(&self, site_id: &str) -> (bool, String) {
        if self.extend_open_tides > 0 {
            if let Some(s) = site(site_id) {
                if s.district == self.district() {
                    if is_pulse_alitak(site_id) {
                        return is_open(&self.openings, self.day, self.district(), site_id);
                    }
                    return (true, self.open_reason());
                }
            }
        }
        is_open(&self.openings, self.day, self.district(), site_id)
    }

    pub fn playable_sites(&self) -> Vec<&'static SiteDef> {
        sites_for_camp(self.camp.id)
    }

    pub fn mammal_status(&self) -> Vec<String> {
        status_lines(&self.wildlife, self.camp.id)
    }

    pub fn note(&mut self, text: impl Into<String>, kind: &str) {
        self.log.push(LogLine {
            tick: self.tick,
            day: self.day,
            text: text.into(),
            kind: kind.into(),
        });
        if self.log.len() > 400 {
            let keep = self.log.split_off(self.log.len() - 300);
            self.log = keep;
        }
    }

    pub fn scare_crew(&mut self, amt: f64) {
        for c in &mut self.crew {
            if c.status != CrewStatus::Quit {
                c.morale = (c.morale - amt).max(0.0);
            }
        }
    }

    pub fn owner_idx(&self) -> usize {
        self.crew.iter().position(|c| c.is_owner).expect("owner")
    }

    pub fn working_deck(&self) -> Vec<&CrewMember> {
        self.crew
            .iter()
            .filter(|c| {
                c.status == CrewStatus::Working
                    && c.role != "cook"
                    && !(c.is_owner && self.skipper_in_town)
            })
            .collect()
    }

    pub fn cook(&self) -> Option<&CrewMember> {
        self.crew
            .iter()
            .find(|c| c.role == "cook" && c.status != CrewStatus::Quit)
    }

    pub fn step(&mut self) -> Vec<LogLine> {
        let start = self.log.len();
        if self.end != GameEnd::None {
            return Vec::new();
        }
        self.tick += 1;
        let (day, tide) = advance(self.day, self.tide);
        self.day = day;
        self.tide = tide;
        if self.day > season_end(self.year) {
            self.finish(GameEnd::Season);
            return Self::log_since(&self.log, start);
        }

        self.weather = roll_weather(self.day, &self.camp, &mut self.rng);
        for line in tick_wildlife(&mut self.wildlife) {
            self.note(line, "mammal");
        }
        if self.bonus_school > 0 {
            self.bonus_school -= 1;
        }
        if self.extend_open_tides > 0 {
            self.extend_open_tides -= 1;
        }

        self.town_clock();
        self.tender_clock();
        self.maybe_midseason_cut();
        if let Some(ev) = maybe_fire(self) {
            self.note(&ev, "event");
            self.notable.push(ev);
        }

        self.soak_all();
        self.skiffs_act();
        self.crew_metabolism();
        self.payroll();
        self.repair_idle();
        self.check_fail();
        Self::log_since(&self.log, start)
    }

    fn log_since(log: &[LogLine], start: usize) -> Vec<LogLine> {
        let start = start.min(log.len());
        log[start..].to_vec()
    }

    fn town_clock(&mut self) {
        if self.skipper_in_town {
            self.skipper_town_eta -= 1;
            if self.skipper_town_eta <= 0 {
                self.skipper_in_town = false;
                let i = self.owner_idx();
                self.crew[i].status = CrewStatus::Working;
                self.note(
                    "Permit holder back on the site. Nets can fish again.",
                    "adfg",
                );
            }
        }
    }

    fn tender_clock(&mut self) {
        if self.tender.present {
            self.tender.stay -= 1;
            if self.tender.stay <= 0 {
                self.tender.present = false;
                let mut gap = if self.district() == "alitak" {
                    self.rng.randint(6, 12)
                } else {
                    self.rng.randint(5, 10)
                };
                if self.tender.late {
                    gap += 2;
                }
                self.tender.eta_tides = gap;
                self.tender.late = false;
                let name = self.tender.name.clone();
                self.note(
                    format!("{name} pulled the hook. Next look in {gap} tides."),
                    "market",
                );
            }
        } else {
            self.tender.eta_tides -= 1;
            if self.tender.eta_tides <= 0 {
                self.tender.present = true;
                self.tender.stay = self.rng.randint(2, 4);
                let name = self.tender.name.clone();
                self.note(
                    format!("{name} in the hole. Weigh and buy. Ice if you need it."),
                    "market",
                );
            }
        }
    }

    fn maybe_midseason_cut(&mut self) {
        if self.midseason_cut || !self.mods.pink_flood {
            return;
        }
        if self.day.month == 8 && self.day.day >= 1 {
            let floor = prices_for_year(self.year).pink * 0.50;
            self.tender.prices.pink = (self.tender.prices.pink * 0.82).max(floor);
            self.midseason_cut = true;
            let px = self.tender.prices.pink;
            self.note(
                format!("Midseason pink cut — flood year. ${px:.2}/lb. Opening estimate is paper."),
                "market",
            );
        }
    }

    fn soak_all(&mut self) {
        let legal_skipper = !self.skipper_in_town;
        if !legal_skipper && self.nets.iter().any(|n| n.in_water) {
            self.note(
                "Permit holder is off-site. Sets cannot legally fish.",
                "adfg",
            );
        }
        let mut any_open = false;
        let n_nets = self.nets.len();
        for i in 0..n_nets {
            if !self.nets[i].in_water {
                continue;
            }
            let Some(site_id) = self.nets[i].site_id.clone() else {
                continue;
            };
            let (open, reason) = self.site_is_open(&site_id);
            if !open {
                if self.last_open {
                    self.note(format!("Closer. {reason}. Gear out of the water."), "adfg");
                }
                if self.nets[i].soak_tides > 0 {
                    self.nets[i].soak_tides += 1;
                    if self.nets[i].soak_tides > 2 {
                        let fine = 250.0;
                        self.ledger.cash -= fine;
                        self.ledger.fines += fine;
                        self.nets[i].in_water = false;
                        let site_name = site_id.clone();
                        self.nets[i].site_id = None;
                        self.note(
                            format!(
                                "Gear still fishing a closer at {site_name}. ADF&G would have the card. \
${fine:.0} (game fine) and the net is on the beach."
                            ),
                            "adfg",
                        );
                    }
                }
                continue;
            }
            if !legal_skipper {
                continue;
            }
            any_open = true;
            let mut avail = site_availability(&site_id, self.day, &self.mods);
            if self.bonus_school > 0 {
                for item in &mut avail {
                    item.1 *= 1.55;
                }
            }
            let sdef = site(&site_id).unwrap();
            let lions = pinnipeds_present(&self.wildlife, &site_id);
            let mut mammal = 1.0;
            if lions {
                mammal -= 0.15 * self.nets[i].sea_lion_pressure * sdef.sea_lion;
                if self.nets[i].soak_tides >= 3 {
                    mammal -= 0.08 * sdef.harbor_seal;
                }
            } else {
                self.nets[i].sea_lion_pressure = self.nets[i].sea_lion_pressure.min(0.04);
            }
            mammal = mammal.max(0.25);
            let scale = f64::from(self.nets[i].fathoms) / 75.0;
            let scatter = resident_scatter(&self.wildlife, &site_id);
            soak_net(
                &mut self.nets[i],
                &avail,
                &self.weather,
                mammal,
                scatter,
                &mut self.rng,
                scale,
                lions,
            );
        }
        self.last_open = any_open || current_window(&self.openings, self.day, self.district()).is_some();
    }

    fn skiffs_act(&mut self) {
        let grounded = skiffs_grounded(&self.weather);
        let n = self.skiffs.len();
        for i in 0..n {
            if self.skiffs[i].wrecked {
                continue;
            }
            if self.skiffs[i].eta > 0 {
                self.skiffs[i].eta -= 1;
                if self.skiffs[i].eta == 0 {
                    if let Some(dest) = self.skiffs[i].dest.take() {
                        self.skiffs[i].location = dest;
                        let name = self.skiffs[i].name.clone();
                        let loc = self.skiffs[i].location.clone();
                        self.note(format!("{name} on {loc}."), "skiff");
                    }
                }
                continue;
            }
            if grounded && self.skiffs[i].job != SkiffJob::Idle {
                let name = self.skiffs[i].name.clone();
                self.note(format!("{name} stays on the running line — weather."), "weather");
                continue;
            }
            match self.skiffs[i].job {
                SkiffJob::Repair => {
                    let loft = self.net_loft;
                    self.skiffs[i].condition = (self.skiffs[i].condition + 8.0 + 2.0 * f64::from(loft)).min(92.0);
                    if self.skiffs[i].condition >= 70.0 {
                        self.skiffs[i].job = SkiffJob::Idle;
                        let name = self.skiffs[i].name.clone();
                        self.note(format!("{name} will start. Don't ask how."), "skiff");
                    }
                }
                SkiffJob::Town => self.run_town(i),
                SkiffJob::Tender => self.run_tender(i),
                SkiffJob::Pick => self.run_pick(i),
                SkiffJob::Idle => {}
            }
        }
    }

    fn crew_on(&self, skiff_i: usize) -> Vec<usize> {
        let mut out = Vec::new();
        let skiff = &self.skiffs[skiff_i];
        for cid in &skiff.crew_ids {
            if let Some(i) = self.crew.iter().position(|c| &c.id == cid) {
                if self.crew[i].status == CrewStatus::Working {
                    out.push(i);
                }
            }
        }
        if !self.skipper_in_town {
            if let Some(own) = self.crew.iter().position(|c| c.is_owner) {
                if self.crew[own].status == CrewStatus::Working {
                    let assigned_here = self.crew[own].assigned.as_deref() == Some(skiff.id.as_str());
                    let unassigned_pick =
                        self.crew[own].assigned.is_none() && skiff.job == SkiffJob::Pick;
                    if (assigned_here || unassigned_pick) && !out.contains(&own) {
                        out.push(own);
                    }
                }
            }
        }
        out
    }

    fn burn_fuel(&mut self, skiff_i: usize, hops: i32) -> bool {
        let need = FUEL_PER_HOP * f64::from(hops);
        if self.skiffs[skiff_i].fuel >= need {
            self.skiffs[skiff_i].fuel -= need;
            return true;
        }
        let take = self
            .fuel_cache
            .min(need - self.skiffs[skiff_i].fuel);
        self.fuel_cache -= take;
        self.skiffs[skiff_i].fuel += take;
        if self.skiffs[skiff_i].fuel >= need {
            self.skiffs[skiff_i].fuel -= need;
            true
        } else {
            false
        }
    }

    fn goto(&mut self, skiff_i: usize, dest: &str) {
        if self.skiffs[skiff_i].location == dest {
            return;
        }
        let mut hops = 1;
        if is_playable(dest) {
            if let Some(s) = site(dest) {
                if let Some(t) = s.travel_from(self.camp.id) {
                    hops = t.max(1);
                }
            }
        }
        if dest == "tender" {
            hops = 1;
        }
        if dest == "town" {
            hops = self.camp.town_tides;
        }
        if !self.burn_fuel(skiff_i, hops) {
            let name = self.skiffs[skiff_i].name.clone();
            self.note(format!("{name} is dry. No fuel, no go."), "skiff");
            self.skiffs[skiff_i].job = SkiffJob::Idle;
            return;
        }
        if hops <= 1 {
            self.skiffs[skiff_i].location = dest.into();
        } else {
            self.skiffs[skiff_i].location = "transit".into();
            self.skiffs[skiff_i].dest = Some(dest.into());
            self.skiffs[skiff_i].eta = hops - 1;
        }
    }

    fn run_pick(&mut self, skiff_i: usize) {
        let mut site_id = self.skiffs[skiff_i].job_site.clone();
        if site_id.is_none() {
            site_id = self
                .nets
                .iter()
                .find(|n| n.in_water)
                .and_then(|n| n.site_id.clone());
            self.skiffs[skiff_i].job_site = site_id.clone();
        }
        let Some(site_id) = site_id else {
            return;
        };
        if self.skiffs[skiff_i].location != site_id {
            self.goto(skiff_i, &site_id);
            if self.skiffs[skiff_i].location != site_id {
                return;
            }
        }
        let net_idxs: Vec<usize> = self
            .nets
            .iter()
            .enumerate()
            .filter(|(_, n)| n.in_water && n.site_id.as_deref() == Some(site_id.as_str()))
            .map(|(i, _)| i)
            .collect();
        if net_idxs.is_empty() {
            return;
        }
        let crew_idxs = self.crew_on(skiff_i);
        for &ci in &crew_idxs {
            self.crew[ci].energy = (self.crew[ci].energy - 9.0).max(0.0);
            self.crew[ci].hunger = (self.crew[ci].hunger + 4.0).min(100.0);
        }
        let cap = if self.skiffs[skiff_i].kind == "holding" {
            HOLDING_CAP
        } else {
            PICKER_CAP
        };
        // Snapshot crew for pick_net (needs owned slice).
        let crew_snap: Vec<CrewMember> = crew_idxs.iter().map(|&i| self.crew[i].clone()).collect();
        for &ni in &net_idxs {
            let (rec, q) = pick_net(&mut self.nets[ni], &crew_snap, &mut self.rng);
            let mut room = cap - self.skiffs[skiff_i].cargo.total();
            let rec_sum: f64 = rec.iter().map(|(_, a)| *a).sum();
            for &(sp, amt) in &rec {
                let take = amt.min(room.max(0.0));
                self.skiffs[skiff_i].cargo.add(sp, take);
                room -= take;
            }
            let ice_q = if self.skiffs[skiff_i].ice > 0.0 {
                1.08
            } else {
                0.82
            };
            self.skiffs[skiff_i].ice = (self.skiffs[skiff_i].ice - 1.2).max(0.0);
            let w = {
                let t = self.skiffs[skiff_i].cargo.total();
                if t == 0.0 {
                    1.0
                } else {
                    t
                }
            };
            let cq = self.skiffs[skiff_i].cargo_quality;
            self.skiffs[skiff_i].cargo_quality =
                (cq * (w - rec_sum + 1.0) + q * ice_q * rec_sum) / (w + 1.0);
        }
        let lbs = self.skiffs[skiff_i].cargo.total();
        let name = self.skiffs[skiff_i].name.clone();
        let nset = net_idxs.len();
        self.note(
            format!("{name} picked {site_id} ({nset} set). Hold {lbs:.0} lb."),
            "skiff",
        );
        if lbs >= cap * 0.85 {
            self.note(
                format!("{name} is in the marks. Run the tender or you stop picking."),
                "skiff",
            );
        }
    }

    fn run_tender(&mut self, skiff_i: usize) {
        if !self.tender.present {
            let name = self.skiffs[skiff_i].name.clone();
            self.note(
                format!("{name} waiting on a tender that isn't here."),
                "market",
            );
            return;
        }
        let anch = self.camp.tender_anchorage;
        let loc = self.skiffs[skiff_i].location.clone();
        if loc != "tender" && loc != anch {
            self.goto(skiff_i, anch);
            let loc2 = self.skiffs[skiff_i].location.clone();
            if loc2 != anch && loc2 != "tender" {
                return;
            }
            self.skiffs[skiff_i].location = "tender".into();
        }
        if self.skiffs[skiff_i].cargo.total() <= 0.5 {
            self.skiffs[skiff_i].job = SkiffJob::Idle;
            self.skiffs[skiff_i].location = "camp".into();
            return;
        }
        self.settle(skiff_i);
    }

    fn run_town(&mut self, skiff_i: usize) {
        if self.skiffs[skiff_i].location != "town" {
            self.goto(skiff_i, "town");
            let owner_assigned = self.crew[self.owner_idx()].assigned.as_deref()
                == Some(self.skiffs[skiff_i].id.as_str());
            if owner_assigned {
                self.skipper_in_town = true;
                self.skipper_town_eta = self.skipper_town_eta.max(self.camp.town_tides * 2);
                let oi = self.owner_idx();
                self.crew[oi].status = CrewStatus::Town;
                self.note(
                    "Permit holder ran town. Sets go dark until you're back on the site.",
                    "adfg",
                );
            }
            return;
        }
        if self.spare_prop <= 0 {
            self.spare_prop = 1;
            self.ledger.cash -= 220.0;
            self.ledger.repairs += 220.0;
        }
        self.skiffs[skiff_i].condition = (self.skiffs[skiff_i].condition + 20.0).min(95.0);
        self.skiffs[skiff_i].job = SkiffJob::Idle;
        self.goto(skiff_i, "camp");
    }

    pub fn settle(&mut self, skiff_i: usize) -> f64 {
        if self.skiffs[skiff_i].cargo.total() <= 0.0 {
            return 0.0;
        }
        let q = self.skiffs[skiff_i].cargo_quality.clamp(0.50, 1.12);
        let claim = if q < 0.70 {
            "quality claim — soft / warm"
        } else if q < 0.85 {
            "docked a bit — not enough ice"
        } else if q >= 1.05 {
            "chilled bonus"
        } else {
            ""
        };
        let mut pay = 0.0;
        let mut bits = Vec::new();
        let cargo = self.skiffs[skiff_i].cargo;
        for sp in SpeciesId::ALL {
            let amt = cargo.get(sp);
            if amt <= 0.0 {
                continue;
            }
            let px = self.tender.prices.get(sp) * q;
            let dol = amt * px;
            pay += dol;
            self.landed.add(sp, amt);
            bits.push(format!("{:.0} {}", amt, sp.as_str()));
        }
        self.ledger.cash += pay;
        self.ledger.gross += pay;
        self.ledger.tickets += 1;
        self.tender.last_ticket = pay;
        self.tender.last_lbs = cargo.total();
        self.tender.last_note = format!(
            "${:.0}  {}  q={q:.2} {claim}",
            pay,
            bits.join(", ")
        )
        .trim()
        .to_string();
        for c in &mut self.crew {
            if c.is_owner || c.status == CrewStatus::Quit || c.share <= 0.0 {
                continue;
            }
            c.accrued_share += pay * c.share;
        }
        self.skiffs[skiff_i].cargo = Lbs::default();
        self.skiffs[skiff_i].cargo_quality = 1.0;
        self.skiffs[skiff_i].location = "camp".into();
        self.skiffs[skiff_i].job = SkiffJob::Idle;
        let note = self.tender.last_note.clone();
        self.note(format!("Fish ticket: {note}"), "market");
        pay
    }

    /// Settle by skiff identity (used by tests that hold a skiff handle conceptually).
    pub fn settle_skiff(&mut self, skiff_id: &str) -> f64 {
        if let Some(i) = self.skiffs.iter().position(|s| s.id == skiff_id) {
            self.settle(i)
        } else {
            0.0
        }
    }

    fn crew_metabolism(&mut self) {
        let eat = self.tide == Tide::Flood;
        let n_people = self
            .crew
            .iter()
            .filter(|c| c.status != CrewStatus::Quit)
            .count() as f64;
        let mut fed = true;
        if eat {
            let need = n_people * FOOD_PER_PERSON_DAY;
            if self.food >= need {
                self.food -= need;
            } else {
                self.food = 0.0;
                fed = false;
                self.note("Cookshack's empty. People get mean on coffee.", "camp");
            }
        }
        let mut cook_mod = 1.0 + 0.12 * f64::from(self.cookshack - 1);
        if let Some(cook) = self.cook() {
            cook_mod += 0.08 * (f64::from(cook.skill) / 10.0);
        } else {
            cook_mod *= 0.55;
        }
        let rest_mod = 1.0 + 0.10 * f64::from(self.bunkhouse - 1);
        let last_open = self.last_open;
        let n_crew = self.crew.len();
        let mut quit_notes = Vec::new();
        let mut rest_notes = Vec::new();
        for i in 0..n_crew {
            if self.crew[i].status == CrewStatus::Quit {
                continue;
            }
            if self.crew[i].status == CrewStatus::Sick {
                self.crew[i].energy = (self.crew[i].energy + 6.0 * rest_mod).min(70.0);
                if self.crew[i].energy > 45.0 && self.crew[i].hunger < 70.0 {
                    self.crew[i].status = CrewStatus::Resting;
                }
                continue;
            }
            if eat {
                if fed {
                    self.crew[i].hunger = (self.crew[i].hunger - 28.0 * cook_mod).max(0.0);
                    self.crew[i].morale = (self.crew[i].morale + 1.2).min(100.0);
                } else {
                    self.crew[i].hunger = (self.crew[i].hunger + 22.0).min(100.0);
                }
            }
            if self.crew[i].status == CrewStatus::Resting {
                self.crew[i].energy = (self.crew[i].energy + 16.0 * rest_mod).min(100.0);
                if self.crew[i].energy > 70.0 {
                    self.crew[i].status = CrewStatus::Working;
                }
            } else if self.crew[i].assigned.is_none()
                && self.crew[i].role != "cook"
                && !self.crew[i].is_owner
            {
                self.crew[i].energy = (self.crew[i].energy + 6.0 * rest_mod).min(100.0);
                if !last_open {
                    self.crew[i].morale = (self.crew[i].morale - 0.4).max(0.0);
                } else {
                    self.crew[i].morale = (self.crew[i].morale - 1.1).max(0.0);
                }
            }
            if self.crew[i].hunger > 70.0 {
                self.crew[i].morale = (self.crew[i].morale - 6.0).max(0.0);
            }
            if self.crew[i].hunger > 90.0 {
                self.crew[i].morale = (self.crew[i].morale - 8.0).max(0.0);
                self.crew[i].energy = (self.crew[i].energy - 5.0).max(0.0);
            }
            if self.crew[i].energy < 15.0 {
                self.crew[i].status = CrewStatus::Resting;
                rest_notes.push(self.crew[i].name.clone());
            }
            if self.crew[i].morale < 8.0 && !self.crew[i].is_owner && self.rng.random() < 0.10 {
                self.crew[i].status = CrewStatus::Quit;
                self.crew[i].assigned = None;
                quit_notes.push(self.crew[i].name.clone());
            }
            if self.crew[i].role == "cook" && self.crew[i].status == CrewStatus::Working {
                self.crew[i].energy = (self.crew[i].energy - 4.0).max(20.0);
            }
        }
        for name in rest_notes {
            self.note(format!("{name} is cooked. Bunkhouse."), "crew");
        }
        for name in quit_notes {
            self.note(format!("{name} quit. Caught the next ride."), "crew");
            self.notable.push(format!("{name} quit"));
        }
    }

    fn payroll(&mut self) {
        self.payday_counter += 1;
        if self.payday_counter < 14 {
            return;
        }
        self.payday_counter = 0;
        let mut bill = 0.0;
        for c in &mut self.crew {
            if c.status == CrewStatus::Quit {
                continue;
            }
            if c.daily_wage != 0 {
                bill += f64::from(c.daily_wage) * 7.0;
            }
            if c.accrued_share > 0.0 {
                bill += c.accrued_share;
                c.accrued_share = 0.0;
            }
        }
        if bill <= 0.0 {
            return;
        }
        if self.ledger.cash >= bill {
            self.ledger.cash -= bill;
            self.ledger.wages += bill;
            let cash = self.ledger.cash;
            self.note(
                format!("Payday. Crew ${bill:.0}. Cash ${cash:.0}."),
                "ledger",
            );
            for c in &mut self.crew {
                if c.status != CrewStatus::Quit {
                    c.morale = (c.morale + 6.0).min(100.0);
                }
            }
        } else {
            self.ledger.cash -= bill;
            self.ledger.wages += bill;
            self.note(
                format!("Payday short. Still owe the crew ${bill:.0}. Books in the hole."),
                "ledger",
            );
            self.scare_crew(15.0);
        }
    }

    fn repair_idle(&mut self) {
        if self.tide != Tide::Ebb {
            return;
        }
        let loft = 2.0 * f64::from(self.net_loft);
        for net in &mut self.nets {
            if !net.in_water && net.condition < 100.0 && self.twine > 0 {
                net.condition = (net.condition + 6.0 + loft).min(100.0);
            }
        }
        for s in &mut self.skiffs {
            if s.location == "camp" && s.job == SkiffJob::Idle && !s.wrecked {
                s.condition = (s.condition + 1.0).min(96.0);
            }
        }
    }

    fn check_fail(&mut self) {
        if self.ledger.cash < -400.0 && self.food <= 0.0 {
            self.finish(GameEnd::Bankrupt);
            return;
        }
        if self.ledger.cash < -2500.0 {
            self.finish(GameEnd::Bankrupt);
            return;
        }
        if !self.skiffs.iter().any(|s| !s.wrecked) {
            self.finish(GameEnd::NoSkiffs);
            return;
        }
        let deck = self.crew.iter().any(|c| {
            matches!(
                c.status,
                CrewStatus::Working | CrewStatus::Resting | CrewStatus::Sick
            ) && c.role != "cook"
        });
        if !deck {
            self.finish(GameEnd::NoCrew);
        }
    }

    pub fn finish(&mut self, end: GameEnd) {
        self.end = end;
        match end {
            GameEnd::Bankrupt => {
                self.note(
                    "Broke. Can't buy fuel, food, or a payday. That's the season.",
                    "end",
                );
            }
            GameEnd::NoSkiffs => {
                self.note(
                    "No skiff will start. You watch the corks from the beach.",
                    "end",
                );
            }
            GameEnd::NoCrew => {
                self.note(
                    "Nobody left who will pick. Permit card doesn't lift fish.",
                    "end",
                );
            }
            _ => {
                self.note("September. Camps pulling. You weigh the books.", "end");
            }
        }
    }

    pub fn deploy_net(&mut self, net_id: &str, site_id: &str) -> String {
        let Some(sdef) = site(site_id) else {
            return "That's not your water. Seiners, or someone else's lease.".into();
        };
        if sdef.travel_from(self.camp.id).is_none() || !sdef.legal_setnet {
            if !sdef.legal_setnet {
                return "Seine water. Your S04K doesn't set here.".into();
            }
            return "That's not your water. Seiners, or someone else's lease.".into();
        }
        let Some(ni) = self.nets.iter().position(|n| n.id == net_id) else {
            return "No such net.".into();
        };
        for (j, other) in self.nets.iter().enumerate() {
            if j != ni && other.in_water && other.site_id.as_deref() == Some(site_id) {
                return "900 feet between sets (game: one net to a site). Waived in some inner Alitak — not here.".into();
            }
        }
        let (open, reason) = self.site_is_open(site_id);
        if !open {
            return format!("Dark. {reason}. Nets stay on the beach.");
        }
        if self.skipper_in_town {
            return "Permit holder has to be on the site.".into();
        }
        self.nets[ni].site_id = Some(site_id.into());
        self.nets[ni].in_water = true;
        self.nets[ni].soak_tides = 0;
        let lease = if sdef.lease == "lease" {
            "Your DNR lease."
        } else {
            "Unleased — scramble with the neighbors."
        };
        let nid = self.nets[ni].id.clone();
        self.note(format!("{nid} in at {}. {lease}", sdef.short), "gear");
        format!("{nid} fishing {}.", sdef.name)
    }

    pub fn pull_net(&mut self, net_id: &str) -> String {
        let Some(ni) = self.nets.iter().position(|n| n.id == net_id) else {
            return "That net is already on the beach.".into();
        };
        if !self.nets[ni].in_water {
            return "That net is already on the beach.".into();
        }
        let where_ = self.nets[ni].site_id.clone();
        let leftover = self.nets[ni].fish.total();
        self.nets[ni].in_water = false;
        self.nets[ni].site_id = None;
        let nid = self.nets[ni].id.clone();
        if leftover > 0.0 {
            self.note(
                format!(
                    "Pulled {nid} off {}. {leftover:.0} lb still in the web.",
                    where_.unwrap_or_default()
                ),
                "gear",
            );
        } else {
            self.note(format!("Pulled {nid}."), "gear");
        }
        format!("{nid} on the beach.")
    }

    pub fn set_mesh(&mut self, net_id: &str, mesh: &str) -> String {
        if !matches!(mesh, "pink" | "mixed" | "red") {
            return "Mesh knob: pink / mixed / red. Not a legal stretched-mesh — Kodiak doesn't spec the gillnet.".into();
        }
        let Some(ni) = self.nets.iter().position(|n| n.id == net_id) else {
            return "No such net.".into();
        };
        if self.nets[ni].in_water {
            return "Pull it first.".into();
        }
        self.nets[ni].mesh = mesh.into();
        format!("{} hung {mesh} (game selectivity).", self.nets[ni].id)
    }

    pub fn assign_skiff(&mut self, skiff_id: &str, job: &str, site_id: Option<&str>) -> String {
        let Some(si) = self.skiffs.iter().position(|s| s.id == skiff_id) else {
            return "Skiff's done.".into();
        };
        if self.skiffs[si].wrecked {
            return "Skiff's done.".into();
        }
        let Some(job_e) = SkiffJob::parse(job) else {
            return "Jobs: pick, tender, idle, town, repair.".into();
        };
        if job == "pick" {
            if let Some(sid) = site_id {
                if !is_playable(sid) {
                    return "Not a setnet site.".into();
                }
            }
        }
        self.skiffs[si].job = job_e;
        self.skiffs[si].job_site = site_id.map(|s| s.to_string());
        let name = self.skiffs[si].name.clone();
        let extra = site_id.map(|s| format!(" @ {s}")).unwrap_or_default();
        self.note(format!("{name} → {job}{extra}"), "skiff");
        format!("{name} assigned {job}.")
    }

    pub fn assign_crew(&mut self, crew_id: &str, where_: &str) -> String {
        let Some(ci) = self.crew.iter().position(|c| c.id == crew_id) else {
            return "They're gone.".into();
        };
        if self.crew[ci].status == CrewStatus::Quit {
            return "They're gone.".into();
        }
        if where_ == "cook" {
            self.crew[ci].assigned = Some("cook".into());
            self.crew[ci].role = "cook".into();
            return format!("{} in the cookshack.", self.crew[ci].name);
        }
        if where_ == "camp" {
            self.crew[ci].assigned = None;
            self.crew[ci].status = CrewStatus::Resting;
            return format!("{} in the bunkhouse.", self.crew[ci].name);
        }
        let Some(si) = self.skiffs.iter().position(|s| s.id == where_) else {
            return "No such skiff.".into();
        };
        let cid = self.crew[ci].id.clone();
        for s in &mut self.skiffs {
            s.crew_ids.retain(|id| id != &cid);
        }
        self.skiffs[si].crew_ids.push(cid);
        self.crew[ci].assigned = Some(self.skiffs[si].id.clone());
        self.crew[ci].status = CrewStatus::Working;
        format!("{} on {}.", self.crew[ci].name, self.skiffs[si].name)
    }

    pub fn hire(&mut self, cand_id: &str) -> String {
        if !self.hire_pool.iter().any(|id| id == cand_id) {
            return "They're not on the beach.".into();
        }
        if !self.tender.present && self.tick > 2 {
            return "Hands come out on the tender.".into();
        }
        let Some(cand) = candidate(cand_id) else {
            return "They're not on the beach.".into();
        };
        self.crew.push(CrewMember {
            id: cand.id.into(),
            name: cand.name.into(),
            role: cand.role.into(),
            skill: cand.skill,
            share: cand.wage_share,
            daily_wage: cand.daily_wage,
            tag: cand.tag.into(),
            energy: f64::from(cand.energy),
            hunger: f64::from(cand.hunger),
            morale: f64::from(cand.morale),
            status: CrewStatus::Working,
            assigned: if cand.role == "cook" {
                Some("cook".into())
            } else {
                None
            },
            is_owner: false,
            accrued_share: 0.0,
        });
        self.hire_pool.retain(|id| id != cand_id);
        self.note(format!("Hired {} ({}).", cand.name, cand.tag), "crew");
        format!("Hired {}.", cand.name)
    }

    pub fn buy(&mut self, item: &str) -> String {
        if !self.tender.present {
            return "Tender's not here. That's the store.".into();
        }
        let Some((qty, cost)) = buy_table(item) else {
            return "Buy: food, fuel, ice, twine, prop.".into();
        };
        if self.ledger.cash < cost {
            return "Not enough cash on the ticket book.".into();
        }
        self.ledger.cash -= cost;
        match item {
            "food" => {
                self.food += qty;
                self.ledger.food += cost;
            }
            "fuel" => {
                self.fuel_cache += qty;
                self.ledger.fuel += cost;
            }
            "ice" => {
                self.ice_cache += qty;
                for s in &mut self.skiffs {
                    if s.kind == "holding" {
                        s.ice += qty * 0.6;
                    }
                }
                self.ledger.other += cost;
            }
            "twine" => {
                self.twine += 2;
                for n in &mut self.nets {
                    n.condition = (n.condition + 12.0).min(100.0);
                }
                self.ledger.gear += cost;
            }
            "prop" => {
                self.spare_prop += 1;
                self.ledger.repairs += cost;
            }
            _ => {}
        }
        self.note(format!("Bought {item} for ${cost:.0}."), "ledger");
        format!("Bought {item} (${cost:.0}).")
    }

    pub fn upgrade(&mut self, what: &str) -> String {
        if !self.tender.present {
            return "Freight comes on the tender.".into();
        }
        let cost = match what {
            "cookshack" => 1800.0,
            "bunkhouse" => 1600.0,
            "loft" => 1400.0,
            "stall" => 2200.0,
            _ => return "Upgrade: cookshack, bunkhouse, loft, stall.".into(),
        };
        if self.ledger.cash < cost {
            return "Can't cover the freight.".into();
        }
        self.ledger.cash -= cost;
        self.ledger.other += cost;
        match what {
            "cookshack" => self.cookshack += 1,
            "bunkhouse" => self.bunkhouse += 1,
            "loft" => self.net_loft += 1,
            "stall" => self.skiff_stalls += 1,
            _ => {}
        }
        self.note(format!("Upgraded {what} (${cost:.0})."), "camp");
        format!("{what} is better. ${cost:.0}.")
    }

    pub fn form_joint_venture(&mut self) -> String {
        if self.joint_venture {
            return "Already doubled up.".into();
        }
        if !self.tender.present {
            return "The other card comes out on a tender.".into();
        }
        let cost = 3500.0;
        if self.ledger.cash < cost {
            return "Need cash on hand to make a joint venture work.".into();
        }
        self.ledger.cash -= cost;
        self.ledger.other += cost;
        self.joint_venture = true;
        let fm = if self.district() == "central" { 100 } else { 75 };
        self.nets.push(Net::new("net-3", None, fm, "mixed"));
        self.note(
            "Joint venture with a second S04K. Three nets, 300 fm (350 Central). They're on the beach.",
            "gear",
        );
        "Joint venture is in. Third net in the loft.".into()
    }

    pub fn recap(&mut self) -> Recap {
        let drift = self.rng.uniform(-0.04, 0.08);
        let settle = self.ledger.gross * drift;
        let cash = self.ledger.cash + settle;
        let stayed: Vec<String> = self
            .crew
            .iter()
            .filter(|c| c.status != CrewStatus::Quit)
            .map(|c| c.name.clone())
            .collect();
        let net = self.ledger.net() + settle;
        let (grade, nick) = grade(net, self.end, stayed.len(), self.landed.total());
        Recap {
            survived: matches!(self.end, GameEnd::None | GameEnd::Season),
            end: if self.end == GameEnd::None {
                GameEnd::Season
            } else {
                self.end
            },
            year: self.year,
            camp: self.camp.name.into(),
            lbs: self.landed,
            gross: self.ledger.gross,
            expenses: self.ledger.expenses(),
            net,
            settle,
            crew_stayed: stayed,
            notable: self.notable.iter().rev().take(10).rev().cloned().collect(),
            grade: grade.into(),
            nickname: nick.into(),
            cash,
            tickets: self.ledger.tickets,
        }
    }

    pub fn phase_label(&self) -> String {
        phase_label(self.day, self.odd_year(), self.district())
    }

    pub fn any_open(&self) -> bool {
        self.playable_sites()
            .iter()
            .any(|s| self.site_is_open(s.id).0)
    }
}

fn grade(net: f64, end: GameEnd, stayed: usize, _lbs: f64) -> (&'static str, &'static str) {
    match end {
        GameEnd::Bankrupt => ("F", "Sold the permit talk"),
        GameEnd::NoSkiffs => ("F", "Walked home"),
        GameEnd::NoCrew => ("D", "Cooked it alone"),
        _ if net >= 18000.0 && stayed >= 3 => ("A", "Salmon King"),
        _ if net >= 8000.0 => ("B", "Made the nut"),
        _ if net >= 0.0 => ("C", "Winter in the black"),
        _ if net >= -5000.0 => ("D", "Winter in the red"),
        _ => ("F", "Books in the stove"),
    }
}

pub fn new_game(seed: u64, camp_id: &str, year: i32) -> Result<Game, String> {
    let camp = *camp_by_id(camp_id).ok_or_else(|| format!("Unknown camp {camp_id}"))?;
    let mut rng = Rng::new(seed);
    let mods = generate_run_mods(year, &mut rng);
    let openings = generate_openings(year, camp.district, &mods, &mut rng);
    let prices = prices_for_year(year);
    let fm = if camp.district == "central" { 87 } else { 75 };
    let home = camp.home_sites;
    let tender_name = rng
        .choice(&["M/V Kodiak Star", "M/V Pacific Pearl", "M/V Uyak Provider"])
        .to_string();
    let mut g = Game {
        rng,
        year,
        camp,
        day: season_start(year),
        tide: Tide::Flood,
        tick: 0,
        weather: Weather::default(),
        nets: vec![
            Net::new("net-1", Some(home[0].into()), fm, "mixed"),
            Net::new(
                "net-2",
                Some(home.get(1).copied().unwrap_or(home[0]).into()),
                fm,
                "mixed",
            ),
        ],
        skiffs: vec![
            {
                let mut s = Skiff::new("skiff-1", "Picking skiff", "picker", "camp");
                s.fuel = 30.0;
                s.ice = 0.0;
                s
            },
            {
                let mut s = Skiff::new("skiff-2", "Holding skiff", "holding", "camp");
                s.fuel = 32.0;
                s.ice = 14.0;
                s
            },
        ],
        crew: Vec::new(),
        hire_pool: Vec::new(),
        tender: Tender {
            present: true,
            stay: 3,
            eta_tides: 0,
            prices,
            name: tender_name,
            last_ticket: 0.0,
            last_lbs: 0.0,
            last_note: "No ticket yet.".into(),
            late: false,
        },
        ledger: Ledger::with_cash(f64::from(camp.starting_cash)),
        log: Vec::new(),
        openings,
        mods,
        food: 55.0,
        fuel_cache: 110.0,
        twine: 3,
        spare_prop: 1,
        ice_cache: 12.0,
        bunkhouse: 1,
        cookshack: 1,
        net_loft: 1,
        skiff_stalls: 2,
        joint_venture: false,
        skipper_in_town: false,
        skipper_town_eta: 0,
        wildlife: new_wildlife(),
        bonus_school: 0,
        extend_open_tides: 0,
        landed: Lbs::default(),
        end: GameEnd::None,
        notable: Vec::new(),
        midseason_cut: false,
        last_open: false,
        payday_counter: 0,
    };

    let owner = CrewMember {
        id: "owner".into(),
        name: OWNER_NAME.into(),
        role: "operator".into(),
        skill: 7,
        share: 0.0,
        daily_wage: 0,
        tag: "S04K permit holder".into(),
        energy: 80.0,
        hunger: 15.0,
        morale: 75.0,
        status: CrewStatus::Working,
        assigned: Some("skiff-1".into()),
        is_owner: true,
        accrued_share: 0.0,
    };
    g.crew.push(owner);
    g.skiffs[0].crew_ids.push("owner".into());

    for cid in starting_hire(camp_id) {
        let cand = candidate(cid).expect("candidate");
        let assigned = if cand.role == "cook" {
            Some("cook".into())
        } else {
            Some("skiff-1".into())
        };
        g.crew.push(CrewMember {
            id: cand.id.into(),
            name: cand.name.into(),
            role: cand.role.into(),
            skill: cand.skill,
            share: cand.wage_share,
            daily_wage: cand.daily_wage,
            tag: cand.tag.into(),
            energy: f64::from(cand.energy),
            hunger: f64::from(cand.hunger),
            morale: f64::from(cand.morale),
            status: CrewStatus::Working,
            assigned,
            is_owner: false,
            accrued_share: 0.0,
        });
        if cand.role != "cook" {
            g.skiffs[0].crew_ids.push(cand.id.into());
        }
    }
    g.hire_pool = CANDIDATES
        .iter()
        .filter(|c| !starting_hire(camp_id).contains(&c.id))
        .map(|c| c.id.to_string())
        .collect();
    g.skiffs[0].job = SkiffJob::Pick;
    g.skiffs[0].job_site = Some(home[0].into());

    let pink_line = if g.odd_year() {
        "Odd-year pink flood."
    } else {
        "Even-year pinks are thin."
    };
    g.note(
        format!(
            "{}. S04K. Two nets, {} fm aggregate. {pink_line}",
            camp.name, camp.max_fathoms
        ),
        "info",
    );
    if g.mods.karluk_early_fail && camp.district == "central" {
        g.note(
            "Karluk early looks weak. After the June tests the Central Section may stay dark until July 6.",
            "adfg",
        );
    }
    let px = prices_for_year(year);
    g.note(
        format!(
            "Opening estimate: red ${:.2}  pink ${:.2}/lb.",
            px.red, px.pink
        ),
        "market",
    );
    Ok(g)
}

pub fn run_headless(game: &mut Game, ticks: Option<i32>) -> Recap {
    let limit = ticks.unwrap_or_else(|| tides_in_season(game.year) + 4);
    for _ in 0..limit {
        if game.end != GameEnd::None {
            break;
        }
        ai(game);
        game.step();
    }
    if game.end == GameEnd::None {
        game.finish(GameEnd::Season);
    }
    game.recap()
}

fn ai(game: &mut Game) {
    let n_nets = game.nets.len();
    for i in 0..n_nets {
        let id = game.nets[i].id.clone();
        if game.nets[i].in_water {
            if let Some(sid) = game.nets[i].site_id.clone() {
                if !game.site_is_open(&sid).0 {
                    game.pull_net(&id);
                }
            }
        }
        if !game.nets[i].in_water {
            let mut candidates: Vec<String> = game
                .camp
                .home_sites
                .iter()
                .map(|s| (*s).to_string())
                .collect();
            for s in game.playable_sites() {
                if !candidates.iter().any(|c| c == s.id) {
                    candidates.push(s.id.to_string());
                }
            }
            for site_id in candidates {
                let open = game.site_is_open(&site_id).0;
                let taken = game
                    .nets
                    .iter()
                    .any(|n| n.in_water && n.site_id.as_deref() == Some(site_id.as_str()));
                if open && !taken {
                    game.deploy_net(&id, &site_id);
                    break;
                }
            }
        }
    }

    if game.skiffs.len() < 2 {
        return;
    }
    if !game.skiffs[0].wrecked {
        if game.tender.present && game.skiffs[0].cargo.total() > 40.0 {
            let id = game.skiffs[0].id.clone();
            game.assign_skiff(&id, "tender", None);
        } else {
            let site = game
                .nets
                .iter()
                .find(|n| n.in_water)
                .and_then(|n| n.site_id.clone())
                .unwrap_or_else(|| game.camp.home_sites[0].to_string());
            let id = game.skiffs[0].id.clone();
            game.assign_skiff(&id, "pick", Some(&site));
        }
    }
    if !game.skiffs[1].wrecked {
        if game.tender.present && game.skiffs[1].cargo.total() > 40.0 {
            let id = game.skiffs[1].id.clone();
            game.assign_skiff(&id, "tender", None);
        } else if game.tender.present
            && game.skiffs[0].cargo.total() > 40.0
            && game.skiffs[0].job != SkiffJob::Tender
        {
            let id = game.skiffs[1].id.clone();
            game.assign_skiff(&id, "tender", None);
        } else if let Some(site) = game
            .nets
            .iter()
            .find(|n| n.in_water)
            .and_then(|n| n.site_id.clone())
        {
            let id = game.skiffs[1].id.clone();
            game.assign_skiff(&id, "pick", Some(&site));
        }
    }
    if game.tender.present {
        if game.food < 12.0 {
            game.buy("food");
        }
        if game.fuel_cache < 25.0 {
            game.buy("fuel");
        }
        if game.ice_cache < 6.0 {
            game.buy("ice");
        }
        if game.nets.iter().any(|n| n.condition < 55.0) {
            game.buy("twine");
        }
        let deck_empty = game
            .working_deck()
            .iter()
            .filter(|c| !c.is_owner)
            .count()
            == 0;
        if deck_empty {
            let pool = game.hire_pool.clone();
            for cid in pool {
                if let Some(cand) = candidate(&cid) {
                    if cand.role == "picker" || cand.role == "operator" {
                        game.hire(&cid);
                        let s1 = game.skiffs[0].id.clone();
                        game.assign_crew(&cid, &s1);
                        break;
                    }
                }
            }
        }
    }
}
