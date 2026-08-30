//! Salmon species tables. Prices are Kodiak prelim ex-vessel by year, not live quotes.

use crate::sim::models::Prices;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SpeciesId {
    King = 0,
    Red = 1,
    Pink = 2,
    Chum = 3,
    Silver = 4,
}

impl SpeciesId {
    pub const ALL: [Self; 5] = [Self::King, Self::Red, Self::Pink, Self::Chum, Self::Silver];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::King => "king",
            Self::Red => "red",
            Self::Pink => "pink",
            Self::Chum => "chum",
            Self::Silver => "silver",
        }
    }

    pub fn idx(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Species {
    pub id: SpeciesId,
    pub common: &'static str,
    pub local: &'static str,
    pub scientific: &'static str,
    pub avg_lb: f64,
    pub peak_doy: i32,
    pub width_days: f64,
    pub notes: &'static str,
}

pub const SPECIES: [Species; 5] = [
    Species {
        id: SpeciesId::King,
        common: "Chinook",
        local: "king",
        scientific: "Oncorhynchus tshawytscha",
        avg_lb: 5.3,
        peak_doy: 165,
        width_days: 16.0,
        notes: "No directed commercial Chinook in the KMA. Incidental, small, scarce. Karluk/Ayakulik stocks of concern.",
    },
    Species {
        id: SpeciesId::Red,
        common: "sockeye",
        local: "red",
        scientific: "Oncorhynchus nerka",
        avg_lb: 4.8,
        peak_doy: 185,
        width_days: 22.0,
        notes: "Westside: Karluk-bound fish passing Central Section beaches. Alitak: Frazer / Upper Station / Akalura.",
    },
    Species {
        id: SpeciesId::Pink,
        common: "pink",
        local: "humpy",
        scientific: "Oncorhynchus gorbuscha",
        avg_lb: 3.2,
        peak_doy: 210,
        width_days: 20.0,
        notes: "Two-year cycle; odd- and even-year lines are genetically distinct. Odd years flood; even years thin.",
    },
    Species {
        id: SpeciesId::Chum,
        common: "chum",
        local: "dog",
        scientific: "Oncorhynchus keta",
        avg_lb: 6.6,
        peak_doy: 205,
        width_days: 18.0,
        notes: "July–August mixed-bag fish. Firm when fresh; go soft if you soak them.",
    },
    Species {
        id: SpeciesId::Silver,
        common: "coho",
        local: "silver",
        scientific: "Oncorhynchus kisutch",
        avg_lb: 6.5,
        peak_doy: 245,
        width_days: 18.0,
        notes: "Mid-August through early October. Skeleton-crew fish after the pink push.",
    },
];

pub fn species(id: SpeciesId) -> &'static Species {
    &SPECIES[id.idx()]
}

/// Kodiak preliminary average $/lb by calendar year (ADF&G season summaries).
/// Used as June opening estimates. 2026 is a generated year, not a live quote.
pub fn prices_for_year(year: i32) -> Prices {
    match year {
        2023 => Prices {
            king: 0.57,
            red: 0.84,
            silver: 0.28,
            pink: 0.26,
            chum: 0.26,
        },
        2024 => Prices {
            king: 0.25,
            red: 1.16,
            silver: 0.56,
            pink: 0.29,
            chum: 0.25,
        },
        2025 => Prices {
            king: 0.10,
            red: 1.34,
            silver: 0.70,
            pink: 0.30,
            chum: 0.39,
        },
        y if y % 2 == 1 => Prices {
            king: 0.34,
            red: 1.09,
            silver: 0.49,
            pink: 0.28,
            chum: 0.32,
        },
        _ => Prices {
            king: 0.25,
            red: 1.05,
            silver: 0.50,
            pink: 0.27,
            chum: 0.28,
        },
    }
}

pub fn short_code(sp: SpeciesId) -> &'static str {
    match sp {
        SpeciesId::King => "KNG",
        SpeciesId::Red => "RED",
        SpeciesId::Pink => "PNK",
        SpeciesId::Chum => "CHM",
        SpeciesId::Silver => "SLV",
    }
}
