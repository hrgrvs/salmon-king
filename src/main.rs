use salmon_king::data::camps::camp;
use salmon_king::sim::engine::{new_game, run_headless};

mod tui;

struct Args {
    headless: bool,
    seed: u64,
    camp: String,
    year: i32,
    ticks: i32,
    quiet: bool,
}

fn print_help() {
    println!(
        "salmon-king — run a Kodiak set-net camp for one summer.\n\n\
         USAGE:\n\
         \x20   salmon-king [OPTIONS]\n\n\
         OPTIONS:\n\
         \x20   --headless          Run the sim with no TUI\n\
         \x20   --camp <CAMP>       uganik | larsen | olga | bailey  (default: uganik)\n\
         \x20   --year <YEAR>       Calendar year; even/odd sets the pink line  (default: 2024)\n\
         \x20   --seed <SEED>       Deterministic seed  (default: 2024)\n\
         \x20   --ticks <N>         Headless tides (0 = full season)\n\
         \x20   --quiet             Headless: no recap print\n\
         \x20   -h, --help          Print help\n\
         \x20   -V, --version       Print version"
    );
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        headless: false,
        seed: 2024,
        camp: "uganik".into(),
        year: 2024,
        ticks: 0,
        quiet: false,
    };
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < raw.len() {
        match raw[i].as_str() {
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("salmon-king {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "--headless" => args.headless = true,
            "--quiet" => args.quiet = true,
            "--camp" => {
                i += 1;
                args.camp = raw.get(i).cloned().ok_or("--camp needs a value")?;
            }
            "--year" => {
                i += 1;
                args.year = raw
                    .get(i)
                    .ok_or("--year needs a value")?
                    .parse()
                    .map_err(|_| "year must be an integer")?;
            }
            "--seed" => {
                i += 1;
                args.seed = raw
                    .get(i)
                    .ok_or("--seed needs a value")?
                    .parse()
                    .map_err(|_| "seed must be an integer")?;
            }
            "--ticks" => {
                i += 1;
                args.ticks = raw
                    .get(i)
                    .ok_or("--ticks needs a value")?
                    .parse()
                    .map_err(|_| "ticks must be an integer")?;
            }
            other => return Err(format!("unknown argument: {other}")),
        }
        i += 1;
    }
    Ok(args)
}

fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            eprintln!("try: salmon-king --help");
            std::process::exit(2);
        }
    };

    if args.headless {
        if camp(&args.camp).is_none() {
            eprintln!(
                "Unknown camp '{}'. Choose uganik, larsen, olga, or bailey.",
                args.camp
            );
            std::process::exit(2);
        }
        match new_game(args.seed, &args.camp, args.year) {
            Ok(mut game) => {
                let ticks = if args.ticks > 0 { Some(args.ticks) } else { None };
                let recap = run_headless(&mut game, ticks);
                if !args.quiet {
                    println!("{}", recap.as_text());
                }
                std::process::exit(if recap.survived { 0 } else { 2 });
            }
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(2);
            }
        }
    }

    if let Err(e) = tui::run() {
        eprintln!("Need a real terminal (alternate screen + keyboard). {e}");
        std::process::exit(1);
    }
}
