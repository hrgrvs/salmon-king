//! Keyboard-first ratatui HUD. The sim is a client of this module, not the reverse.

mod screens;
mod widgets;

use std::io::{self, stdout};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::{Frame, Terminal};

use salmon_king::data::camps::CAMP_IDS;
use salmon_king::sim::engine::{new_game, Game};
use salmon_king::sim::models::GameEnd;

use screens::{
    buy_items, crew_items, hire_items, mesh_items, pull_items, skiff_items, upgrade_items, ActionItem,
};
use widgets::{
    render_camp, render_clock, render_crew, render_log, render_map_panel, render_tender,
};

const BG: Color = Color::Rgb(14, 20, 24);
const CARD: Color = Color::Rgb(18, 27, 33);
const HUD_BG: Color = Color::Rgb(26, 36, 43);
const TITLE: Color = Color::Rgb(232, 213, 181);
const MUTED: Color = Color::Rgb(143, 163, 173);
const FLAVOR: Color = Color::Rgb(196, 180, 154);
const COPPER: Color = Color::Rgb(180, 83, 42);
const MAP_BORDER: Color = Color::Rgb(61, 90, 76);
const OPEN: Color = Color::Rgb(143, 188, 143);
const DARK: Color = Color::Rgb(196, 92, 74);

pub fn run() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = App::new();
    let result = app_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

struct App {
    screen: Screen,
    should_quit: bool,
}

enum Screen {
    Title,
    NewSeason(NewSeason),
    Play(Play),
}

struct NewSeason {
    camp_i: usize,
    year: String,
    seed: String,
    field: Field,
}

enum Field {
    Camp,
    Year,
    Seed,
}

struct Play {
    game: Game,
    speed: u32,
    paused: bool,
    cursor_i: usize,
    overlay: Overlay,
    last_tick: Instant,
    recap_text: String,
}

enum Overlay {
    None,
    Help,
    Quit,
    Recap,
    Picker {
        title: String,
        kind: String,
        items: Vec<ActionItem>,
        selected: usize,
    },
}

impl App {
    fn new() -> Self {
        Self {
            screen: Screen::Title,
            should_quit: false,
        }
    }
}

fn interval(speed: u32) -> Duration {
    match speed {
        4 => Duration::from_millis(400),
        16 => Duration::from_millis(120),
        _ => Duration::from_millis(1600),
    }
}

fn app_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|f| draw(f, app))?;

        let timeout = match &app.screen {
            Screen::Play(p) if matches!(p.overlay, Overlay::None) && !p.paused && p.game.end == GameEnd::None => {
                let due = p.last_tick + interval(p.speed);
                due.saturating_duration_since(Instant::now()).min(Duration::from_millis(50))
            }
            _ => Duration::from_millis(100),
        };

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key(app, key);
                }
            }
        }

        if let Screen::Play(p) = &mut app.screen {
            if matches!(p.overlay, Overlay::None)
                && !p.paused
                && p.game.end == GameEnd::None
                && p.last_tick.elapsed() >= interval(p.speed)
            {
                p.game.step();
                p.last_tick = Instant::now();
                if p.game.end != GameEnd::None {
                    p.recap_text = p.game.recap().as_text();
                    p.overlay = Overlay::Recap;
                }
            }
        }
    }
    Ok(())
}

