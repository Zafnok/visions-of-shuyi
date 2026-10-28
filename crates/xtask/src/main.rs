//! Repo tooling commands. See CLAUDE.md.
//!
//! The generic tools moved to Shuyi Engine's `shuyi-cli` (ticket 1117):
//! `cargo shuyi <command>`. Their old names here forward to it, with this
//! game's tools ([`game_tools::Trpg`]); xtask is also the game's tool
//! program `cargo shuyi` runs for `frame-png` and `lang-status`.
//!
//! A CLI tool prints to stdout by design, so `print_stdout` is allowed here.
#![allow(clippy::print_stdout)]

mod battler_import;
mod check_script;
mod clean_targets;
mod code_lines;
mod edge_marks;
mod effect_marks;
mod engine;
mod game_tools;
mod icon;
mod lines;
mod map_sprite_import;
mod playdate;
mod playdate_audio;
mod playdate_images;
mod playtest;
mod portrait_import;
mod sfx;
mod skill_pack;
mod test_auto;
mod test_card;
mod test_tileset;
mod test_units;
mod tickets;
mod tileset_import;
mod tileset_ron;
mod title_art;
mod voice_test;

use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::SystemTime;

const USAGE: &str = "usage: cargo xtask <command>\n\n\
available commands:\n  \
ticket-lint [--pr-branch <name>]   check tickets/{open,done} against tickets/README.md\n    \
[--check-engine]                 and look up each engine#NNNN in the engine's tickets\n  \
engine [--off]                     build the game on Shuyi Engine checked out in shuyi-engine/\n  \
engine-pin [<commit> | --check]    move the engine's pinned commit, or check it and its ledger (ADR-0063 §8)\n  \
engine-pin --report                the engine PRs past the pin, sorted, and whether a move is due (ADR-0096)\n  \
skill-pack [--check]               copy the pinned engine's skill pack into .claude/skills/, or check the copy (ADR-0097)\n  \
clean-merged-targets [--dry-run]   delete target/ in worktrees whose PR has merged\n  \
code-lines [--base <ref>]          production Rust lines added and removed per crate (ADR-0092)\n  \
sfx [--check]                      render our own sounds into assets/audio/sfx/\n  \
test-card                          write the sprite test image, assets-test/images/test_card.png\n  \
test-tileset                       write the sprite map skins' test tilesets, assets-test/tilesets/test*\n  \
effect-marks                       write the effect arrows, assets/images/effect_marks.png\n  \
edge-marks                         write the map's edge arrows, assets/images/edge_marks.png\n  \
title-art [--bought]               write the title logo and placeholder backdrops, assets/title/\n  \
map-sprite-import [--list]         copy the bought map sprites the game uses into assets-private/game/\n  \
battler-import [--list]            copy the bought combat pictures and backdrops into assets-private/game/\n  \
tileset-import [--list]            pack the bought terrain tiles and write the game's tileset there\n  \
playtest <battle-id> [options]     a bot plays a battle many times and reports (playtest --help)\n  \
portrait-import <busts> <id>       cut bought busts into portraits (portrait-import --help)\n  \
lines [scene]                      list every dialogue line with its line id (lines --help)\n  \
voice-test-clips                   make the tone clips that test voice playback (needs ffmpeg)\n  \
playdate [--release] [--bench] [--private-assets] [--device|--simulator|--both]\n                                     build the Playdate game into target/playdate/VisionsOfShuyi.pdx\n                                     (--bench: the build that times the renderer, its own .pdx;\n                                     --private-assets: with the bought art)
  playdate-images                    write the Playdate's one-bit pictures to look at, into
                                     target/playdate/images/ (and the bought ones' into assets-private/)

moved to Shuyi's tools, `cargo shuyi <command>` (these names still run them):
  check-keys, check-text, lang-status, font-atlas, frame-png, web, private-assets
  (`cargo shuyi` is `cargo xtask shuyi`: Shuyi's tools with this game's)";

fn main() -> ExitCode {
    ExitCode::from(dispatch(env::args().skip(1)))
}

/// The actual command dispatch, as a plain exit code (0 success) rather than
/// `ExitCode` so it's directly comparable in tests.
fn dispatch(mut args: impl Iterator<Item = String>) -> u8 {
    match args.next().as_deref() {
        Some(
            name @ ("check-keys" | "check-text" | "lang-status" | "font-atlas" | "frame-png"
            | "web" | "private-assets"),
        ) => shuyi(name, args),
        Some("shuyi") => shuyi_cli::main_with(args, Some(&game_tools::Trpg)),
        Some("ticket-lint") => ticket_lint(&args.collect::<Vec<_>>()),
        Some("engine") => engine(&args.collect::<Vec<_>>()),
        Some("engine-pin") => engine_pin(&args.collect::<Vec<_>>()),
        Some("skill-pack") => skill_pack(&args.collect::<Vec<_>>()),
        Some("clean-merged-targets") => clean_merged_targets(&args.collect::<Vec<_>>()),
        Some("code-lines") => code_lines(&args.collect::<Vec<_>>()),
        Some("icon") => icon(&args.collect::<Vec<_>>()),
        Some("playdate") => playdate(&args.collect::<Vec<_>>()),
        Some("playdate-images") => playdate_images_in(&repo_root(), &args.collect::<Vec<_>>()),
        Some("sfx") => sfx(&args.collect::<Vec<_>>()),
        Some("test-card") => test_card(&args.collect::<Vec<_>>()),
        Some("test-tileset") => test_tileset(&args.collect::<Vec<_>>()),
        Some("edge-marks") => edge_marks(&args.collect::<Vec<_>>()),
        Some("effect-marks") => effect_marks(&args.collect::<Vec<_>>()),
        Some("title-art") => title_art_in(&repo_root(), &args.collect::<Vec<_>>()),
        Some("map-sprite-import") => map_sprite_import(&args.collect::<Vec<_>>()),
        Some("battler-import") => battler_import_in(&repo_root(), &args.collect::<Vec<_>>()),
        Some("tileset-import") => tileset_import(&args.collect::<Vec<_>>()),
        Some("playtest") => playtest(&args.collect::<Vec<_>>()),
        Some("portrait-import") => portrait_import(&args.collect::<Vec<_>>()),
        Some("lines") => lines(&args.collect::<Vec<_>>()),
        Some("check-script") => check_script(&args.collect::<Vec<_>>()),
        Some("voice-test-clips") => voice_test_clips(&args.collect::<Vec<_>>()),
        Some(command) => {
            eprintln!("unknown command: {command}");
            eprintln!("{USAGE}");
            2
        }
        None => {
            println!("{USAGE}");
            2
        }
    }
}

/// Runs `command` of Shuyi's tools (ticket 1117) with `args` on this
/// repository, with the game's tools.
fn shuyi(command: &str, args: impl Iterator<Item = String>) -> u8 {
    let project = match shuyi_cli::Project::load(&repo_root()) {
        Ok(project) => project,
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };
    let args: Vec<String> = std::iter::once(command.to_owned()).chain(args).collect();
    shuyi_cli::run(&project, &args, Some(&game_tools::Trpg))
}

fn ticket_lint(args: &[String]) -> u8 {
    let (pr_branch, check_engine) = match parse_ticket_lint_args(args) {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };

    let repo_root = repo_root();
    let errors = tickets::run(&repo_root, pr_branch.as_deref(), check_engine);

    if errors.is_empty() {
        println!("ticket-lint: OK");
        return 0;
    }

    eprintln!("ticket-lint: {} error(s)", errors.len());
    for error in &errors {
        eprintln!("  {error}");
    }
    1
}

/// `engine [--off]`: the local checkout of Shuyi Engine (ticket 1131).
fn engine(args: &[String]) -> u8 {
    let off = match args {
        [] => false,
        [flag] if flag == "--off" => true,
        _ => {
            eprintln!("{}", engine::USAGE);
            return 2;
        }
    };
    report(engine::checkout(&repo_root(), off))
}

/// `engine-pin [<commit> | --check | --report]`: the engine's pinned commit.
fn engine_pin(args: &[String]) -> u8 {
    match args {
        [flag] if flag == "--check" => match engine::check(&repo_root()) {
            Ok(said) => {
                println!("{said}");
                0
            }
            Err(problems) => {
                eprintln!("engine-pin: {} problem(s)", problems.len());
                for problem in &problems {
                    eprintln!("  {problem}");
                }
                1
            }
        },
        [flag] if flag == "--report" => report(engine::watch_report(&repo_root())),
        [commit] if !commit.starts_with('-') => report(engine::pin(&repo_root(), Some(commit))),
        [] => report(engine::pin(&repo_root(), None)),
        _ => {
            eprintln!("{}", engine::USAGE);
            2
        }
    }
}

/// `skill-pack [--check]`: the pinned engine's skill pack in `.claude/skills/`.
fn skill_pack(args: &[String]) -> u8 {
    let pack = shuyi_cli::skills::Pack::embedded();
    match args {
        [] => report(skill_pack::write(&repo_root(), &pack)),
        [flag] if flag == "--check" => match skill_pack::check(&repo_root(), &pack) {
            Ok(said) => {
                println!("{said}");
                0
            }
            Err(problems) => {
                eprintln!(
                    "skill-pack: {} problem(s); `cargo xtask skill-pack` writes the copy",
                    problems.len()
                );
                for problem in &problems {
                    eprintln!("  {problem}");
                }
                1
            }
        },
        _ => {
            eprintln!("{}", skill_pack::USAGE);
            2
        }
    }
}

/// Prints what a command did, or why it failed.
fn report(outcome: Result<String, String>) -> u8 {
    match outcome {
        Ok(said) => {
            println!("{said}");
            0
        }
        Err(e) => {
            eprintln!("{e}");
            1
        }
    }
}

fn clean_merged_targets(args: &[String]) -> u8 {
    let dry_run = match clean_targets::parse_args(args) {
        Ok(dry_run) => dry_run,
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };
    let repo_root = repo_root();
    let tools = clean_targets::RealTools {
        repo_root: &repo_root,
    };
    let print = |line| println!("{line}");
    match clean_targets::run(&tools, &repo_root, SystemTime::now(), dry_run, print) {
        Ok(outcome) => u8::from(outcome.failed),
        Err(e) => {
            eprintln!("clean-merged-targets: {e}");
            1
        }
    }
}

fn icon(args: &[String]) -> u8 {
    match icon::run(args, &repo_root()) {
        Ok(summary) => {
            println!("{summary}");
            0
        }
        Err(e) => {
            eprintln!("icon: {e}");
            2
        }
    }
}

fn voice_test_clips(args: &[String]) -> u8 {
    if !args.is_empty() {
        eprintln!("usage: cargo xtask voice-test-clips");
        return 2;
    }
    match voice_test::run(&repo_root()) {
        Ok(summary) => {
            println!("{summary}");
            0
        }
        Err(e) => {
            eprintln!("voice-test-clips: {e}");
            1
        }
    }
}

fn sfx(args: &[String]) -> u8 {
    let Some(check) = parse_sfx_check(args) else {
        eprintln!("usage: cargo xtask sfx [--check]");
        return 2;
    };
    match sfx::run(&repo_root(), check) {
        Ok(summary) => {
            println!("{summary}");
            0
        }
        Err(e) => {
            eprintln!("sfx: {e}");
            1
        }
    }
}

/// `sfx`'s arguments: `Some(check)`, or `None` if they're wrong.
fn parse_sfx_check(args: &[String]) -> Option<bool> {
    match args {
        [] => Some(false),
        [flag] if flag == "--check" => Some(true),
        _ => None,
    }
}

fn test_card(args: &[String]) -> u8 {
    if !args.is_empty() {
        eprintln!("usage: cargo xtask test-card");
        return 2;
    }
    match test_card::run(&repo_root()) {
        Ok(summary) => {
            println!("{summary}");
            0
        }
        Err(e) => {
            eprintln!("test-card: {e}");
            1
        }
    }
}

fn test_tileset(args: &[String]) -> u8 {
    if !args.is_empty() {
        eprintln!("usage: cargo xtask test-tileset");
        return 2;
    }
    let root = repo_root();
    let written = test_tileset::run(&root)
        .and_then(|tiles| Ok([tiles, test_units::run(&root)?, test_auto::run(&root)?]));
    match written {
        Ok(summaries) => {
            println!("{}", summaries.join("\n"));
            0
        }
        Err(e) => {
            eprintln!("test-tileset: {e}");
            1
        }
    }
}

fn map_sprite_import(args: &[String]) -> u8 {
    map_sprite_import_in(&repo_root(), args)
}

/// `map-sprite-import` with the repo at `root`.
fn map_sprite_import_in(root: &Path, args: &[String]) -> u8 {
    use map_sprite_import::{SPRITES, USAGE, list, run};
    match args {
        [] => match run(root, &SPRITES) {
            Ok(summary) => {
                println!("{summary}");
                0
            }
            Err(e) => {
                eprintln!("map-sprite-import: {e}");
                1
            }
        },
        [flag] if flag == "--list" => {
            print!("{}", list(&SPRITES));
            0
        }
        [flag] if flag == "--help" => {
            println!("{USAGE}");
            0
        }
        _ => {
            eprintln!("{USAGE}");
            2
        }
    }
}

/// `battler-import` with the repo at `root`.
fn battler_import_in(root: &Path, args: &[String]) -> u8 {
    use battler_import::{BACKDROPS, BATTLERS, USAGE, list, run, write_backdrops_ron};
    match args {
        [] => match run(root, &BATTLERS, &BACKDROPS).and_then(|said| {
            write_backdrops_ron(root)?;
            Ok(said)
        }) {
            Ok(summary) => {
                println!("{summary}");
                0
            }
            Err(e) => {
                eprintln!("battler-import: {e}");
                1
            }
        },
        [flag] if flag == "--list" => {
            print!("{}", list(&BATTLERS, &BACKDROPS));
            0
        }
        [flag] if flag == "--help" => {
            println!("{USAGE}");
            0
        }
        _ => {
            eprintln!("{USAGE}");
            2
        }
    }
}

fn title_art_in(root: &Path, args: &[String]) -> u8 {
    use title_art::{BACKDROPS, USAGE, run, run_bought};
    let result = match args {
        [] => run(root),
        [flag] if flag == "--bought" => run_bought(root, &BACKDROPS),
        [flag] if flag == "--help" => {
            println!("{USAGE}");
            return 0;
        }
        _ => {
            eprintln!("{USAGE}");
            return 2;
        }
    };
    match result {
        Ok(summary) => {
            println!("{summary}");
            0
        }
        Err(e) => {
            eprintln!("title-art: {e}");
            1
        }
    }
}

fn tileset_import(args: &[String]) -> u8 {
    tileset_import_in(&repo_root(), args)
}

/// `tileset-import` with the repo at `root`.
fn tileset_import_in(root: &Path, args: &[String]) -> u8 {
    use tileset_import::{USAGE, list, read_mapping, run};
    let done = match args {
        [] => run(root, &map_sprite_import::SPRITES).map(|summary| format!("{summary}\n")),
        [flag] if flag == "--list" => read_mapping(root).map(|mapping| list(&mapping)),
        [flag] if flag == "--help" => Ok(format!("{USAGE}\n")),
        _ => {
            eprintln!("{USAGE}");
            return 2;
        }
    };
    match done {
        Ok(text) => {
            print!("{text}");
            0
        }
        Err(e) => {
            eprintln!("tileset-import: {e}");
            1
        }
    }
}

fn edge_marks(args: &[String]) -> u8 {
    if !args.is_empty() {
        eprintln!("usage: cargo xtask edge-marks");
        return 2;
    }
    match edge_marks::run(&repo_root()) {
        Ok(summary) => {
            println!("{summary}");
            0
        }
        Err(e) => {
            eprintln!("edge-marks: {e}");
            1
        }
    }
}

fn effect_marks(args: &[String]) -> u8 {
    if !args.is_empty() {
        eprintln!("usage: cargo xtask effect-marks");
        return 2;
    }
    match effect_marks::run(&repo_root()) {
        Ok(summary) => {
            println!("{summary}");
            0
        }
        Err(e) => {
            eprintln!("effect-marks: {e}");
            1
        }
    }
}

fn playdate(args: &[String]) -> u8 {
    let options = match playdate::parse_args(args) {
        Ok(options) => options,
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };
    match playdate::run(&repo_root(), options) {
        Ok(summary) => {
            println!("{summary}");
            0
        }
        Err(e) => {
            eprintln!("playdate: {e}");
            1
        }
    }
}

fn playdate_images_in(root: &Path, args: &[String]) -> u8 {
    match args {
        [] => {}
        [flag] if flag == "--help" => {
            println!("{}", playdate_images::USAGE);
            return 0;
        }
        _ => {
            eprintln!("{}", playdate_images::USAGE);
            return 2;
        }
    }
    match playdate_images::run(root) {
        Ok(summary) => {
            println!("{summary}");
            0
        }
        Err(e) => {
            eprintln!("playdate-images: {e}");
            1
        }
    }
}

fn playtest(args: &[String]) -> u8 {
    if args.iter().any(|a| a == "--help") {
        println!("{}", playtest::USAGE);
        return 0;
    }
    let options = match playtest::parse_args(args) {
        Ok(options) => options,
        Err(e) => {
            eprintln!("{e}\n\n{}", playtest::USAGE);
            return 2;
        }
    };
    match playtest::run(&repo_root(), &options) {
        Ok(ran) => {
            print!("{}", ran.text);
            if ran.blocked {
                eprintln!("playtest: blocked: Casual succeeds in too few tries");
                1
            } else {
                0
            }
        }
        Err(e) => {
            eprintln!("playtest: {e}");
            1
        }
    }
}

fn portrait_import(args: &[String]) -> u8 {
    if args.iter().any(|a| a == "--help") {
        println!("{}", portrait_import::USAGE);
        return 0;
    }
    let options = match portrait_import::parse_args(args) {
        Ok(options) => options,
        Err(e) => {
            eprintln!("{e}\n\n{}", portrait_import::USAGE);
            return 2;
        }
    };
    match portrait_import::run(&repo_root(), &options) {
        Ok(summary) => {
            println!("{summary}");
            0
        }
        Err(e) => {
            eprintln!("portrait-import: {e}");
            1
        }
    }
}

/// `ticket-lint`'s arguments: the `--pr-branch` value, and whether
/// `--check-engine` was given.
fn parse_ticket_lint_args(args: &[String]) -> Result<(Option<String>, bool), String> {
    let mut pr_branch = None;
    let mut check_engine = false;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--pr-branch" => match iter.next() {
                Some(branch) => pr_branch = Some(branch.clone()),
                None => return Err("--pr-branch requires a value".to_string()),
            },
            "--check-engine" => check_engine = true,
            other => return Err(format!("unknown argument to ticket-lint: {other}")),
        }
    }
    Ok((pr_branch, check_engine))
}

