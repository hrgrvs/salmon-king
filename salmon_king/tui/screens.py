from __future__ import annotations

from textual.app import ComposeResult
from textual.containers import Horizontal, Vertical
from textual.screen import ModalScreen, Screen
from textual.widgets import Button, Input, Label, ListItem, ListView, Static

from salmon_king.data.crew_pool import CANDIDATES
from salmon_king.sim.engine import Game, Recap


class TitleScreen(Screen):
    BINDINGS = [
        ("n", "new", "New season"),
        ("q", "quit", "Quit"),
    ]

    def compose(self) -> ComposeResult:
        with Vertical(id="title-wrap"):
            with Vertical(id="title-card"):
                yield Static("S A L M O N   K I N G", classes="title")
                yield Static("Kodiak Island · S04K set gillnet · one summer", classes="sub")
                yield Static(
                    "Two nets, a couple of skiffs, a cook who doesn't quit,\n"
                    "and an emergency order on the VHF. That's a season.\n"
                    "Central Section or inner Alitak. Nowhere else.",
                    classes="flavor",
                )
                yield Horizontal(
                    Button("New season", id="new", variant="success"),
                    Button("Quit", id="quit"),
                )

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "new":
            self.action_new()
        else:
            self.app.exit()

    def action_new(self) -> None:
        self.app.push_screen(NewSeasonScreen())

    def action_quit(self) -> None:
        self.app.exit()


class NewSeasonScreen(ModalScreen):
    def compose(self) -> ComposeResult:
        with Vertical(id="modal-body"):
            yield Static("NEW SEASON — pick a camp, a year, a seed", classes="title")
            yield Static(
                "Odd years: pink flood (2023 harvest 24.7M, 2025 34.6M).\n"
                "Even years: thin pinks (2024 7.3M). Prices follow that year's Kodiak prelim table."
            )
            yield Label("Camp")
            yield ListView(
                ListItem(Label("Larsen Bay / Uyak — village, cannery ghost, outer Uyak"), id="c-larsen"),
                ListItem(Label("Uganik outer / NE Arm — Shelikof setnet country"), id="c-uganik"),
                ListItem(Label("Olga Bay — Alitak pulse water, tender-dependent"), id="c-olga"),
                ListItem(Label("Port Bailey / Dry Spruce — Kupreanof williwaws, nearer town"), id="c-bailey"),
                id="camp-list",
            )
            yield Label("Year (even/odd sets the pink line; known years use real prelim $/lb)")
            yield Input(value="2025", id="year")
            yield Label("Seed")
            yield Input(value="1701", id="seed")
            yield Horizontal(
                Button("Start", id="start", variant="success"),
                Button("Back", id="back"),
            )

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "back":
            self.app.pop_screen()
            return
        camp_list = self.query_one("#camp-list", ListView)
        idx = camp_list.index if camp_list.index is not None else 0
        camp_id = ("larsen", "uganik", "olga", "bailey")[int(idx)]
        try:
            year = int(self.query_one("#year", Input).value)
            seed = int(self.query_one("#seed", Input).value)
        except ValueError:
            year, seed = 2025, 1701
        self.app.start_season(camp_id, year, seed)


class HelpScreen(ModalScreen):
    BINDINGS = [("escape", "close", "Close"), ("?", "close", "Close")]

    def compose(self) -> ComposeResult:
        yield Static(
            "KEYS\n"
            "  space pause/run    1 4 x speeds (1x / 4x / 16x)\n"
            "  ← →  site cursor   enter  set a net on the highlighted site\n"
            "  p    pull nets      s     skiff job     c  crew assign\n"
            "  h    hire           b     buy from tender   u  upgrade camp\n"
            "  j    joint venture (2nd S04K, 3rd net)   t  send a skiff to the tender\n"
            "  m    mesh knob      ? help   q quit\n\n"
            "A TIDE is one tick (flood or ebb; two a day). Nets only fish during an EO opener,\n"
            "and only if the permit holder is on the site. Pull on a closer. Pick 2+ times a day\n"
            "or the lions and the sun do it for you. Tender buys fish and sells food/fuel/ice.\n"
            "Transients (ω) chase Stellers and seals off a bay — they do not pick your salmon.\n"
            "Residents (r) make fish dive; lions stay. Mixed blessing, written on the map.\n\n"
            "Legal water: Central Section (west-side Shelikof) or Alitak inner bays.\n"
            "Karluk, Ayakulik, Afognak, Eastside = seiners. You can see them. You don't set there.\n"
            "esc / ?",
            id="help-body",
        )

    def action_close(self) -> None:
        self.app.pop_screen()


