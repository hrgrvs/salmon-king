use crate::data::species::SpeciesId;
use crate::sim::clock::GameDate;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkiffJob {
    Idle,
    Pick,
    Tender,
    Town,
    Repair,
}

impl SkiffJob {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Pick => "pick",
            Self::Tender => "tender",
            Self::Town => "town",
            Self::Repair => "repair",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "idle" => Some(Self::Idle),
            "pick" => Some(Self::Pick),
            "tender" => Some(Self::Tender),
            "town" => Some(Self::Town),
            "repair" => Some(Self::Repair),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrewStatus {
    Working,
    Resting,
    Sick,
    Quit,
    Town,
}

impl CrewStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Working => "working",
            Self::Resting => "resting",
            Self::Sick => "sick",
            Self::Quit => "quit",
            Self::Town => "town",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameEnd {
    None,
    Season,
    Bankrupt,
    NoSkiffs,
    NoCrew,
}

impl GameEnd {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Season => "season",
            Self::Bankrupt => "bankrupt",
            Self::NoSkiffs => "no_skiffs",
            Self::NoCrew => "no_crew",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Lbs {
    pub king: f64,
    pub red: f64,
    pub pink: f64,
    pub chum: f64,
    pub silver: f64,
}

impl Lbs {
    pub fn add(&mut self, species: SpeciesId, amount: f64) {
        *self.get_mut(species) += amount;
    }

    pub fn get(self, species: SpeciesId) -> f64 {
        match species {
            SpeciesId::King => self.king,
            SpeciesId::Red => self.red,
            SpeciesId::Pink => self.pink,
            SpeciesId::Chum => self.chum,
            SpeciesId::Silver => self.silver,
        }
    }

    pub fn get_mut(&mut self, species: SpeciesId) -> &mut f64 {
        match species {
            SpeciesId::King => &mut self.king,
            SpeciesId::Red => &mut self.red,
            SpeciesId::Pink => &mut self.pink,
            SpeciesId::Chum => &mut self.chum,
            SpeciesId::Silver => &mut self.silver,
        }
    }

