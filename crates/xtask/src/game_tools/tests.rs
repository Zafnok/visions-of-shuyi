//! Shuyi's tools on this repository (ticket 1117): the game's `frame-png`
//! and `lang-status`, the checks on the real code, and the committed font
//! atlas.

use std::fs;
use std::path::{Path, PathBuf};

use shuyi_assets::font::{CellWidths, FontAtlasDef};
use shuyi_cli::Project;
use shuyi_cli::codec::decode_png;
use shuyi_cli::contract::{check, lint, playtest, replay, target};
use shuyi_cli::font_atlas::{ATLAS_RANGES, AtlasChars, ONE_BIT_FILE, read_font};
use shuyi_cli::frame_png::{self, Options};
use shuyi_cli::lang_status::{self, Part};

use super::*;
use crate::repo_root;

fn args(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_string()).collect()
}

/// This repository's project.
fn project() -> Project {
    Project::load(&repo_root()).unwrap()
}

/// `frame-png`'s arguments, with the game's options.
fn parse(items: &[&str]) -> Result<Options, String> {
    frame_png::parse_args(&args(items), &OPTIONS)
}

/// Runs `frame-png` with `options`, writing under `dir`.
fn run(dir: &Path, options: &Options) -> Result<String, String> {
    frame_png::run(dir, options, &Trpg, None)
}

