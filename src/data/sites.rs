//! Named water. Playable nodes are legal S04K set-gillnet water only.

use crate::data::species::SpeciesId;

#[derive(Clone, Copy, Debug)]
pub struct SiteDef {
    pub id: &'static str,
    pub name: &'static str,
    pub short: &'static str,
    pub district: &'static str,
    pub legal_setnet: bool,
    pub exposure: &'static str,
    pub travel_from_camp: &'static [(&'static str, i32)],
    pub affinity: [f64; 5],
    pub sea_lion: f64,
    pub harbor_seal: f64,
    pub lease: &'static str,
    pub note: &'static str,
    pub mark: &'static str,
}

const fn aff(king: f64, red: f64, pink: f64, chum: f64, silver: f64) -> [f64; 5] {
    [king, red, pink, chum, silver]
}

pub const SITES: [SiteDef; 25] = [
    // --- Central Section, Northwest Kodiak District (legal setnet) ---
    SiteDef {
        id: "raspberry",
        name: "Raspberry Cape",
        short: "Rasp. Cape",
        district: "central",
        legal_setnet: true,
        exposure: "shelikof",
        travel_from_camp: &[("bailey", 1), ("uganik", 2), ("larsen", 3)],
        affinity: aff(0.25, 0.70, 0.85, 0.55, 0.40),
        sea_lion: 0.55,
        harbor_seal: 0.15,
        lease: "scramble",
        note: "North end of Central Section, looking at Raspberry Island and Shelikof.",
        mark: "R",
    },
    SiteDef {
        id: "malina",
        name: "Malina Point / Onion Bay mouth",
        short: "Malina",
        district: "central",
        legal_setnet: true,
        exposure: "shelikof",
        travel_from_camp: &[("bailey", 1), ("uganik", 1), ("larsen", 3)],
        affinity: aff(0.20, 0.75, 0.90, 0.60, 0.45),
        sea_lion: 0.50,
        harbor_seal: 0.15,
        lease: "scramble",
        note: "Onion Bay mouth. Shelikof leaks in; fog hangs.",
        mark: "M",
    },
    SiteDef {
        id: "viekoda",
        name: "Outlet Cape / Viekoda mouth",
        short: "Viekoda",
        district: "central",
        legal_setnet: true,
        exposure: "shelikof",
        travel_from_camp: &[("bailey", 0), ("uganik", 1), ("larsen", 2)],
        affinity: aff(0.22, 0.80, 0.95, 0.65, 0.50),
        sea_lion: 0.48,
        harbor_seal: 0.15,
        lease: "scramble",
        note: "Viekoda Bay mouth. Classic west-side shoreline set.",
        mark: "O",
    },
    SiteDef {
        id: "dryspruce",
        name: "Dry Spruce / Port Bailey",
        short: "Dry Spruce",
        district: "central",
        legal_setnet: true,
        exposure: "shelikof",
        travel_from_camp: &[("bailey", 0), ("uganik", 1), ("larsen", 2)],
        affinity: aff(0.18, 0.85, 1.00, 0.70, 0.55),
        sea_lion: 0.42,
        harbor_seal: 0.15,
        lease: "lease",
        note: "Old cannery water. Williwaws off Kupreanof Mountain. Whale Passage rips if you run town.",
        mark: "D",
    },
    SiteDef {
        id: "uganik_pass",
        name: "Uganik Passage / outer Uganik",
        short: "Uganik Pass",
        district: "central",
        legal_setnet: true,
        exposure: "shelikof",
        travel_from_camp: &[("bailey", 1), ("uganik", 0), ("larsen", 2)],
        affinity: aff(0.20, 0.95, 1.05, 0.75, 0.60),
        sea_lion: 0.50,
        harbor_seal: 0.15,
        lease: "lease",
        note: "Outer Uganik / Northeast Arm approaches. Inner Uganik is seine-only.",
        mark: "U",
    },
    SiteDef {
        id: "spiridon_outer",
        name: "Spiridon outer / Telrod-adjacent",
        short: "Spiridon outer",
        district: "central",
        legal_setnet: true,
        exposure: "shelikof",
        travel_from_camp: &[("bailey", 2), ("uganik", 1), ("larsen", 1)],
        affinity: aff(0.15, 1.10, 0.90, 0.55, 0.45),
        sea_lion: 0.40,
        harbor_seal: 0.15,
        lease: "scramble",
        note: "Fish the outside mixed fishery. Telrod Cove SHA is seine-only.",
        mark: "S",
    },
    SiteDef {
        id: "zachar_outer",
        name: "Zachar outer / Carlsen Point",
        short: "Carlsen Pt",
        district: "central",
        legal_setnet: true,
        exposure: "shelikof",
        travel_from_camp: &[("bailey", 2), ("uganik", 1), ("larsen", 1)],
        affinity: aff(0.16, 1.00, 0.95, 0.60, 0.50),
        sea_lion: 0.38,
        harbor_seal: 0.15,
        lease: "scramble",
        note: "Carlsen Point / outer Zachar. Inner Zachar is seine-only.",
        mark: "Z",
    },
    SiteDef {
        id: "harvester",
        name: "Harvester Island / Uyak entrance",
        short: "Harvester",
        district: "central",
        legal_setnet: true,
        exposure: "protected",
        travel_from_camp: &[("bailey", 2), ("uganik", 2), ("larsen", 0)],
        affinity: aff(0.18, 1.15, 1.00, 0.70, 0.65),
        sea_lion: 0.35,
        harbor_seal: 0.28,
        lease: "lease",
        note: "Uyak entrance. More protected than open Shelikof. Inner Uyak is seine-only.",
        mark: "H",
    },
    SiteDef {
        id: "cape_uyak",
        name: "Cape Uyak / Rocky Point",
        short: "Cape Uyak",
        district: "central",
        legal_setnet: true,
        exposure: "shelikof",
        travel_from_camp: &[("bailey", 3), ("uganik", 2), ("larsen", 1)],
        affinity: aff(0.30, 1.25, 0.85, 0.50, 0.55),
        sea_lion: 0.58,
        harbor_seal: 0.15,
        lease: "scramble",
        note: "District line toward Karluk. Karluk-bound reds slide this beach. You are not fishing the lagoon.",
        mark: "C",
    },
    // --- Alitak District inner sections (set gillnet only until Sept 4) ---
    SiteDef {
        id: "lazy_bay",
        name: "Alitak Bay / Lazy Bay",
        short: "Lazy Bay",
        district: "alitak",
        legal_setnet: true,
        exposure: "south_end",
        travel_from_camp: &[("olga", 1)],
        affinity: aff(0.10, 0.90, 1.10, 0.80, 0.50),
        sea_lion: 0.30,
        harbor_seal: 0.25,
        lease: "scramble",
        note: "Alitak Bay section. Setnet-only until Sept 4, then seines also allowed.",
        mark: "A",
    },
    SiteDef {
        id: "moser",
        name: "Moser Bay",
        short: "Moser",
        district: "alitak",
        legal_setnet: true,
        exposure: "protected",
        travel_from_camp: &[("olga", 1)],
        affinity: aff(0.08, 1.05, 1.15, 0.85, 0.55),
        sea_lion: 0.22,
        harbor_seal: 0.35,
        lease: "scramble",
        note: "Inside water. Seals work the leads. Pulse openers.",
        mark: "M",
    },
    SiteDef {
        id: "olga_narrows",
        name: "Olga Bay / Olga Narrows",
        short: "Olga Narrows",
        district: "alitak",
        legal_setnet: true,
        exposure: "protected",
        travel_from_camp: &[("olga", 0)],
        affinity: aff(0.08, 1.20, 1.20, 0.90, 0.70),
        sea_lion: 0.18,
        harbor_seal: 0.40,
        lease: "lease",
        note: "Far from town. Tender is the trade wagon. 63-hour dark every ten days unless the weirs say otherwise.",
        mark: "N",
    },
    SiteDef {
        id: "dog_salmon",
        name: "Dog Salmon Flats",
        short: "Dog Sal. Flats",
        district: "alitak",
        legal_setnet: true,
        exposure: "protected",
        travel_from_camp: &[("olga", 1)],
        affinity: aff(0.06, 1.30, 0.95, 0.70, 0.60),
        sea_lion: 0.15,
        harbor_seal: 0.32,
        lease: "scramble",
        note: "Often dark unless Frazer fish are past the traditional water. Pulse site.",
        mark: "F",
    },
    SiteDef {
        id: "upper_station_outer",
        name: "Outer Upper Station",
        short: "Outer U.Sta",
        district: "alitak",
        legal_setnet: true,
        exposure: "protected",
        travel_from_camp: &[("olga", 1)],
        affinity: aff(0.05, 1.40, 0.70, 0.45, 0.75),
        sea_lion: 0.12,
        harbor_seal: 0.30,
        lease: "scramble",
        note: "Upper Station sockeye. Opens when the weir can stand it.",
        mark: "T",
    },
    SiteDef {
        id: "akalura_outer",
        name: "Outer Akalura",
        short: "Akalura",
        district: "alitak",
        legal_setnet: true,
        exposure: "protected",
        travel_from_camp: &[("olga", 1)],
        affinity: aff(0.05, 1.15, 0.75, 0.50, 0.80),
        sea_lion: 0.12,
        harbor_seal: 0.28,
        lease: "scramble",
        note: "Akalura system. Late silver water if you're still here.",
        mark: "K",
    },
    // --- Seine / NPC landmarks. Visible. You cannot set a net here. ---
    SiteDef {
        id: "karluk_lagoon",
        name: "Karluk Lagoon",
        short: "Karluk",
        district: "seine",
        legal_setnet: false,
        exposure: "landmark",
        travel_from_camp: &[],
        affinity: aff(0.0, 0.0, 0.0, 0.0, 0.0),
        sea_lion: 0.35,
        harbor_seal: 0.15,
        lease: "scramble",
        note: "Southwest Kodiak. Purse/beach seine. Karluk-bound reds pass Central Section beaches first.",
        mark: "◇",
    },
    SiteDef {
        id: "ayakulik",
        name: "Ayakulik",
        short: "Ayakulik",
        district: "seine",
        legal_setnet: false,
        exposure: "landmark",
        travel_from_camp: &[],
        affinity: aff(0.0, 0.0, 0.0, 0.0, 0.0),
        sea_lion: 0.35,
        harbor_seal: 0.15,
        lease: "scramble",
        note: "Southwest Kodiak seine water. King stock of concern.",
        mark: "◇",
    },
    SiteDef {
        id: "halibut_bay",
        name: "Halibut Bay",
        short: "Halibut Bay",
        district: "seine",
        legal_setnet: false,
        exposure: "landmark",
        travel_from_camp: &[],
        affinity: aff(0.0, 0.0, 0.0, 0.0, 0.0),
        sea_lion: 0.35,
        harbor_seal: 0.15,
        lease: "scramble",
        note: "Southwest Kodiak. Seine only.",
        mark: "◇",
    },
    SiteDef {
        id: "telrod_sha",
        name: "Telrod Cove SHA",
        short: "Telrod SHA",
        district: "seine",
        legal_setnet: false,
        exposure: "landmark",
        travel_from_camp: &[],
        affinity: aff(0.0, 0.0, 0.0, 0.0, 0.0),
        sea_lion: 0.35,
        harbor_seal: 0.15,
        lease: "scramble",
        note: "Spiridon Bay sockeye SHA. Seine-only. Setnets fish the outside mixed fishery.",
        mark: "◇",
    },
    SiteDef {
        id: "cape_alitak_seine",
        name: "Cape Alitak (seine)",
        short: "Cape Alitak",
        district: "seine",
        legal_setnet: false,
        exposure: "landmark",
        travel_from_camp: &[],
        affinity: aff(0.0, 0.0, 0.0, 0.0, 0.0),
        sea_lion: 0.35,
        harbor_seal: 0.15,
        lease: "scramble",
        note: "Seine section. Set gillnet allowed here only after September 4 (not a starting site).",
        mark: "◇",
    },
    SiteDef {
        id: "humpy_deadman",
        name: "Humpy-Deadman",
        short: "Humpy-Deadman",
        district: "seine",
        legal_setnet: false,
        exposure: "landmark",
        travel_from_camp: &[],
        affinity: aff(0.0, 0.0, 0.0, 0.0, 0.0),
        sea_lion: 0.35,
        harbor_seal: 0.15,
        lease: "scramble",
        note: "Alitak District seine section.",
        mark: "◇",
    },
    SiteDef {
        id: "afognak",
        name: "Afognak",
        short: "Afognak",
        district: "seine",
        legal_setnet: false,
        exposure: "landmark",
        travel_from_camp: &[],
        affinity: aff(0.0, 0.0, 0.0, 0.0, 0.0),
        sea_lion: 0.35,
        harbor_seal: 0.15,
        lease: "scramble",
        note: "Afognak District: purse and beach seine only.",
        mark: "◇",
    },
    SiteDef {
        id: "ugak",
        name: "Ugak Bay",
        short: "Ugak",
        district: "seine",
        legal_setnet: false,
        exposure: "landmark",
        travel_from_camp: &[],
        affinity: aff(0.0, 0.0, 0.0, 0.0, 0.0),
        sea_lion: 0.35,
        harbor_seal: 0.15,
        lease: "scramble",
        note: "Eastside. Seine only.",
        mark: "◇",
    },
    SiteDef {
        id: "womens_bay",
        name: "Women's Bay",
        short: "Women's Bay",
        district: "seine",
        legal_setnet: false,
        exposure: "landmark",
        travel_from_camp: &[],
        affinity: aff(0.0, 0.0, 0.0, 0.0, 0.0),
        sea_lion: 0.35,
        harbor_seal: 0.15,
        lease: "scramble",
        note: "Near town. Not setnet water.",
        mark: "◇",
    },
    SiteDef {
        id: "swikshak",
        name: "Mainland / Swikshak",
        short: "Swikshak",
        district: "seine",
        legal_setnet: false,
        exposure: "landmark",
        travel_from_camp: &[],
        affinity: aff(0.0, 0.0, 0.0, 0.0, 0.0),
        sea_lion: 0.35,
        harbor_seal: 0.15,
        lease: "scramble",
        note: "Mainland District. Seine only.",
        mark: "◇",
    },
];

