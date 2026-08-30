"""Named water. Playable nodes are legal S04K set-gillnet water only."""

from __future__ import annotations

from dataclasses import dataclass, field

from salmon_king.data.species import SpeciesId


@dataclass(frozen=True)
class SiteDef:
    id: str
    name: str
    short: str
    district: str  # "central" | "alitak" | "seine"
    legal_setnet: bool
    exposure: str  # shelikof, protected, south_end, landmark
    travel_from_camp: dict[str, int]  # camp_id -> tides of run time
    # Relative run affinities. Game numbers, timed to real systems.
    affinity: dict[SpeciesId, float] = field(default_factory=dict)
    sea_lion: float = 0.35
    harbor_seal: float = 0.15
    lease: str = "scramble"  # "lease" home beaches vs scramble
    note: str = ""
    mark: str = "?"  # map glyph key


# --- Central Section, Northwest Kodiak District (legal setnet) ---
_CENTRAL = [
    SiteDef(
        id="raspberry",
        name="Raspberry Cape",
        short="Rasp. Cape",
        district="central",
        legal_setnet=True,
        exposure="shelikof",
        travel_from_camp={"bailey": 1, "uganik": 2, "larsen": 3},
        affinity={
            SpeciesId.KING: 0.25,
            SpeciesId.RED: 0.70,
            SpeciesId.PINK: 0.85,
            SpeciesId.CHUM: 0.55,
            SpeciesId.SILVER: 0.40,
        },
        sea_lion=0.55,
        lease="scramble",
        note="North end of Central Section, looking at Raspberry Island and Shelikof.",
        mark="R",
    ),
    SiteDef(
        id="malina",
        name="Malina Point / Onion Bay mouth",
        short="Malina",
        district="central",
        legal_setnet=True,
        exposure="shelikof",
        travel_from_camp={"bailey": 1, "uganik": 1, "larsen": 3},
        affinity={
            SpeciesId.KING: 0.20,
            SpeciesId.RED: 0.75,
            SpeciesId.PINK: 0.90,
            SpeciesId.CHUM: 0.60,
            SpeciesId.SILVER: 0.45,
        },
        sea_lion=0.50,
        lease="scramble",
        note="Onion Bay mouth. Shelikof leaks in; fog hangs.",
        mark="M",
    ),
    SiteDef(
        id="viekoda",
        name="Outlet Cape / Viekoda mouth",
        short="Viekoda",
        district="central",
        legal_setnet=True,
        exposure="shelikof",
        travel_from_camp={"bailey": 0, "uganik": 1, "larsen": 2},
        affinity={
            SpeciesId.KING: 0.22,
            SpeciesId.RED: 0.80,
            SpeciesId.PINK: 0.95,
            SpeciesId.CHUM: 0.65,
            SpeciesId.SILVER: 0.50,
        },
        sea_lion=0.48,
        lease="scramble",
        note="Viekoda Bay mouth. Classic west-side shoreline set.",
        mark="O",
    ),
    SiteDef(
        id="dryspruce",
        name="Dry Spruce / Port Bailey",
        short="Dry Spruce",
        district="central",
        legal_setnet=True,
        exposure="shelikof",
        travel_from_camp={"bailey": 0, "uganik": 1, "larsen": 2},
        affinity={
            SpeciesId.KING: 0.18,
            SpeciesId.RED: 0.85,
            SpeciesId.PINK: 1.00,
            SpeciesId.CHUM: 0.70,
            SpeciesId.SILVER: 0.55,
        },
        sea_lion=0.42,
        lease="lease",
        note="Old cannery water. Williwaws off Kupreanof Mountain. Whale Passage rips if you run town.",
        mark="D",
    ),
    SiteDef(
        id="uganik_pass",
        name="Uganik Passage / outer Uganik",
        short="Uganik Pass",
        district="central",
        legal_setnet=True,
        exposure="shelikof",
        travel_from_camp={"bailey": 1, "uganik": 0, "larsen": 2},
        affinity={
            SpeciesId.KING: 0.20,
            SpeciesId.RED: 0.95,
            SpeciesId.PINK: 1.05,
            SpeciesId.CHUM: 0.75,
            SpeciesId.SILVER: 0.60,
        },
        sea_lion=0.50,
        lease="lease",
        note="Outer Uganik / Northeast Arm approaches. Inner Uganik is seine-only.",
        mark="U",
    ),
    SiteDef(
        id="spiridon_outer",
        name="Spiridon outer / Telrod-adjacent",
        short="Spiridon outer",
        district="central",
        legal_setnet=True,
        exposure="shelikof",
        travel_from_camp={"bailey": 2, "uganik": 1, "larsen": 1},
        affinity={
            SpeciesId.KING: 0.15,
            SpeciesId.RED: 1.10,
            SpeciesId.PINK: 0.90,
            SpeciesId.CHUM: 0.55,
            SpeciesId.SILVER: 0.45,
        },
        sea_lion=0.40,
        lease="scramble",
        note="Fish the outside mixed fishery. Telrod Cove SHA is seine-only.",
        mark="S",
    ),
    SiteDef(
        id="zachar_outer",
        name="Zachar outer / Carlsen Point",
        short="Carlsen Pt",
        district="central",
        legal_setnet=True,
        exposure="shelikof",
        travel_from_camp={"bailey": 2, "uganik": 1, "larsen": 1},
        affinity={
            SpeciesId.KING: 0.16,
            SpeciesId.RED: 1.00,
            SpeciesId.PINK: 0.95,
            SpeciesId.CHUM: 0.60,
            SpeciesId.SILVER: 0.50,
        },
        sea_lion=0.38,
        lease="scramble",
        note="Carlsen Point / outer Zachar. Inner Zachar is seine-only.",
        mark="Z",
    ),
    SiteDef(
        id="harvester",
        name="Harvester Island / Uyak entrance",
        short="Harvester",
        district="central",
        legal_setnet=True,
        exposure="protected",
        travel_from_camp={"bailey": 2, "uganik": 2, "larsen": 0},
        affinity={
            SpeciesId.KING: 0.18,
            SpeciesId.RED: 1.15,
            SpeciesId.PINK: 1.00,
            SpeciesId.CHUM: 0.70,
            SpeciesId.SILVER: 0.65,
        },
        sea_lion=0.35,
        harbor_seal=0.28,
        lease="lease",
        note="Uyak entrance. More protected than open Shelikof. Inner Uyak is seine-only.",
        mark="H",
    ),
    SiteDef(
        id="cape_uyak",
        name="Cape Uyak / Rocky Point",
        short="Cape Uyak",
        district="central",
        legal_setnet=True,
        exposure="shelikof",
        travel_from_camp={"bailey": 3, "uganik": 2, "larsen": 1},
        affinity={
            SpeciesId.KING: 0.30,
            SpeciesId.RED: 1.25,
            SpeciesId.PINK: 0.85,
            SpeciesId.CHUM: 0.50,
            SpeciesId.SILVER: 0.55,
        },
        sea_lion=0.58,
        lease="scramble",
        note="District line toward Karluk. Karluk-bound reds slide this beach. You are not fishing the lagoon.",
        mark="C",
    ),
]

