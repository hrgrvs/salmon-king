//! Keyboard-first ratatui HUD. The sim is a client of this module, not the reverse.

mod art;
mod screens;
mod theme;
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
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::{Frame, Terminal};

use salmon_king::data::camps::CAMP_IDS;
use salmon_king::sim::engine::{new_game, Game};
use salmon_king::sim::models::GameEnd;

use art::{help_header, title_art_scaled, NEW_SEASON_HEAD};
use screens::{
    buy_items, crew_items, hire_items, mesh_items, pull_items, radio_items, skiff_items,
    upgrade_items, ActionItem,
};
use salmon_king::sim::hints::{dismiss_hint, set_hints};
use theme::{
    body, cork_bold, dark_tag, foam_bold, muted, open_tag, CARD, CORK, FOAM, HUD, NIGHT, WOOL,
};
use widgets::{
    draw_bay, draw_boats, draw_camp, draw_clock, draw_crew, draw_hint, draw_log, draw_map,
    draw_radio, draw_tender,
};

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

fn app_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|f| draw(f, app))?;

        let timeout = match &app.screen {
            Screen::Play(p)
                if matches!(p.overlay, Overlay::None)
                    && !p.paused
                    && p.game.end == GameEnd::None =>
            {
                let due = p.last_tick + interval(p.speed);
                due.saturating_duration_since(Instant::now())
                    .min(Duration::from_millis(50))
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
            if matches!(
                key.code,
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q')
            ) {
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
            KeyCode::Char('r') => {
                p.overlay = Overlay::Picker {
                    title: "VHF — who do you call?".into(),
                    kind: "radio".into(),
                    items: radio_items(&p.game),
                    selected: 0,
                };
            }
            KeyCode::Char('i') => {
                let on = !p.game.hints_on;
                set_hints(&mut p.game, on);
                p.game.note(
                    if on {
                        "Hints on. Cook will mutter when something needs doing."
                    } else {
                        "Hints off. You're on your own."
                    },
                    "camp",
                );
            }
            KeyCode::Esc => {
                if p.game.hint.is_some() {
                    dismiss_hint(&mut p.game);
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
        "radio" => game.radio_call(ident, cursor),
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
        Block::default().style(Style::default().bg(NIGHT).fg(FOAM)),
        area,
    );
    match &app.screen {
        Screen::Title => draw_title(f, area),
        Screen::NewSeason(ns) => draw_new_season(f, area, ns),
        Screen::Play(p) => draw_play(f, area, p),
    }
}

fn draw_title(f: &mut Frame, area: Rect) {
    let h = area.height.min(30).max(16);
    let w = area.width.min(78).max(60);
    let card = centered(area, w, h);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CORK))
        .style(Style::default().bg(CARD).fg(FOAM));
    let inner = block.inner(card);
    f.render_widget(block, card);

    let mut lines = title_art_scaled(inner.height);
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Two nets, a couple of skiffs, a cook who doesn't quit,",
        body(),
    )));
    lines.push(Line::from(Span::styled(
        "and an emergency order on the VHF. That's a season.",
        body(),
    )));
    lines.push(Line::from(Span::styled(
        "Central Section or inner Alitak. Nowhere else.",
        muted(),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  n  New season          q  Quit",
        foam_bold(),
    )));
    f.render_widget(
        Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center),
        inner,
    );
}