fn handle_key(app: &mut App, key: KeyEvent) {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        app.should_quit = true;
        return;
    }
    match &mut app.screen {
        Screen::Title => match key.code {
            KeyCode::Char('n') | KeyCode::Enter => {
                app.screen = Screen::NewSeason(NewSeason {
                    camp_i: 1,
                    year: "2025".into(),
                    seed: "1701".into(),
                    field: Field::Camp,
                });
            }
            KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
            _ => {}
        },
        Screen::NewSeason(ns) => match key.code {
            KeyCode::Esc => app.screen = Screen::Title,
            KeyCode::Tab | KeyCode::Down if matches!(ns.field, Field::Camp) => {
                if key.code == KeyCode::Tab {
                    ns.field = Field::Year;
                } else {
                    ns.camp_i = (ns.camp_i + 1) % CAMP_IDS.len();
                }
            }
            KeyCode::Up if matches!(ns.field, Field::Camp) => {
                ns.camp_i = (ns.camp_i + CAMP_IDS.len() - 1) % CAMP_IDS.len();
            }
            KeyCode::Char('j') if matches!(ns.field, Field::Camp) => {
                ns.camp_i = (ns.camp_i + 1) % CAMP_IDS.len();
            }
            KeyCode::Char('k') if matches!(ns.field, Field::Camp) => {
                ns.camp_i = (ns.camp_i + CAMP_IDS.len() - 1) % CAMP_IDS.len();
            }
            KeyCode::Tab => {
                ns.field = match ns.field {
                    Field::Camp => Field::Year,
                    Field::Year => Field::Seed,
                    Field::Seed => Field::Camp,
                };
            }
            KeyCode::BackTab => {
                ns.field = match ns.field {
                    Field::Camp => Field::Seed,
                    Field::Year => Field::Camp,
                    Field::Seed => Field::Year,
                };
            }
            KeyCode::Backspace => match ns.field {
                Field::Year => {
                    ns.year.pop();
                }
                Field::Seed => {
                    ns.seed.pop();
                }
                Field::Camp => {}
            },
            KeyCode::Char(c) if c.is_ascii_digit() => match ns.field {
                Field::Year if ns.year.len() < 4 => ns.year.push(c),
                Field::Seed if ns.seed.len() < 8 => ns.seed.push(c),
                Field::Year | Field::Seed => {}
                Field::Camp => {}
            },
            KeyCode::Enter => {
                let year: i32 = ns.year.parse().unwrap_or(2025);
                let seed: u64 = ns.seed.parse().unwrap_or(1701);
                let camp = CAMP_IDS[ns.camp_i];
                match new_game(seed, camp, year) {
                    Ok(game) => {
                        app.screen = Screen::Play(Play {
                            game,
                            speed: 1,
                            paused: false,
                            cursor_i: 0,
                            overlay: Overlay::None,
                            last_tick: Instant::now(),
                            recap_text: String::new(),
                        });
                    }
                    Err(_) => {}
                }
            }
            _ => {}
        },
        Screen::Play(_) => handle_key_play(app, key),
    }
}

