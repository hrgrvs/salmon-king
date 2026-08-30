//! Limited fish-camp palette: wet wool, diesel, kelp, cork orange, night water.

use ratatui::style::{Color, Modifier, Style};

/// Night water — panel ground.
pub const NIGHT: Color = Color::Rgb(10, 16, 22);
/// Slightly lifted card.
pub const CARD: Color = Color::Rgb(16, 24, 30);
/// HUD strip.
pub const HUD: Color = Color::Rgb(22, 32, 38);
/// Foam / title cream.
pub const FOAM: Color = Color::Rgb(220, 214, 196);
/// Wet wool.
pub const WOOL: Color = Color::Rgb(138, 148, 148);
/// Diesel brown.
pub const DIESEL: Color = Color::Rgb(78, 64, 46);
/// Kelp.
pub const KELP: Color = Color::Rgb(58, 90, 62);
/// Bright kelp / open water.
pub const KELP_LT: Color = Color::Rgb(106, 148, 96);
/// Cork orange.
pub const CORK: Color = Color::Rgb(204, 108, 42);
/// Copper / dark-period red.
pub const COPPER: Color = Color::Rgb(180, 83, 42);
/// Salmon flesh.
pub const FLESH: Color = Color::Rgb(196, 92, 74);
/// Flavor body text.
pub const FLAVOR: Color = Color::Rgb(196, 180, 154);
/// Deep water ink for waves.
pub const WATER: Color = Color::Rgb(42, 72, 92);
/// Orca / night accent.
pub const INK: Color = Color::Rgb(186, 196, 204);

pub fn foam_bold() -> Style {
    Style::default().fg(FOAM).add_modifier(Modifier::BOLD)
}

pub fn cork_bold() -> Style {
    Style::default().fg(CORK).add_modifier(Modifier::BOLD)
}

pub fn kelp_bold() -> Style {
    Style::default().fg(KELP_LT).add_modifier(Modifier::BOLD)
}

pub fn muted() -> Style {
    Style::default().fg(WOOL)
}

pub fn body() -> Style {
    Style::default().fg(FLAVOR)
}

pub fn warn() -> Style {
    Style::default().fg(FLESH).add_modifier(Modifier::BOLD)
}

pub fn open_tag() -> Style {
    Style::default().fg(KELP_LT).add_modifier(Modifier::BOLD)
}

pub fn dark_tag() -> Style {
    Style::default().fg(COPPER).add_modifier(Modifier::BOLD)
}
