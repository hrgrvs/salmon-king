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
    camp_pos: &[("bailey", (1, 5)), ("uganik", (1, 6)), ("larsen", (1, 10))],
    tender_pos: &[("bailey", (2, 5)), ("uganik", (2, 6)), ("larsen", (1, 9))],
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

/// Marks the TUI paints onto a designed map. Display only.
pub struct MapMarks<'a> {
    pub deployed: &'a HashMap<String, String>,
    /// (skiff id, location, kind) — picker vs holding read differently.
    pub skiffs: &'a [(String, String, String)],
    pub tender_here: bool,
    pub closed_sites: &'a HashSet<String>,
    pub cursor_site: Option<&'a str>,
    pub transient_sites: &'a HashSet<String>,
    pub resident_sites: &'a HashSet<String>,
    pub fish_showing: bool,
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
    let skiffs: Vec<(String, String, String)> = skiff_sites
        .iter()
        .map(|(id, loc)| (id.clone(), loc.clone(), String::from("picker")))
        .collect();
    render_map_marked(
        map_id,
        camp_id,
        &MapMarks {
            deployed,
            skiffs: &skiffs,
            tender_here,
            closed_sites,
            cursor_site,
            transient_sites,
            resident_sites,
            fish_showing: false,
        },
    )
}

pub fn render_map_marked(map_id: &str, camp_id: &str, marks: &MapMarks<'_>) -> String {
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

    let stamp_if_blank = |grid: &mut [Vec<char>], xy: (usize, usize), ch: char| -> bool {
        let (x, y) = xy;
        if y < grid.len() && x < grid[y].len() && grid[y][x] == ' ' {
            grid[y][x] = ch;
            true
        } else {
            false
        }
    };

    for (sid, xy) in m.pos {
        if marks.closed_sites.contains(*sid) {
            stamp(&mut grid, *xy, '×');
        } else {
            stamp(&mut grid, *xy, '·');
        }
    }

    for site_id in marks.deployed.values() {
        if let Some(xy) = lookup(m.pos, site_id) {
            stamp(&mut grid, xy, '╫');
        }
    }

    if let Some(xy) = lookup(m.camp_pos, camp_id) {
        stamp(&mut grid, xy, '▲');
    }

    if marks.tender_here {
        if let Some(xy) = lookup(m.tender_pos, camp_id) {
            stamp(&mut grid, xy, '▣');
        }
    }

    for (_, loc, kind) in marks.skiffs {
        if let Some(xy) = lookup(m.pos, loc) {
            let ch = if kind == "holding" { '»' } else { '›' };
            stamp(&mut grid, xy, ch);
        }
    }

    if let Some(cur) = marks.cursor_site {
        if let Some(xy) = lookup(m.pos, cur) {
            stamp(&mut grid, xy, '●');
        }
    }

    // Mammal / fish silhouettes sit beside the site mark so the node stays readable.
    for sid in marks.transient_sites {
        if let Some((x, y)) = lookup(m.pos, sid) {
            if x > 0 {
                stamp_if_blank(&mut grid, (x - 1, y), 'ω');
            }
            if x > 1 {
                stamp_if_blank(&mut grid, (x - 2, y), '≈');
            }
        }
    }
    for sid in marks.resident_sites {
        if let Some((x, y)) = lookup(m.pos, sid) {
            // Salmon silhouette: ><>
            if x + 2 < grid.get(y).map(|r| r.len()).unwrap_or(0) {
                stamp_if_blank(&mut grid, (x + 1, y), '>');
                stamp_if_blank(&mut grid, (x + 2, y), '<');
                if x + 3 < grid[y].len() {
                    stamp_if_blank(&mut grid, (x + 3, y), '>');
                }
            } else if x > 2 {
                stamp_if_blank(&mut grid, (x - 3, y), '>');
                stamp_if_blank(&mut grid, (x - 2, y), '<');
                stamp_if_blank(&mut grid, (x - 1, y), '>');
            }
        }
    }
    if marks.fish_showing {
        for (sid, xy) in m.pos {
            if marks.closed_sites.contains(*sid) {
                continue;
            }
            let (x, y) = *xy;
            if y + 1 < grid.len() {
                stamp_if_blank(&mut grid, (x, y + 1), '>');
                if x + 1 < grid[y + 1].len() {
                    stamp_if_blank(&mut grid, (x + 1, y + 1), '<');
                }
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
