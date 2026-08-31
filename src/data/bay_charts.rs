//! Little bay charts of a camp's home water. Not the island. Display only.

use std::collections::HashSet;

use crate::data::sites::site;

#[derive(Clone, Copy, Debug)]
pub struct BayChart {
    pub camp_id: &'static str,
    pub title: &'static str,
    pub lines: &'static [&'static str],
    pub sites: &'static [(&'static str, (usize, usize))],
    pub camp: (usize, usize),
    pub tender: (usize, usize),
    pub town: (usize, usize),
    pub channel: &'static [(usize, usize)],
}

pub const UGANIK_BAY: BayChart = BayChart {
    camp_id: "uganik",
    title: "outer Uganik / NE Arm",
    lines: &[
        "~~~~ SHELIKOF ~~~~        ",
        "  ·O Viekoda              ",
        "                          ",
        "▲CAMP         ·U Pass     ",
        "Uganik        ▣ hole      ",
        "                          ",
    ],
    sites: &[("viekoda", (2, 1)), ("uganik_pass", (14, 3))],
    camp: (0, 3),
    tender: (14, 4),
    town: (24, 0),
    channel: &[(6, 3), (9, 3), (12, 3)],
};

pub const LARSEN_BAY: BayChart = BayChart {
    camp_id: "larsen",
    title: "Uyak / Harvester",
    lines: &[
        "  Uyak entrance           ",
        "  ·H Harvester            ",
        "                          ",
        "▲CAMP         ·C Cape Uyak",
        "Larsen        ▣ hole      ",
        "                          ",
    ],
    sites: &[("harvester", (2, 1)), ("cape_uyak", (14, 3))],
    camp: (0, 3),
    tender: (14, 4),
    town: (24, 0),
    channel: &[(6, 2), (9, 2), (12, 3)],
};

pub const OLGA_BAY: BayChart = BayChart {
    camp_id: "olga",
    title: "Olga / Moser",
    lines: &[
        "  inner Alitak            ",
        "  ·M Moser                ",
        "                          ",
        "▲CAMP         ·N Narrows  ",
        "Olga Bay      ▣ hole      ",
        "                          ",
    ],
    sites: &[("moser", (2, 1)), ("olga_narrows", (14, 3))],
    camp: (0, 3),
    tender: (14, 4),
    town: (24, 0),
    channel: &[(6, 3), (9, 3), (12, 3)],
};

pub const BAILEY_BAY: BayChart = BayChart {
    camp_id: "bailey",
    title: "Dry Spruce / Viekoda",
    lines: &[
        "  Kupreanof shore         ",
        "  ·O Viekoda              ",
        "                          ",
        "▲CAMP         ·D Dry Spruce",
        "Port Bailey   ▣ hole      ",
        "                          ",
    ],
    sites: &[("viekoda", (2, 1)), ("dryspruce", (14, 3))],
    camp: (0, 3),
    tender: (14, 4),
    town: (24, 0),
    channel: &[(6, 3), (9, 3), (12, 3)],
};

pub fn bay_chart_for(camp_id: &str) -> Option<&'static BayChart> {
    match camp_id {
        "uganik" => Some(&UGANIK_BAY),
        "larsen" => Some(&LARSEN_BAY),
        "olga" => Some(&OLGA_BAY),
        "bailey" => Some(&BAILEY_BAY),
        _ => None,
    }
}

/// Marks the TUI paints on the home-water inset.
pub struct BayMarks<'a> {
    pub skiffs: &'a [BaySkiff],
    pub tender_here: bool,
    pub transient_sites: &'a HashSet<String>,
    pub resident_sites: &'a HashSet<String>,
    pub seal_sites: &'a HashSet<String>,
}

pub struct BaySkiff {
    pub location: String,
    pub dest: Option<String>,
    pub from: Option<String>,
    pub eta: i32,
    pub kind: String,
}

fn stamp(grid: &mut [Vec<char>], xy: (usize, usize), ch: char) {
    let (x, y) = xy;
    if y < grid.len() && x < grid[y].len() {
        grid[y][x] = ch;
    }
}

fn stamp_if_blank(grid: &mut [Vec<char>], xy: (usize, usize), ch: char) {
    let (x, y) = xy;
    if y < grid.len() && x < grid[y].len() && grid[y][x] == ' ' {
        grid[y][x] = ch;
    }
}

impl BayChart {
    pub fn xy(&self, id: &str) -> Option<(usize, usize)> {
        match id {
            "camp" => Some(self.camp),
            "tender" => Some(self.tender),
            "town" => Some(self.town),
            other => self
                .sites
                .iter()
                .find(|(k, _)| *k == other)
                .map(|(_, xy)| *xy),
        }
    }

    fn transit_xy(&self, from: Option<&str>, dest: Option<&str>, eta: i32) -> (usize, usize) {
        if !self.channel.is_empty() {
            let n = self.channel.len();
            let total = (eta + 1).max(1) as usize;
            let toward = 1usize;
            let idx = if dest == Some("camp") || dest == Some("tender") {
                n.saturating_sub(1 + toward.min(eta as usize))
            } else {
                ((n - 1) * toward / total).min(n - 1)
            };
            return self.channel[idx];
        }
        let a = from.and_then(|id| self.xy(id)).unwrap_or(self.camp);
        let b = dest.and_then(|id| self.xy(id)).unwrap_or(self.tender);
        let t = 1.0 / f64::from((eta + 1).max(1));
        (
            a.0 + ((b.0 as f64 - a.0 as f64) * t).round() as usize,
            a.1 + ((b.1 as f64 - a.1 as f64) * t).round() as usize,
        )
    }
}

pub fn render_bay_chart(camp_id: &str, marks: &BayMarks<'_>) -> String {
    let Some(c) = bay_chart_for(camp_id) else {
        return String::from("(no bay)");
    };
    let mut grid: Vec<Vec<char>> = c.lines.iter().map(|line| line.chars().collect()).collect();

    stamp(&mut grid, c.camp, '▲');
    if marks.tender_here {
        stamp(&mut grid, c.tender, '▣');
    }

    for sid in marks.transient_sites {
        if let Some((x, y)) = c.xy(sid) {
            if x > 0 {
                stamp_if_blank(&mut grid, (x - 1, y), 'ω');
            }
        }
    }
    for sid in marks.resident_sites {
        if let Some((x, y)) = c.xy(sid) {
            stamp_if_blank(&mut grid, (x + 1, y), '>');
            stamp_if_blank(&mut grid, (x + 2, y), '<');
        }
    }
    for sid in marks.seal_sites {
        if let Some((x, y)) = c.xy(sid) {
            let ssl = site(sid).is_some_and(|s| s.sea_lion >= 0.35);
            let ch = if ssl { 'S' } else { 's' };
            stamp_if_blank(&mut grid, (x + 1, y), ch);
        }
    }

    for sk in marks.skiffs {
        let xy = if sk.location == "transit" {
            c.transit_xy(sk.from.as_deref(), sk.dest.as_deref(), sk.eta)
        } else {
            c.xy(&sk.location).unwrap_or(c.camp)
        };
        let ch = if sk.kind == "holding" { '»' } else { '›' };
        let xy = if sk.location == "camp" {
            let off = if sk.kind == "holding" { 6 } else { 5 };
            (c.camp.0 + off, c.camp.1)
        } else if sk.location == "tender" {
            (c.tender.0.saturating_sub(1), c.tender.1)
        } else {
            xy
        };
        stamp(&mut grid, xy, ch);
    }

    let body: String = grid
        .into_iter()
        .map(|row| row.into_iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n");
    format!("{}\n{}", c.title, body)
}
