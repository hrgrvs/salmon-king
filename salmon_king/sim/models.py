from __future__ import annotations

from dataclasses import dataclass, field
from datetime import date
from enum import StrEnum

from salmon_king.data.species import SpeciesId


class Tide(StrEnum):
    FLOOD = "flood"
    EBB = "ebb"


class SkiffJob(StrEnum):
    IDLE = "idle"
    PICK = "pick"
    TENDER = "tender"
    TOWN = "town"
    REPAIR = "repair"


class CrewStatus(StrEnum):
    WORKING = "working"
    RESTING = "resting"
    SICK = "sick"
    QUIT = "quit"
    TOWN = "town"


class GameEnd(StrEnum):
    NONE = ""
    SEASON = "season"
    BANKRUPT = "bankrupt"
    NO_SKIFFS = "no_skiffs"
    NO_CREW = "no_crew"


@dataclass
class Lbs:
    king: float = 0.0
    red: float = 0.0
    pink: float = 0.0
    chum: float = 0.0
    silver: float = 0.0

    def add(self, species: SpeciesId, amount: float) -> None:
        setattr(self, species.value, getattr(self, species.value) + amount)

    def get(self, species: SpeciesId) -> float:
        return float(getattr(self, species.value))

    def total(self) -> float:
        return self.king + self.red + self.pink + self.chum + self.silver

    def clear(self) -> None:
        self.king = self.red = self.pink = self.chum = self.silver = 0.0

    def as_dict(self) -> dict[SpeciesId, float]:
        return {
            SpeciesId.KING: self.king,
            SpeciesId.RED: self.red,
            SpeciesId.PINK: self.pink,
            SpeciesId.CHUM: self.chum,
            SpeciesId.SILVER: self.silver,
        }

    def drain_into(self, other: Lbs) -> None:
        for sp, amt in self.as_dict().items():
            other.add(sp, amt)
        self.clear()


@dataclass
class Weather:
    wind_kt: float = 12.0
    seas_ft: float = 2.0
    fog: float = 0.1
    precip: str = "overcast"
    williwaw: bool = False
    label: str = "northwest 12, 2 ft"


@dataclass
class Net:
    id: str
    site_id: str | None
    fathoms: int
    mesh: str  # "pink" | "mixed" | "red" — game selectivity knob, not a legal mesh spec
    condition: float = 100.0
    soak_tides: int = 0
    fish: Lbs = field(default_factory=Lbs)
    quality: float = 1.0  # 1.0 chilled-fresh → 0.4 soft/sunburn
    sea_lion_pressure: float = 0.0
    in_water: bool = False


@dataclass
class Skiff:
    id: str
    name: str
    kind: str  # "picker" | "holding"
    location: str  # site_id, "camp", "tender", "transit", "town"
    dest: str | None = None
    eta: int = 0
    fuel: float = 28.0
    condition: float = 90.0
    job: SkiffJob = SkiffJob.IDLE
    job_site: str | None = None
    cargo: Lbs = field(default_factory=Lbs)
    cargo_quality: float = 1.0
    ice: float = 0.0  # gallons-equivalent slush; 0 = unchilled
    wrecked: bool = False
    crew_ids: list[str] = field(default_factory=list)


@dataclass
class CrewMember:
    id: str
    name: str
    role: str
    skill: int
    share: float
    daily_wage: int
    trait: str
    energy: float
    hunger: float
    morale: float
    status: CrewStatus = CrewStatus.WORKING
    assigned: str | None = None  # skiff id or "cook" or "camp"
    is_owner: bool = False
    accrued_share: float = 0.0


@dataclass
class Tender:
    present: bool = False
    eta_tides: int = 4
    stay: int = 0
    name: str = "M/V Kodiak Star"
    prices: dict[SpeciesId, float] = field(default_factory=dict)
    last_ticket: float = 0.0
    last_lbs: float = 0.0
    last_note: str = "No ticket yet."
    late: bool = False


@dataclass
class Ledger:
    cash: float = 12000.0
    gross: float = 0.0
    wages: float = 0.0
    food: float = 0.0
    fuel: float = 0.0
    gear: float = 0.0
    repairs: float = 0.0
    fines: float = 0.0
    other: float = 0.0
    tickets: int = 0

    def expenses(self) -> float:
        return self.wages + self.food + self.fuel + self.gear + self.repairs + self.fines + self.other

    def net(self) -> float:
        return self.gross - self.expenses()


@dataclass
class LogLine:
    tick: int
    day: date
    text: str
    kind: str = "info"


@dataclass
class OpeningWindow:
    start_doy: int
    end_doy: int  # inclusive
    hours: int
    district: str
    reason: str
    pulse_sites_only: bool = False


@dataclass
class RunMods:
    """Season-scale abundance. Generated from seed + even/odd, not a 2026 EO."""

    karluk_early: float = 1.0
    karluk_late: float = 1.0
    frazer: float = 1.0
    upper_station: float = 1.0
    pink: float = 1.0
    chum: float = 1.0
    silver: float = 1.0
    king: float = 0.35
    pink_flood: bool = False
    karluk_early_fail: bool = False
    frazer_goals_ok: bool = False
