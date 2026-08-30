from __future__ import annotations

from pathlib import Path

from textual.app import App, ComposeResult
from textual.containers import Horizontal, Vertical
from textual.screen import Screen
from textual.widgets import Static

from salmon_king.sim.engine import Game, new_game
from salmon_king.sim.models import GameEnd
from salmon_king.tui.screens import (
    ActionScreen,
    HelpScreen,
    NewSeasonScreen,
    QuitScreen,
    RecapScreen,
    TitleScreen,
    buy_items,
    crew_items,
    hire_items,
    mesh_items,
    pull_items,
    skiff_items,
    upgrade_items,
)
from salmon_king.tui.widgets import (
    render_camp,
    render_clock,
    render_crew,
    render_log,
    render_map_panel,
    render_tender,
)

CSS = Path(__file__).with_name("theme.tcss")

SPEEDS = {0: None, 1: 1.6, 4: 0.4, 16: 0.12}


class MainScreen(Screen):
    BINDINGS = [
        ("space", "pause", "Pause"),
        ("1", "spd1", "1x"),
        ("4", "spd4", "4x"),
        ("x", "spd16", "16x"),
        ("left", "prev_site", "Site-"),
        ("right", "next_site", "Site+"),
        ("enter", "deploy", "Set net"),
        ("p", "pull", "Pull"),
        ("s", "skiff", "Skiff"),
        ("c", "crew", "Crew"),
        ("h", "hire", "Hire"),
        ("b", "buy", "Buy"),
        ("u", "upgrade", "Upgrade"),
        ("j", "joint", "Joint venture"),
        ("t", "tender_run", "Tender"),
        ("m", "mesh", "Mesh"),
        ("question_mark", "help", "Help"),
        ("q", "quit", "Quit"),
    ]

    def __init__(self, game: Game) -> None:
        super().__init__()
        self.game = game
        self.speed = 1
        self.paused = False
        self.cursor_i = 0
        self._timer = None

    def compose(self) -> ComposeResult:
        yield Static(id="hud")
        with Horizontal(id="body"):
            with Vertical(id="map-pane"):
                yield Static(id="map-body")
            with Vertical(id="side"):
                yield Static(id="crew-pane", classes="panel")
                yield Static(id="tender-pane", classes="panel")
                yield Static(id="camp-pane", classes="panel")
                yield Static(id="clock-pane", classes="panel")
        yield Static(id="log")
        yield Static(
            "space pause  1/4/x speed  ←→ site  enter set  p pull  s skiff  "
            "c crew  h hire  b buy  u upg  t tender  ? help  q quit",
            id="keys",
        )

    def on_mount(self) -> None:
        self.refresh_all()
        self._arm()

    def _arm(self) -> None:
        if self._timer:
            self._timer.stop()
            self._timer = None
        interval = SPEEDS[self.speed]
        if interval and not self.paused:
            self._timer = self.set_interval(interval, self._tick)

    def _tick(self) -> None:
        if self.paused or self.game.end is not GameEnd.NONE:
            return
        self.game.step()
        self.refresh_all()
        if self.game.end is not GameEnd.NONE:
            if self._timer:
                self._timer.stop()
            self.app.push_screen(RecapScreen(self.game.recap()))

    def sites(self):
        return list(self.game.playable_sites())

    def cursor_site(self):
        sites = self.sites()
        if not sites:
            return None
        return sites[self.cursor_i % len(sites)].id

    def refresh_all(self) -> None:
        g = self.game
        spd = "PAUSE" if self.paused else f"{self.speed}x"
        open_any = any(g.site_is_open(s.id)[0] for s in g.playable_sites())
        tag = "OPEN" if open_any else "DARK"
        self.query_one("#hud", Static).update(
            f" SALMON KING   {g.camp.name}   {g.day:%d %b %Y}   {g.tide.value}   "
            f"{tag}   {spd}   seed-run  S04K"
        )
        self.query_one("#map-body", Static).update(render_map_panel(g, self.cursor_site()))
        self.query_one("#crew-pane", Static).update(render_crew(g))
        self.query_one("#tender-pane", Static).update(render_tender(g))
        self.query_one("#camp-pane", Static).update(render_camp(g))
        self.query_one("#clock-pane", Static).update(render_clock(g))
        self.query_one("#log", Static).update(render_log(g))

    def action_pause(self) -> None:
        self.paused = not self.paused
        if not self.paused and self.speed == 0:
            self.speed = 1
        self._arm()
        self.refresh_all()

    def action_spd1(self) -> None:
        self.speed, self.paused = 1, False
        self._arm()
        self.refresh_all()

    def action_spd4(self) -> None:
        self.speed, self.paused = 4, False
        self._arm()
        self.refresh_all()

    def action_spd16(self) -> None:
        self.speed, self.paused = 16, False
        self._arm()
        self.refresh_all()

    def action_next_site(self) -> None:
        self.cursor_i = (self.cursor_i + 1) % max(1, len(self.sites()))
        self.refresh_all()

    def action_prev_site(self) -> None:
        self.cursor_i = (self.cursor_i - 1) % max(1, len(self.sites()))
        self.refresh_all()

    def action_deploy(self) -> None:
        site = self.cursor_site()
        if not site:
            return
        net = next((n for n in self.game.nets if not n.in_water), None)
        if not net:
            # move first net
            net = self.game.nets[0]
            self.game.pull_net(net.id)
        msg = self.game.deploy_net(net.id, site)
        self.game.note(msg, "gear")
        self.refresh_all()

    def action_pull(self) -> None:
        items = pull_items(self.game)
        self.app.push_screen(ActionScreen("Pull which net?", items, "pull"))

    def action_skiff(self) -> None:
        self.app.push_screen(ActionScreen("Skiff job", skiff_items(self.game), "skiff"))

    def action_crew(self) -> None:
        self.app.push_screen(ActionScreen("Assign crew", crew_items(self.game), "crew"))

    def action_hire(self) -> None:
        self.app.push_screen(ActionScreen("Hire (tender)", hire_items(self.game), "hire"))

    def action_buy(self) -> None:
        self.app.push_screen(ActionScreen("Tender store", buy_items(), "buy"))

    def action_upgrade(self) -> None:
        self.app.push_screen(ActionScreen("Freight / upgrade", upgrade_items(), "upgrade"))

    def action_mesh(self) -> None:
        self.app.push_screen(ActionScreen("Mesh (game selectivity)", mesh_items(self.game), "mesh"))

    def action_joint(self) -> None:
        self.game.note(self.game.form_joint_venture(), "gear")
        self.refresh_all()

    def action_tender_run(self) -> None:
        fat = max(self.game.skiffs, key=lambda s: s.cargo.total())
        self.game.assign_skiff(fat.id, "tender")
        self.refresh_all()

    def action_help(self) -> None:
        self.app.push_screen(HelpScreen())

    def action_quit(self) -> None:
        self.app.push_screen(QuitScreen())

    def apply_action(self, kind: str, ident: str) -> None:
        g = self.game
        msg = ""
        if kind == "pull":
            msg = g.pull_net(ident)
        elif kind == "skiff":
            sid, job = ident.split("|", 1)
            site = self.cursor_site() if job == "pick" else None
            msg = g.assign_skiff(sid, job, site)
        elif kind == "crew":
            cid, where = ident.split("|", 1)
            msg = g.assign_crew(cid, where)
        elif kind == "hire":
            if ident != "none":
                msg = g.hire(ident)
        elif kind == "buy":
            msg = g.buy(ident)
        elif kind == "upgrade":
            msg = g.upgrade(ident)
        elif kind == "mesh":
            nid, mesh = ident.split("|", 1)
            msg = g.set_mesh(nid, mesh)
        if msg:
            g.note(msg, "action")
        self.refresh_all()


class SalmonKingApp(App):
    CSS_PATH = CSS
    TITLE = "Salmon King"

    def on_mount(self) -> None:
        self.push_screen(TitleScreen())

    def start_season(self, camp_id: str, year: int, seed: int) -> None:
        game = new_game(seed=seed, camp_id=camp_id, year=year)
        # pop new-season + stay on title then push main? pop new-season first
        self.pop_screen()
        self.push_screen(MainScreen(game))

    def apply_action(self, kind: str, ident: str) -> None:
        scr = self.screen
        if isinstance(scr, MainScreen):
            scr.apply_action(kind, ident)


def run_app() -> None:
    SalmonKingApp().run()
