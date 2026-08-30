"""Hire pool. Working-waterfront names, not a joke list."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class Candidate:
    id: str
    name: str
    role: str  # picker, operator, cook
    skill: int  # 1-10
    wage_share: float  # crew share of landed value; cook uses daily_wage instead
    daily_wage: int
    trait: str
    blurb: str
    energy: int = 78
    hunger: int = 20
    morale: int = 72


CANDIDATES: tuple[Candidate, ...] = (
    Candidate(
        id="petroff",
        name="Aiden Petroff",
        role="operator",
        skill=8,
        wage_share=0.13,
        daily_wage=0,
        trait="veteran skiff man",
        blurb="Ran a holding skiff out of Uyak before he was twenty. Quiet. Hates a soft fish.",
        energy=82,
        morale=70,
    ),
    Candidate(
        id="noya",
        name="Malia Noya",
        role="picker",
        skill=7,
        wage_share=0.12,
        daily_wage=0,
        trait="veteran picker",
        blurb="Hands like leather. Will tell you when the lead is rolling. Wants coffee at 0400.",
        energy=80,
        morale=74,
    ),
    Candidate(
        id="haakanson",
        name="Ben Haakanson",
        role="picker",
        skill=4,
        wage_share=0.10,
        daily_wage=0,
        trait="green",
        blurb="First full season. Strong back. Still learns mesh from corkline the hard way.",
        energy=90,
        hunger=25,
        morale=80,
    ),
    Candidate(
        id="pestrikoff",
        name="June Pestrikoff",
        role="cook",
        skill=8,
        wage_share=0.0,
        daily_wage=70,  # game number: daily cook wage
        trait="grouchy cook",
        blurb="Feeds a crew that comes in wet. Does not teach you to cook. Do not be late to the table.",
        energy=70,
        morale=62,
    ),
    Candidate(
        id="squartsoff",
        name="Cody Squartsoff",
        role="operator",
        skill=6,
        wage_share=0.12,
        daily_wage=0,
        trait="heavy on the throttle",
        blurb="Gets there. Fuel and lower units notice. Fine in a williwaw if you like living.",
        energy=84,
        morale=68,
    ),
    Candidate(
        id="knagin",
        name="Elena Knagin",
        role="picker",
        skill=9,
        wage_share=0.15,
        daily_wage=0,
        trait="site boss",
        blurb="Has pulled more torn leads than you've owned. Share is not cheap. Neither is she.",
        energy=76,
        morale=66,
    ),
    Candidate(
        id="santos",
        name="Miguel Santos",
        role="picker",
        skill=6,
        wage_share=0.11,
        daily_wage=0,
        trait="cannery hands",
        blurb="Came out from town after the slime line. Fast on pinks. Still learning a king buoy.",
        energy=85,
        morale=75,
    ),
    Candidate(
        id="chichenoff",
        name="Ruth Chichenoff",
        role="cook",
        skill=6,
        wage_share=0.0,
        daily_wage=60,
        trait="patient cook",
        blurb="Bread, coffee, and a pot that is never empty. Will sit with a green kid.",
        energy=72,
        morale=78,
    ),
    Candidate(
        id="wollak",
        name="Travis Wollak",
        role="operator",
        skill=5,
        wage_share=0.11,
        daily_wage=0,
        trait="townie",
        blurb="Knows Whale Passage and the parts houses. Gets itchy if the tender is late.",
        energy=80,
        morale=70,
    ),
    Candidate(
        id="lukin",
        name="Daria Lukin",
        role="picker",
        skill=7,
        wage_share=0.12,
        daily_wage=0,
        trait="night picker",
        blurb="Will pick a flood at 0200 without a speech. Hates idle days in the bunkhouse.",
        energy=74,
        hunger=18,
        morale=73,
    ),
    Candidate(
        id="anderson",
        name="Keiko Anderson",
        role="picker",
        skill=5,
        wage_share=0.10,
        daily_wage=0,
        trait="green but careful",
        blurb="Second summer. Ties a decent hanging twine. Will not guess in a rip.",
        energy=88,
        morale=77,
    ),
    Candidate(
        id="naumoff",
        name="Sam Naumoff",
        role="operator",
        skill=7,
        wage_share=0.13,
        daily_wage=0,
        trait="ice man",
        blurb="Obsessed with slush. Fish come off his skiff looking like they still owe him money.",
        energy=79,
        morale=71,
    ),
)


STARTING_HIRE = {
    "larsen": ("noya", "pestrikoff"),
    "uganik": ("petroff", "chichenoff"),
    "olga": ("lukin", "pestrikoff"),
    "bailey": ("santos", "chichenoff"),
}

OWNER_NAME = "You (permit holder)"