fn draw_new_season(f: &mut Frame, area: Rect, ns: &NewSeason) {
    let card = centered(area, 80, 24);
    let block = Block::default()
        .title(" NEW SEASON — pick a camp, a year, a seed ")
        .title_style(foam_bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CORK))
        .style(Style::default().bg(CARD).fg(FOAM));
    let inner = block.inner(card);
    f.render_widget(block, card);

    let camps = [
        "Larsen Bay / Uyak — village, cannery ghost, outer Uyak",
        "Uganik outer / NE Arm — Shelikof setnet country",
        "Olga Bay — Alitak pulse water, tender-dependent",
        "Port Bailey / Dry Spruce — Kupreanof williwaws, nearer town",
    ];
    let mut lines: Vec<Line> = NEW_SEASON_HEAD
        .iter()
        .map(|row| Line::from(Span::styled((*row).to_string(), muted())))
        .collect();
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Odd years: pink flood. Even years: thin pinks.",
        body(),
    )));
    lines.push(Line::from(Span::styled(
        "Known years (2023–2025) use that year's Kodiak prelim $/lb table.",
        muted(),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Camp  (↑↓)  Tab year/seed  Enter start  Esc back",
        muted(),
    )));
    for (i, label) in camps.iter().enumerate() {
        let mark = if i == ns.camp_i { "●" } else { " " };
        let style = if i == ns.camp_i && matches!(ns.field, Field::Camp) {
            foam_bold()
        } else {
            body()
        };
        lines.push(Line::from(Span::styled(format!(" {mark} {label}"), style)));
    }
    let y_mark = if matches!(ns.field, Field::Year) {
        ">"
    } else {
        " "
    };
    let s_mark = if matches!(ns.field, Field::Seed) {
        ">"
    } else {
        " "
    };
    lines.push(Line::from(""));
    lines.push(Line::from(format!("{y_mark} Year  [{}]", ns.year)));
    lines.push(Line::from(format!("{s_mark} Seed  [{}]", ns.seed)));
    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_play(f: &mut Frame, area: Rect, p: &Play) {
    // Full-width boats strip so both skiffs stay readable at 80x24.
    let boats_h = if area.height >= 32 { 9 } else { 6 };
    let mid_h = if area.height >= 36 {
        8
    } else if area.height >= 28 {
        7
    } else {
        5
    };
    let log_h = if area.height >= 34 {
        7
    } else if area.height >= 28 {
        5
    } else {
        3
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(8),
            Constraint::Length(boats_h),
            Constraint::Length(mid_h),
            Constraint::Length(log_h),
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
        open_tag()
    } else {
        dark_tag()
    };
    let hud = Line::from(vec![
        Span::styled(
            format!(
                " SALMON KING   {}   {}   {}   ",
                p.game.camp.name,
                p.game.day.fmt_short(),
                p.game.tide.as_str()
            ),
            foam_bold().bg(HUD),
        ),
        Span::styled(format!("{tag}   {spd}   S04K "), tag_style.bg(HUD)),
    ]);
    f.render_widget(
        Paragraph::new(hud).style(Style::default().bg(HUD)),
        chunks[0],
    );

    let sites = p.game.playable_sites();
    let cursor = sites.get(p.cursor_i % sites.len().max(1)).map(|s| s.id);

    if area.width >= 110 && chunks[1].height >= 8 {
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(38),
                Constraint::Percentage(28),
                Constraint::Percentage(34),
            ])
            .split(chunks[1]);
        draw_map(f, body[0], &p.game, cursor);
        draw_bay(f, body[1], &p.game);
        draw_crew(f, body[2], &p.game);
    } else {
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
            .split(chunks[1]);
        let map_col = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(7)])
            .split(body[0]);
        draw_map(f, map_col[0], &p.game, cursor);
        draw_bay(f, map_col[1], &p.game);
        draw_crew(f, body[1], &p.game);
    }

    draw_boats(f, chunks[2], &p.game);

    let mid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(36),
            Constraint::Percentage(32),
            Constraint::Percentage(32),
        ])
        .split(chunks[3]);
    draw_camp(f, mid[0], &p.game);
    draw_tender(f, mid[1], &p.game);
    draw_clock(f, mid[2], &p.game);

    let foot = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(44), Constraint::Percentage(56)])
        .split(chunks[4]);
    draw_radio(f, foot[0], &p.game);
    draw_log(f, foot[1], &p.game);

    f.render_widget(
        Paragraph::new(
            " space pause  1/4/x  ←→ site  enter set  p pull  r radio  s skiff  c crew  t tender  i hints  ? help  q quit",
        )
        .style(Style::default().fg(WOOL).bg(HUD)),
        chunks[5],
    );

    if p.game.hints_on {
        draw_hint(f, area, &p.game);
    }

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