fn handle_key_play(app: &mut App, key: KeyEvent) {
    let Screen::Play(p) = &mut app.screen else {
        return;
    };
    match &mut p.overlay {
        Overlay::Help => {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q')) {
                p.overlay = Overlay::None;
            }
        }
        Overlay::Quit => match key.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                app.should_quit = true;
            }
            KeyCode::Char('n') | KeyCode::Esc | KeyCode::Char('q') => {
                p.overlay = Overlay::None;
            }
            _ => {}
        },
        Overlay::Recap => {
            if matches!(key.code, KeyCode::Enter | KeyCode::Char('q') | KeyCode::Esc) {
                app.screen = Screen::Title;
            }
        }
        Overlay::Picker {
            kind,
            items,
            selected,
            ..
        } => match key.code {
            KeyCode::Esc | KeyCode::Char('q') => p.overlay = Overlay::None,
            KeyCode::Up | KeyCode::Char('k') => {
                if *selected > 0 {
                    *selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if *selected + 1 < items.len() {
                    *selected += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(item) = items.get(*selected).cloned() {
                    let kind = kind.clone();
                    let cursor = {
                        let sites = p.game.playable_sites();
                        sites
                            .get(p.cursor_i % sites.len().max(1))
                            .map(|s| s.id.to_string())
                    };
                    p.overlay = Overlay::None;
                    apply_action(&mut p.game, &kind, &item.id, cursor.as_deref());
                }
            }
            _ => {}
        },
        Overlay::None => match key.code {
            KeyCode::Char(' ') => {
                p.paused = !p.paused;
                if !p.paused && p.speed == 0 {
                    p.speed = 1;
                }
                p.last_tick = Instant::now();
            }
            KeyCode::Char('1') => {
                p.speed = 1;
                p.paused = false;
                p.last_tick = Instant::now();
            }
            KeyCode::Char('4') => {
                p.speed = 4;
                p.paused = false;
                p.last_tick = Instant::now();
            }
            KeyCode::Char('x') => {
                p.speed = 16;
                p.paused = false;
                p.last_tick = Instant::now();
            }
            KeyCode::Left => {
                let n = p.game.playable_sites().len().max(1);
                p.cursor_i = (p.cursor_i + n - 1) % n;
            }
            KeyCode::Right => {
                let n = p.game.playable_sites().len().max(1);
                p.cursor_i = (p.cursor_i + 1) % n;
            }
            KeyCode::Enter => {
                let sites = p.game.playable_sites();
                if let Some(site) = sites.get(p.cursor_i % sites.len().max(1)) {
                    let site_id = site.id.to_string();
                    let net = p
                        .game
                        .nets
                        .iter()
                        .find(|n| !n.in_water)
                        .map(|n| n.id.clone())
                        .or_else(|| p.game.nets.first().map(|n| n.id.clone()));
                    if let Some(nid) = net {
                        if p.game.nets.iter().all(|n| n.in_water) {
                            p.game.pull_net(&nid);
                        }
                        let msg = p.game.deploy_net(&nid, &site_id);
                        p.game.note(msg, "gear");
                    }
                }
            }
            KeyCode::Char('p') => {
                p.overlay = Overlay::Picker {
                    title: "Pull which net?".into(),
                    kind: "pull".into(),
                    items: pull_items(&p.game),
                    selected: 0,
                };
            }
            KeyCode::Char('s') => {
                p.overlay = Overlay::Picker {
                    title: "Skiff job".into(),
                    kind: "skiff".into(),
                    items: skiff_items(&p.game),
                    selected: 0,
                };
            }
            KeyCode::Char('c') => {
                p.overlay = Overlay::Picker {
                    title: "Assign crew".into(),
                    kind: "crew".into(),
                    items: crew_items(&p.game),
                    selected: 0,
                };
            }
            KeyCode::Char('h') => {
                p.overlay = Overlay::Picker {
                    title: "Hire (tender)".into(),
                    kind: "hire".into(),
                    items: hire_items(&p.game),
                    selected: 0,
                };
            }
            KeyCode::Char('b') => {
                p.overlay = Overlay::Picker {
                    title: "Tender store".into(),
                    kind: "buy".into(),
                    items: buy_items(),
                    selected: 0,
                };
            }
            KeyCode::Char('u') => {
                p.overlay = Overlay::Picker {
                    title: "Freight / upgrade".into(),
                    kind: "upgrade".into(),
                    items: upgrade_items(),
                    selected: 0,
                };
            }
            KeyCode::Char('m') => {
                p.overlay = Overlay::Picker {
                    title: "Mesh (game selectivity)".into(),
                    kind: "mesh".into(),
                    items: mesh_items(&p.game),
                    selected: 0,
                };
            }
            KeyCode::Char('j') => {
                let msg = p.game.form_joint_venture();
                p.game.note(msg, "gear");
            }
            KeyCode::Char('t') => {
                if let Some(fat) = p
                    .game
                    .skiffs
                    .iter()
                    .max_by(|a, b| {
                        a.cargo
                            .total()
                            .partial_cmp(&b.cargo.total())
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|s| s.id.clone())
                {
                    p.game.assign_skiff(&fat, "tender", None);
                }
            }
            KeyCode::Char('?') => p.overlay = Overlay::Help,
            KeyCode::Char('q') => p.overlay = Overlay::Quit,
            _ => {}
        },
    }
}

fn apply_action(game: &mut Game, kind: &str, ident: &str, cursor: Option<&str>) {
    let msg = match kind {
        "pull" => game.pull_net(ident),
        "skiff" => {
            let mut parts = ident.splitn(2, '|');
            let sid = parts.next().unwrap_or("");
            let job = parts.next().unwrap_or("idle");
            let site = if job == "pick" { cursor } else { None };
            game.assign_skiff(sid, job, site)
        }
        "crew" => {
            let mut parts = ident.splitn(2, '|');
            let cid = parts.next().unwrap_or("");
            let where_ = parts.next().unwrap_or("camp");
            game.assign_crew(cid, where_)
        }
        "hire" if ident != "none" => game.hire(ident),
        "hire" => String::new(),
        "buy" => game.buy(ident),
        "upgrade" => game.upgrade(ident),
        "mesh" => {
            let mut parts = ident.splitn(2, '|');
            let nid = parts.next().unwrap_or("");
            let mesh = parts.next().unwrap_or("mixed");
            game.set_mesh(nid, mesh)
        }
        _ => String::new(),
    };
    if !msg.is_empty() {
        game.note(msg, "action");
    }
}

fn draw(f: &mut Frame, app: &App) {
    let area = f.area();
    f.render_widget(
        Block::default().style(Style::default().bg(BG).fg(FLAVOR)),
        area,
    );
    match &app.screen {
        Screen::Title => draw_title(f, area),
        Screen::NewSeason(ns) => draw_new_season(f, area, ns),
        Screen::Play(p) => draw_play(f, area, p),
    }
}

fn draw_title(f: &mut Frame, area: Rect) {
    let card = centered(area, 78, 14);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COPPER))
        .style(Style::default().bg(CARD).fg(FLAVOR));
    let inner = block.inner(card);
    f.render_widget(block, card);
    let lines = vec![
        Line::from(Span::styled(
            "S A L M O N   K I N G",
            Style::default().fg(TITLE).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Kodiak Island · S04K set gillnet · one summer",
            Style::default().fg(MUTED),
        )),
        Line::from(""),
        Line::from("Two nets, a couple of skiffs, a cook who doesn't quit,"),
        Line::from("and an emergency order on the VHF. That's a season."),
        Line::from("Central Section or inner Alitak. Nowhere else."),
        Line::from(""),
        Line::from(Span::styled(
            "  n  New season     q  Quit",
            Style::default().fg(TITLE),
        )),
    ];
    f.render_widget(Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center), inner);
}

