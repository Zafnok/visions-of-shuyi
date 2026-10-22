//! Shuyi's tools on this repository (ticket 1117): the game's `frame-png`
//! and `lang-status`, the checks on the real code, and the committed font
//! atlas.

use std::fs;
use std::path::{Path, PathBuf};

use shuyi_assets::font::FontAtlasDef;
use shuyi_cli::Project;
use shuyi_cli::codec::decode_png;
use shuyi_cli::font_atlas::{ATLAS_RANGES, ONE_BIT_FILE, read_font};
use shuyi_cli::frame_png::{self, Image, Options};
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
    let mut options = parse(&["camera.png", "--keys", "F2 Up Up f f"]).unwrap();
    options.scale = Some(1);
    let printed = run(&dir, &options).unwrap();
    assert!(printed.contains("scene_camera"), "{printed}");
    let frame = Image::decode(&fs::read(dir.join("camera.png")).unwrap()).unwrap();
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

/// Ticket 1039: `--crank` winds the Playdate's crank after the steps and
/// `--then-pad` presses buttons after it, in turn: wound back on the
/// Quick Battle's map, the rewind screen; A asks there.
#[test]
fn the_crank_and_buttons_after_the_steps_open_the_rewind_screen() {
    let given = parse(&["r.png", "--crank", "-30", "--then-pad", "South"]).unwrap();
    let after = Start::of(&given.game).unwrap().after;
    assert_eq!(after, [After::Crank(-30.0), After::Pad("South".into())]);
    let bad = parse(&["r.png", "--crank", "far"]).unwrap();
    let error = Start::of(&bad.game).unwrap_err();
    assert_eq!(error, "--crank: not a number of degrees: far");
    let dir = temp_dir("crank");
    let quick = [
        "r.png",
        "--playdate",
        "--pad",
        "DpadDown South",
        "--wait",
        "0.1",
        "--pad",
        "DpadLeft South",
        "--wait",
        "0.1",
        "--pad",
        "South",
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
    let report = lang_status::report(&lang.lang, &lang.variants, "test", &options).unwrap();
    let first = report.lines().next().unwrap_or_default();
    assert!(
        first.starts_with("lang-status test (TEST, machine-made): "),
        "{first}"
    );
    assert!(first.ends_with(" missing, 2 stale, 1 orphaned"), "{first}");
    assert!(report.contains("missing:\n  title.credits\n"), "{report}");
    // A data name and a dialogue line nobody translated.
    assert!(
        report.contains("  items.iron_spear.name\n    english: \"Iron Spear\"\n"),
        "{report}"
    );
    assert!(
        report.contains("  test_36a692b7\n    english: \"Let's move.\"\n"),
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
        lang_status::report(&lang.lang, &lang.variants, "test", options).unwrap()
    };
    let dialogue = report(&lang_status::Options {
        only: Some(Part::Dialogue),
        file: None,
    });
    assert!(!dialogue.contains("left out"), "{dialogue}");
    assert!(
        dialogue.contains("  test_36a692b7\n    english:"),
        "{dialogue}"
    );
    // The test pack lists the test scene, and only the file asked for.
    let test = report(&lang_status::Options {
        only: None,
        file: Some("test".into()),
    });
    assert!(test.contains("  test_36a692b7\n    english:"), "{test}");
    assert!(!test.contains("screen text"), "{test}");
    let lines = &lang.lang.source().lines;
    for (key, line) in lines.iter().filter(|(_, line)| line.file != "test") {
        assert!(
            !test.contains(&format!("  {key}\n    english:")),
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

/// The font sources the committed atlas is built from, relative to the
/// repo root: Terminus, then our own extra glyphs.
const SOURCES: [&str; 2] = [
    "assets-src/fonts/ter-u16n.bdf",
    "assets-src/fonts/pad-shapes.bdf",
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

/// The committed atlas must be exactly what the tool produces from the
/// committed font source.
#[test]
fn committed_atlas_is_up_to_date() {
    let project = project();
    let root = &project.root;
    let rules = project.config.font_atlas.as_ref().unwrap();
    let dir = std::env::temp_dir().join(format!("xtask-atlas-check-{}", std::process::id()));
    let fonts = SOURCES.map(|source| root.join(source));
    let msg = shuyi_cli::font_atlas::run(rules, project.config.command(), &fonts, &dir).unwrap();
    assert!(msg.starts_with("font-atlas: "));
    let read = |p: &Path| fs::read_to_string(p).unwrap().replace("\r\n", "\n");
    let rerun = "rerun `cargo shuyi font-atlas assets-src/fonts/ter-u16n.bdf \
                 assets-src/fonts/pad-shapes.bdf assets/fonts`";
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