# --- Alitak District inner sections (set gillnet only until Sept 4) ---
_ALITAK = [
    SiteDef(
        id="lazy_bay",
        name="Alitak Bay / Lazy Bay",
        short="Lazy Bay",
        district="alitak",
        legal_setnet=True,
        exposure="south_end",
        travel_from_camp={"olga": 1},
        affinity={
            SpeciesId.KING: 0.10,
            SpeciesId.RED: 0.90,
            SpeciesId.PINK: 1.10,
            SpeciesId.CHUM: 0.80,
            SpeciesId.SILVER: 0.50,
        },
        sea_lion=0.30,
        harbor_seal=0.25,
        lease="scramble",
        note="Alitak Bay section. Setnet-only until Sept 4, then seines also allowed.",
        mark="A",
    ),
    SiteDef(
        id="moser",
        name="Moser Bay",
        short="Moser",
        district="alitak",
        legal_setnet=True,
        exposure="protected",
        travel_from_camp={"olga": 1},
        affinity={
            SpeciesId.KING: 0.08,
            SpeciesId.RED: 1.05,
            SpeciesId.PINK: 1.15,
            SpeciesId.CHUM: 0.85,
            SpeciesId.SILVER: 0.55,
        },
        sea_lion=0.22,
        harbor_seal=0.35,
        lease="scramble",
        note="Inside water. Seals work the leads. Pulse openers.",
        mark="M",
    ),
    SiteDef(
        id="olga_narrows",
        name="Olga Bay / Olga Narrows",
        short="Olga Narrows",
        district="alitak",
        legal_setnet=True,
        exposure="protected",
        travel_from_camp={"olga": 0},
        affinity={
            SpeciesId.KING: 0.08,
            SpeciesId.RED: 1.20,
            SpeciesId.PINK: 1.20,
            SpeciesId.CHUM: 0.90,
            SpeciesId.SILVER: 0.70,
        },
        sea_lion=0.18,
        harbor_seal=0.40,
        lease="lease",
        note="Far from town. Tender is the trade wagon. 63-hour dark every ten days unless the weirs say otherwise.",
        mark="N",
    ),
    SiteDef(
        id="dog_salmon",
        name="Dog Salmon Flats",
        short="Dog Sal. Flats",
        district="alitak",
        legal_setnet=True,
        exposure="protected",
        travel_from_camp={"olga": 1},
        affinity={
            SpeciesId.KING: 0.06,
            SpeciesId.RED: 1.30,
            SpeciesId.PINK: 0.95,
            SpeciesId.CHUM: 0.70,
            SpeciesId.SILVER: 0.60,
        },
        sea_lion=0.15,
        harbor_seal=0.32,
        lease="scramble",
        note="Often dark unless Frazer fish are past the traditional water. Pulse site.",
        mark="F",
        # treated as excess-escapement pulse in the opener generator
    ),
    SiteDef(
        id="upper_station_outer",
        name="Outer Upper Station",
        short="Outer U.Sta",
        district="alitak",
        legal_setnet=True,
        exposure="protected",
        travel_from_camp={"olga": 1},
        affinity={
            SpeciesId.KING: 0.05,
            SpeciesId.RED: 1.40,
            SpeciesId.PINK: 0.70,
            SpeciesId.CHUM: 0.45,
            SpeciesId.SILVER: 0.75,
        },
        sea_lion=0.12,
        harbor_seal=0.30,
        lease="scramble",
        note="Upper Station sockeye. Opens when the weir can stand it.",
        mark="T",
    ),
    SiteDef(
        id="akalura_outer",
        name="Outer Akalura",
        short="Akalura",
        district="alitak",
        legal_setnet=True,
        exposure="protected",
        travel_from_camp={"olga": 1},
        affinity={
            SpeciesId.KING: 0.05,
            SpeciesId.RED: 1.15,
            SpeciesId.PINK: 0.75,
            SpeciesId.CHUM: 0.50,
            SpeciesId.SILVER: 0.80,
        },
        sea_lion=0.12,
        harbor_seal=0.28,
        lease="scramble",
        note="Akalura system. Late silver water if you're still here.",
        mark="K",
    ),
]