fn draw_new_season(f: &mut Frame, area: Rect, ns: &NewSeason) {
    let card = centered(area, 80, 20);
    let block = Block::default()
        .title(" NEW SEASON — pick a camp, a year, a seed ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COPPER))
        .style(Style::default().bg(CARD).fg(FLAVOR));
    let inner = block.inner(card);
    f.render_widget(block, card);

    let camps = [
        "Larsen Bay / Uyak — village, cannery ghost, outer Uyak",
        "Uganik outer / NE Arm — Shelikof setnet country",
        "Olga Bay — Alitak pulse water, tender-dependent",
        "Port Bailey / Dry Spruce — Kupreanof williwaws, nearer town",
    ];
    let mut lines = vec![
        Line::from("Odd years: pink flood. Even years: thin pinks."),
        Line::from("Known years (2023–2025) use that year's Kodiak prelim $/lb table."),
        Line::from(""),
        Line::from(Span::styled(
            "Camp  (↑↓)  Tab year/seed  Enter start  Esc back",
            Style::default().fg(MUTED),
        )),
    ];
    for (i, label) in camps.iter().enumerate() {
        let mark = if i == ns.camp_i { "●" } else { " " };
        let style = if i == ns.camp_i && matches!(ns.field, Field::Camp) {
            Style::default().fg(TITLE).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        lines.push(Line::from(Span::styled(format!(" {mark} {label}"), style)));
    }
    let y_mark = if matches!(ns.field, Field::Year) { ">" } else { " " };
    let s_mark = if matches!(ns.field, Field::Seed) { ">" } else { " " };
    lines.push(Line::from(""));
    lines.push(Line::from(format!("{y_mark} Year  [{}]", ns.year)));
    lines.push(Line::from(format!("{s_mark} Seed  [{}]", ns.seed)));
    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_play(f: &mut Frame, area: Rect, p: &Play) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(12),
            Constraint::Length(8),
            Constraint::Length(1),
        ])
        .split(area);

    let spd = if p.paused {
        "PAUSE".to_string()
    } else {
        format!("{}x", p.speed)
    };
    let tag = if p.game.any_open() { "OPEN" } else { "DARK" };
    let tag_style = if p.game.any_open() {
        Style::default().fg(OPEN).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(DARK).add_modifier(Modifier::BOLD)
    };
    let hud = Line::from(vec![
        Span::styled(
            format!(
                " SALMON KING   {}   {}   {}   ",
                p.game.camp.name,
                p.game.day.fmt_short(),
                p.game.tide.as_str()
            ),
            Style::default().fg(TITLE).add_modifier(Modifier::BOLD).bg(HUD_BG),
        ),
        Span::styled(format!("{tag}   {spd}   S04K "), tag_style.bg(HUD_BG)),
    ]);
    f.render_widget(Paragraph::new(hud).style(Style::default().bg(HUD_BG)), chunks[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(chunks[1]);

    let sites = p.game.playable_sites();
    let cursor = sites
        .get(p.cursor_i % sites.len().max(1))
        .map(|s| s.id);
    let map_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MAP_BORDER))
        .title(" map ");
    let map_inner = map_block.inner(body[0]);
    f.render_widget(map_block, body[0]);
    f.render_widget(
        Paragraph::new(render_map_panel(&p.game, cursor)).style(Style::default().fg(FLAVOR)),
        map_inner,
    );

    let side = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(18),
            Constraint::Percentage(24),
            Constraint::Percentage(24),
        ])
        .split(body[1]);

    panel(f, side[0], " crew ", &render_crew(&p.game));
    panel(f, side[1], " tender ", &render_tender(&p.game));
    panel(f, side[2], " camp ", &render_camp(&p.game));
    panel(f, side[3], " clock ", &render_clock(&p.game));

    let log_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(90, 61, 42)))
        .title(" event log ");
    let log_inner = log_block.inner(chunks[2]);
    f.render_widget(log_block, chunks[2]);
    f.render_widget(
        Paragraph::new(render_log(&p.game, 6)).style(Style::default().fg(FLAVOR)),
        log_inner,
    );

    f.render_widget(
        Paragraph::new(
            " space pause  1/4/x speed  ←→ site  enter set  p pull  s skiff  c crew  h hire  b buy  u upg  t tender  ? help  q quit",
        )
        .style(Style::default().fg(MUTED).bg(HUD_BG)),
        chunks[3],
    );

    match &p.overlay {
        Overlay::None => {}
        Overlay::Help => draw_help(f, area),
        Overlay::Quit => draw_quit(f, area),
        Overlay::Recap => {
            let text = if p.recap_text.is_empty() {
                p.game_recap_text()
            } else {
                format!("{}\n\nenter / q  title", p.recap_text)
            };
            draw_modal(f, area, " season recap ", &text);
        }
        Overlay::Picker {
            title,
            items,
            selected,
            ..
        } => draw_picker(f, area, title, items, *selected),
    }
}