fn code_lines(args: &[String]) -> u8 {
    let base = match args {
        [] => "origin/main",
        [flag, base] if flag == "--base" => base.as_str(),
        [help] if help == "--help" => {
            println!("{}", code_lines::USAGE);
            return 0;
        }
        _ => {
            eprintln!("{}", code_lines::USAGE);
            return 2;
        }
    };
    match code_lines::run(&repo_root(), base) {
        Ok(report) => {
            print!("{report}");
            0
        }
        Err(e) => {
            eprintln!("code-lines: {e}");
            1
        }
    }
}

fn lines(args: &[String]) -> u8 {
    if args.iter().any(|a| a == "--help") {
        println!("{}", lines::USAGE);
        return 0;
    }
    let scene = match args {
        [] => None,
        [scene] => Some(scene.as_str()),
        _ => {
            eprintln!("{}", lines::USAGE);
            return 2;
        }
    };
    let rows = trpg_content::load_embedded()
        .map_err(|errors| errors.to_string())
        .and_then(|content| lines::rows(&content.dialogue, scene));
    match rows {
        Ok(rows) => {
            for row in rows {
                println!("{row}");
            }
            0
        }
        Err(e) => {
            eprintln!("lines: {e}");
            1
        }
    }
}

fn check_script(args: &[String]) -> u8 {
    if args.iter().any(|a| a == "--help") {
        println!("{}", check_script::USAGE);
        return 0;
    }
    let file = match args {
        [] => None,
        [file] => Some(Path::new(file)),
        _ => {
            eprintln!("{}", check_script::USAGE);
            return 2;
        }
    };
    match check_script::scripts(&repo_root(), file) {
        Ok((scripts, only)) => {
            let problems = check_script::problems(&scripts);
            let only = only.as_deref();
            print!("{}", check_script::report(&problems, only, scripts.len()));
            u8::from(!check_script::clean(&problems, only))
        }
        Err(e) => {
            eprintln!("check-script: {e}");
            2
        }
    }
}