class QuitScreen(ModalScreen):
    def compose(self) -> ComposeResult:
        with Vertical(id="modal-body"):
            yield Static("Pull the season? Unsaved — this is a local camp.")
            yield Horizontal(
                Button("Keep fishing", id="no"),
                Button("Quit", id="yes", variant="error"),
            )

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "yes":
            self.app.exit()
        else:
            self.app.pop_screen()


class RecapScreen(ModalScreen):
    def __init__(self, recap: Recap) -> None:
        super().__init__()
        self.recap = recap

    def compose(self) -> ComposeResult:
        yield Static(self.recap.as_text() + "\n\nenter / q  title", id="recap-body")

    def on_key(self, event) -> None:
        if event.key in {"enter", "q", "escape"}:
            self.app.pop_screen()
            self.app.pop_screen()


class ActionScreen(ModalScreen):
    """Generic picker: title, items (id, label), callback name on app."""

    def __init__(self, title: str, items: list[tuple[str, str]], kind: str) -> None:
        super().__init__()
        self._title = title
        self.items = items
        self.kind = kind

    def compose(self) -> ComposeResult:
        with Vertical(id="modal-body"):
            yield Static(self._title)
            yield ListView(*[ListItem(Label(lab), id=f"i-{i}") for i, (_, lab) in enumerate(self.items)], id="alist")
            yield Button("Cancel", id="cancel")

    def on_button_pressed(self, event: Button.Pressed) -> None:
        self.app.pop_screen()

    def on_list_view_selected(self, event: ListView.Selected) -> None:
        idx = event.list_view.index or 0
        ident = self.items[int(idx)][0]
        self.app.pop_screen()
        self.app.apply_action(self.kind, ident)


def skiff_items(game: Game) -> list[tuple[str, str]]:
    out = []
    for s in game.skiffs:
        out.append((f"{s.id}|pick", f"{s.name}: pick nets"))
        out.append((f"{s.id}|tender", f"{s.name}: run tender"))
        out.append((f"{s.id}|idle", f"{s.name}: idle / beach"))
        out.append((f"{s.id}|repair", f"{s.name}: repair"))
        out.append((f"{s.id}|town", f"{s.name}: town run (permit leaves → nets dark)"))
    return out


def crew_items(game: Game) -> list[tuple[str, str]]:
    out = []
    for c in game.crew:
        if c.status.value == "quit":
            continue
        for s in game.skiffs:
            out.append((f"{c.id}|{s.id}", f"{c.name} → {s.name}"))
        out.append((f"{c.id}|camp", f"{c.name} → bunkhouse"))
        if c.role == "cook" or c.id.endswith("cook"):
            out.append((f"{c.id}|cook", f"{c.name} → cookshack"))
    return out


def hire_items(game: Game) -> list[tuple[str, str]]:
    out = []
    for cid in game.hire_pool:
        cand = next(c for c in CANDIDATES if c.id == cid)
        out.append((cid, f"{cand.name}  {cand.role}  {cand.trait}  share {cand.wage_share:.0%}" if cand.wage_share else f"{cand.name}  cook  ${cand.daily_wage}/d"))
    return out or [("none", "Nobody on the beach.")]


def buy_items() -> list[tuple[str, str]]:
    return [
        ("food", "Food — 18 person-days  $160"),
        ("fuel", "Fuel — 40 gal  $170"),
        ("ice", "Slush ice  $55"),
        ("twine", "Needles / twine / corks  $90"),
        ("prop", "Spare prop  $220"),
    ]


def upgrade_items() -> list[tuple[str, str]]:
    return [
        ("cookshack", "Cookshack — feeds better  $1800"),
        ("bunkhouse", "Bunkhouse — rest  $1600"),
        ("loft", "Net loft — faster mend  $1400"),
        ("stall", "Skiff stall  $2200"),
    ]


def mesh_items(game: Game) -> list[tuple[str, str]]:
    out = []
    for n in game.nets:
        for m in ("pink", "mixed", "red"):
            out.append((f"{n.id}|{m}", f"{n.id} hang {m} (game knob, not a legal mesh spec)"))
    return out


def pull_items(game: Game) -> list[tuple[str, str]]:
    return [(n.id, f"Pull {n.id}  ({n.site_id or 'beach'})") for n in game.nets]
