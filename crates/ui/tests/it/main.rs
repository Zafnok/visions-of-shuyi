//! `trpg-ui`'s one integration-test program (ticket 0114). Cargo links every
//! file directly under `tests/` as its own program, so new integration tests
//! go here as a module: add `tests/it/<name>.rs` and a `mod <name>;` line
//! below. Never add a file directly under `tests/` (an `xtask` test fails).
//! Run one module with `cargo test -p trpg-ui --test it <name>::`.

/// `insta::assert_snapshot!` of `value` against its copy in the private
/// repository's `content/tests/snapshots/` (ADR-0118 §3), for a picture
/// test that must check the real content's look, named or not: `$dir` is
/// `trpg_content::bundle::content_tests()`'s folder. The file is named as
/// insta names it here (`it__<module>__<test>.snap`).
macro_rules! assert_private_snapshot {
    ($dir:expr, $value:expr) => {{
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path($dir.join("snapshots"));
        settings.bind(|| insta::assert_snapshot!($value));
    }};
    ($dir:expr, $name:expr, $value:expr) => {{
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path($dir.join("snapshots"));
        settings.bind(|| insta::assert_snapshot!($name, $value));
    }};
}

mod auto_advance;
mod battle;
mod ch01_lessons;
mod ch01_tips;
mod ch01_winnable;
mod ch02_winnable;
mod ch03_winnable;
mod ch04_winnable;
mod ch05_winnable;
mod cinematic;
mod class_change;
mod controller;
mod crash_reports;
mod credits;
mod dialogue;
mod dialogue_log;
mod flow;
mod focus;
mod guides;
mod key_bindings;
mod key_bindings_screen;
mod language;
mod layout_picker;
mod lessons;
mod map_skin;
mod map_zoom;
mod openings;
mod options;
mod places;
mod playdate;
mod playthrough;
mod preparations;
mod profiles;
mod rebind_buttons;
mod save;
mod save_files;
mod save_import;
mod scene_camera;
mod scene_preview;
mod sessions;
mod side_quests;
mod skirmishes;
mod split_keys;
mod sprite_test;
mod threat;
mod title;
mod title_cinematic;
mod voice;
mod world_map;
