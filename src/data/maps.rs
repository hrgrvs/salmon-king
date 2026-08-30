//! Designed strategic maps. Not GIS. Playable nodes vs seiner landmarks.

use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug)]
pub struct MapDef {
    pub id: &'static str,
    pub title: &'static str,
    pub lines: &'static [&'static str],
    pub pos: &'static [(&'static str, (usize, usize))],
    pub camp_pos: &'static [(&'static str, (usize, usize))],
    pub tender_pos: &'static [(&'static str, (usize, usize))],
}

pub const WESTSIDE: MapDef = MapDef {
    id: "westside",
    title: "Central Section · Northwest Kodiak · Shelikof shore",
    lines: &[
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
    ],
    pos: &[
        ("raspberry", (4, 2)),
        ("malina", (7, 3)),
        ("viekoda", (10, 4)),
        ("dryspruce", (13, 5)),
        ("uganik_pass", (16, 6)),
        ("spiridon_outer", (19, 7)),
        ("zachar_outer", (22, 8)),
        ("harvester", (17, 9)),
        ("cape_uyak", (23, 11)),
    ],
    camp_pos: &[
        ("bailey", (1, 5)),
        ("uganik", (1, 6)),
        ("larsen", (1, 10)),
    ],
    tender_pos: &[
        ("bailey", (2, 5)),
        ("uganik", (2, 6)),
        ("larsen", (1, 9)),
    ],
};

pub const ALITAK: MapDef = MapDef {
    id: "alitak",
    title: "Alitak District · inner sections · setnet until 4 Sep",
    lines: &[
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
    ],
    pos: &[
        ("lazy_bay", (5, 2)),
        ("moser", (8, 3)),
        ("olga_narrows", (11, 4)),
        ("dog_salmon", (14, 5)),
        ("upper_station_outer", (17, 6)),
        ("akalura_outer", (20, 7)),
    ],
    camp_pos: &[("olga", (1, 4))],
    tender_pos: &[("olga", (2, 4))],
};

pub fn map_by_id(id: &str) -> Option<&'static MapDef> {
    match id {
        "westside" => Some(&WESTSIDE),
        "alitak" => Some(&ALITAK),
        _ => None,
    }
}

fn lookup(pairs: &[(&'static str, (usize, usize))], key: &str) -> Option<(usize, usize)> {
    pairs.iter().find(|(k, _)| *k == key).map(|(_, xy)| *xy)
}

pub fn render_map(
    map_id: &str,
    camp_id: &str,
    deployed: &HashMap<String, String>,
    skiff_sites: &HashMap<String, String>,
    tender_here: bool,
    closed_sites: &HashSet<String>,
    cursor_site: Option<&str>,
    transient_sites: &HashSet<String>,
    resident_sites: &HashSet<String>,
) -> String {
    let Some(m) = map_by_id(map_id) else {
        return String::from("(no map)");
    };
    let mut grid: Vec<Vec<char>> = m.lines.iter().map(|line| line.chars().collect()).collect();

    let stamp = |grid: &mut [Vec<char>], xy: (usize, usize), ch: char| {
        let (x, y) = xy;
        if y < grid.len() && x < grid[y].len() {
            grid[y][x] = ch;
        }
    };

    for (sid, xy) in m.pos {
        if closed_sites.contains(*sid) {
            stamp(&mut grid, *xy, 'x');
        } else {
            stamp(&mut grid, *xy, '·');
        }
    }

    for site_id in deployed.values() {
        if let Some(xy) = lookup(m.pos, site_id) {
            stamp(&mut grid, xy, '╫');
        }
    }

    if let Some(xy) = lookup(m.camp_pos, camp_id) {
        stamp(&mut grid, xy, '▲');
    }

    if tender_here {
        if let Some(xy) = lookup(m.tender_pos, camp_id) {
            stamp(&mut grid, xy, '■');
        }
    }

    for loc in skiff_sites.values() {
        if let Some(xy) = lookup(m.pos, loc) {
            stamp(&mut grid, xy, '›');
        }
    }

    if let Some(cur) = cursor_site {
        if let Some(xy) = lookup(m.pos, cur) {
            stamp(&mut grid, xy, '●');
        }
    }

    for sid in transient_sites {
        if let Some((x, y)) = lookup(m.pos, sid) {
            if y < grid.len() && x > 0 && x - 1 < grid[y].len() && grid[y][x - 1] == ' ' {
                stamp(&mut grid, (x - 1, y), 'ω');
            }
        }
    }
    for sid in resident_sites {
        if let Some((x, y)) = lookup(m.pos, sid) {
            if y < grid.len() && x > 0 && x - 1 < grid[y].len() && grid[y][x - 1] == ' ' {
                stamp(&mut grid, (x - 1, y), 'r');
            }
        }
    }

    let body: String = grid
        .into_iter()
        .map(|row| row.into_iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n");
    format!("{}\n{}", m.title, body)
}