# --- Seine / NPC landmarks. Visible. You cannot set a net here. ---
_SEINE = [
    SiteDef(
        id="karluk_lagoon",
        name="Karluk Lagoon",
        short="Karluk",
        district="seine",
        legal_setnet=False,
        exposure="landmark",
        travel_from_camp={},
        affinity={},
        note="Southwest Kodiak. Purse/beach seine. Karluk-bound reds pass Central Section beaches first.",
        mark="◇",
    ),
    SiteDef(
        id="ayakulik",
        name="Ayakulik",
        short="Ayakulik",
        district="seine",
        legal_setnet=False,
        exposure="landmark",
        travel_from_camp={},
        note="Southwest Kodiak seine water. King stock of concern.",
        mark="◇",
    ),
    SiteDef(
        id="halibut_bay",
        name="Halibut Bay",
        short="Halibut Bay",
        district="seine",
        legal_setnet=False,
        exposure="landmark",
        travel_from_camp={},
        note="Southwest Kodiak. Seine only.",
        mark="◇",
    ),
    SiteDef(
        id="telrod_sha",
        name="Telrod Cove SHA",
        short="Telrod SHA",
        district="seine",
        legal_setnet=False,
        exposure="landmark",
        travel_from_camp={},
        note="Spiridon Bay sockeye SHA. Seine-only. Setnets fish the outside mixed fishery.",
        mark="◇",
    ),
    SiteDef(
        id="cape_alitak_seine",
        name="Cape Alitak (seine)",
        short="Cape Alitak",
        district="seine",
        legal_setnet=False,
        exposure="landmark",
        travel_from_camp={},
        note="Seine section. Set gillnet allowed here only after September 4 (not a starting site).",
        mark="◇",
    ),
    SiteDef(
        id="humpy_deadman",
        name="Humpy-Deadman",
        short="Humpy-Deadman",
        district="seine",
        legal_setnet=False,
        exposure="landmark",
        travel_from_camp={},
        note="Alitak District seine section.",
        mark="◇",
    ),
    SiteDef(
        id="afognak",
        name="Afognak",
        short="Afognak",
        district="seine",
        legal_setnet=False,
        exposure="landmark",
        travel_from_camp={},
        note="Afognak District: purse and beach seine only.",
        mark="◇",
    ),
    SiteDef(
        id="ugak",
        name="Ugak Bay",
        short="Ugak",
        district="seine",
        legal_setnet=False,
        exposure="landmark",
        travel_from_camp={},
        note="Eastside. Seine only.",
        mark="◇",
    ),
    SiteDef(
        id="womens_bay",
        name="Women's Bay",
        short="Women's Bay",
        district="seine",
        legal_setnet=False,
        exposure="landmark",
        travel_from_camp={},
        note="Near town. Not setnet water.",
        mark="◇",
    ),
    SiteDef(
        id="swikshak",
        name="Mainland / Swikshak",
        short="Swikshak",
        district="seine",
        legal_setnet=False,
        exposure="landmark",
        travel_from_camp={},
        note="Mainland District. Seine only.",
        mark="◇",
    ),
]


