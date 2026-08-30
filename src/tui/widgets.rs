//! Glanceable HUD panes. Boats and crew speak in words, not just bars.

use std::collections::{HashMap, HashSet};

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use salmon_king::data::maps::{render_map_marked, MapMarks};
use salmon_king::data::sites::site;
use salmon_king::data::species::{short_code, SpeciesId};
use salmon_king::sim::engine::Game;
use salmon_king::sim::mammals::{empty_haulout_sites, resident_sites, transient_sites};
use salmon_king::sim::models::CrewStatus;
use salmon_king::sim::status::{crew_glance, skiff_status};

use super::art::{boat_for_kind, crew_glyph, stamp_block, toast_for, wave_rule, CABIN, WILLIWAW};
use super::theme::{
    body, cork_bold, dark_tag, foam_bold, kelp_bold, muted, open_tag, warn, CARD, CORK, DIESEL,
    FLESH, FOAM, KELP, KELP_LT, WATER, WOOL,
};

pub fn bar(value: f64, width: usize) -> String {
    let v = value.clamp(0.0, 100.0);
    let fill = ((v / 100.0) * width as f64).round() as usize;
    let fill = fill.min(width);
    format!("{}{}", "█".repeat(fill), "░".repeat(width - fill))
}

fn pane(title: &str, border: ratatui::style::Color) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border))
        .title(format!(" {title} "))
        .title_style(Style::default().fg(FOAM).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(CARD).fg(FOAM))
}

fn colorize_map(text: &str) -> Vec<Line<'static>> {
    text.lines()
        .enumerate()
        .map(|(i, line)| {
            if i == 0 {
                return Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(WOOL).add_modifier(Modifier::ITALIC),
                ));
            }
            let spans: Vec<Span> = line
                .chars()
                .map(|ch| {
                    let style = match ch {
                        '~' | '≈' => Style::default().fg(WATER),
                        '╫' => Style::default().fg(CORK).add_modifier(Modifier::BOLD),
                        '▲' => Style::default().fg(KELP_LT).add_modifier(Modifier::BOLD),
                        '›' | '»' => Style::default().fg(FOAM).add_modifier(Modifier::BOLD),
                        '▣' | '■' => Style::default().fg(DIESEL).add_modifier(Modifier::BOLD),
                        '●' => Style::default().fg(CORK).add_modifier(Modifier::BOLD),
                        '·' => Style::default().fg(KELP_LT),
                        '×' | 'x' => Style::default().fg(FLESH),
                        'ω' => Style::default().fg(FOAM).add_modifier(Modifier::BOLD),
                        '>' | '<' => Style::default().fg(FLESH),
                        '*' => Style::default().fg(CORK),
                        '[' | ']' => Style::default().fg(WOOL),
                        _ => body(),
                    };
                    Span::styled(ch.to_string(), style)
                })
                .collect();
            Line::from(spans)
        })
        .collect()
}