    pub fn total(self) -> f64 {
        self.king + self.red + self.pink + self.chum + self.silver
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Prices {
    pub king: f64,
    pub red: f64,
    pub pink: f64,
    pub chum: f64,
    pub silver: f64,
}

impl Prices {
    pub fn get(&self, species: SpeciesId) -> f64 {
        match species {
            SpeciesId::King => self.king,
            SpeciesId::Red => self.red,
            SpeciesId::Pink => self.pink,
            SpeciesId::Chum => self.chum,
            SpeciesId::Silver => self.silver,
        }
    }

    pub fn get_mut(&mut self, species: SpeciesId) -> &mut f64 {
        match species {
            SpeciesId::King => &mut self.king,
            SpeciesId::Red => &mut self.red,
            SpeciesId::Pink => &mut self.pink,
            SpeciesId::Chum => &mut self.chum,
            SpeciesId::Silver => &mut self.silver,
        }
    }

    pub fn scale_all(&mut self, factor: f64) {
        self.king *= factor;
        self.red *= factor;
        self.pink *= factor;
        self.chum *= factor;
        self.silver *= factor;
    }
}

#[derive(Clone, Debug)]
pub struct Weather {
    pub wind_kt: f64,
    pub seas_ft: f64,
    pub fog: f64,
    pub precip: String,
    pub williwaw: bool,
    pub label: String,
}

impl Default for Weather {
    fn default() -> Self {
        Self {
            wind_kt: 12.0,
            seas_ft: 2.0,
            fog: 0.1,
            precip: "overcast".into(),
            williwaw: false,
            label: "northwest 12, 2 ft".into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Net {
    pub id: String,
    pub site_id: Option<String>,
    pub fathoms: i32,
    pub mesh: String,
    pub condition: f64,
    pub soak_tides: i32,
    pub fish: Lbs,
    pub quality: f64,
    pub sea_lion_pressure: f64,
    pub in_water: bool,
}

impl Net {
    pub fn new(id: impl Into<String>, site_id: Option<String>, fathoms: i32, mesh: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            site_id,
            fathoms,
            mesh: mesh.into(),
            condition: 100.0,
            soak_tides: 0,
            fish: Lbs::default(),
            quality: 1.0,
            sea_lion_pressure: 0.0,
            in_water: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Skiff {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub location: String,
    pub dest: Option<String>,
    pub eta: i32,
    pub fuel: f64,
    pub condition: f64,
    pub job: SkiffJob,
    pub job_site: Option<String>,
    pub cargo: Lbs,
    pub cargo_quality: f64,
    pub ice: f64,
    pub wrecked: bool,
    pub crew_ids: Vec<String>,
}

impl Skiff {
    pub fn new(id: impl Into<String>, name: impl Into<String>, kind: impl Into<String>, location: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind: kind.into(),
            location: location.into(),
            dest: None,
            eta: 0,
            fuel: 28.0,
            condition: 90.0,
            job: SkiffJob::Idle,
            job_site: None,
            cargo: Lbs::default(),
            cargo_quality: 1.0,
            ice: 0.0,
            wrecked: false,
            crew_ids: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CrewMember {
    pub id: String,
    pub name: String,
    pub role: String,
    pub skill: i32,
    pub share: f64,
    pub daily_wage: i32,
    pub tag: String,
    pub energy: f64,
    pub hunger: f64,
    pub morale: f64,
    pub status: CrewStatus,
    pub assigned: Option<String>,
    pub is_owner: bool,
    pub accrued_share: f64,
}

#[derive(Clone, Debug)]
pub struct Tender {
    pub present: bool,
    pub eta_tides: i32,
    pub stay: i32,
    pub name: String,
    pub prices: Prices,
    pub last_ticket: f64,
    pub last_lbs: f64,
    pub last_note: String,
    pub late: bool,
}

#[derive(Clone, Debug)]
pub struct Ledger {
    pub cash: f64,
    pub gross: f64,
    pub wages: f64,
    pub food: f64,
    pub fuel: f64,
    pub gear: f64,
    pub repairs: f64,
    pub fines: f64,
    pub other: f64,
    pub tickets: i32,
}

impl Ledger {
    pub fn with_cash(cash: f64) -> Self {
        Self {
            cash,
            gross: 0.0,
            wages: 0.0,
            food: 0.0,
            fuel: 0.0,
            gear: 0.0,
            repairs: 0.0,
            fines: 0.0,
            other: 0.0,
            tickets: 0,
        }
    }

    pub fn expenses(&self) -> f64 {
        self.wages + self.food + self.fuel + self.gear + self.repairs + self.fines + self.other
    }

    pub fn net(&self) -> f64 {
        self.gross - self.expenses()
    }
}

#[derive(Clone, Debug)]
pub struct LogLine {
    pub tick: i32,
    pub day: GameDate,
    pub text: String,
    pub kind: String,
}

#[derive(Clone, Debug)]
pub struct OpeningWindow {
    pub start_doy: i32,
    pub end_doy: i32,
    pub hours: i32,
    pub district: String,
    pub reason: String,
    pub pulse_sites_only: bool,
}

#[derive(Clone, Debug)]
pub struct RunMods {
    pub karluk_early: f64,
    pub karluk_late: f64,
    pub frazer: f64,
    pub upper_station: f64,
    pub pink: f64,
    pub chum: f64,
    pub silver: f64,
    pub king: f64,
    pub pink_flood: bool,
    pub karluk_early_fail: bool,
    pub frazer_goals_ok: bool,
}

impl Default for RunMods {
    fn default() -> Self {
        Self {
            karluk_early: 1.0,
            karluk_late: 1.0,
            frazer: 1.0,
            upper_station: 1.0,
            pink: 1.0,
            chum: 1.0,
            silver: 1.0,
            king: 0.35,
            pink_flood: false,
            karluk_early_fail: false,
            frazer_goals_ok: false,
        }
    }
}
