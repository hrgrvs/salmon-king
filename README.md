# Salmon King

You hold one **S04K** Kodiak set-gillnet permit and run one summer at a west-side or Alitak fish camp: two nets, a couple of skiffs, a cook, a tender that may or may not show, and ADF&G on the VHF. Keep the crew fed, the gear in the water only when it is legal, and the books out of the stove. Local TUI. No network. Not an MMO.

## Requirements

- **Python 3.12+**
- A **real terminal** (macOS Terminal, iTerm, Windows Terminal, gnome-terminal, and the like). Not a dumb pipe, and not an IDE output panel that does not speak alternate-screen / keyboard.
- No API keys. Runtime is offline.

## Install

From GitHub:

```bash
python3 -m pip install "git+https://github.com/hrgrvs/salmon-king.git"
salmon-king
```

Or with pipx:

```bash
pipx install git+https://github.com/hrgrvs/salmon-king.git
salmon-king
```

From a clone:

```bash
git clone https://github.com/hrgrvs/salmon-king.git
cd salmon-king
python3 -m pip install .
salmon-king
```

Editable (if you want to hack on it):

```bash
git clone https://github.com/hrgrvs/salmon-king.git
cd salmon-king
python3 -m pip install -e .
salmon-king
# or: python3 -m salmon_king
```

### Tests (optional)

```bash
python3 -m pip install -e ".[dev]"
pytest
```

Headless season check (no TUI):

```bash
python3 -m salmon_king --headless --camp uganik --year 2025 --seed 1701
```

## How to play

### New season

Title screen: **New season** (`n`) or **Quit** (`q`). Then pick a camp, a **year**, and a seed.

| Camp | Character |
|------|-----------|
| Larsen Bay / Uyak | Village + cannery ghost. Outer Uyak / Harvester. More protected than open Shelikof. |
| Uganik (outer / NE Arm) | Classic west-side setnet. Shelikof weather leaks in. |
| Olga Bay | Alitak inner water. Pulse openers. Tender-dependent. Far from town. |
| Port Bailey / Dry Spruce | Old cannery. Williwaws off Kupreanof. Nearer town via Whale Passage. |

Legal setnet water is only the **Central Section** (west-side Shelikof shore) and **inner Alitak**. Karluk, Ayakulik, Afognak, and the Eastside are seiners on the map. You do not set there.

### Even vs odd year

Pinks are a two-year cycle. **Odd years flood. Even years are thin.** The year you type sets that line and, for 2023 / 2024 / 2025, the Kodiak prelim $/lb table. It does not load a real 2026 emergency-order board.

### Map, nets, crew, tender

The main screen is one HUD:

- **Map** — your beaches, camp (`▲`), nets (`╫`), skiffs (`›`), tender (`■`). `←` `→` moves the site cursor (`●`). `enter` sets a net on that site. `p` pulls.
- **Crew** — energy / hunger / morale bars and the job. Hire on the tender (`h`). Assign with `c`. A town run by the permit holder darkens the sets (you have to be on the site).
- **Tender** — live $/lb and the last fish ticket. `t` sends the fullest skiff. `b` buys food, fuel, ice, twine, a prop, only while the boat is in the hole.
- **Camp** — cash, food days, fuel, ice. Miss the tender and fish rot, or you stop picking when the holding skiff is full.

### Emergency orders

One tick is one tide (flood, then ebb; two a day). Nets fish only during a generated ADF&G opener. On a closer, pull. Leave gear in and you get a game fine and the net on the beach.

Westside clock (5 AAC 18.362, generated, not a 2026 EO list): June tests (33-hour), Karluk-early reds passing the beaches, July–August pinks, late reds, then skeleton-crew coho. Weak Karluk early can mean those June tests are all you get until 6 July. Alitak (5 AAC 18.361): 5–7 days open per 10, then 63 hours dark unless the weirs will make it.

`space` pause. `1` / `4` / `x` = 1× / 4× / 16×. Season runs 1 June–18 September (game tear-down).

### Orcas and sea lions

This is a sim chain, not flavor. Transient killer whales in a bay run the sea lions off. Raids stop while they are there. Residents make the salmon act weird. Neither type picks your web.

### Keys

| Key | Action |
|-----|--------|
| `space` | Pause / run |
| `1` `4` `x` | 1× / 4× / 16× |
| `←` `→` | Site cursor |
| `enter` | Set a net on the highlighted site |
| `p` | Pull a net |
| `s` | Skiff job (pick / tender / idle / repair / town) |
| `c` | Assign crew |
| `h` | Hire (tender) |
| `b` | Buy supplies (tender) |
| `u` | Upgrade camp |
| `j` | Joint venture (second S04K → third net) |
| `t` | Send the fullest skiff to the tender |
| `m` | Mesh knob (game selectivity, not a legal mesh spec) |
| `?` | Help |
| `q` | Quit (confirm) |

Season ends 18 September and scores the books and who stayed. Early fail: bankrupt, every skiff wrecked, or nobody left who will pick. Grade is a letter plus a nickname.

## Flavor and sources

Constraints come from public ADF&G / NOAA / setnetter material. Invented balance numbers are marked **game** in the code. This will not get you a permit, an EO, or a fish ticket.

- 5 AAC 18.200, 18.330, 18.331, 18.361 (Alitak), 18.362 (Westside)
- ADF&G Kodiak Management Area salmon pages, harvest strategies, and preliminary season summaries (2023–2025 ex-vessel $/lb and weights)
- NOAA MMPA List of Fisheries: AK Kodiak salmon set gillnet (Category II; harbor porpoise, SSL, harbor seal, northern sea otter, humpback)
- Northwest Setnetters Association (camp / picking / tender practice)
- *Kodiak’s Setnet Salmon Fishery in the Context of Alaska’s Limited Access Management System* (FAO tenure paper)
