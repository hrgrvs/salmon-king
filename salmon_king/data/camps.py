"""Four legal starting camps. No Karluk/Ayakulik/Eastside setnet camp."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class CampDef:
    id: str
    name: str
    village: str
    district: str  # "central" | "alitak"
    blurb: str
    long: str
    home_sites: tuple[str, ...]
    tender_anchorage: str
    starting_cash: int
    williwaw: float
    fog: float
    town_tides: int  # tides to run town (Whale Passage / Alitak haul)
    max_fathoms: int  # 175 Central, 150 Alitak
    map_id: str


CAMPS: dict[str, CampDef] = {
    "larsen": CampDef(
        id="larsen",
        name="Larsen Bay",
        village="Larsen Bay",
        district="central",
        blurb="Village + historic cannery. Outer Uyak / Amook / Harvester is Central Section water.",
        long=(
            "Larsen Bay still has a village and the ghost of a cannery. Your water is the "
            "outer Uyak — Harvester Island, the entrance, Cape Uyak. Inner Uyak is seiner "
            "country. More protected than open Shelikof, which is why the old company put "
            "a plant here. Tender still has to come around. If you run town you're gone a day."
        ),
        home_sites=("harvester", "cape_uyak"),
        tender_anchorage="harvester",
        starting_cash=12000,
        williwaw=0.18,
        fog=0.22,
        town_tides=3,
        max_fathoms=175,
        map_id="westside",
    ),
    "uganik": CampDef(
        id="uganik",
        name="Uganik (outer / Northeast Arm)",
        village="Uganik",
        district="central",
        blurb="Classic west-side setnet country. Shelikof weather leaks in.",
        long=(
            "Outer Uganik and the Northeast Arm approaches. This is what people mean when "
            "they say west-side setnet: corkline on a Shelikof beach, Karluk-bound reds "
            "sliding past, williwaws you hear before you see. Inner Uganik is closed to "
            "your gear. You fish the passage and the outside."
        ),
        home_sites=("uganik_pass", "viekoda"),
        tender_anchorage="uganik_pass",
        starting_cash=11500,
        williwaw=0.32,
        fog=0.28,
        town_tides=3,
        max_fathoms=175,
        map_id="westside",
    ),
    "olga": CampDef(
        id="olga",
        name="Olga Bay",
        village="Olga Bay",
        district="alitak",
        blurb="Setnet-only inner water until Sept 4. Pulse openers. Far from town.",
        long=(
            "South end. Alitak Bay, Moser, Olga Narrows — set gillnet only until September 4. "
            "ADF&G runs this on Frazer and Upper Station sockeye and a 10-day clock: five to "
            "seven days on, then sixty-three hours dark unless the weirs are going to make it. "
            "Odd-year pinks can bury you. There is no town run that makes sense. The tender "
            "is fuel, food, ice, mail, and the only buyer."
        ),
        home_sites=("olga_narrows", "moser"),
        tender_anchorage="olga_narrows",
        starting_cash=10500,
        williwaw=0.12,
        fog=0.20,
        town_tides=5,
        max_fathoms=150,
        map_id="alitak",
    ),
    "bailey": CampDef(
        id="bailey",
        name="Port Bailey / Dry Spruce",
        village="Port Bailey",
        district="central",
        blurb="Old cannery. Nearer town via Whale Passage. Williwaws off Kupreanof Mountain.",
        long=(
            "Kupreanof / Dry Spruce / the old Port Bailey cannery. You are still Central "
            "Section — Spruce Island down toward Uganik — but Whale Passage is a 5–7 knot "
            "rip and town is actually thinkable if the outboard lives. Williwaws come off "
            "Kupreanof Mountain like someone opened a door. Viekoda and Malina are your "
            "neighbors' beaches unless you lease them."
        ),
        home_sites=("dryspruce", "viekoda"),
        tender_anchorage="dryspruce",
        starting_cash=12500,
        williwaw=0.40,
        fog=0.24,
        town_tides=2,
        max_fathoms=175,
        map_id="westside",
    ),
}
