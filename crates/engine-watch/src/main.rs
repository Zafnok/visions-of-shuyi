//! `cargo run -p trpg-engine-watch -- [--json] [--trigger <dispatch|schedule|manual>]
//! [--dispatch-seen] [--dispatched <sha>] [--missed-dispatch]`: the engine
//! watcher's report (ADR-0096). Markdown by default; `--json` for the
//! workflow (`.github/workflows/engine-watch.yml`), which passes the run's
//! trigger and what the last issue's marker said, for the new marker.
//!
//! A CLI tool prints to stdout by design, so `print_stdout` is allowed here.
#![allow(clippy::print_stdout)]

use std::path::Path;
use std::process::ExitCode;

use trpg_engine_watch::{Run, fetch, json, markdown, render};

const USAGE: &str = "usage: trpg-engine-watch [--json] [--trigger <dispatch|schedule|manual>] \
[--dispatch-seen] [--dispatched <sha>] [--missed-dispatch]";

/// The options, or `None` for a bad command line.
fn parse(mut args: impl Iterator<Item = String>) -> Option<(bool, Run)> {
    let mut as_json = false;
    let mut run = Run {
        trigger: "manual".to_owned(),
        ..Run::default()
    };
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => as_json = true,
            "--dispatch-seen" => run.dispatch_seen = true,
            "--missed-dispatch" => run.missed_dispatch = true,
            "--dispatched" => run.dispatched = args.next()?,
            "--trigger" => {
                run.trigger = args
                    .next()
                    .filter(|t| ["dispatch", "schedule", "manual"].contains(&t.as_str()))?;
            }
            _ => return None,
        }
    }
    Some((as_json, run))
}

fn main() -> ExitCode {
    let Some((as_json, run)) = parse(std::env::args().skip(1)) else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let watch = match fetch::watch(&root, fetch::now()) {
        Ok(watch) => watch,
        Err(e) => {
            eprintln!("engine-watch: {e}");
            return ExitCode::FAILURE;
        }
    };
    let rendered = render(&watch.report, &watch.ledger, &watch.open_ids, &run);
    if as_json {
        println!("{}", json(&watch.report, rendered.as_ref()));
    } else {
        print!("{}", markdown(&watch.report, rendered.as_ref()));
    }
    ExitCode::SUCCESS
}
