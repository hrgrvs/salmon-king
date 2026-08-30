"""Event table. Mammal behavior matches the fishery, not a generic tycoon."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class EventDef:
    id: str
    weight: float
    title: str
    kind: str  # weather, mammal, mechanical, market, adfg, camp, crew


EVENT_TABLE: tuple[EventDef, ...] = (
    EventDef("williwaw", 1.1, "Williwaw", "weather"),
    EventDef("advection_fog", 1.0, "Advection fog", "weather"),
    EventDef("shelikof_seas", 0.9, "Shelikof seas", "weather"),
    EventDef("whale_pass_rip", 0.35, "Whale Passage rip", "weather"),
    EventDef("september_winter", 0.5, "September weather", "weather"),
    EventDef("sea_lion_raid", 1.2, "Steller sea lions", "mammal"),
    EventDef("harbor_seal_raid", 0.8, "Harbor seals", "mammal"),
    EventDef("otter_foul", 0.45, "Sea otter / gear foul", "mammal"),
    EventDef("orca_residents", 0.35, "Resident killer whales", "mammal"),
    EventDef("orca_transients", 0.30, "Transient killer whales", "mammal"),
    EventDef("porpoise_bycatch", 0.08, "Harbor porpoise", "mammal"),
    EventDef("humpback_wrap", 0.12, "Humpback", "mammal"),
    EventDef("outboard_down", 0.55, "Outboard down", "mechanical"),
    EventDef("net_tear", 0.70, "Torn net", "mechanical"),
    EventDef("skiff_swamped", 0.25, "Skiff swamped", "mechanical"),
    EventDef("ice_failure", 0.40, "Ice / RSW failure", "mechanical"),
    EventDef("tender_late", 0.55, "Tender late", "market"),
    EventDef("price_cut", 0.40, "Price cut", "market"),
    EventDef("cannery_dark", 0.22, "Cannery dark", "market"),
    EventDef("adfg_extension", 0.35, "ADF&G extension", "adfg"),
    EventDef("adfg_pulse", 0.30, "ADF&G pulse", "adfg"),
    EventDef("food_spoil", 0.28, "Food spoilage", "camp"),
    EventDef("crew_fight", 0.30, "Crew fight", "crew"),
    EventDef("bonus_school", 0.45, "Fish showing", "adfg"),
)
