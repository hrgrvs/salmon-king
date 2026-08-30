"""Salmon species tables. Prices are Kodiak prelim ex-vessel by year, not live quotes."""

from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum


class SpeciesId(StrEnum):
    KING = "king"
    RED = "red"
    PINK = "pink"
    CHUM = "chum"
    SILVER = "silver"


@dataclass(frozen=True)
class Species:
    id: SpeciesId
    common: str
    local: str
    scientific: str
    avg_lb: float  # Kodiak commercial mean dressed-ish weight
    # Peak day-of-year and width (game-shaped curves, timed to real run windows)
    peak_doy: int
    width_days: float
    notes: str


SPECIES: dict[SpeciesId, Species] = {
    SpeciesId.KING: Species(
        id=SpeciesId.KING,
        common="Chinook",
        local="king",
        scientific="Oncorhynchus tshawytscha",
        avg_lb=5.3,  # incidental Kodiak fish; not SE troll kings
        peak_doy=165,  # mid-June
        width_days=16,
        notes="No directed commercial Chinook in the KMA. Incidental, small, scarce. Karluk/Ayakulik stocks of concern.",
    ),
    SpeciesId.RED: Species(
        id=SpeciesId.RED,
        common="sockeye",
        local="red",
        scientific="Oncorhynchus nerka",
        avg_lb=4.8,
        peak_doy=185,  # early July; late Karluk/Upper Station is a second pulse in-sim
        width_days=22,
        notes="Westside: Karluk-bound fish passing Central Section beaches. Alitak: Frazer / Upper Station / Akalura.",
    ),
    SpeciesId.PINK: Species(
        id=SpeciesId.PINK,
        common="pink",
        local="humpy",
        scientific="Oncorhynchus gorbuscha",
        avg_lb=3.2,
        peak_doy=210,  # late July
        width_days=20,
        notes="Two-year cycle; odd- and even-year lines are genetically distinct. Odd years flood; even years thin.",
    ),
    SpeciesId.CHUM: Species(
        id=SpeciesId.CHUM,
        common="chum",
        local="dog",
        scientific="Oncorhynchus keta",
        avg_lb=6.6,
        peak_doy=205,
        width_days=18,
        notes="July–August mixed-bag fish. Firm when fresh; go soft if you soak them.",
    ),
    SpeciesId.SILVER: Species(
        id=SpeciesId.SILVER,
        common="coho",
        local="silver",
        scientific="Oncorhynchus kisutch",
        avg_lb=6.5,
        peak_doy=245,  # early September
        width_days=18,
        notes="Mid-August through early October. Skeleton-crew fish after the pink push.",
    ),
}

# Kodiak preliminary average $/lb by calendar year (ADF&G season summaries).
# Used as June opening estimates. In-season the tender walks them around.
# 2026 is a generated year, not a live quote.
YEAR_PRICES: dict[int, dict[SpeciesId, float]] = {
    2023: {
        SpeciesId.KING: 0.57,
        SpeciesId.RED: 0.84,
        SpeciesId.SILVER: 0.28,
        SpeciesId.PINK: 0.26,
        SpeciesId.CHUM: 0.26,
    },
    2024: {
        SpeciesId.KING: 0.25,
        SpeciesId.RED: 1.16,
        SpeciesId.SILVER: 0.56,
        SpeciesId.PINK: 0.29,
        SpeciesId.CHUM: 0.25,
    },
    2025: {
        SpeciesId.KING: 0.10,
        SpeciesId.RED: 1.34,
        SpeciesId.SILVER: 0.70,
        SpeciesId.PINK: 0.30,
        SpeciesId.CHUM: 0.39,
    },
}

# Fallback when the player types a year we don't have a table for.
EVEN_YEAR_PRICES: dict[SpeciesId, float] = {
    SpeciesId.KING: 0.25,
    SpeciesId.RED: 1.05,
    SpeciesId.SILVER: 0.50,
    SpeciesId.PINK: 0.27,
    SpeciesId.CHUM: 0.28,
}
ODD_YEAR_PRICES: dict[SpeciesId, float] = {
    SpeciesId.KING: 0.34,
    SpeciesId.RED: 1.09,
    SpeciesId.SILVER: 0.49,
    SpeciesId.PINK: 0.28,
    SpeciesId.CHUM: 0.32,
}


def prices_for_year(year: int) -> dict[SpeciesId, float]:
    if year in YEAR_PRICES:
        return dict(YEAR_PRICES[year])
    return dict(ODD_YEAR_PRICES if year % 2 else EVEN_YEAR_PRICES)


SHORT = {
    SpeciesId.KING: "KNG",
    SpeciesId.RED: "RED",
    SpeciesId.PINK: "PNK",
    SpeciesId.CHUM: "CHM",
    SpeciesId.SILVER: "SLV",
}