fn draw_help(f: &mut Frame, area: Rect) {
    let mut lines = help_header();
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("KEYS", cork_bold())));
    for row in [
        "  space pause/run    1 4 x speeds (1x / 4x / 16x)",
        "  ← →  site cursor   enter  set a net on the highlighted site",
        "  p    pull nets      s     skiff job     c  crew assign     r  VHF",
        "  h    hire           b     buy from tender   u  upgrade camp",
        "  j    joint venture (2nd S04K, 3rd net)   t  send a skiff to the tender",
        "  m    mesh knob      i  hints on/off     esc dismiss aside",
        "  ? help   q quit",
    ] {
        lines.push(Line::from(Span::styled(row, body())));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "A TIDE is one tick (flood or ebb; two a day). Nets only fish during an EO opener,",
        muted(),
    )));
    lines.push(Line::from(Span::styled(
        "and only if the permit holder is on the site. Pull on a closer. Pick 2+ times a day",
        muted(),
    )));
    lines.push(Line::from(Span::styled(
        "or the lions and the sun do it for you. Tender buys fish and sells food/fuel/ice.",
        muted(),
    )));
    lines.push(Line::from(Span::styled(
        "VHF: r calls a skiff, a hand, the tender (live board + gossip), or 16 (ADF&G).",
        body(),
    )));
    lines.push(Line::from(Span::styled(
        "Tender rumors are chatter. Official openers/closers and the once-a-day report are gospel.",
        muted(),
    )));
    lines.push(Line::from(Span::styled(
        "Bay inset: camp, live boats, whales whenever present, seals only while you pick.",
        body(),
    )));
    lines.push(Line::from(Span::styled(
        "Hints default on (i toggles for this season). Asides, not a tutorial wall.",
        muted(),
    )));
    lines.push(Line::from(Span::styled(
        "Transients (ω) chase Stellers off a bay — they do not pick your salmon.",
        body(),
    )));
    lines.push(Line::from(Span::styled(
        "Residents (><>) make fish dive; lions stay. Mixed blessing, written on the map.",
        body(),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Legal water: Central Section (west-side Shelikof) or Alitak inner bays.",
        muted(),
    )));
    lines.push(Line::from(Span::styled(
        "Karluk, Ayakulik, Afognak, Eastside = seiners. You can see them. You don't set there.",
        muted(),
    )));
    lines.push(Line::from(Span::styled("esc / ?", foam_bold())));
    draw_modal_lines(f, area, " help ", lines);
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
    let lines: Vec<Line> = body.lines().map(|l| Line::from(l.to_string())).collect();
    draw_modal_lines(f, area, title, lines);
}

fn draw_modal_lines(f: &mut Frame, area: Rect, title: &str, lines: Vec<Line>) {
    let height = lines.len() as u16 + 2;
    let width = lines
        .iter()
        .map(|l| l.width())
        .max()
        .unwrap_or(40)
        .clamp(40, 86) as u16
        + 4;
    let card = centered(
        area,
        width.min(area.width.saturating_sub(2)),
        height.min(area.height.saturating_sub(2)),
    );
    f.render_widget(Clear, card);
    let block = Block::default()
        .title(format!(" {title} "))
        .title_style(foam_bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CORK))
        .style(Style::default().bg(CARD).fg(FOAM));
    let inner = block.inner(card);
    f.render_widget(block, card);
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

#[cfg(test)]
mod hud_tests {
    use super::*;
    use ratatui::backend::TestBackend;

