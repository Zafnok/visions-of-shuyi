//! The game's tool program (ADR-0129): `cargo shuyi` runs Shuyi's tools
//! here with this game's ([`game_tools::Trpg`]), and the commands that
//! build the game live here: the bots, the voice, the importers, the
//! generators and the Playdate build. The repository's process checks are
//! in `xtask`, which builds without the game and runs each command of this
//! program by its old name (`cargo xtask playtest` runs `playtest` here).
//!
//! The generic tools moved to Shuyi Engine's `shuyi-cli` (ticket 1117):
//! `cargo shuyi <command>`. Their old names here forward to it.
//!
//! A CLI tool prints to stdout by design, so `print_stdout` is allowed here.
#![allow(clippy::print_stdout)]

mod battler_import;
mod bot_wins;
mod check_script;
mod content_layer;
mod game_tools;
mod icon;
mod lettering;
mod lines;
mod map_sprite_import;
mod playdate;
mod playdate_audio;
mod playdate_images;
mod playtest;
mod portrait_import;
mod readme_pictures;
mod sfx;
mod skill_pack;
mod skirmish;
mod story_import;
mod test_auto;
mod test_tileset;
mod test_units;
mod tileset_import;
mod tileset_ron;
mod title_art;
mod voice;
mod voice_test;