/// A fresh empty directory under the system temp directory.
fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("xtask-frame-png-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Ticket 1244: `--battle` and `--scene` open the game there, one of them
/// once, and never with `--prompt`, which is the title's.
#[test]
fn the_game_opens_on_a_battle_or_a_scene() {
    let opens = |args: &[&str]| {
        let mut all = vec!["a.png"];
        all.extend(args);
        Start::of(&parse(&all).unwrap().game).map(|start| start.opens)
    };
    assert_eq!(opens(&[]), Ok(Opens::Title));
    assert_eq!(
        opens(&["--battle", "ch04"]),
        Ok(Opens::Battle("ch04".into()))
    );
    assert_eq!(opens(&["--scene", "a"]), Ok(Opens::Scene("a".into())));
    let twice = Err("--battle and --scene: one or the other, once".to_owned());
    assert_eq!(opens(&["--battle", "a", "--scene", "b"]), twice);
    assert_eq!(opens(&["--battle", "a", "--battle", "b"]), twice);
    assert_eq!(
        opens(&["--prompt", "--scene", "a"]),
        Err("--prompt: only at the title, not with --battle or --scene".into())
    );
}

/// The battle `--battle` opens is that battle at its start, and an
/// unknown one is refused with the battles there are.
#[test]
fn a_session_opens_on_the_battle_asked_for() {
    let start = |args: &[&str]| {
        let mut all = vec!["a.png"];
        all.extend(args);
        Start::of(&parse(&all).unwrap().game).unwrap()
    };
    let h = harness(&start(&["--battle", "ch04"])).unwrap();
    assert_eq!(h.top_screen(), "battle");
    let battle = h.battle().unwrap().state();
    // Its own characters fill all eight slots, Gil's too (1071).
    assert_eq!((battle.turn(), battle.units().len()), (1, 8 + 13));
    let Err(unknown) = harness(&start(&["--battle", "nope"])) else {
        panic!("no battle nope");
    };
    assert!(unknown.starts_with("no battle \"nope\" in assets/battles/; the battles are: "));
    assert!(unknown.contains("ch04"), "{unknown}");
    let h = harness(&start(&["--scene", "ch04_intro"])).unwrap();
    assert_eq!(h.top_screen(), "dialogue");
}

/// `bench` on the Playdate plays the game on its 50 × 15 console, each
/// frame drawn in one bit; there is no other one-bit target.
#[test]
fn the_playdate_session_plays_on_its_console() {
    let battle = [GameArg {
        flag: "--battle".into(),
        value: Some("ch04".into()),
    }];
    let mut session = Trpg.session_on("playdate", &battle).unwrap();
    session.idle(2);
    assert_eq!(session.frame_count(), 2);
    assert_eq!(session.screens().last().map(String::as_str), Some("battle"));
    let snapshot = session.snapshot();
    let row = snapshot.lines().next().unwrap_or_default();
    assert_eq!(row.chars().count(), 50, "{snapshot}");
    let Err(desktop) = Trpg.session_on("desktop", &battle) else {
        panic!("the desktop isn't one-bit");
    };
    assert_eq!(desktop, "desktop: the game's one-bit target is playdate");
}

#[test]
fn the_games_options_are_read() {
    let given = parse(&[
        "a.png",
        "--layout",
        "left",
        "--prompt",
        "--light",
        "dusk",
        "--tips",
        "--map",
        "--playdate",
    ])
    .unwrap();
    assert_eq!(
        Start::of(&given.game),
        Ok(Start {
            layout: LEFT_HANDED,
            playdate: true,
            shows: Shows::MapAlone,
            prompt: true,
            light: Some("dusk".into()),
            tips: true,
            opens: Opens::Title,
            after: Vec::new(),
        })
    );
    let plain = Start::of(&parse(&["a.png"]).unwrap().game).unwrap();
    assert_eq!(
        (
            plain.layout,
            plain.playdate,
            plain.shows,
            plain.prompt,
            plain.light,
            plain.tips
        ),
        (RIGHT_HANDED, false, Shows::Screen, false, None, false)
    );
    let middle = parse(&["a.png", "--layout", "middle"]).unwrap();
    assert_eq!(
        Start::of(&middle.game),
        Err("--layout: right or left, not middle".into())
    );
    let unknown = [GameArg {
        flag: "--bogus".into(),
        value: None,
    }];
    assert!(Start::of(&unknown).is_err());
    assert!(parse(&["a.png", "--light"]).is_err());
    assert!(parse(&["a.png", "--bogus"]).is_err());
}

#[test]
fn the_game_gives_its_options_and_notes() {
    assert_eq!(Trpg.frame_options(), &OPTIONS);
    assert_eq!(Trpg.frame_notes(), NOTES);
}

#[test]
fn the_help_names_the_games_options_and_the_bought_art() {
    let usage = frame_png::usage("cargo shuyi", &OPTIONS, NOTES);
    assert!(usage.contains("\n  --light NAME         fight every battle in this light"));
    assert!(usage.contains("\n  --layout right|left  the key layout picked before"));
    assert!(usage.ends_with("(CLAUDE.md, Bought art)."), "{usage}");
}

#[test]
fn writes_the_title_screen_and_the_same_bytes_twice() {
    let dir = temp_dir("title");
    let options = parse(&["out/title.png", "--scale", "1"]).unwrap();
    let summary = run(&dir, &options).unwrap();
    assert!(summary.starts_with("frame-png: title → "), "{summary}");
    let first = fs::read(dir.join("out/title.png")).unwrap();
    let (w, h, rgba) = decode_png(&first).unwrap();
    assert_eq!((w, h), (800, 512));
    // The console background, opaque, in the bottom corner (the help
    // line's row: the title's backdrop ends above it).
    let corner = 511 * 800 * 4;
    assert_eq!(&rgba[corner..corner + 4], &[0, 0, 0, 255]);
    // Something is drawn.
    assert!(rgba.chunks(4).any(|p| p[..3] != [0, 0, 0]));
    run(&dir, &options).unwrap();
    assert_eq!(fs::read(dir.join("out/title.png")).unwrap(), first);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn keys_reach_the_quick_battle_map() {
    let dir = temp_dir("battle");
    let options = parse(&["b.png", "--keys", "Down f Left f", "--scale", "1"]).unwrap();
    let summary = run(&dir, &options).unwrap();
    assert!(summary.starts_with("frame-png: battle → "), "{summary}");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn the_prompt_start_draws_another_title() {
    let dir = temp_dir("prompt");
    let render = |extra: &[&str]| {
        let mut all = vec!["t.png", "--scale", "1"];
        all.extend_from_slice(extra);
        run(&dir, &parse(&all).unwrap()).unwrap();
        fs::read(dir.join("t.png")).unwrap()
    };
    assert_ne!(render(&["--prompt"]), render(&[]));
    fs::remove_dir_all(&dir).unwrap();
}

/// Ticket 0455: `--tips` shows the start tip over the Quick Battle.
#[test]
fn tips_show_with_the_tips_option() {
    let dir = temp_dir("tips");
    let render = |extra: &[&str]| {
        let mut all = vec!["t.png", "--keys", "Down f Left f f", "--scale", "1"];
        all.extend_from_slice(extra);
        run(&dir, &parse(&all).unwrap()).unwrap();
        fs::read(dir.join("t.png")).unwrap()
    };
    assert_ne!(render(&["--tips"]), render(&[]));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_light_must_be_one_of_the_lights() {
    let dir = temp_dir("light");
    let options = |light| parse(&["out.png", "--scale", "1", "--light", light]);
    let err = run(&dir, &options("midnight").unwrap()).unwrap_err();
    assert_eq!(
        err,
        "--light: no light \"midnight\"; the lights are \"dawn\", \"dusk\", \"night\", \"noon\""
    );
    assert!(!dir.join("out.png").exists());
    assert!(run(&dir, &options("night").unwrap()).is_ok());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn the_scene_camera_frame_shows_the_map_in_its_window() {
    let dir = temp_dir("frame-png-backdrop");
    let mut options = parse(&["camera.png", "--keys", "F2 Up Up Up f f"]).unwrap();
    options.scale = Some(1);
    let printed = run(&dir, &options).unwrap();
    assert!(printed.contains("scene_camera"), "{printed}");
    let frame = frame_png::decode_image(&fs::read(dir.join("camera.png")).unwrap()).unwrap();
    // The middle of the window (cells 26..74 × 4..18) isn't the clear
    // colour: the map is there.
    let middle = frame.pixel(50 * 8, 11 * 16).unwrap();
    assert_ne!(middle, [0, 0, 0, 255]);
    let _ = fs::remove_dir_all(&dir);
}

/// Ticket 1032: `--playdate` draws the Playdate's screen, black and white.
#[test]
fn the_playdate_frame_is_400_by_240_in_black_and_white() {
    let dir = temp_dir("playdate");
    // The debug menu: no pictures, which are grey boxes until 1040.
    let options = parse(&["p.png", "--playdate", "--keys", "F2"]).unwrap();
    let summary = run(&dir, &options).unwrap();
    assert!(summary.contains("(400×240)"), "{summary}");
    let (w, h, rgba) = decode_png(&fs::read(dir.join("p.png")).unwrap()).unwrap();
    assert_eq!((w, h), (400, 240));
    let black = rgba.chunks(4).filter(|p| *p == [0, 0, 0, 255]).count();
    let white = rgba.chunks(4).filter(|p| *p == [255; 4]).count();
    assert_eq!(black + white, 400 * 240);
    // Mostly the white ground, and something drawn on it.
    assert!(white > black && black > 0, "{white} white, {black} black");
    let options = parse(&["p.png", "--playdate", "--scale", "2"]).unwrap();
    assert!(run(&dir, &options).unwrap().contains("(800×480)"));
    fs::remove_dir_all(&dir).unwrap();
}

/// Ticket 1039: the engine's `--crank` step winds the Playdate's crank
/// (the game's own until engine#2540) and `--then-pad` presses buttons
/// after the steps, in turn: wound back on the Quick Battle's map, the
/// rewind screen; A asks there.
#[test]
fn the_crank_and_buttons_after_the_steps_open_the_rewind_screen() {
    let given = parse(&["r.png", "--crank", "-30", "--then-pad", "East"]).unwrap();
    let after = Start::of(&given.game).unwrap().after;
    assert_eq!(after, [After::Pad("East".into())]);
    let dir = temp_dir("crank");
    let quick = [
        "r.png",
        "--playdate",
        "--pad",
        "DpadDown East",
        "--wait",
        "0.1",
        "--pad",
        "DpadLeft East",
        "--wait",
        "0.1",
        "--pad",
        "East",
    ];
    let shows = |extra: &[&str]| {
        let options = parse(&[&quick[..], extra].concat()).unwrap();
        run(&dir, &options).unwrap()
    };
    assert!(shows(&[]).starts_with("frame-png: battle → "));
    assert!(shows(&["--crank", "-30"]).starts_with("frame-png: rewind → "));
    fs::remove_dir_all(&dir).unwrap();
}

/// Ticket 1037: `--map` draws the battle's map alone over the whole
/// screen in the map skin: on the Playdate, its own; with no battle, an
/// error.
#[test]
fn the_map_alone_fills_the_screen() {
    let dir = temp_dir("map");
    let quick = ["m.png", "--keys", "Down f Left f f", "--map"];
    let picture = |extra: &[&str]| {
        let options = parse(&[&quick[..], extra].concat()).unwrap();
        let summary = run(&dir, &options).unwrap();
        assert!(summary.starts_with("frame-png: battle → "), "{summary}");
        decode_png(&fs::read(dir.join("m.png")).unwrap()).unwrap()
    };
    let (w, h, rgba) = picture(&["--playdate"]);
    assert_eq!((w, h), (400, 240));
    // The Quick Battle's small map in the middle of the screen, not where
    // the battle screen's PC layout puts it.
    let black = |x: usize, y: usize| rgba[(y * 400 + x) * 4] == 0;
    let column = |x| (0..240).any(|y| black(x, y));
    let left = (0..400).find(|&x| column(x)).unwrap();
    let right = (0..400).rev().find(|&x| column(x)).unwrap();
    assert!(left >= 8 && right < 392, "{left}..={right}");
    assert!(left.abs_diff(399 - right) <= 16, "{left}..={right}");
    let options = parse(&[&quick[..3], &["--playdate"][..]].concat()).unwrap();
    run(&dir, &options).unwrap();
    let (_, _, screen) = decode_png(&fs::read(dir.join("m.png")).unwrap()).unwrap();
    assert_ne!(screen, rgba);
    // The PC's map alone: the glyph skin over the whole console.
    let (w, h, _) = picture(&["--scale", "1"]);
    assert_eq!((w, h), (800, 512));
    let options = parse(&["t.png", "--map"]).unwrap();
    let error = run(&dir, &options).unwrap_err();
    assert_eq!(error, "--map: no battle on screen (title)");
    fs::remove_dir_all(&dir).unwrap();
}

/// The game's languages and the variants of its lines.
fn lang() -> GameLang {
    Trpg.lang().unwrap()
}

/// The test pack keeps one of each on purpose (tickets 0233, 0235),
/// and translates only a little of the data and one scene.
#[test]
fn the_test_pack_has_stale_missing_and_orphaned_text() {
    let lang = lang();
    assert_eq!(lang.variants, ["m", "f"]);
    let options = lang_status::Options::default();
    let report = lang_status::report(
        &lang.lang,
        &lang.variants,
        &CellWidths::NARROW,
        "test",
        &options,
    )
    .unwrap();
    let first = report.lines().next().unwrap_or_default();
    assert!(
        first.starts_with("lang-status test (TEST, machine-made): "),
        "{first}"
    );
    assert!(first.ends_with(" missing, 2 stale, 1 orphaned"), "{first}");
    assert!(report.contains("missing:\n  title.credits\n"), "{report}");
    // A data name and a dialogue line nobody translated.
    assert!(
        report.contains("  items.iron_spear.name\n    source: \"Iron Spear\"\n"),
        "{report}"
    );
    assert!(
        report.contains("  test_36a692b7\n    source: \"Let's move.\"\n"),
        "{report}"
    );
    assert!(report.contains("stale:\n  title.continue\n"), "{report}");
    let axe = "  items.iron_axe.name\n    was: \"Old Axe\"\n    now: \"Iron Axe\"\n";
    assert!(report.contains(axe), "{report}");
    // The orphan is paired with the reworded line.
    assert!(
        report.ends_with(
            "orphaned:\n  test_00000000\n    was: \"It is always about you.\"\n    \
             text: \"IT IS ALWAYS ABOUT YOU.\"\n    nearest: test_92dc3bd5\n    \
             now: \"It's always about you.\"\n"
        ),
        "{report}"
    );
}

#[test]
fn the_test_pack_keeps_the_test_text_and_lists_a_files_lines() {
    let lang = lang();
    let report = |options: &lang_status::Options| {
        lang_status::report(
            &lang.lang,
            &lang.variants,
            &CellWidths::NARROW,
            "test",
            options,
        )
        .unwrap()
    };
    let dialogue = report(&lang_status::Options {
        only: Some(Part::Dialogue),
        file: None,
    });
    assert!(!dialogue.contains("left out"), "{dialogue}");
    assert!(
        dialogue.contains("  test_36a692b7\n    source:"),
        "{dialogue}"
    );
    // The test pack lists the test scene, and only the file asked for.
    let test = report(&lang_status::Options {
        only: None,
        file: Some("test".into()),
    });
    assert!(test.contains("  test_36a692b7\n    source:"), "{test}");
    assert!(!test.contains("screen text"), "{test}");
    let lines = &lang.lang.source().lines;
    for (key, line) in lines.iter().filter(|(_, line)| line.file != "test") {
        assert!(
            !test.contains(&format!("  {key}\n    source:")),
            "{key} is from {}",
            line.file
        );
    }
}

/// The checks pass on the real code, and the title screen (ticket 0233,
/// the first one converted) has no literal left.
#[test]
fn the_checks_pass_on_this_repository() {
    let project = project();
    let keys = project.config.check_keys.as_ref().unwrap();
    let errors = shuyi_cli::check_keys::run(keys, &project.root);
    assert!(errors.is_empty(), "check-keys errors: {errors:#?}");
    let text = project.config.check_text.as_ref().unwrap();
    let (hits, errors) = shuyi_cli::check_text::run(text, &project.root);
    assert_eq!(errors, Vec::<String>::new());
    assert_eq!(hits.len(), text.max_literals);
    assert_eq!(text.max_literals, 0);
    assert_eq!(
        shuyi_cli::check_text::run_names(text, &project.root),
        Vec::<String>::new()
    );
}

/// Ticket 0138: `shuyi.ron` lists the game's three targets, at the
/// console's sizes, and `check --target all` reaches each with no problem
/// and no limit about a missing setting.
#[test]
fn the_project_declares_the_targets_at_the_consoles_sizes() {
    let project = project();
    let setting = |id: &str| project.config.targets[id];
    for (id, console) in [
        ("desktop", ConsoleSpec::PC),
        ("web", ConsoleSpec::PC),
        ("playdate", ConsoleSpec::PLAYDATE),
    ] {
        let (width, height) = console.px();
        let (_, cell_height) = console.cell_px();
        let found = setting(id);
        assert_eq!(
            (i32::from(found.frame.0), i32::from(found.frame.1)),
            (width, height),
            "{id}"
        );
        assert_eq!(i32::from(found.text_px), cell_height, "{id}");
    }
    let mut input = check::CheckInput::default();
    input.target = Some(target::ALL.to_owned());
    let answer = check::targets(&project.config, &project.root, &input, Some(&Trpg));
    assert!(answer.ok, "{:?} {:?}", answer.error, answer.result);
    let reports = answer.result.unwrap().targets;
    let reached: Vec<&str> = reports.iter().map(|r| r.target.as_str()).collect();
    assert_eq!(reached, ["desktop", "playdate", "web"]);
    assert!(reports.iter().all(|r| r.problems.is_empty()), "{reports:?}");
    let lacking: Vec<&String> = answer
        .limits
        .iter()
        .filter(|limit| limit.contains("`targets`") || limit.contains("target profile"))
        .collect();
    assert_eq!(lacking, Vec::<&String>::new());
}

/// Ticket 0140: `check --target all` checks the controls and the one-bit
/// patterns: neither is a limit.
#[test]
fn check_target_checks_the_controls_and_the_one_bit_patterns() {
    let project = project();
    let mut input = check::CheckInput::default();
    input.target = Some(target::ALL.to_owned());
    let answer = check::targets(&project.config, &project.root, &input, Some(&Trpg));
    assert!(answer.ok, "{:?} {:?}", answer.error, answer.result);
    let unchecked: Vec<&String> = answer
        .limits
        .iter()
        .filter(|limit| limit.contains("aren't checked"))
        .collect();
    assert_eq!(unchecked, Vec::<&String>::new());
}

/// Every screen is on the list once, and the controls reach the Playdate's
/// table: its required actions fit, and a chord it lacks would be found.
#[test]
fn the_controls_list_each_screen_once_and_see_a_missing_chord() {
    let screens = trpg_ui::screens::actions::every_screen();
    let names: std::collections::BTreeSet<&str> = screens.iter().map(|s| s.screen).collect();
    assert_eq!(names.len(), screens.len());
    assert!(names.contains("battle") && names.contains("title") && names.contains("options"));

    let project = project();
    let profile = target::profile(&project.config, &project.root, "playdate")
        .unwrap()
        .profile;
    let controls = Trpg.controls().unwrap().unwrap();
    assert_eq!(controls.fit(&profile), Vec::new());
    let mut lacking = profile.clone();
    lacking.input.chords.clear();
    let found = controls.fit(&lacking);
    assert!(
        found
            .iter()
            .any(|p| p.screen == Some("battle") && p.actions == ["EndTurn"]),
        "{found:?}"
    );
}

/// The map is the file the Playdate's build draws with, and what must
/// read apart does.
#[test]
fn the_one_bit_skin_reads_the_map_file() {
    use shuyi_assets::onebit::{self, Pattern};
    let skin = Trpg.one_bit("playdate").unwrap().unwrap();
    assert_eq!(onebit::check(&skin.palette, &skin.map), Vec::new());
    let fg = |role: &str| skin.map.get(role).unwrap().fg;
    assert_eq!(fg("map_player_tile"), Pattern::BLACK);
    assert_eq!(fg("black"), Pattern::WHITE);
    assert_ne!(fg("move_range"), fg("attack_range"));
    assert_eq!(skin.map.roles.len(), skin.palette.colors.len());
    assert_eq!(skin.map.distinct.len(), 2);
    assert!(
        skin.map_file.ends_with("playdate@onebit.ron"),
        "{}",
        skin.map_file
    );
    assert_eq!(Some(skin.map), trpg_content::ink::load_playdate_map().ok());
}

/// Text is never dithered on the Playdate (`one_bit.rs` says why the map's
/// `text` is empty).
#[test]
fn the_text_roles_are_solid_black() {
    use trpg_content::ink::Ink;
    let skin = Trpg.one_bit("playdate").unwrap().unwrap();
    for role in ["text", "text_dim", "text_highlight"] {
        assert_eq!(skin.palette.get(role), Some(Ink::Black.code()), "{role}");
    }
}

/// A cargo that finds nothing and keeps the commands it was given.
#[derive(Default)]
struct QuietCargo(std::cell::RefCell<Vec<String>>);

impl lint::Cargo for QuietCargo {
    fn run(&self, _root: &Path, args: &[&str]) -> Result<lint::Ran, String> {
        self.0.borrow_mut().push(args.join(" "));
        Ok(lint::Ran {
            success: true,
            ..lint::Ran::default()
        })
    }
}

/// Ticket 0138: `lint` fences `trpg-core`, with nothing left out. The
/// cargo is a fake here: CI's `cargo shuyi lint` runs the real one.
#[test]
fn lint_fences_the_rules_crate() {
    let project = project();
    assert_eq!(project.config.rules.as_deref(), Some("trpg-core"));
    let cargo = QuietCargo::default();
    let answer = lint::lint(&cargo, &project, &lint::LintInput::default());
    assert!(answer.ok, "{:?}", answer.result);
    assert_eq!(answer.limits, Vec::<String>::new());
    let ran = cargo.0.borrow();
    assert!(
        ran[0].starts_with("clippy --quiet -p trpg-core "),
        "{ran:?}"
    );
    assert!(ran[1].starts_with("tree -p trpg-core "), "{ran:?}");
}

/// Ticket 0138: the fence's lints have bans to read: without the rules
/// crate's `clippy.toml`, `lint` would pass having checked nothing.
#[test]
fn the_rules_crate_has_the_fences_bans() {
    let bans = fs::read_to_string(repo_root().join("crates/core/clippy.toml")).unwrap();
    for ban in [
        "std::collections::HashMap",
        "std::collections::HashSet",
        "std::time::Instant::now",
        "std::fs::read_to_string",
        "std::thread::spawn",
        "rand::random",
        "f32",
        "f64",
    ] {
        assert!(bans.contains(&format!("\"{ban}\"")), "{ban}");
    }
    let lib = fs::read_to_string(repo_root().join("crates/core/src/lib.rs")).unwrap();
    assert!(lib.contains("#![cfg_attr(not(test), deny(clippy::float_arithmetic))]"));
}

/// `shuyi.ron` names the glyphs the game's content requires.
#[test]
fn the_project_requires_the_glyphs_the_content_does() {
    let project = project();
    let atlas = project.config.font_atlas.unwrap();
    assert_eq!(atlas.required_glyphs, trpg_content::font::REQUIRED_GLYPHS);
}

/// `shuyi.ron` names the files the content does for the credits check.
#[test]
fn the_project_checks_the_credits_the_content_reads() {
    use trpg_content::bundle::{PRIVATE_DISPLAY_ROOT, display_path};
    let credits = project().config.private_assets.unwrap().credits.unwrap();
    assert_eq!(
        credits.file,
        display_path(trpg_content::credits::CREDITS_PATH)
    );
    assert_eq!(
        credits.audio_manifest,
        display_path(trpg_content::audio::AUDIO_PATH)
    );
    assert_eq!(credits.private_root, PRIVATE_DISPLAY_ROOT);
}

/// Ticket 1208: every track and audio license `export.beside` ships is on the
/// `disclosure` list, as each target names it, so a new one fails here and not
/// when a release is cut. Ticket 0911: so is every file of `voice/` (the clips,
/// their manifest and the voice tool's files beside it), when the private
/// repository's copy is here (`cargo shuyi private-assets`).
#[test]
fn the_disclosure_list_names_the_music_voices_and_licenses_a_package_ships() {
    let root = repo_root();
    let allowed = project().config.disclosure.unwrap().allowed_files;
    let mut shipped = Vec::new();
    for (folder, shipped_as) in [
        ("music", "music"),
        ("assets/audio/licenses", "audio-licenses"),
        ("voice", "voice"),
    ] {
        if folder == "voice" && !root.join(folder).is_dir() {
            continue;
        }
        files_under(&root.join(folder), shipped_as, &mut shipped);
    }
    assert!(shipped.len() > 40, "the music wasn't found: {shipped:?}");
    for path in shipped {
        let in_app = format!("visions-of-shuyi.app/Contents/Resources/{path}");
        assert!(allowed.contains(&path), "{path}");
        assert!(allowed.contains(&in_app), "{in_app}");
    }
}

/// Every file under `dir`, at any depth, as `prefix/<path>` with `/`.
fn files_under(dir: &Path, prefix: &str, out: &mut Vec<String>) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = format!("{prefix}/{}", entry.file_name().to_string_lossy());
        if entry.file_type().unwrap().is_dir() {
            files_under(&entry.path(), &path, out);
        } else {
            out.push(path);
        }
    }
}

/// Ticket 0909: `NOTICES.txt` carries the license text of every shipped
/// row of `THIRD_PARTY_ASSETS.md` whose text is under `assets/`, and
/// `assets/NOTICES-game.txt` credits those rows and every work of the audio
/// manifest.
#[test]
fn the_notices_name_the_games_works_and_license_texts() {
    let root = repo_root();
    let texts = project().config.notices.unwrap().texts;
    let table = fs::read_to_string(root.join("THIRD_PARTY_ASSETS.md")).unwrap();
    let listed = fs::read_to_string(root.join("assets/NOTICES-game.txt"))
        .unwrap()
        .replace("\r\n", "\n");
    let mut rows = 0;
    for row in table.lines().filter(|line| line.starts_with("| ")) {
        let cells: Vec<&str> = row.split(" | ").collect();
        let Some((_, link)) = cells.get(4).and_then(|cell| cell.split_once("](assets/")) else {
            continue;
        };
        let file = format!("assets/{}", link.trim_end_matches(')'));
        assert!(texts.contains(&file), "{file} isn't in `notices`");
        assert!(listed.contains(cells[1]), "{} isn't listed", cells[1]);
        rows += 1;
    }
    assert!(rows > 40, "the table wasn't found: {rows} rows");
    let entries: Vec<&str> = listed.split("\n\n").collect();
    let audio = trpg_content::audio::load().unwrap();
    let works = audio.credits.values();
    for work in works.filter(|work| work.license != trpg_content::audio::OWN) {
        let entry = entries
            .iter()
            .find(|entry| entry.lines().next() == Some(work.title.as_str()));
        let entry = entry.unwrap_or_else(|| panic!("{} isn't listed", work.title));
        for line in [
            format!("  Author: {}\n", work.author),
            format!("  Source: {}\n", work.source),
            format!("  License: {}", work.license),
        ] {
            assert!(entry.contains(&line), "{}: no {line}", work.title);
        }
    }
}

/// The font sources the committed atlas is built from, relative to the
/// repo root: Terminus, then our own extra glyphs (the controller
/// buttons, the building tiles).
const SOURCES: [&str; 3] = [
    "assets-src/fonts/ter-u16n.bdf",
    "assets-src/fonts/pad-shapes.bdf",
    "assets-src/fonts/tile-shapes.bdf",
];

/// The controller button shapes are our own glyphs, in the atlas next
/// to Terminus's, each 7×7 on the baseline.
#[test]
fn the_pad_shapes_are_in_the_atlas() {
    let root = repo_root();
    let shapes = read_font(&root.join(SOURCES[1])).unwrap();
    assert_eq!(shapes.glyphs.keys().collect::<String>(), "□△◯✕");
    let grid = |c: char| -> Vec<String> {
        shapes.glyphs[&c]
            .chunks(8)
            .map(|row| row.iter().map(|&on| if on { '#' } else { '.' }).collect())
            .collect()
    };
    assert_eq!(
        grid('△')[5..12],
        [
            "...#....", "...#....", "..#.#...", "..#.#...", ".#...#..", ".#...#..", "#######."
        ]
    );
    for (c, pixels) in &shapes.glyphs {
        let rows: Vec<&[bool]> = pixels.chunks(8).collect();
        let lit = |row: &&[bool]| row.iter().any(|&on| on);
        assert_eq!(rows.iter().position(lit), Some(5), "{c}");
        assert_eq!(rows.iter().rposition(lit), Some(11), "{c}");
        assert!(
            rows.iter().all(|row| !row[7]),
            "{c}: the last column is the gap"
        );
        assert!(ATLAS_RANGES.iter().any(|(lo, hi)| (lo..=hi).contains(&c)));
    }
    let atlas = fs::read_to_string(root.join("assets/fonts/atlas.ron")).unwrap();
    let atlas: FontAtlasDef = ron::from_str(&atlas).unwrap();
    assert!(shapes.glyphs.keys().all(|c| atlas.glyphs.contains_key(c)));
}

/// The house, gate and throne (ticket 0322) are each one 16×16 picture
/// over the tile's two cells: our own glyphs, the terrain's pair, in the
/// atlas.
#[test]
fn the_tile_shapes_are_the_terrains_pictures() {
    let root = repo_root();
    let shapes = read_font(&root.join(SOURCES[2])).unwrap();
    assert_eq!(shapes.glyphs.keys().collect::<String>(), "☖☗♔♕⛨⛩");
    let picture = |[left, right]: [char; 2]| -> Vec<String> {
        let half = |c: char| shapes.glyphs[&c].chunks(8).collect::<Vec<_>>();
        let (left, right) = (half(left), half(right));
        (0..16)
            .map(|y| {
                let row = left[y].iter().chain(right[y]);
                row.map(|&on| if on { '#' } else { '.' }).collect()
            })
            .collect()
    };
    let terrain = trpg_content::terrain::TerrainDef::load(None).unwrap();
    let glyphs = |id: &str| {
        let id = terrain.display.id_of(id).unwrap();
        terrain.display.get(id).unwrap().glyphs
    };
    assert_eq!(picture(glyphs("house"))[13], ".##############.");
    assert_eq!(picture(glyphs("gate"))[7], "################");
    assert_eq!(picture(glyphs("throne"))[0], "...#...##...#...");
    let atlas = fs::read_to_string(root.join("assets/fonts/atlas.ron")).unwrap();
    let atlas: FontAtlasDef = ron::from_str(&atlas).unwrap();
    for c in shapes.glyphs.keys() {
        assert!(ATLAS_RANGES.iter().any(|(lo, hi)| (lo..=hi).contains(&c)));
        assert!(atlas.glyphs.contains_key(c), "{c}");
    }
}

/// The committed atlas must be exactly what the tool produces from the
/// committed font source and the characters the game's text uses, as
/// `cargo shuyi font-atlas` keeps them (ticket 1243).
#[test]
fn committed_atlas_is_up_to_date() {
    let project = project();
    let root = &project.root;
    let rules = project.config.font_atlas.as_ref().unwrap();
    let dir = std::env::temp_dir().join(format!("xtask-atlas-check-{}", std::process::id()));
    let fonts = SOURCES.map(|source| root.join(source));
    let chars =
        AtlasChars::of_lang(&lang().lang, &rules.required_glyphs).with_wide(&rules.wide_glyphs);
    let msg =
        shuyi_cli::font_atlas::run(rules, project.config.command(), &fonts, &[], &dir, &chars)
            .unwrap();
    assert!(msg.starts_with("font-atlas: "));
    let read = |p: &Path| fs::read_to_string(p).unwrap().replace("\r\n", "\n");
    let rerun = "rerun `cargo shuyi font-atlas assets-src/fonts/ter-u16n.bdf \
                 assets-src/fonts/pad-shapes.bdf assets-src/fonts/tile-shapes.bdf \
                 assets/fonts`";
    assert_eq!(
        read(&dir.join("atlas.ron")),
        read(&root.join("assets/fonts/atlas.ron")),
        "assets/fonts/atlas.ron is stale: {rerun}"
    );
    let png = |p: &Path| decode_png(&fs::read(p).unwrap()).unwrap();
    assert!(
        png(&dir.join("atlas.png")) == png(&root.join("assets/fonts/atlas.png")),
        "assets/fonts/atlas.png is stale: {rerun}"
    );
    assert!(
        fs::read(dir.join(ONE_BIT_FILE)).unwrap()
            == fs::read(root.join("assets/fonts").join(ONE_BIT_FILE)).unwrap(),
        "assets/fonts/{ONE_BIT_FILE} is stale: {rerun}"
    );
    fs::remove_dir_all(&dir).unwrap();
}

/// The committed one-bit atlas has exactly the pixels of the committed
/// `atlas.png`, glyph by glyph, at the cells of `atlas.ron`.
#[test]
fn the_one_bit_atlas_matches_the_png() {
    let fonts = repo_root().join("assets/fonts");
    let def: FontAtlasDef =
        ron::from_str(&fs::read_to_string(fonts.join("atlas.ron")).unwrap()).unwrap();
    let (width, _, rgba) = decode_png(&fs::read(fonts.join("atlas.png")).unwrap()).unwrap();
    let bits = fs::read(fonts.join(ONE_BIT_FILE)).unwrap();
    let to_usize = |v: u32| usize::try_from(v).unwrap();
    let rows = to_usize(def.cell_h);
    assert_eq!(bits.len(), def.glyphs.len() * rows);
    for (&c, &index) in &def.glyphs {
        let cell = def.cell_rect(index);
        for row in 0..cell.h {
            let byte = bits[to_usize(index) * rows + to_usize(row)];
            for col in 0..cell.w {
                let (x, y) = (cell.x + col, cell.y + row);
                let alpha = rgba[(to_usize(y) * to_usize(width) + to_usize(x)) * 4 + 3];
                let bit = byte >> (7 - col) & 1 == 1;
                assert_eq!(bit, alpha > 0, "{c:?} at ({col}, {row})");
            }
        }
    }
    // 'A' has its left stroke in the left half of its cell.
    let a = to_usize(def.glyphs[&'A']) * rows;
    assert!(bits[a..a + rows].iter().any(|&row| row & 0xf0 != 0));
}

/// Ticket 1181: the session (`cargo shuyi run`, `scene`, the MCP server's)
/// reaches the Quick Battle, reads its map scene as the Harness's
/// `map_text` and paints `frame-png`'s picture.
#[test]
fn the_session_reads_the_battles_scene_and_paints_its_frame() {
    let mut session = Trpg.session(&[]).unwrap();
    let error = session.scene().unwrap_err();
    assert!(error.contains("`title`"), "{error}");
    let script = Script::new(vec![frame_png::Step::Keys("Down f Left f".into())]);
    session.play(&script);
    let scene = session.scene().unwrap();
    assert_eq!(
        (scene.screen.as_str(), scene.kind.as_str()),
        ("battle", "grid map")
    );
    let mut h = harness(&Start::of(&[]).unwrap()).unwrap();
    script.play(&mut h);
    assert_ne!(scene.text, "");
    assert_eq!(scene.text, h.map_text());
    let units = scene.data["units"].as_array();
    assert!(units.is_some_and(|u| !u.is_empty()), "{}", scene.data);
    let picture = session.picture(Some(1)).unwrap();
    let shot = frame(&Start::of(&[]).unwrap(), &script, Some(1)).unwrap();
    assert_eq!((picture.width, picture.height), (800, 512));
    assert_eq!(picture, shot.image);
}

/// Ticket 1210: `run --session` plays a kept record from the game's real
/// start to its last frame, and reads the battle it ends in.
#[test]
fn a_session_record_plays_from_the_real_start() {
    let path = repo_root().join("crates/ui/tests/it/sessions/quick_battle.session.ron");
    let text = fs::read_to_string(path).unwrap();
    let record = SessionRecord::from_ron(&text).unwrap();
    let mut session = Trpg.session_from(&record, Stamps::Check).unwrap();
    let played = session.play_record().unwrap();
    assert_eq!(played.first_difference, None);
    assert_eq!(played.frames as usize, record.frames.len());
    assert_eq!(session.scene().unwrap().screen, "battle");
}

/// Ticket 1193: `check` (the MCP server's) passes on the game's content,
/// its text checked against the font atlas too (ticket 1243).
#[test]
fn check_passes_on_the_content() {
    let answer = check::check(&Trpg, &check::CheckInput::default());
    let errors = answer.result.as_ref().map(|r| r.errors.clone());
    assert!(answer.ok, "{:?} {errors:?}", answer.error);
    assert_eq!(errors, Some(Vec::new()));
    assert_eq!(answer.limits, Vec::<String>::new());
}

/// Ticket 1193: a broken file is an error of `check`, with its file and
/// line.
#[test]
fn check_finds_a_broken_file_by_file_and_line() {
    static BROKEN: &[(&str, &[u8])] = &[("data/ai.ron", b"(\n  oops\n")];
    let mut layers = vec![trpg_content::bundle::Layer::table("assets", BROKEN)];
    layers.extend(trpg_content::bundle::layers());
    let bundle = Box::leak(Box::new(trpg_content::bundle::Bundle::new(layers)));
    let answer = trpg_content::bundle::reading(bundle, || {
        check::check(&Trpg, &check::CheckInput::default())
    });
    assert!(!answer.ok);
    let errors = answer.result.unwrap().errors;
    let ai = errors.iter().find(|e| e.file == "assets/data/ai.ron");
    assert!(ai.is_some_and(|e| e.line.is_some()), "{errors:?}");
}

/// Ticket 1243: a test pack line with a character the font atlas can't
/// draw fails `check`, naming the character on the pack's file (as the
/// engine names it, from the language's folder).
#[test]
fn check_finds_a_character_the_atlas_cant_draw() {
    const UI: &str = "lang/test/ui.ron";
    let ui = fs::read_to_string(repo_root().join("assets-test").join(UI)).unwrap();
    let snowman = ui.replace("text: \"NEW GAME\"", "text: \"NEW GAME \u{2603}\"");
    assert_ne!(snowman, ui);
    let bytes: &'static [u8] = Box::leak(snowman.into_boxed_str()).as_bytes();
    let files: &'static [(&str, &[u8])] = Box::leak(Box::new([(UI, bytes)]));
    let mut layers = vec![trpg_content::bundle::Layer::table("assets-test", files).test()];
    layers.extend(trpg_content::bundle::layers());
    let bundle = Box::leak(Box::new(trpg_content::bundle::Bundle::new(layers)));
    let answer = trpg_content::bundle::reading(bundle, || {
        check::check(&Trpg, &check::CheckInput::default())
    });
    assert!(!answer.ok);
    let errors = answer.result.unwrap().errors;
    let named = errors
        .iter()
        .any(|e| e.file == "test/ui.ron" && e.message.contains('\u{2603}'));
    assert!(named, "{errors:?}");
}

/// `playtest` as a plain bot run of the story's first battle: `tries`
/// tries from seed 1, no history kept.
fn bot_run(tries: u32) -> playtest::PlaytestOutput {
    let content = trpg_content::load_embedded().unwrap();
    let chapter = &content.chapters[&content.new_game.first_chapter];
    let mut input = playtest::PlaytestInput::default();
    input.stage = Some(chapter.battle.clone());
    input.tries = Some(tries);
    input.seed = Some(1);
    input.no_history = true;
    let answer = playtest::playtest(&Trpg, &input);
    assert!(answer.ok, "{:?}", answer.error);
    answer.result.unwrap().remove(0)
}

/// Ticket 1193: the bot run plays as many games as asked, each with a
/// record (since engine 2544, as `playtest` with no checks).
#[test]
fn bots_play_three_games_with_records() {
    let output = bot_run(3);
    let [baseline] = &output.entrants[..] else {
        panic!("one entrant: {:?}", output.entrants)
    };
    assert_eq!(baseline.outcomes.values().sum::<u32>(), 3);
    let seeds: Vec<u64> = baseline.tries.iter().map(|t| t.seed).collect();
    assert_eq!(seeds, [1, 2, 3]);
    assert!(baseline.tries.iter().all(|t| t.record.is_some()));
    assert!(
        baseline.measures.contains_key("won"),
        "{:?}",
        baseline.measures
    );
    assert_eq!(
        output.report.checks.len(),
        0,
        "no checks: {:?}",
        output.report.checks
    );
}

/// Ticket 1193: a record the bots wrote replays to the same hashes twice,
/// and to the play's own end.
#[test]
fn a_bots_record_replays_the_same_twice() {
    let output = bot_run(1);
    let kept = output.entrants[0].tries[0].clone();
    let tried = output.report.entrants[0].tries[0].clone();
    let dir = std::env::temp_dir().join(format!("xtask-replay-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("play.ron"), kept.record.unwrap()).unwrap();
    let answer = replay::replay(&Trpg, &dir, &replay::ReplayInput::new("play.ron"));
    let _ = fs::remove_dir_all(&dir);
    assert!(answer.ok, "{:?} {:?}", answer.error, answer.limits);
    let output = answer.result.unwrap();
    let [first, second] = &output.runs[..] else {
        panic!("two runs: {output:?}")
    };
    assert_eq!(first, second);
    assert_eq!(
        first.steps.len(),
        usize::try_from(kept.commands).unwrap() + 1
    );
    let refused = first.steps.iter().filter(|s| s.refused.is_some()).count();
    assert_eq!(refused, 0);
    assert_eq!(first.outcome, tried.outcome);
}

/// Acceptance (1245): the battle record a crash report holds replays from
/// the battle's first state to where it crashed: the commands applied, then
/// the one the rules were applying.
#[test]
fn a_crash_record_replays_to_the_crash() {
    use trpg_ui::crash::{BattleRecord, crash_record};
    use trpg_ui::harness::Harness;

    let mut h = Harness::with_layout(trpg_ui::input::RIGHT_HANDED);
    // Quick Battle, `Fight!`, the banner closed; the lord moves one tile
    // right and waits.
    h.keys("Down f Left f f");
    h.keys("f Right f").wait(1.0).keys("f").wait(1.0);
    let text = crash_record(h.game()).unwrap();
    let mut record: BattleRecord = ron::from_str(&text).unwrap();
    let applied = record.history.len();
    let dir = std::env::temp_dir().join(format!("xtask-crash-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("crash.ron"), &text).unwrap();
    // As if the rules hadn't returned from ending the phase.
    record.applying = Some(trpg_core::Command::EndPhase);
    fs::write(dir.join("applying.ron"), ron::to_string(&record).unwrap()).unwrap();
    let crashed = replay::replay(&Trpg, &dir, &replay::ReplayInput::new("crash.ron"));
    let applying = replay::replay(&Trpg, &dir, &replay::ReplayInput::new("applying.ron"));
    let _ = fs::remove_dir_all(&dir);
    assert!(crashed.ok, "{:?} {:?}", crashed.error, crashed.limits);
    let run = &crashed.result.unwrap().runs[0];
    assert_eq!(run.steps.len(), applied + 1);
    assert!(run.steps.iter().all(|s| s.refused.is_none()), "{run:?}");
    let last = run.steps.last().map(|s| s.hash.clone());
    assert_eq!(last, h.game().state_hash().map(|hash| hash.to_string()));
    assert!(applying.ok, "{:?}", applying.error);
    let run = &applying.result.unwrap().runs[0];
    assert_eq!(run.steps.len(), applied + 2);
    assert!(run.steps.iter().all(|s| s.refused.is_none()), "{run:?}");
}

/// Ticket 1193: a record that isn't the game's is an error naming it.
#[test]
fn replay_refuses_a_record_that_isnt_one() {
    let record = Record {
        file: "x.ron".to_owned(),
        text: "(version: 9, battle: \"ch01\", mode: Classic, seed: 0, commands: [])".to_owned(),
    };
    let error = Trpg.replay(&record).unwrap_err();
    assert!(error.starts_with("x.ron: a record of version 9"), "{error}");
}

/// Ticket 1200: the game's copy of the Playdate's profile says what the
/// engine's says, at the pin.
#[test]
fn the_playdates_facts_are_the_engines() {
    let facts = trpg_ui::platform::playdate_facts().unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(facts, engine_facts("playdate"));
}

/// The facts of the engine's profile `id`.
fn engine_facts(id: &str) -> trpg_ui::PlatformFacts {
    let profiles = target::engine_profiles().unwrap_or_else(|e| panic!("{e:?}"));
    trpg_ui::PlatformFacts::from_profile(&profiles[id].profile)
}

/// Ticket 1200: each target's title and Options, by the engine's profiles:
/// a Quit only on the desktop, a Fullscreen row except on the Playdate.
#[test]
fn the_title_and_options_follow_the_targets_facts() {
    use trpg_ui::screens::TitleScreen;
    use trpg_ui::screens::options::{Row, shown_rows};
    for (id, quit, fullscreen) in [
        ("desktop", true, true),
        ("web", false, true),
        ("playdate", false, false),
    ] {
        let facts = engine_facts(id);
        let h = Harness::with_layout_and(RIGHT_HANDED, |c| c.platform = facts);
        let ctx = h.game().ctx();
        let view = h
            .game()
            .screen::<TitleScreen>()
            .unwrap_or_else(|| panic!("no title screen"))
            .view(ctx);
        let menu = view.menu.unwrap_or_else(|| panic!("no menu"));
        let has_quit = menu.items.iter().any(|i| ctx.label(&i.label) == "Quit");
        assert_eq!(has_quit, quit, "{id}: Quit");
        let has_row = shown_rows(ctx).contains(&Row::Fullscreen);
        assert_eq!(has_row, fullscreen, "{id}: Fullscreen");
    }
}

/// What a test reads of a `bench/*.bench.ron` scenario: how it starts and
/// what it plays (the engine's own reader is the engine's).
#[derive(serde::Deserialize)]
struct BenchScenario {
    #[serde(default)]
    start: Vec<GameArg>,
    #[serde(default)]
    script: frame_png::Script,
    #[serde(default)]
    frames: u32,
}

/// `scenario` of `bench/` played on the colour targets' session or, with
/// `playdate`, the Playdate's, as `cargo shuyi bench` plays it.
fn bench_played(scenario: &str, playdate: bool) -> Box<dyn Session> {
    let path = crate::repo_root().join(format!("bench/{scenario}.bench.ron"));
    let text = fs::read_to_string(&path).unwrap();
    // As the engine reads a scenario: a value's `Some` is implied.
    let options =
        ron::Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);
    let bench: BenchScenario = options
        .from_str(&text)
        .unwrap_or_else(|e| panic!("{scenario}: {e}"));
    let mut session = if playdate {
        Trpg.session_on("playdate", &bench.start).unwrap()
    } else {
        Trpg.session(&bench.start).unwrap()
    };
    session.play(&bench.script);
    session.idle(bench.frames);
    session
}

/// The battle's units in `session`, as the scene's data gives them.
fn bench_units(session: &dyn Session) -> Vec<serde_json::Value> {
    let scene = session.scene().unwrap();
    scene.data["units"].as_array().unwrap().clone()
}

/// The unit playing `character`.
fn unit_of<'a>(units: &'a [serde_json::Value], character: &str) -> &'a serde_json::Value {
    let found = units.iter().find(|u| u["character"] == character);
    found.unwrap_or_else(|| panic!("no {character}"))
}

/// Ticket 1244: each scenario plays what it says, on the colour targets
/// and on the Playdate, its actions pressed by its keys or its buttons.
#[test]
fn the_bench_scenarios_play_what_they_say() {
    for playdate in [false, true] {
        let on = if playdate { "playdate" } else { "desktop" };
        let session = bench_played("battle-idle", playdate);
        assert_eq!(bench_units(&*session).len(), 7 + 15, "{on}");
        // The attack: the rider on the bridge, having fought the swordsman.
        let session = bench_played("battle-attack", playdate);
        let units = bench_units(&*session);
        let rider = unit_of(&units, "sergeant");
        assert_eq!(rider["pos"], serde_json::json!({"x": 14, "y": 10}), "{on}");
        assert_eq!(rider["acted"], true, "{on}");
        assert!(
            rider["weapon_exp"]["Spear"].as_u64() > Some(0),
            "{on}: {rider}"
        );
        // The enemy's turn, still going when it ends, five of them moved.
        let start = bench_units(&*bench_played("battle-idle", playdate));
        let session = bench_played("battle-enemy-turn", playdate);
        let scene = session.scene().unwrap();
        assert_eq!(scene.data["phase"], "Enemy", "{on}");
        let moved = bench_units(&*session)
            .iter()
            .filter(|u| u["faction"] == "Enemy" && !start.contains(u))
            .count();
        assert!(moved >= 5, "{on}: {moved} moved");
        // The cursor swept the map round to the lead's row, a tile left
        // of the lead: the right edge stopped it a tile early.
        let session = bench_played("battle-cursor-sweep", playdate);
        assert!(
            session.scene().unwrap().text.contains("cursor (9,12)"),
            "{on}"
        );
        let options = bench_played("options-menu", playdate);
        assert_eq!(
            options.screens().last().map(String::as_str),
            Some("options"),
            "{on}"
        );
        let dialogue = bench_played("dialogue-portrait", playdate);
        assert_eq!(
            dialogue.screens().last().map(String::as_str),
            Some("dialogue"),
            "{on}"
        );
    }
}
