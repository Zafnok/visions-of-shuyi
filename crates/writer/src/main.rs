//! `story-check [folder or file]`: the checks a dialogue script goes
//! through, for a writer with the mirror's files and no source code
//! (ticket 0152). Run in the mirror's folder: it reads `dialogue/`.
//!
//! A CLI tool prints to stdout by design, so `print_stdout` is allowed here.
#![allow(clippy::print_stdout)]

use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (code, out, err) = trpg_writer::story_check(&args, Path::new(trpg_writer::DIALOGUE));
    print!("{out}");
    eprint!("{err}");
    ExitCode::from(code)
}
