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

/// The sound cues `h` played once since its audio was last cleared, in
/// order: [`Harness::sounds`](trpg_ui::harness::Harness) without the
/// sounds started to loop, such as the title's campfire (ticket 0922).
fn one_shots(h: &trpg_ui::harness::Harness) -> Vec<String> {
    h.audio_requests()
        .into_iter()
        .filter_map(|r| match r {
            trpg_ui::audio::AudioRequest::PlaySound { cue, .. } => Some(cue),
            _ => None,
        })
        .collect()
}

/// Whether `r` asks for music (a track or a stop), not a sound.
fn is_music(r: &trpg_ui::audio::AudioRequest) -> bool {
    use trpg_ui::audio::AudioRequest;
    !matches!(
        r,
        AudioRequest::PlaySound { .. }
            | AudioRequest::StartSound { .. }
            | AudioRequest::StopSound { .. }
            | AudioRequest::SoundVolume { .. }
    )
}

/// `line` with each run of three or more spaces read as the separator
/// `·`: the bar under the battle map lines its hints up in columns with
/// spaces (ticket 0445), and a test names them in order.
fn unpadded(line: &str) -> String {
    let parts = line.split("   ").map(str::trim).filter(|p| !p.is_empty());
    parts.collect::<Vec<_>>().join(" · ")
}

mod ambience;
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
mod debug_view;
mod demo;
mod dialogue;
mod dialogue_log;
mod flow;
mod focus;
mod guides;
mod hot_reload;
mod key_bindings;
mod key_bindings_screen;
mod key_caps;
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
mod sound_cues;
mod split_keys;
mod sprite_test;
mod story_preview;
mod streamed_pack;
mod threat;
mod title;
mod title_cinematic;
mod voice;
mod world_map;
