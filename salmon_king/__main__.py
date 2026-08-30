from __future__ import annotations

import argparse
import sys


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="salmon-king",
        description="Salmon King — run a Kodiak set-net camp for one summer.",
    )
    parser.add_argument(
        "--headless",
        action="store_true",
        help="Run the sim with no TUI (for tests and season checks).",
    )
    parser.add_argument("--seed", type=int, default=2024)
    parser.add_argument(
        "--camp",
        choices=("uganik", "larsen", "olga", "bailey"),
        default="uganik",
    )
    parser.add_argument("--year", type=int, default=2024)
    parser.add_argument("--ticks", type=int, default=0, help="Headless: number of tides to run (0 = full season).")
    parser.add_argument("--quiet", action="store_true")
    args = parser.parse_args(argv)

    if args.headless:
        from salmon_king.sim.engine import new_game, run_headless

        game = new_game(seed=args.seed, camp_id=args.camp, year=args.year)
        recap = run_headless(game, ticks=args.ticks or None, quiet=args.quiet)
        print(recap.as_text())
        return 0 if recap.survived else 2

    from salmon_king.tui.app import run_app

    run_app()
    return 0


if __name__ == "__main__":
    sys.exit(main())
