"""Designed strategic maps. Not GIS. Playable nodes vs seiner landmarks."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class MapDef:
    id: str
    title: str
    lines: tuple[str, ...]
    # site_id -> (x, y) character cell
    pos: dict[str, tuple[int, int]]
    camp_pos: dict[str, tuple[int, int]]
    tender_pos: dict[str, tuple[int, int]]


WESTSIDE = MapDef(
    id="westside",
    title="Central Section · Northwest Kodiak · Shelikof shore",
    lines=(
        "~~~~~~~~ SHELIKOF STRAIT  (short steep seas)  ~~~~~~~~",
        " Raspberry I.    [Afognak seine]                      ",
        "    *R Raspberry Cape                                 ",
        "       *M Malina / Onion Bay mouth                    ",
        "          *O Outlet Cape / Viekoda                    ",
        "             *D Dry Spruce / Port Bailey              ",
        "                *U Uganik Passage / outer Uganik      ",
        "                   *S Spiridon outer  [Telrod SHA]    ",
        "                      *Z Zachar outer / Carlsen Pt    ",
        "                 *H Harvester I. / Uyak entrance      ",
        "                    Larsen Bay village                ",
        "                       *C Cape Uyak / Rocky Point     ",
        "                          [Karluk Lagoon  seine]      ",
        "                             [Halibut Bay / Ayakulik] ",
        " inner arms (Terror, Spiridon, Zachar, Uyak) = seine  ",
    ),
    # Stamp the leading '*' so names stay readable.
    pos={
        "raspberry": (4, 2),
        "malina": (7, 3),
        "viekoda": (10, 4),
        "dryspruce": (13, 5),
        "uganik_pass": (16, 6),
        "spiridon_outer": (19, 7),
        "zachar_outer": (22, 8),
        "harvester": (17, 9),
        "cape_uyak": (23, 11),
    },
    camp_pos={
        "bailey": (1, 5),
        "uganik": (1, 6),
        "larsen": (1, 10),
    },
    tender_pos={
        "bailey": (2, 5),
        "uganik": (2, 6),
        "larsen": (1, 9),
    },
)


ALITAK = MapDef(
    id="alitak",
    title="Alitak District · inner sections · setnet until 4 Sep",
    lines=(
        " south end · Shelikof wrap / Gulf swell               ",
        "  [Cape Alitak  seine]     [Humpy-Deadman  seine]     ",
        "     *A Alitak Bay / Lazy Bay                         ",
        "        *M Moser Bay                                  ",
        "           *N Olga Narrows / Olga Bay                 ",
        "              *F Dog Salmon Flats                     ",
        "                 *T Outer Upper Station               ",
        "                    *K Outer Akalura                  ",
        "                       [Ayakulik  seine, west]        ",
        "                  [Frazer weir]  [Horse Marine]        ",
        " inner Upper Station / Akalura pulse if weirs allow   ",
        " 63 hr dark / 10 days unless Frazer+U.Sta will make it",
    ),
    pos={
        "lazy_bay": (5, 2),
        "moser": (8, 3),
        "olga_narrows": (11, 4),
        "dog_salmon": (14, 5),
        "upper_station_outer": (17, 6),
        "akalura_outer": (20, 7),
    },
    camp_pos={
        "olga": (1, 4),
    },
    tender_pos={
        "olga": (2, 4),
    },
)


MAPS = {"westside": WESTSIDE, "alitak": ALITAK}


def render_map(
    map_id: str,
    camp_id: str,
    deployed: dict[str, str],
    skiff_sites: dict[str, str],
    tender_here: bool,
    closed_sites: set[str],
    cursor_site: str | None,
    transient_sites: set[str] | None = None,
    resident_sites: set[str] | None = None,
) -> str:
    """Return a unicode map with live markers.

    deployed: net_id -> site_id
    skiff_sites: skiff_id -> site_id or 'camp' or 'tender' or 'transit'
    """
    m = MAPS[map_id]
    grid = [list(line) for line in m.lines]

    def stamp(xy: tuple[int, int], ch: str) -> None:
        x, y = xy
        if 0 <= y < len(grid) and 0 <= x < len(grid[y]):
            grid[y][x] = ch

    for sid, xy in m.pos.items():
        if sid in closed_sites:
            stamp(xy, "x")
        else:
            stamp(xy, "·")

    for site_id in deployed.values():
        if site_id in m.pos:
            stamp(m.pos[site_id], "╫")

    if camp_id in m.camp_pos:
        stamp(m.camp_pos[camp_id], "▲")

    if tender_here and camp_id in m.tender_pos:
        stamp(m.tender_pos[camp_id], "■")

    for loc in skiff_sites.values():
        if loc in m.pos:
            stamp(m.pos[loc], "›")
        elif loc == "camp" and camp_id in m.camp_pos:
            # already camp glyph; leave it
            pass

    if cursor_site and cursor_site in m.pos:
        x, y = m.pos[cursor_site]
        stamp((x, y), "●")

    # ω / r sit one cell left of the site mark when that cell is free.
    for sid in transient_sites or ():
        if sid in m.pos:
            x, y = m.pos[sid]
            if 0 <= y < len(grid) and 0 <= x - 1 < len(grid[y]) and grid[y][x - 1] == " ":
                stamp((x - 1, y), "ω")
    for sid in resident_sites or ():
        if sid in m.pos:
            x, y = m.pos[sid]
            cell = (x - 1, y)
            if 0 <= y < len(grid) and 0 <= x - 1 < len(grid[y]) and grid[y][x - 1] == " ":
                stamp(cell, "r")

    body = "\n".join("".join(row) for row in grid)
    return f"{m.title}\n{body}"