pub const PULSE_ALITAK: &[&str] = &["dog_salmon", "upper_station_outer", "akalura_outer"];

pub struct Bay {
    pub id: &'static str,
    pub label: &'static str,
    pub sites: &'static [&'static str],
}

pub const BAYS: [Bay; 9] = [
    Bay { id: "raspberry_onion", label: "Raspberry / Onion Bay", sites: &["raspberry", "malina"] },
    Bay { id: "viekoda", label: "Viekoda / Dry Spruce", sites: &["viekoda", "dryspruce"] },
    Bay { id: "uganik", label: "outer Uganik", sites: &["uganik_pass"] },
    Bay { id: "spiridon", label: "Spiridon outer", sites: &["spiridon_outer"] },
    Bay { id: "zachar", label: "Zachar outer", sites: &["zachar_outer"] },
    Bay { id: "uyak", label: "Uyak", sites: &["harvester", "cape_uyak"] },
    Bay { id: "alitak", label: "Alitak / Moser", sites: &["lazy_bay", "moser"] },
    Bay { id: "olga", label: "Olga Bay", sites: &["olga_narrows", "dog_salmon"] },
    Bay { id: "station", label: "Upper Station / Akalura", sites: &["upper_station_outer", "akalura_outer"] },
];

impl SiteDef {
    pub fn affinity_of(&self, sp: SpeciesId) -> f64 {
        self.affinity[sp.idx()]
    }