impl Play {
    fn game_recap_text(&self) -> String {
        // Recap needs &mut for the postseason settle RNG. Overlay is shown after end;
        // we format a snapshot without mutating by cloning the needed fields.
        // The official recap() is called once when we open — here we rebuild text from state.
        // Use a note: we store text when overlay opens instead. For simplicity, format books now
        // without the random settle (shown as 0) if we can't mut-borrow. Better: compute without rng.
        format!(
            "SEASON {}  {}\nEnd: {}\nKings {:.0} lb   Reds {:.0} lb   Pinks {:.0} lb\nChums {:.0} lb   Silvers {:.0} lb\nGross ${:.0}   Expenses ${:.0}   Net ${:.0}\nCash ${:.0}   Tickets {}\n\nenter / q  title",
            self.game.year,
            self.game.camp.name,
            if self.game.end == GameEnd::None { "season" } else { self.game.end.as_str() },
            self.game.landed.king,
            self.game.landed.red,
            self.game.landed.pink,
            self.game.landed.chum,
            self.game.landed.silver,
            self.game.ledger.gross,
            self.game.ledger.expenses(),
            self.game.ledger.net(),
            self.game.ledger.cash,
            self.game.ledger.tickets,
        )
    }
}

fn panel(f: &mut Frame, area: Rect, title: &str, body: &str) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(61, 74, 82)))
        .title(title);
    let inner = block.inner(area);
    f.render_widget(block, area);
    f.render_widget(Paragraph::new(body.to_string()).style(Style::default().fg(FLAVOR)), inner);
}