use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: cargo xtask <command> (or cargo run -p trpg-tools -- <command>)\n\n\
the game's tool program's commands (the process checks are xtask's: cargo xtask):\n  \
skill-pack [--check]               copy the pinned engine's skill pack into .claude/skills/, or check the copy (ADR-0097)\n  \
icon preview <dir> | build <n>     draw the icon's options, or build the chosen one\n  \
sfx [--check]                      render our own sounds into assets/audio/sfx/\n  \
test-tileset                       write the sprite map skins' test tilesets, assets-test/tilesets/test*\n  \
lettering                          bake the chapter card's Cinzel sheets, assets/title/chapter_*\n  \
title-art [--bought]               write the title logo and placeholder backdrops, assets/title/\n  \
map-sprite-import [--list]         copy the bought map sprites the game uses into assets-private/game/\n  \
battler-import [--list]            copy the bought combat pictures and backdrops into assets-private/game/\n  \
tileset-import [--list]            pack the bought terrain tiles and write the game's tileset there\n  \
playtest <battle-id> [options]     a bot plays a battle many times and reports (playtest --help)\n  \
bot-wins (<battle-id> | --all)     record the bots' own wins, the lines their armies come from (bot-wins --help)\n  \
skirmish new --at <x,y> --level <n> generate a skirmish whose bots land in their bands (skirmish --help)\n  \
portrait-import <busts> <id>       cut bought busts into portraits (portrait-import --help)\n  \
lines [scene]                      list every dialogue line with its line id (lines --help)\n  \
check-script [file]                check the dialogue scripts (check-script --help)\n  \
story-import <n> [--dry-run]       take a mirror pull request Nick approved into the assets repository\n  \
voice <status|generate|script|audit> the voice clips of the dialogue lines (voice --help)\n  \
voice-test-clips                   make the tone clips that test voice playback (needs ffmpeg)\n  \
playdate [--release] [--bench] [--private-assets] [--device|--simulator|--both]\n                                     build the Playdate game into target/playdate/VisionsOfShuyi.pdx\n                                     (--bench: the build that times the renderer, its own .pdx;\n                                     --private-assets: with the bought art)
  readme-pictures                    draw the README's intro animation again, docs/img/intro.gif
                                     (with --features private-assets; readme-pictures --help)
  playdate-images                    write the Playdate's one-bit pictures to look at, into
                                     target/playdate/images/ (and the bought ones' into assets-private/)

moved to Shuyi's tools, `cargo shuyi <command>` (these names still run them):
  check-keys, check-text, lang-status, font-atlas, frame-png, web, private-assets
  (`cargo shuyi` is `cargo run -p trpg-tools -- shuyi`: Shuyi's tools with this game's)";

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
        Some("shuyi") => shuyi_main(args.collect()),
        Some("skill-pack") => skill_pack(&args.collect::<Vec<_>>()),
        Some("icon") => icon(&args.collect::<Vec<_>>()),
        Some("playdate") => playdate(&args.collect::<Vec<_>>()),
        Some("playdate-images") => playdate_images_in(&repo_root(), &args.collect::<Vec<_>>()),
        Some("sfx") => sfx(&args.collect::<Vec<_>>()),
        Some("test-tileset") => test_tileset(&args.collect::<Vec<_>>()),
        Some("lettering") => lettering(&args.collect::<Vec<_>>()),
        Some("title-art") => title_art_in(&repo_root(), &args.collect::<Vec<_>>()),
        Some("map-sprite-import") => map_sprite_import(&args.collect::<Vec<_>>()),
        Some("battler-import") => battler_import_in(&repo_root(), &args.collect::<Vec<_>>()),
        Some("tileset-import") => tileset_import(&args.collect::<Vec<_>>()),
        Some("playtest") => playtest(&args.collect::<Vec<_>>()),
        Some("bot-wins") => bot_wins::command(&repo_root(), &args.collect::<Vec<_>>()),
        Some("skirmish") => skirmish::command(&repo_root(), &args.collect::<Vec<_>>()),
        Some("portrait-import") => portrait_import(&args.collect::<Vec<_>>()),
        Some("lines") => lines(&args.collect::<Vec<_>>()),
        Some("check-script") => check_script(&args.collect::<Vec<_>>()),
        Some("story-import") => {
            story_import::command(&repo_root(), &args.collect::<Vec<_>>(), &story_import::gh)
        }
        Some("voice") => voice(&args.collect::<Vec<_>>()),
        Some("readme-pictures") => readme_pictures(&args.collect::<Vec<_>>()),
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
/// `cargo shuyi`: Shuyi's tools with this game's. `check-keys` and `lint`
/// run on the project as [`content_layer::project`] makes it, so they scan
/// the content when it is there (ADR-0117).
fn shuyi_main(args: Vec<String>) -> u8 {
    let layered = args
        .split_first()
        .filter(|(name, _)| scans_the_content(name));
    if let Some((name, rest)) = layered {
        return shuyi(name, rest.iter().cloned());
    }
    shuyi_cli::main_with(args.into_iter(), Some(&game_tools::Trpg))
}

/// Whether `cargo shuyi <command>` runs on the project with the content in
/// what it scans ([`content_layer::project`]): `check-keys` and `lint`.
fn scans_the_content(command: &str) -> bool {
    command == "check-keys" || command == "lint"
}

/// Runs the Shuyi tool `command` with `args` on this repository's project,
/// with the content in what `check-keys` scans ([`content_layer::project`]).
fn shuyi(command: &str, args: impl Iterator<Item = String>) -> u8 {
    let project = match shuyi_cli::Project::load(&repo_root()) {
        Ok(project) => content_layer::project(project),
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };
    let args: Vec<String> = std::iter::once(command.to_owned()).chain(args).collect();
    shuyi_cli::run(&project, &args, Some(&game_tools::Trpg))
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

/// `readme-pictures` (ticket 1264): the README's intro animation, drawn by
/// `frame-png` with the bought art.
fn readme_pictures(args: &[String]) -> u8 {
    // `not` in the cfg, not a `!`: without the art no test may draw the
    // picture, and a mutant that deletes a `!` would draw it for minutes.
    if cfg!(not(feature = "private-assets")) {
        eprintln!("readme-pictures: the picture shows the bought art; build with --features private-assets
");
        eprintln!("{}", readme_pictures::USAGE);
        return 2;
    }
    readme_pictures::command(&repo_root(), args, |args| {
        shuyi("frame-png", args.iter().cloned())
    })
}

// engine-friction: engine#2578, moves (game 1224 deletes it)
/// `skill-pack [--check]`: the pinned engine's skill pack in `.claude/skills/`.
fn skill_pack(args: &[String]) -> u8 {
    skill_pack_in(&repo_root(), args)
}

// engine-friction: engine#2578, moves (game 1224 deletes it)
/// `skill-pack` with the repo at `root`.
fn skill_pack_in(root: &Path, args: &[String]) -> u8 {
    let pack = shuyi_cli::skills::Pack::embedded();
    match args {
        [] => report(skill_pack::write(root, &pack)),
        [flag] if flag == "--check" => match skill_pack::check(root, &pack) {
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

fn voice(args: &[String]) -> u8 {
    if args.iter().any(|a| a == "--help") {
        println!("{}", voice::USAGE);
        return 0;
    }
    let args = match voice::parse(args) {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };
    match voice::run(&repo_root(), &args) {
        Ok(report) => {
            println!("{report}");
            0
        }
        Err(e) => {
            eprintln!("voice: {e}");
            1
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

fn lettering(args: &[String]) -> u8 {
    if !args.is_empty() {
        eprintln!("usage: cargo xtask lettering");
        return 2;
    }
    match lettering::run(&repo_root()) {
        Ok(summary) => {
            println!("{summary}");
            0
        }
        Err(e) => {
            eprintln!("lettering: {e}");
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
    if args.first().is_some_and(|a| a == "--merge") {
        let paths: Vec<PathBuf> = args[1..].iter().map(PathBuf::from).collect();
        if paths.is_empty() {
            eprintln!("--merge needs the parts' reports\n\n{}", playtest::USAGE);
            return 2;
        }
        return match playtest::merge_files(&paths) {
            Ok(ran) => {
                print!("{}", ran.text);
                u8::from(ran.blocked)
            }
            Err(e) => {
                eprintln!("playtest --merge: {e}");
                1
            }
        };
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
            let (code, out) = trpg_writer::check(&scripts, only.as_deref(), check_script::COMMAND);
            print!("{out}");
            code
        }
        Err(e) => {
            eprintln!("check-script: {e}");
            2
        }
    }
}

/// The program runs from its build in this repository (`cargo run`, or
/// `cargo xtask` forwarding), so `CARGO_MANIFEST_DIR` (this crate's
/// directory, `<repo>/crates/tools`) locates the repo root.
#[allow(clippy::expect_used)] // CARGO_MANIFEST_DIR is baked in at compile time; the two
// `parent()` calls can only fail if this crate's own manifest ever moves.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("trpg-tools is at <repo>/crates/tools")
        .to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn playtest_help_and_bad_args() {
        assert_eq!(playtest(&args(&["--help"])), 0);
        assert_eq!(playtest(&args(&["quick", "--help"])), 0);
        assert_eq!(playtest(&[]), 2);
        assert_eq!(playtest(&args(&["quick", "--runs", "0"])), 2);
        assert_eq!(dispatch(args(&["playtest", "--bogus", "1"]).into_iter()), 2);
        assert_eq!(playtest(&args(&["--merge"])), 2);
        assert_eq!(playtest(&args(&["--merge", "no/such/part.json"])), 1);
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

    /// Without the bought art the README's intro can't be drawn (1264).
    #[cfg(not(feature = "private-assets"))]
    #[test]
    fn readme_pictures_needs_the_bought_art() {
        assert_eq!(dispatch(args(&["readme-pictures"]).into_iter()), 2);
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
    fn lines_lists_a_scene_and_fails_on_an_unknown_one() {
        assert_eq!(dispatch(args(&["lines", "test_intro"]).into_iter()), 0);
        assert_eq!(lines(&[]), 0);
        assert_eq!(lines(&args(&["--help"])), 0);
        assert_eq!(lines(&args(&["no_such_scene"])), 1);
        assert_eq!(lines(&args(&["test_intro", "test_victory"])), 2);
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
        assert!(root.join("shuyi.ron").is_file());
    }

    #[test]
    fn dispatch_with_no_command_prints_usage_and_fails() {
        assert_eq!(dispatch(std::iter::empty()), 2);
    }

    /// `check-keys` and `lint` read the content when it is there; the other
    /// commands run on the project as `shuyi.ron` gives it.
    #[test]
    fn only_check_keys_and_lint_scan_the_content() {
        assert!(scans_the_content("check-keys"));
        assert!(scans_the_content("lint"));
        assert!(!scans_the_content("check-text"));
        assert!(!scans_the_content("frame-png"));
    }

    /// With no arguments the pack is written; then `--check` passes; any
    /// other argument is refused.
    #[test]
    fn skill_pack_writes_checks_and_rejects_bad_arguments() {
        let dir = env::temp_dir().join(format!("trpg-tools-skill-pack-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".claude/skills")).unwrap();
        assert_eq!(skill_pack_in(&dir, &args(&["--check"])), 1);
        assert_eq!(skill_pack_in(&dir, &[]), 0);
        assert_eq!(skill_pack_in(&dir, &args(&["--check"])), 0);
        assert_eq!(skill_pack_in(&dir, &args(&["--bogus"])), 2);
        assert_eq!(
            dispatch(args(&["skill-pack", "--check", "x"]).into_iter()),
            2
        );
        // A skills folder that is a file can't be written: the error, 1.
        std::fs::remove_dir_all(dir.join(".claude/skills")).unwrap();
        std::fs::write(dir.join(".claude/skills"), "not a folder").unwrap();
        assert_eq!(skill_pack_in(&dir, &[]), 1);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// `voice`: its help, 2 for arguments it can't read, 1 for a run that
    /// fails (a scene or a language the game doesn't have).
    #[test]
    fn voice_help_bad_args_and_a_failed_run() {
        assert_eq!(voice(&args(&["status", "--help"])), 0);
        assert_eq!(voice(&args(&["--bogus"])), 2);
        assert_eq!(voice(&args(&["status", "--lang"])), 2);
        assert_eq!(voice(&args(&["status", "--lang", "zz"])), 1);
        assert_eq!(
            dispatch(args(&["voice", "status", "--scene", "no_such_scene"]).into_iter()),
            1
        );
    }

    #[test]
    fn dispatch_with_unknown_command_fails() {
        assert_eq!(dispatch(args(&["bogus"]).into_iter()), 2);
        // The process checks are xtask's (ADR-0129).
        assert_eq!(dispatch(args(&["ticket-lint"]).into_iter()), 2);
    }

    /// `cargo xtask` runs here each command it lists as moved, and every
    /// command here is one of them, so none is lost to an old name
    /// (ADR-0129).
    #[test]
    fn xtask_forwards_every_command_of_this_program() {
        let listed: Vec<&str> = USAGE
            .lines()
            .filter_map(|l| l.strip_prefix("  "))
            .filter(|l| l.starts_with(|c: char| c.is_ascii_lowercase()))
            .filter_map(|l| l.split([' ', ',']).next())
            .collect();
        let mut ours: Vec<&str> = listed.clone();
        // The line of Shuyi's tools that moved, and `shuyi` itself.
        let shuyi = USAGE.lines().find(|l| l.contains("check-keys, ")).unwrap();
        ours.extend(shuyi.split(", ").map(str::trim));
        ours.push("shuyi");
        ours.sort_unstable();
        ours.dedup();
        let mut forwarded = xtask::FORWARDED.to_vec();
        forwarded.sort_unstable();
        assert_eq!(ours, forwarded);
        // Each answers here: an unknown command would be 2 with no help.
        for command in [
            "playtest",
            "bot-wins",
            "lines",
            "check-script",
            "voice",
            "skirmish",
        ] {
            assert_eq!(
                dispatch(args(&[command, "--help"]).into_iter()),
                0,
                "{command}"
            );
        }
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
    fn lettering_rejects_args() {
        assert_eq!(lettering(&args(&["--bogus"])), 2);
        assert_eq!(dispatch(args(&["lettering", "x"]).into_iter()), 2);
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
}