SITES: dict[str, SiteDef] = {s.id: s for s in (*_CENTRAL, *_ALITAK, *_SEINE)}
PLAYABLE = {s.id: s for s in SITES.values() if s.legal_setnet}
CENTRAL_SITES = [s.id for s in _CENTRAL]
ALITAK_SITES = [s.id for s in _ALITAK]
PULSE_ALITAK = {"dog_salmon", "upper_station_outer", "akalura_outer"}
SEINE_LANDMARKS = [s.id for s in _SEINE]

# Adjacent beaches that share a haulout / a pod of transients.
BAYS: dict[str, tuple[str, ...]] = {
    "raspberry_onion": ("raspberry", "malina"),
    "viekoda": ("viekoda", "dryspruce"),
    "uganik": ("uganik_pass",),
    "spiridon": ("spiridon_outer",),
    "zachar": ("zachar_outer",),
    "uyak": ("harvester", "cape_uyak"),
    "alitak": ("lazy_bay", "moser"),
    "olga": ("olga_narrows", "dog_salmon"),
    "station": ("upper_station_outer", "akalura_outer"),
}

BAY_LABEL: dict[str, str] = {
    "raspberry_onion": "Raspberry / Onion Bay",
    "viekoda": "Viekoda / Dry Spruce",
    "uganik": "outer Uganik",
    "spiridon": "Spiridon outer",
    "zachar": "Zachar outer",
    "uyak": "Uyak",
    "alitak": "Alitak / Moser",
    "olga": "Olga Bay",
    "station": "Upper Station / Akalura",
}


def bay_id_for_site(site_id: str) -> str | None:
    for bid, members in BAYS.items():
        if site_id in members:
            return bid
    return None


def sites_in_bay(bay_id: str) -> tuple[str, ...]:
    return BAYS.get(bay_id, ())


def sites_for_camp(camp_id: str) -> list[SiteDef]:
    return [s for s in PLAYABLE.values() if camp_id in s.travel_from_camp]


def travel_tides(camp_id: str, site_id: str) -> int:
    site = SITES[site_id]
    if camp_id in site.travel_from_camp:
        return site.travel_from_camp[camp_id]
    return 99