pub fn draw_map(f: &mut Frame, area: Rect, game: &Game, cursor: Option<&str>) {
    let block = pane(
        " map · ▲ camp  ╫ net  › pick  » hold  ▣ tender  ω orca  ><> fish ",
        KELP,
    );
    let inner = block.inner(area);
    f.render_widget(block, area);

    let deployed: HashMap<String, String> = game
        .nets
        .iter()
        .filter(|n| n.in_water)
        .filter_map(|n| n.site_id.clone().map(|s| (n.id.clone(), s)))
        .collect();
    let skiffs: Vec<(String, String, String)> = game
        .skiffs
        .iter()
        .filter(|s| !s.wrecked)
        .map(|s| (s.id.clone(), s.location.clone(), s.kind.clone()))
        .collect();
    let mut closed = HashSet::new();
    for s in game.playable_sites() {
        if !game.site_is_open(s.id).0 {
            closed.insert(s.id.to_string());
        }
    }
    let trans = transient_sites(&game.wildlife);
    let res = resident_sites(&game.wildlife);
    let text = render_map_marked(
        game.camp.map_id,
        game.camp.id,
        &MapMarks {
            deployed: &deployed,
            skiffs: &skiffs,
            tender_here: game.tender.present,
            closed_sites: &closed,
            cursor_site: cursor,
            transient_sites: &trans,
            resident_sites: &res,
            fish_showing: game.bonus_school > 0,
        },
    );

    let mut lines = colorize_map(&text);
    lines.push(Line::from(Span::styled(
        wave_rule(inner.width as usize),
        Style::default().fg(WATER),
    )));
    for n in &game.nets {
        if n.in_water {
            if let Some(sid) = &n.site_id {
                if let Some(sdef) = site(sid) {
                    let lions = if empty_haulout_sites(&game.wildlife).contains(sid) {
                        "  lions gone"
                    } else {
                        ""
                    };
                    lines.push(Line::from(vec![
                        Span::styled(" ╫ ", cork_bold()),
                        Span::styled(
                            format!(
                                "{}  {}  {:.0} lb  soak {}  cond {:.0}%  {}{lions}",
                                n.id,
                                sdef.short,
                                n.fish.total(),
                                n.soak_tides,
                                n.condition,
                                n.mesh
                            ),
                            body(),
                        ),
                    ]));
                }
            }
        } else {
            lines.push(Line::from(Span::styled(
                format!(
                    " ╫ {} on the beach  cond {:.0}%  {}",
                    n.id, n.condition, n.mesh
                ),
                muted(),
            )));
        }
    }
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

pub fn draw_boats(f: &mut Frame, area: Rect, game: &Game) {
    let block = pane(" boats ", CORK);
    let inner = block.inner(area);
    f.render_widget(block, area);

    if game.skiffs.is_empty() {
        f.render_widget(Paragraph::new("No skiffs.").style(muted()), inner);
        return;
    }

    let n = game.skiffs.len().max(1);
    let constraints: Vec<Constraint> = game
        .skiffs
        .iter()
        .map(|_| Constraint::Ratio(1, n as u32))
        .collect();
    // Full-width strip: boats sit side-by-side. Narrow panes stack.
    let dir = if inner.width >= 64 {
        Direction::Horizontal
    } else {
        Direction::Vertical
    };
    let slots = Layout::default()
        .direction(dir)
        .constraints(constraints)
        .split(inner);

    for (i, skiff) in game.skiffs.iter().enumerate() {
        let st = skiff_status(game, skiff);
        let art = boat_for_kind(&st.kind, st.wrecked);
        let crew = if st.crew_names.is_empty() {
            "no crew".to_string()
        } else {
            st.crew_names.join(", ")
        };
        let job_style = if st.wrecked {
            warn()
        } else if st.job_words == "idle in the hole" || st.job_words == "idle" {
            muted()
        } else {
            cork_bold()
        };
        // Words first so a short pane still answers "what is this skiff doing?"
        let mut lines = vec![Line::from(vec![
            Span::styled(art[0].to_string(), Style::default().fg(FOAM)),
            Span::styled(format!("  {}  ", st.name.to_uppercase()), foam_bold()),
            Span::styled(st.kind.clone(), muted()),
        ])];
        if slots[i].height >= 7 {
            for row in art.iter().skip(1) {
                lines.push(Line::from(Span::styled(
                    (*row).to_string(),
                    Style::default().fg(FOAM),
                )));
            }
        }
        lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(st.where_words.clone(), body()),
            Span::styled("  ·  ", muted()),
            Span::styled(st.job_words.clone(), job_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(crew, Style::default().fg(KELP_LT)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                format!(
                    "{:.0} lb   fuel {:.0}   cond {:.0}%",
                    st.cargo_lb, st.fuel, st.condition
                ),
                body(),
            ),
        ]));
        f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), slots[i]);
    }
}

