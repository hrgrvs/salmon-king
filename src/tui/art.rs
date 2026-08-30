//! Classic ASCII / ANSI fish-camp drawings. No emoji as the main art.

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use super::theme::{
    body, cork_bold, foam_bold, muted, CORK, FLESH, FOAM, INK, KELP_LT, WATER, WOOL,
};

pub const TITLE_CAMP: &[&str] = &[
    r"          ~    ~       ><{{{{o>        ~     ~",
    r"     ~~~~    ~~~~    ~~~~    ~~~~    ~~~~    ~~~~",
    r"    /     Shelikof Strait · short steep seas     \",
    r"   |      ,-=\                                   |",
    r"   |     /____\     ╫═══════  corkline           |",
    r"   |     o    o                                  |",
    r"   |               .--^--.                       |",
    r"   |              / cabin \   [cook]   [loft]    |",
    r"   |             |_________|                     |",
    r"   |                      ▣  tender in the hole  |",
    r"    \____________________________________________/",
];

pub const TITLE_BANNER: &[&str] = &[
    "╔══════════════════════════════════════════════════╗",
    "║            S A L M O N   K I N G                 ║",
    "║       Kodiak Island · S04K set gillnet           ║",
    "╚══════════════════════════════════════════════════╝",
];

pub const TITLE_SKIFF: &[&str] = &[
    r"      ,-=\              ,=======.",
    r"     /____\            / [HOLD]  \",
    r"     o    o            `--o---o--'",
    r"    picking skiff       holding skiff",
];

pub const PICKER_BOAT: &[&str] = &[r"  ,-=\ ", r" /____\ ", r" o    o "];

pub const HOLDING_BOAT: &[&str] = &[r" ,=======.", r#"/ [HOLD]  \"#, r"`--o---o--'"];

pub const WRECK_BOAT: &[&str] = &[r"   ~~~  ", r" \____/ ", r"  x    x "];

pub const CABIN: &[&str] = &[
    r"  .--^--. ",
    r#" / cabin \"#,
    r"| cook ## |",
    r" [ loft ] ",
];

pub const SEA_LION: &[&str] = &[r"   (o o)  ", r#"  / V  \  "#, r"  ^^^^^^  "];

pub const ORCA: &[&str] = &[
    r#"    __/\_   "#,
    r#" __/    \__ "#,
    r#"/   ω      \"#,
    r#"\_________/ "#,
];

pub const WILLIWAW: &[&str] = &[r"  \\\\  //// ", r" ~~~~~~~~~~ ", r"  katabatic "];

pub const HELP_HEAD: &[&str] = &[
    r"  ,-=\   ╫════  ▲ camp   ▣ tender",
    r" /____\  corkline on a Shelikof beach",
    r" o    o",
];

pub const NEW_SEASON_HEAD: &[&str] = &[
    r"  .--^--.     pick a beach, a year, a seed",
    r" / cabin \    odd year: pink flood   even: thin",
    r"|_________|",
];

/// One-glyph crew figure so a picker, cook, and skipper read differently.
pub fn crew_glyph(role: &str, activity: &str) -> &'static str {
    if activity == "quit" {
        return " x ";
    }
    if activity == "sick" {
        return " o~";
    }
    if activity == "sleeping" {
        return "_o_";
    }
    if activity == "town" || activity.starts_with("on the town") {
        return " o>";
    }
    if activity == "cooking" {
        return " ô)";
    }
    if activity.starts_with("mending") {
        return " o~";
    }
    if activity.starts_with("on the tender") {
        return " o»";
    }
    if activity.starts_with("picking") {
        return " o/";
    }
    match role {
        "cook" => " ô)",
        "operator" => " O/",
        "picker" => " o/",
        _ => " o ",
    }
}

pub fn picker_boat(wrecked: bool) -> &'static [&'static str] {
    if wrecked {
        WRECK_BOAT
    } else {
        PICKER_BOAT
    }
}

pub fn holding_boat(wrecked: bool) -> &'static [&'static str] {
    if wrecked {
        WRECK_BOAT
    } else {
        HOLDING_BOAT
    }
}

pub fn boat_for_kind(kind: &str, wrecked: bool) -> &'static [&'static str] {
    if kind == "holding" {
        holding_boat(wrecked)
    } else {
        picker_boat(wrecked)
    }
}