fn draw_help(f: &mut Frame, area: Rect) {
    let text = "\
KEYS\n\
  space pause/run    1 4 x speeds (1x / 4x / 16x)\n\
  ← →  site cursor   enter  set a net on the highlighted site\n\
  p    pull nets      s     skiff job     c  crew assign\n\
  h    hire           b     buy from tender   u  upgrade camp\n\
  j    joint venture (2nd S04K, 3rd net)   t  send a skiff to the tender\n\
  m    mesh knob      ? help   q quit\n\n\
A TIDE is one tick (flood or ebb; two a day). Nets only fish during an EO opener,\n\
and only if the permit holder is on the site. Pull on a closer. Pick 2+ times a day\n\
or the lions and the sun do it for you. Tender buys fish and sells food/fuel/ice.\n\
Transients (ω) chase Stellers and seals off a bay — they do not pick your salmon.\n\
Residents (r) make fish dive; lions stay. Mixed blessing, written on the map.\n\n\
Legal water: Central Section (west-side Shelikof) or Alitak inner bays.\n\
Karluk, Ayakulik, Afognak, Eastside = seiners. You can see them. You don't set there.\n\
esc / ?";
    draw_modal(f, area, " help ", text);
}

fn draw_quit(f: &mut Frame, area: Rect) {
    draw_modal(
        f,
        area,
        " quit ",
        "Pull the season? Unsaved — this is a local camp.\n\n  y / enter  quit     n / esc  keep fishing",
    );
}

fn draw_picker(f: &mut Frame, area: Rect, title: &str, items: &[ActionItem], selected: usize) {
    let mut body = String::new();
    for (i, item) in items.iter().enumerate() {
        let mark = if i == selected { "●" } else { " " };
        body.push_str(&format!(" {mark} {}\n", item.label));
    }
    body.push_str("\n enter select   esc cancel");
    draw_modal(f, area, title, &body);
}

fn draw_modal(f: &mut Frame, area: Rect, title: &str, body: &str) {
    let lines = body.lines().count() as u16 + 2;
    let width = body
        .lines()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(40)
        .clamp(40, 86) as u16
        + 4;
    let card = centered(area, width.min(area.width.saturating_sub(2)), lines.min(area.height.saturating_sub(2)));
    f.render_widget(Clear, card);
    let block = Block::default()
        .title(format!(" {title} "))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COPPER))
        .style(Style::default().bg(CARD).fg(FLAVOR));
    let inner = block.inner(card);
    f.render_widget(block, card);
    f.render_widget(Paragraph::new(body.to_string()), inner);
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}