pub fn draw_crew(f: &mut Frame, area: Rect, game: &Game) {
    let block = pane(" crew ", KELP_LT);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines = Vec::new();
    let compact = inner.height < (game.crew.len() as u16 * 2).saturating_add(1);
    for c in &game.crew {
        let g = crew_glance(game, c);
        let glyph = crew_glyph(&g.role, &g.activity);
        let own = if g.is_owner { "*" } else { " " };
        let act_style = match g.status {
            CrewStatus::Quit => warn(),
            CrewStatus::Sick => warn(),
            CrewStatus::Resting => muted(),
            CrewStatus::Town => Style::default().fg(CORK),
            CrewStatus::Working => kelp_bold(),
        };
        let name = if compact {
            g.name
                .split_whitespace()
                .next()
                .unwrap_or(&g.name)
                .to_string()
        } else {
            g.name.clone()
        };
        let activity = if compact {
            compact_activity(&g.activity, g.skiff_name.as_deref())
        } else {
            g.activity.clone()
        };
        let mut spans = vec![
            Span::styled(format!("{glyph} "), Style::default().fg(FOAM)),
            Span::styled(format!("{name}{own}  "), foam_bold()),
        ];
        if !compact {
            spans.push(Span::styled(format!("{}  ", g.role), muted()));
        }
        spans.push(Span::styled(activity, act_style));
        lines.push(Line::from(spans));
        if !compact && g.status != CrewStatus::Quit {
            lines.push(Line::from(vec![
                Span::styled("     E ", muted()),
                Span::styled(bar(g.energy, 8), Style::default().fg(KELP_LT)),
                Span::styled("  H ", muted()),
                Span::styled(bar(100.0 - g.hunger, 8), Style::default().fg(CORK)),
                Span::styled("  M ", muted()),
                Span::styled(bar(g.morale, 8), Style::default().fg(FOAM)),
            ]));
        }
    }
    let para = Paragraph::new(lines);
    if compact {
        f.render_widget(para, inner);
    } else {
        f.render_widget(para.wrap(Wrap { trim: true }), inner);
    }
}

fn compact_activity(activity: &str, skiff: Option<&str>) -> String {
    // Keep the verb and the place; the boats pane already names the hull.
    if let Some(rest) = activity.strip_prefix("picking net at ") {
        let place = rest.split(" (").next().unwrap_or(rest);
        let mark = skiff_mark(skiff);
        return format!("pick @ {place}{mark}");
    }
    if activity.starts_with("on the tender run") {
        return match skiff {
            Some(s) => format!("tender run ({s})"),
            None => "tender run".into(),
        };
    }
    if activity.starts_with("on the town run") {
        return match skiff {
            Some(s) => format!("town run ({s})"),
            None => "town run".into(),
        };
    }
    if let Some(rest) = activity.strip_prefix("mending (") {
        return format!("mending ({})", rest.trim_end_matches(')'));
    }
    if let Some(rest) = activity.strip_prefix("idle on ") {
        return format!("idle on {rest}");
    }
    activity.to_string()
}

fn skiff_mark(skiff: Option<&str>) -> String {
    match skiff {
        Some(s) if s.to_ascii_lowercase().contains("hold") => " »".into(),
        Some(s) if !s.is_empty() => " ›".into(),
        _ => String::new(),
    }
}