pub fn toast_for(text: &str) -> Option<(&'static str, &'static [&'static str])> {
    let t = text.to_ascii_lowercase();
    if t.contains("williwaw") || t.contains("katabatic") {
        Some(("WILLIWAW", WILLIWAW))
    } else if t.contains("transient")
        || (t.contains("orca") && !t.contains("resident"))
        || t.contains("killer whale")
    {
        Some(("TRANSIENTS", ORCA))
    } else if t.contains("steller") || t.contains("sea lion") || t.contains("lions on") {
        Some(("STELLERS", SEA_LION))
    } else {
        None
    }
}

pub fn styled_art_lines(rows: &[&str], style: Style) -> Vec<Line<'static>> {
    rows.iter()
        .map(|row| Line::from(Span::styled((*row).to_string(), style)))
        .collect()
}

pub fn title_art_scaled(height: u16) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let camp = if height >= 26 {
        TITLE_CAMP
    } else {
        &TITLE_CAMP[0..6]
    };
    for (i, row) in camp.iter().enumerate() {
        let style = if i <= 1 {
            Style::default().fg(WATER)
        } else {
            body()
        };
        if row.contains("╫") || row.contains(",-=") {
            lines.push(color_camp_row(row));
        } else {
            lines.push(Line::from(Span::styled((*row).to_string(), style)));
        }
    }
    for (i, row) in TITLE_BANNER.iter().enumerate() {
        let style = if i == 1 {
            foam_bold()
        } else if i == 2 {
            Style::default().fg(WOOL)
        } else {
            Style::default().fg(CORK)
        };
        lines.push(Line::from(Span::styled((*row).to_string(), style)));
    }
    if height >= 22 {
        for row in TITLE_SKIFF {
            lines.push(color_camp_row(row));
        }
    }
    lines
}

fn color_camp_row(row: &str) -> Line<'static> {
    let mut spans = Vec::new();
    let chars: Vec<char> = row.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        let (style, n) = match ch {
            '~' => (Style::default().fg(WATER), 1),
            '╫' | '═' => (Style::default().fg(CORK), 1),
            'o' if i + 1 < chars.len() && chars[i + 1] == ' ' || ch == 'o' => {
                (Style::default().fg(FOAM), 1)
            }
            '▣' | '▲' | '^' => (Style::default().fg(KELP_LT), 1),
            '>' | '<' => (Style::default().fg(FLESH), 1),
            _ => (body(), 1),
        };
        spans.push(Span::styled(ch.to_string(), style));
        i += n;
    }
    Line::from(spans)
}

pub fn help_header() -> Vec<Line<'static>> {
    let mut lines = styled_art_lines(HELP_HEAD, body());
    if let Some(first) = lines.get_mut(0) {
        *first = Line::from(vec![
            Span::styled("  ,-=\\   ", Style::default().fg(FOAM)),
            Span::styled("╫════  ", cork_bold()),
            Span::styled("▲ camp   ", Style::default().fg(KELP_LT)),
            Span::styled("▣ tender", Style::default().fg(WOOL)),
        ]);
    }
    lines
}

pub fn wave_rule(width: usize) -> String {
    let n = width.max(8);
    "~ ".repeat(n / 2 + 1).chars().take(n).collect()
}

pub fn stamp_block(label: &str, art: &[&str], text: &str) -> Vec<Line<'static>> {
    let mut lines = vec![Line::from(Span::styled(
        format!(" {label} "),
        Style::default()
            .fg(FOAM)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    ))];
    let art_w = art.iter().map(|r| r.chars().count()).max().unwrap_or(0);
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut wrapped = Vec::new();
    let mut cur = String::new();
    let wrap_at = 48usize;
    for w in words {
        if !cur.is_empty() && cur.len() + 1 + w.len() > wrap_at {
            wrapped.push(std::mem::take(&mut cur));
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(w);
    }
    if !cur.is_empty() {
        wrapped.push(cur);
    }
    let rows = art.len().max(wrapped.len());
    for i in 0..rows {
        let a = art.get(i).copied().unwrap_or("");
        let t = wrapped.get(i).map(String::as_str).unwrap_or("");
        let pad = " ".repeat(art_w.saturating_sub(a.chars().count()) + 2);
        lines.push(Line::from(vec![
            Span::styled(format!("{a}{pad}"), Style::default().fg(INK)),
            Span::styled(t.to_string(), muted()),
        ]));
    }
    lines
}