    fn dump(backend: &TestBackend) -> String {
        let buf = backend.buffer();
        let mut out = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                out.push_str(buf[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    fn play_app() -> App {
        let game = new_game(1701, "uganik", 2025).expect("game");
        App {
            screen: Screen::Play(Play {
                game,
                speed: 1,
                paused: true,
                cursor_i: 0,
                overlay: Overlay::None,
                last_tick: Instant::now(),
                recap_text: String::new(),
            }),
            should_quit: false,
        }
    }

    #[test]
    fn title_keys_fit_twenty_four_rows() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = App {
            screen: Screen::Title,
            should_quit: false,
        };
        terminal.draw(|f| draw(f, &app)).unwrap();
        let text = dump(terminal.backend());
        assert!(text.contains("S A L M O N"), "{text}");
        assert!(text.to_ascii_lowercase().contains("new season"), "{text}");
    }

    #[test]
    fn title_is_illustrated() {
        let backend = TestBackend::new(80, 28);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = App {
            screen: Screen::Title,
            should_quit: false,
        };
        terminal.draw(|f| draw(f, &app)).unwrap();
        let text = dump(terminal.backend());
        assert!(text.contains("S A L M O N"), "{text}");
        assert!(
            text.contains("corkline") || text.contains("cabin") || text.contains(",-="),
            "{text}"
        );
        assert!(
            text.contains("n") && text.to_ascii_lowercase().contains("new season"),
            "{text}"
        );
        assert!(text.contains("Quit") || text.contains("q  Quit"), "{text}");
    }

    #[test]
    fn hud_answers_boats_and_crew_at_a_glance() {
        let backend = TestBackend::new(100, 36);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = play_app();
        terminal.draw(|f| draw(f, &app)).unwrap();
        let text = dump(terminal.backend());
        assert!(
            text.contains("boats") || text.contains("BOATS") || text.contains("PICKING SKIFF"),
            "{text}"
        );
        assert!(
            text.contains("HOLDING SKIFF") || text.contains("Holding"),
            "{text}"
        );
        assert!(text.contains("idle in the hole"), "{text}");
        assert!(text.contains("picking"), "{text}");
        assert!(text.contains("cooking"), "{text}");
        assert!(text.contains("crew"), "{text}");
        assert!(text.contains("Energy"), "{text}");
        assert!(text.contains("Hunger"), "{text}");
        assert!(text.contains("Morale"), "{text}");
        assert!(
            text.contains("VHF") || text.contains("CH 68") || text.contains("squelch"),
            "{text}"
        );
        assert!(
            text.contains("CAMP") || text.contains("bay") || text.contains("Uganik"),
            "{text}"
        );
    }

    #[test]
    fn crew_hud_stacks_stat_words_when_the_pane_is_narrow() {
        let game = new_game(1701, "uganik", 2025).expect("game");
        // Inner width ~28 after borders — too tight for Energy/Hunger/Morale on one line.
        let backend = TestBackend::new(32, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| draw_crew(f, f.area(), &game)).unwrap();
        let text = dump(terminal.backend());
        assert!(text.contains("Energy"), "{text}");
        assert!(text.contains("Hunger"), "{text}");
        assert!(text.contains("Morale"), "{text}");
    }

    #[test]
    fn help_and_toasts_use_the_same_look() {
        let backend = TestBackend::new(80, 28);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = play_app();
        if let Screen::Play(p) = &mut app.screen {
            p.overlay = Overlay::Help;
        }
        terminal.draw(|f| draw(f, &app)).unwrap();
        let text = dump(terminal.backend());
        assert!(text.contains("corkline") || text.contains(",-="), "{text}");
        assert!(text.contains("KEYS") || text.contains("space"), "{text}");

        if let Screen::Play(p) = &mut app.screen {
            p.overlay = Overlay::None;
            p.game.note("Williwaw. Katabatic, no warning. Skiffs stay on the running line if you like them.", "weather");
            p.game.weather.williwaw = true;
        }
        terminal.draw(|f| draw(f, &app)).unwrap();
        let text = dump(terminal.backend());
        assert!(
            text.contains("WILLIWAW") || text.contains("katabatic") || text.contains("Williwaw"),
            "{text}"
        );
    }

    #[test]
    fn hud_fits_eighty_by_twenty_four() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = play_app();
        terminal.draw(|f| draw(f, &app)).unwrap();
        let text = dump(terminal.backend());
        assert!(text.contains("idle in the hole"), "{text}");
        assert!(text.contains("picking"), "{text}");
        assert!(text.contains("cooking"), "{text}");
    }
}
