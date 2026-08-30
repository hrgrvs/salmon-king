//! Event table. Mammal behavior matches the fishery, not a generic tycoon.

#[derive(Clone, Copy, Debug)]
pub struct EventDef {
    pub id: &'static str,
    pub weight: f64,
    pub title: &'static str,
    pub kind: &'static str,
}

pub const EVENT_TABLE: [EventDef; 24] = [
    EventDef { id: "williwaw", weight: 1.1, title: "Williwaw", kind: "weather" },
    EventDef { id: "advection_fog", weight: 1.0, title: "Advection fog", kind: "weather" },
    EventDef { id: "shelikof_seas", weight: 0.9, title: "Shelikof seas", kind: "weather" },
    EventDef { id: "whale_pass_rip", weight: 0.35, title: "Whale Passage rip", kind: "weather" },
    EventDef { id: "september_winter", weight: 0.5, title: "September weather", kind: "weather" },
    EventDef { id: "sea_lion_raid", weight: 1.2, title: "Steller sea lions", kind: "mammal" },
    EventDef { id: "harbor_seal_raid", weight: 0.8, title: "Harbor seals", kind: "mammal" },
    EventDef { id: "otter_foul", weight: 0.45, title: "Sea otter / gear foul", kind: "mammal" },
    EventDef { id: "orca_residents", weight: 0.35, title: "Resident killer whales", kind: "mammal" },
    EventDef { id: "orca_transients", weight: 0.30, title: "Transient killer whales", kind: "mammal" },
    EventDef { id: "porpoise_bycatch", weight: 0.08, title: "Harbor porpoise", kind: "mammal" },
    EventDef { id: "humpback_wrap", weight: 0.12, title: "Humpback", kind: "mammal" },
    EventDef { id: "outboard_down", weight: 0.55, title: "Outboard down", kind: "mechanical" },
    EventDef { id: "net_tear", weight: 0.70, title: "Torn net", kind: "mechanical" },
    EventDef { id: "skiff_swamped", weight: 0.25, title: "Skiff swamped", kind: "mechanical" },
    EventDef { id: "ice_failure", weight: 0.40, title: "Ice / RSW failure", kind: "mechanical" },
    EventDef { id: "tender_late", weight: 0.55, title: "Tender late", kind: "market" },
    EventDef { id: "price_cut", weight: 0.40, title: "Price cut", kind: "market" },
    EventDef { id: "cannery_dark", weight: 0.22, title: "Cannery dark", kind: "market" },
    EventDef { id: "adfg_extension", weight: 0.35, title: "ADF&G extension", kind: "adfg" },
    EventDef { id: "adfg_pulse", weight: 0.30, title: "ADF&G pulse", kind: "adfg" },
    EventDef { id: "food_spoil", weight: 0.28, title: "Food spoilage", kind: "camp" },
    EventDef { id: "crew_fight", weight: 0.30, title: "Crew fight", kind: "crew" },
    EventDef { id: "bonus_school", weight: 0.45, title: "Fish showing", kind: "adfg" },
];