/// `xtask` always runs via `cargo xtask`, so `CARGO_MANIFEST_DIR` (this
/// crate's directory, `<repo>/crates/xtask`) locates the repo root.
#[allow(clippy::expect_used)] // CARGO_MANIFEST_DIR is baked in at compile time; the two
// `parent()` calls can only fail if xtask's own manifest ever moves.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("xtask is at <repo>/crates/xtask")
        .to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_string()).collect()
    }

    /// The lines of `manifest`'s `[header]` table, without comments and
    /// blank lines.
    fn table<'a>(manifest: &'a str, header: &str) -> Vec<&'a str> {
        let lines = manifest.lines().skip_while(|l| l.trim() != header).skip(1);
        lines
            .take_while(|l| !l.starts_with('['))
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
            .collect()
    }

    /// The platform crate here, `trpg-playdate`, copies the workspace lints
    /// because one of them differs (`unsafe_code`, ADR-0070 §4); the copy
    /// must not drift.
    #[test]
    fn platform_lints_are_the_workspace_lints_except_unsafe_code() {
        let root = repo_root();
        let workspace = std::fs::read_to_string(root.join("Cargo.toml")).unwrap();
        let clippy = table(&workspace, "[workspace.lints.clippy]");
        assert!(clippy.len() > 5, "{clippy:?}");
        let rust: Vec<String> = table(&workspace, "[workspace.lints.rust]")
            .iter()
            .map(|l| l.replace("unsafe_code = \"forbid\"", "unsafe_code = \"deny\""))
            .collect();
        assert!(
            rust.contains(&"unsafe_code = \"deny\"".to_owned()),
            "{rust:?}"
        );
        // `shuyi-app`'s copy is checked by a test in the engine's
        // repository (ticket 1131).
        let manifest = std::fs::read_to_string(root.join("crates/playdate/Cargo.toml")).unwrap();
        assert_eq!(table(&manifest, "[lints.clippy]"), clippy);
        assert_eq!(table(&manifest, "[lints.rust]"), rust);
        // Every other crate inherits the workspace's `forbid`, the game's
        // binary too: its platform code is the engine's (ticket 1110).
        for other in ["app", "bots", "core", "content", "ui", "xtask"] {
            let manifest =
                std::fs::read_to_string(root.join(format!("crates/{other}/Cargo.toml"))).unwrap();
            assert_eq!(table(&manifest, "[lints]"), ["workspace = true"], "{other}");
        }
    }

    /// The playtest bots are dev tooling (ADR-0033): nothing the game is
    /// built from may depend on them, and they may not depend on the game's
    /// look (ADR-0038). Every way to a crate of this workspace is a
    /// manifest here, so reading the manifests covers indirect ones too.
    #[test]
    fn bots_stay_out_of_the_game() {
        // A manifest without its comments.
        let manifest = |name: &str| {
            let path = repo_root().join(format!("crates/{name}/Cargo.toml"));
            let text = std::fs::read_to_string(path).unwrap();
            let lines: Vec<&str> = text.lines().filter(|l| !l.starts_with('#')).collect();
            lines.join("\n")
        };
        for game in ["core", "content", "ui", "app"] {
            assert!(!manifest(game).contains("trpg-bots"), "{game}");
        }
        for bots in ["bots", "core", "content"] {
            let manifest = manifest(bots);
            assert!(!manifest.contains("trpg-ui"), "{bots}");
            assert!(!manifest.contains("trpg-app"), "{bots}");
        }
        assert!(manifest("xtask").contains("trpg-bots"));
    }

    /// The names of the `.rs` files directly in `dir` (not in its subfolders).
    fn top_level_rs_files(dir: &std::path::Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.is_file() && path.extension().is_some_and(|e| e == "rs"))
            .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    /// The workflow and action files under `.github`.
    fn workflow_files() -> Vec<PathBuf> {
        let github = repo_root().join(".github");
        let mut files = Vec::new();
        for entry in std::fs::read_dir(github.join("workflows")).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "yml" || e == "yaml") {
                files.push(path);
            }
        }
        for entry in std::fs::read_dir(github.join("actions")).unwrap() {
            let action = entry.unwrap().path().join("action.yml");
            if action.is_file() {
                files.push(action);
            }
        }
        files.sort();
        files
    }

    /// GitHub runs no job of a workflow file that is not valid YAML, and only
    /// says so after the merge: an unquoted `run:` line ending in `::`
    /// stopped every Pages deploy (ticket 0118).
    #[test]
    fn every_workflow_file_parses() {
        let files = workflow_files();
        assert!(files.len() >= 9, "{files:?}");
        for file in files {
            let text = std::fs::read_to_string(&file).unwrap();
            if let Err(e) = serde_norway::from_str::<serde_norway::Value>(&text) {
                panic!("{} is not valid YAML: {e}", file.display());
            }
        }

        // The check sees the mistake.
        let broken = "steps:
  - run: cargo test private_assets::
";
        assert!(serde_norway::from_str::<serde_norway::Value>(broken).is_err());
    }

    /// The `.rs` files under `dir`, at any depth.
    fn rs_files_under(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                rs_files_under(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }

    /// Which crates read each Markdown file of the repository, from the
    /// string literals that name it (`"../../docs/design/audio.md"`,
    /// `"dialogue/README.md"` in the embedded assets): the file's path
    /// from the root, to the packages whose sources name it.
    fn markdown_the_code_reads() -> BTreeMap<String, BTreeSet<String>> {
        let root = repo_root();
        let mut read: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for krate in std::fs::read_dir(root.join("crates")).unwrap() {
            let krate = krate.unwrap().path();
            let manifest = std::fs::read_to_string(krate.join("Cargo.toml")).unwrap();
            let package = manifest
                .lines()
                .find_map(|l| l.strip_prefix("name = \""))
                .and_then(|n| n.strip_suffix('"'))
                .expect("a package name")
                .to_string();
            let mut files = Vec::new();
            rs_files_under(&krate, &mut files);
            for file in files {
                let text = std::fs::read_to_string(&file).unwrap();
                // Every other piece between quotes is a literal; a comment
                // names a file without reading it.
                let code = text.lines().filter(|l| !l.trim_start().starts_with("//"));
                for literal in code.flat_map(|l| l.split('"').skip(1).step_by(2)) {
                    let markdown = Path::new(literal).extension().is_some_and(|e| e == "md");
                    if !markdown || !literal.contains('/') {
                        continue;
                    }
                    let path = literal.trim_start_matches("../");
                    for base in ["", "assets/", "assets-test/"] {
                        let path = format!("{base}{path}");
                        if root.join(&path).is_file() {
                            read.entry(path).or_default().insert(package.clone());
                        }
                    }
                }
            }
        }
        read
    }

    /// CI skips its build and tests on a pull request that changes only
    /// docs, Markdown among them (ADR-0014), so PR #464 changed only
    /// `docs/design/audio.md`, merged green and broke `main`. Each Markdown
    /// file the code reads has a `read by` line in `changes.yml` naming the
    /// crates that read it, whose tests CI's `doc-tests` job runs (ticket
    /// 0252, ADR-0098).
    #[test]
    fn every_markdown_file_the_code_reads_is_tested() {
        let changes =
            std::fs::read_to_string(repo_root().join(".github/workflows/changes.yml")).unwrap();
        let mut listed: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for line in changes.lines().filter(|l| l.contains("echo \"read by ")) {
            let (paths, rest) = line.trim().split_once(')').unwrap();
            let crates = rest
                .split_once("read by ")
                .unwrap()
                .1
                .split_once(':')
                .unwrap()
                .0;
            assert!(
                rest.contains(&format!("doc_crates=\"$doc_crates {crates}\"")),
                "{line}"
            );
            let crates: BTreeSet<String> = crates.split(' ').map(str::to_string).collect();
            for path in paths.split('|') {
                assert!(
                    listed.insert(path.to_string(), crates.clone()).is_none(),
                    "{path}"
                );
            }
        }
        let mut read = markdown_the_code_reads();
        // A file name in playdate_images' test of a made-up bundle, not read.
        assert!(read.remove("assets/battlers/README.md").is_some());
        assert_eq!(listed, read, "changes.yml's `read by` lines");
    }

    /// Cargo links every file directly under a crate's `tests/` as its own
    /// program (about 100 MB each), so each crate keeps one: new integration
    /// tests are modules of `tests/it/main.rs` (ticket 0114).
    #[test]
    fn each_crate_has_at_most_one_integration_test_program() {
        let crates = std::fs::read_dir(repo_root().join("crates")).unwrap();
        let mut checked = 0;
        for krate in crates {
            let tests = krate.unwrap().path().join("tests");
            if !tests.is_dir() {
                continue;
            }
            checked += 1;
            let files = top_level_rs_files(&tests);
            assert!(
                files.len() <= 1,
                "{} has {files:?}: put new integration tests in tests/it/ \
                 as a module listed in tests/it/main.rs",
                tests.display()
            );
        }
        assert!(checked >= 4, "{checked}");

        // The check sees a second file.
        let dir = std::env::temp_dir().join(format!("xtask-tests-dir-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("it")).unwrap();
        std::fs::write(dir.join("it/main.rs"), "").unwrap();
        std::fs::write(dir.join("a.rs"), "").unwrap();
        std::fs::write(dir.join("notes.md"), "").unwrap();
        assert_eq!(top_level_rs_files(&dir), ["a.rs"]);
        std::fs::write(dir.join("b.rs"), "").unwrap();
        assert_eq!(top_level_rs_files(&dir), ["a.rs", "b.rs"]);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn playtest_help_and_bad_args() {
        assert_eq!(playtest(&args(&["--help"])), 0);
        assert_eq!(playtest(&args(&["quick", "--help"])), 0);
        assert_eq!(playtest(&[]), 2);
        assert_eq!(playtest(&args(&["quick", "--runs", "0"])), 2);
        assert_eq!(dispatch(args(&["playtest", "--bogus", "1"]).into_iter()), 2);
    }

    #[test]
    fn playtest_reports_a_battle_and_fails_on_an_unknown_one() {
        let history = std::env::temp_dir().join(format!("xtask-playtest-{}", std::process::id()));
        let history_arg = history.to_string_lossy().into_owned();
        let run = |battle: &str| {
            let run = ["playtest", battle, "--runs", "2", "--history", &history_arg];
            dispatch(args(&run).into_iter())
        };
        assert_eq!(run("quick"), 0);
        assert!(history.join("quick-baseline-classic.jsonl").is_file());
        assert_eq!(run("no_such_battle"), 1);
        // Casual can't win the Quick Battle in one turn: blocked.
        let blocked = [
            "playtest",
            "quick",
            "--bot",
            "casual",
            "--runs",
            "1",
            "--turn-cap",
            "1",
            "--history",
            &history_arg,
        ];
        assert_eq!(dispatch(args(&blocked).into_iter()), 1);
        assert!(history.join("quick-casual.jsonl").is_file());
        std::fs::remove_dir_all(&history).unwrap();
    }

    #[test]
    fn frame_png_help_bad_args_and_a_run() {
        let frame_png = |items: &[String]| shuyi("frame-png", items.iter().cloned());
        assert_eq!(frame_png(&args(&["--help"])), 0);
        assert_eq!(frame_png(&[]), 2);
        assert_eq!(
            dispatch(args(&["frame-png", "a.png", "--bogus"]).into_iter()),
            2
        );
        let out = std::env::temp_dir().join(format!("xtask-frame-png-{}.png", std::process::id()));
        let out_arg = out.to_string_lossy().into_owned();
        assert_eq!(frame_png(&args(&[&out_arg, "--scale", "1"])), 0);
        assert!(out.is_file());
        std::fs::remove_file(&out).unwrap();
    }

    #[test]
    fn check_script_exits_zero_on_the_repository_scripts() {
        assert_eq!(dispatch(args(&["check-script"]).into_iter()), 0);
        assert_eq!(check_script(&args(&["--help"])), 0);
        assert_eq!(check_script(&args(&["a.dlg", "b.dlg"])), 2);
        assert_eq!(check_script(&args(&["no_such_script.dlg"])), 2);
        // A file with problems: a Markdown file whose example is no scene.
        let draft = std::env::temp_dir().join(format!("{}_check_script.md", std::process::id()));
        std::fs::write(
            &draft,
            "```dlg
> Outside a scene.
```
",
        )
        .unwrap();
        let code = check_script(&args(&[&draft.to_string_lossy()]));
        std::fs::remove_file(&draft).unwrap();
        assert_eq!(code, 1);
    }

    #[test]
    fn code_lines_reports_and_rejects_bad_arguments() {
        assert_eq!(
            dispatch(args(&["code-lines", "--base", "HEAD"]).into_iter()),
            0
        );
        assert_eq!(code_lines(&args(&["--help"])), 0);
        assert_eq!(code_lines(&args(&["--base"])), 2);
        assert_eq!(code_lines(&args(&["--base", "no-such-ref-1151"])), 1);
    }

    #[test]
    fn lines_lists_a_scene_and_fails_on_an_unknown_one() {
        assert_eq!(dispatch(args(&["lines", "ch01_intro"]).into_iter()), 0);
        assert_eq!(lines(&[]), 0);
        assert_eq!(lines(&args(&["--help"])), 0);
        assert_eq!(lines(&args(&["no_such_scene"])), 1);
        assert_eq!(lines(&args(&["ch01_intro", "ch01_prebattle"])), 2);
    }

    #[test]
    fn icon_exits_with_the_outcome() {
        assert_eq!(dispatch(args(&["icon"]).into_iter()), 2);
        let dir = env::temp_dir().join(format!("icon-dispatch-{}", std::process::id()));
        let dir = dir.to_str().unwrap().to_owned();
        assert_eq!(dispatch(args(&["icon", "preview", &dir]).into_iter()), 0);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn repo_root_points_at_the_workspace_root() {
        let root = repo_root();
        assert!(root.join("Cargo.toml").is_file());
        assert!(root.join("tickets/README.md").is_file());
    }

    #[test]
    fn parse_ticket_lint_args_with_no_args_is_none() {
        assert_eq!(parse_ticket_lint_args(&[]), Ok((None, false)));
    }

    #[test]
    fn parse_ticket_lint_args_reads_the_pr_branch_value() {
        assert_eq!(
            parse_ticket_lint_args(&args(&["--pr-branch", "t0106-x"])),
            Ok((Some("t0106-x".to_string()), false))
        );
    }

    #[test]
    fn parse_ticket_lint_args_reads_check_engine_in_any_order() {
        let want = Ok((Some("t0106-x".to_string()), true));
        assert_eq!(
            parse_ticket_lint_args(&args(&["--check-engine", "--pr-branch", "t0106-x"])),
            want
        );
        assert_eq!(
            parse_ticket_lint_args(&args(&["--pr-branch", "t0106-x", "--check-engine"])),
            want
        );
    }

    #[test]
    fn parse_ticket_lint_args_requires_a_pr_branch_value() {
        assert_eq!(
            parse_ticket_lint_args(&args(&["--pr-branch"])),
            Err("--pr-branch requires a value".to_string())
        );
    }

    #[test]
    fn parse_ticket_lint_args_rejects_unknown_flags() {
        assert_eq!(
            parse_ticket_lint_args(&args(&["--bogus"])),
            Err("unknown argument to ticket-lint: --bogus".to_string())
        );
    }

    #[test]
    fn ticket_lint_fails_fast_on_bad_args() {
        assert_eq!(ticket_lint(&args(&["--pr-branch"])), 2);
    }

    #[test]
    fn ticket_lint_succeeds_on_the_real_repo() {
        assert_eq!(ticket_lint(&[]), 0);
    }

    #[test]
    fn ticket_lint_fails_when_pr_branch_names_an_unknown_ticket() {
        // Ticket 9999 will never exist (see tickets/README.md's numbering),
        // unlike a real open/done ticket id, whose status changes as tickets
        // are worked — this exercises the wrapper's non-zero exit path
        // without depending on the repo's current ticket state.
        assert_eq!(ticket_lint(&args(&["--pr-branch", "t9999-ghost"])), 1);
    }

    #[test]
    fn dispatch_with_no_command_prints_usage_and_fails() {
        assert_eq!(dispatch(std::iter::empty()), 2);
    }

    #[test]
    fn dispatch_with_unknown_command_fails() {
        assert_eq!(dispatch(args(&["bogus"]).into_iter()), 2);
    }

    fn font_atlas(items: &[String]) -> u8 {
        shuyi("font-atlas", items.iter().cloned())
    }

    #[test]
    fn font_atlas_needs_two_args() {
        assert_eq!(font_atlas(&args(&["only-one"])), 2);
        assert_eq!(dispatch(args(&["font-atlas"]).into_iter()), 2);
    }

    #[test]
    fn font_atlas_reports_failure() {
        assert_eq!(font_atlas(&args(&["no/such/font.bdf", "out"])), 1);
    }

    #[test]
    fn playdate_fails_fast_on_bad_args() {
        // Only checks argument parsing: a real build needs the Playdate SDK
        // (`playdate::tests` check the build's steps; CI's `playdate` job
        // runs it).
        assert_eq!(playdate(&args(&["--bogus"])), 2);
        assert_eq!(
            dispatch(args(&["playdate", "--device", "--simulator"]).into_iter()),
            2
        );
    }

    #[test]
    fn playdate_images_takes_no_args_but_help() {
        // `playdate_images::tests` run it in a scratch folder.
        assert_eq!(
            dispatch(args(&["playdate-images", "--bogus"]).into_iter()),
            2
        );
        assert_eq!(playdate_images_in(&repo_root(), &args(&["--help"])), 0);
        let empty = std::env::temp_dir().join(format!("xtask-pi-{}", std::process::id()));
        assert_eq!(playdate_images_in(&empty, &[]), 1);
    }

    #[test]
    fn web_fails_fast_on_bad_args() {
        // Only checks argument parsing: a real `--release`/no-args run
        // spawns `cargo build`, which is exercised by `web::tests` and
        // manually, not here.
        assert_eq!(shuyi("web", args(&["--bogus"]).into_iter()), 2);
        assert_eq!(dispatch(args(&["web", "--bogus"]).into_iter()), 2);
    }

    #[test]
    fn private_assets_fails_fast_on_bad_args() {
        // Only the argument check: a real run needs the private repository
        // and would move this checkout's `assets-private/`.
        assert_eq!(shuyi("private-assets", args(&["--bogus"]).into_iter()), 2);
        assert_eq!(
            dispatch(args(&["private-assets", "--pin", "--library"]).into_iter()),
            2
        );
    }

    #[test]
    fn portrait_import_help_and_bad_args() {
        // Only the argument check: a real run writes into `assets-private/`.
        assert_eq!(portrait_import(&args(&["--help"])), 0);
        assert_eq!(portrait_import(&[]), 2);
        assert_eq!(portrait_import(&args(&["busts", "Not An Id"])), 2);
        // A source that isn't there fails before anything is written.
        assert_eq!(portrait_import(&args(&["no/such/folder", "k"])), 1);
        assert_eq!(
            dispatch(args(&["portrait-import", "busts", "k", "--shift-x", "9"]).into_iter()),
            2
        );
    }

    #[test]
    fn parse_sfx_check_reads_the_flag() {
        assert_eq!(parse_sfx_check(&[]), Some(false));
        assert_eq!(parse_sfx_check(&args(&["--check"])), Some(true));
        assert_eq!(parse_sfx_check(&args(&["--bogus"])), None);
    }

    #[test]
    fn voice_test_clips_takes_no_arguments() {
        assert_eq!(voice_test_clips(&args(&["x"])), 2);
        let bogus = args(&["voice-test-clips", "--bogus"]);
        assert_eq!(dispatch(bogus.into_iter()), 2);
    }

    #[test]
    fn sfx_rejects_unknown_args() {
        assert_eq!(sfx(&args(&["--bogus"])), 2);
        assert_eq!(dispatch(args(&["sfx", "--check", "x"]).into_iter()), 2);
    }

    #[test]
    fn sfx_check_passes_on_the_committed_files() {
        assert_eq!(sfx(&args(&["--check"])), 0);
    }

    #[test]
    fn test_card_rejects_args() {
        assert_eq!(test_card(&args(&["--bogus"])), 2);
        assert_eq!(dispatch(args(&["test-card", "x"]).into_iter()), 2);
    }

    #[test]
    fn tileset_import_help_list_and_bad_args() {
        assert_eq!(tileset_import(&args(&["--help"])), 0);
        // The committed mapping file is read and listed.
        assert_eq!(tileset_import(&args(&["--list"])), 0);
        assert_eq!(tileset_import(&args(&["--bogus"])), 2);
        // Without the checkout of the bought art (or the mapping file):
        // an error, and nothing written.
        let bare = std::env::temp_dir().join(format!("xtask-ti-bare-{}", std::process::id()));
        assert_eq!(tileset_import_in(&bare, &[]), 1);
        assert_eq!(tileset_import_in(&bare, &args(&["--list"])), 1);
        assert!(!bare.exists());
        assert_eq!(
            dispatch(args(&["tileset-import", "--list", "x"]).into_iter()),
            2
        );
    }

    #[test]
    fn map_sprite_import_help_list_and_bad_args() {
        // Only the argument check: a real run writes into `assets-private/`.
        assert_eq!(map_sprite_import(&args(&["--help"])), 0);
        assert_eq!(map_sprite_import(&args(&["--list"])), 0);
        assert_eq!(map_sprite_import(&args(&["--bogus"])), 2);
        // No arguments runs it: in a folder with no checkout, that fails.
        let bare = std::env::temp_dir().join(format!("xtask-msi-none-{}", std::process::id()));
        assert_eq!(map_sprite_import_in(&bare, &[]), 1);
        assert_eq!(
            dispatch(args(&["map-sprite-import", "--list", "x"]).into_iter()),
            2
        );
    }

    #[test]
    fn battler_import_help_list_and_bad_args() {
        // Only the argument check: a real run writes into `assets-private/`.
        let here = repo_root();
        assert_eq!(battler_import_in(&here, &args(&["--help"])), 0);
        assert_eq!(battler_import_in(&here, &args(&["--list"])), 0);
        assert_eq!(battler_import_in(&here, &args(&["--bogus"])), 2);
        // No arguments runs it: in a folder with no checkout, that fails.
        let bare = std::env::temp_dir().join(format!("xtask-bi-none-{}", std::process::id()));
        assert_eq!(battler_import_in(&bare, &[]), 1);
        assert_eq!(
            dispatch(args(&["battler-import", "--list", "x"]).into_iter()),
            2
        );
    }

    #[test]
    fn title_art_help_and_bad_args() {
        // Only the argument check: a real run writes into `assets/`.
        let here = repo_root();
        assert_eq!(title_art_in(&here, &args(&["--help"])), 0);
        assert_eq!(title_art_in(&here, &args(&["--bogus"])), 2);
        assert_eq!(title_art_in(&here, &args(&["--bought", "x"])), 2);
        // In a folder with no logo source and no checkout, both runs fail.
        let bare = std::env::temp_dir().join(format!("xtask-ta-none-{}", std::process::id()));
        assert_eq!(title_art_in(&bare, &[]), 1);
        assert_eq!(title_art_in(&bare, &args(&["--bought"])), 1);
    }

    #[test]
    fn edge_marks_rejects_args() {
        assert_eq!(edge_marks(&args(&["--bogus"])), 2);
        assert_eq!(dispatch(args(&["edge-marks", "x"]).into_iter()), 2);
    }

    #[test]
    fn effect_marks_rejects_args() {
        assert_eq!(effect_marks(&args(&["--bogus"])), 2);
        assert_eq!(dispatch(args(&["effect-marks", "x"]).into_iter()), 2);
    }

    #[test]
    fn test_tileset_rejects_args() {
        assert_eq!(test_tileset(&args(&["--bogus"])), 2);
        assert_eq!(dispatch(args(&["test-tileset", "x"]).into_iter()), 2);
    }

    fn check_keys(items: &[String]) -> u8 {
        shuyi("check-keys", items.iter().cloned())
    }

    fn check_text(items: &[String]) -> u8 {
        shuyi("check-text", items.iter().cloned())
    }

    fn lang_status(items: &[String]) -> u8 {
        shuyi("lang-status", items.iter().cloned())
    }

    #[test]
    fn check_keys_passes_on_the_real_repo() {
        assert_eq!(check_keys(&[]), 0);
        assert_eq!(dispatch(args(&["check-keys"]).into_iter()), 0);
    }

    #[test]
    fn engine_pin_check_passes_on_the_real_repo() {
        assert_eq!(engine_pin(&args(&["--check"])), 0);
        assert_eq!(engine_pin(&args(&["--bogus"])), 2);
        assert_eq!(engine_pin(&args(&["a", "b"])), 2);
        assert_eq!(engine(&args(&["--bogus"])), 2);
        assert_eq!(engine(&args(&["--off", "x"])), 2);
    }

    #[test]
    fn check_text_passes_on_the_real_repo() {
        assert_eq!(check_text(&[]), 0);
        assert_eq!(dispatch(args(&["check-text"]).into_iter()), 0);
        assert_eq!(check_text(&args(&["--bogus"])), 2);
    }

    #[test]
    fn lang_status_exits_zero_with_missing_and_stale_keys() {
        // The test pack has one of each.
        assert_eq!(lang_status(&args(&["test"])), 0);
        assert_eq!(dispatch(args(&["lang-status", "test"]).into_iter()), 0);
        assert_eq!(lang_status(&args(&["zz"])), 2);
        assert_eq!(lang_status(&[]), 2);
        assert_eq!(lang_status(&args(&["test", "ja"])), 2);
        assert_eq!(lang_status(&args(&["test", "--only", "dialogue"])), 0);
        assert_eq!(lang_status(&args(&["test", "--file", "test"])), 0);
        assert_eq!(
            lang_status(&args(&["test", "--only", "ui", "--file", "x"])),
            2
        );
        assert_eq!(lang_status(&args(&["test", "--bogus"])), 2);
    }

    #[test]
    fn check_keys_rejects_args() {
        assert_eq!(check_keys(&args(&["--bogus"])), 2);
    }

    #[test]
    fn clean_merged_targets_rejects_unknown_args() {
        // Only the argument check: a real run needs `gh` and would delete
        // build folders.
        assert_eq!(clean_merged_targets(&args(&["--bogus"])), 2);
        assert_eq!(
            dispatch(args(&["clean-merged-targets", "--dry-run", "x"]).into_iter()),
            2
        );
    }

    #[test]
    fn dispatch_runs_ticket_lint() {
        assert_eq!(dispatch(args(&["ticket-lint"]).into_iter()), 0);
    }
}