pub fn draw_tender(f: &mut Frame, area: Rect, game: &Game) {
    let block = pane(" tender ", DIESEL);
    let inner = block.inner(area);
    f.render_widget(block, area);
    let p = &game.tender.prices;
    let eta = if game.tender.present {
        Line::from(Span::styled("IN THE HOLE", kelp_bold()))
    } else {
        Line::from(Span::styled(
            format!("offshore  ETA {} tides", game.tender.eta_tides),
            muted(),
        ))
    };
    let prices = SpeciesId::ALL
        .iter()
        .map(|&sp| format!("{} ${:.2}", short_code(sp), p.get(sp)))
        .collect::<Vec<_>>()
        .join("  ");
    let lines = vec![
        Line::from(vec![
            Span::styled(format!("{}  ", game.tender.name), foam_bold()),
            eta.spans.first().cloned().unwrap_or_else(|| Span::raw("")),
        ]),
        Line::from(Span::styled("  ▣═══▣", Style::default().fg(DIESEL))),
        Line::from(Span::styled(prices, body())),
        Line::from(Span::styled(
            format!("last: {}", game.tender.last_note),
            muted(),
        )),
    ];
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

pub fn draw_camp(f: &mut Frame, area: Rect, game: &Game) {
    let block = pane(" camp ", DIESEL);
    let inner = block.inner(area);
    f.render_widget(block, area);
    let people = game
        .crew
        .iter()
        .filter(|c| c.status != CrewStatus::Quit)
        .count()
        .max(1);
    let days = game.food / people as f64;
    let jv = if game.joint_venture {
        "  JV 3rd net"
    } else {
        ""
    };
    let mut lines = vec![Line::from(Span::styled(
        format!("{}{jv}", game.camp.name),
        foam_bold(),
    ))];
    if inner.height >= 6 {
        for row in CABIN {
            lines.push(Line::from(Span::styled(
                (*row).to_string(),
                Style::default().fg(WOOL),
            )));
        }
    } else {
        lines.push(Line::from(Span::styled(
            CABIN[0].to_string(),
            Style::default().fg(WOOL),
        )));
    }
    lines.push(Line::from(Span::styled(
        format!(
            "cash ${:.0}   food {days:.1} d   fuel {:.0} gal",
            game.ledger.cash, game.fuel_cache
        ),
        body(),
    )));
    lines.push(Line::from(Span::styled(
        format!(
            "ice {:.0}   twine {}   prop {}   cook {} bunk {} loft {} stall {}",
            game.ice_cache,
            game.twine,
            game.spare_prop,
            game.cookshack,
            game.bunkhouse,
            game.net_loft,
            game.skiff_stalls
        ),
        muted(),
    )));
    lines.push(Line::from(Span::styled(
        format!(
            "gross ${:.0}  exp ${:.0}  tickets {}",
            game.ledger.gross,
            game.ledger.expenses(),
            game.ledger.tickets
        ),
        body(),
    )));
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

pub fn draw_clock(f: &mut Frame, area: Rect, game: &Game) {
    let block = pane(" clock ", WATER);
    let inner = block.inner(area);
    f.render_widget(block, area);
    let open = game.any_open();
    let tag = if open { "OPEN" } else { "DARK" };
    let tag_style = if open { open_tag() } else { dark_tag() };
    let skip = if game.skipper_in_town {
        "  PERMIT OFF-SITE"
    } else {
        ""
    };
    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                format!("{}  {}  ", game.day.fmt_long(), game.tide.as_str()),
                foam_bold(),
            ),
            Span::styled(tag, tag_style),
            Span::styled(skip, warn()),
        ]),
        Line::from(Span::styled(game.phase_label(), body())),
        Line::from(Span::styled(game.open_reason(), muted())),
        Line::from(Span::styled(game.weather.label.clone(), body())),
    ];
    if game.weather.williwaw {
        lines.push(Line::from(Span::styled(WILLIWAW[0], warn())));
        lines.push(Line::from(Span::styled(
            WILLIWAW[1],
            Style::default().fg(WATER),
        )));
    }
    for m in game.mammal_status().into_iter().take(2) {
        lines.push(Line::from(Span::styled(m, muted())));
    }
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

pub fn draw_log(f: &mut Frame, area: Rect, game: &Game) {
    let block = pane(" event log ", CORK);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let toast = game.log.iter().rev().find_map(|ln| {
        toast_for(&ln.text).map(|(label, art)| (label, art, ln.text.as_str(), ln.day))
    });

    let stamp_h = if toast.is_some() { 5 } else { 0 };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(stamp_h), Constraint::Min(2)])
        .split(inner);

    if let Some((label, art, text, day)) = toast {
        let head = format!("{}  {}", day.fmt_md(), text);
        f.render_widget(Paragraph::new(stamp_block(label, art, &head)), chunks[0]);
    }

    let n = chunks[1].height as usize;
    let lines: Vec<_> = game.log.iter().rev().take(n.max(3)).rev().collect();
    let body_lines: Vec<Line> = if lines.is_empty() {
        vec![Line::from(Span::styled("Waiting on the VHF.", muted()))]
    } else {
        lines
            .iter()
            .map(|ln| {
                let kind_style = match ln.kind.as_str() {
                    "mammal" | "event" => Style::default().fg(FLESH),
                    "weather" => Style::default().fg(WATER),
                    "adfg" => cork_bold(),
                    "skiff" | "gear" => kelp_bold(),
                    _ => body(),
                };
                let stamp = if toast_for(&ln.text).is_some() {
                    match toast_for(&ln.text).map(|(k, _)| k) {
                        Some("WILLIWAW") => " \\\\",
                        Some("TRANSIENTS") => " ω ",
                        Some("STELLERS") => " ^ ",
                        _ => " · ",
                    }
                } else {
                    " · "
                };
                Line::from(vec![
                    Span::styled(format!(" {} ", ln.day.fmt_md()), muted()),
                    Span::styled(stamp, kind_style),
                    Span::styled(ln.text.clone(), kind_style),
                ])
            })
            .collect()
    };
    f.render_widget(
        Paragraph::new(body_lines).wrap(Wrap { trim: true }),
        chunks[1],
    );
}