    pub fn travel_from(&self, camp_id: &str) -> Option<i32> {
        self.travel_from_camp
            .iter()
            .find(|(c, _)| *c == camp_id)
            .map(|(_, t)| *t)
    }
}

pub fn site(id: &str) -> Option<&'static SiteDef> {
    SITES.iter().find(|s| s.id == id)
}

pub fn is_playable(id: &str) -> bool {
    site(id).map(|s| s.legal_setnet).unwrap_or(false)
}

pub fn is_pulse_alitak(id: &str) -> bool {
    PULSE_ALITAK.contains(&id)
}

pub fn bay_id_for_site(site_id: &str) -> Option<&'static str> {
    BAYS.iter()
        .find(|b| b.sites.contains(&site_id))
        .map(|b| b.id)
}

pub fn bay_by_id(bay_id: &str) -> Option<&'static Bay> {
    BAYS.iter().find(|b| b.id == bay_id)
}

pub fn sites_in_bay(bay_id: &str) -> &'static [&'static str] {
    bay_by_id(bay_id).map(|b| b.sites).unwrap_or(&[])
}

pub fn sites_for_camp(camp_id: &str) -> Vec<&'static SiteDef> {
    SITES
        .iter()
        .filter(|s| s.legal_setnet && s.travel_from(camp_id).is_some())
        .collect()
}

pub fn travel_tides(camp_id: &str, site_id: &str) -> i32 {
    site(site_id)
        .and_then(|s| s.travel_from(camp_id))
        .unwrap_or(99)
}
