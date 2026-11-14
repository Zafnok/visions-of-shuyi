//! `story-preview [folder]`: the writer's script preview (ticket 0764,
//! [`trpg_ui::story_preview`]). It plays the scenes of the `.dlg` files in
//! a folder in the game's own dialogue screen and reads a file again when
//! it is saved. The folder is the one named on the command line (a folder
//! dragged onto the program, or the folder of a file dragged onto it),
//! else `dialogue/` beside the program, as in a clone of the writer's
//! mirror; with neither it starts on no scenes and says how to start it.
//!
//! The kit's build embeds none of the game's scripts (`shuyi.ron`'s
//! `story-preview` edition): it holds only the scenes of its folder.

// A release build on Windows is a GUI program: no console window.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[path = "../app_info.rs"]
mod app_info;

#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};

use macroquad::window::Conf;
use shuyi::assets::bundle::Layer;
use shuyi_app::storage::DesktopFolder;
use shuyi_app::{AppConfig, Window};
use trpg_content::{bundle, font};

use crate::app_info::APP;

// As the game's (`main.rs`): a build that ships has none of the test files.
#[cfg(not(any(debug_assertions, feature = "debug-tools")))]
const _: () = assert!(
    !trpg_content::bundle::TEST_FILES,
    "this release build would embed the test files (assets-test/): build the program \
     alone, with `-p trpg-app` (ADR-0057)"
);

/// The game's window.
const WINDOW: Window = Window::new(APP.name, 1640, 1064).with_icon(APP.window_icon);

/// The folder of scripts a writer's clone of the mirror has at its root.
const DIALOGUE: &str = "dialogue";

/// What a wrong command line is answered with.
#[cfg(not(target_arch = "wasm32"))]
const USAGE: &str = "usage: story-preview [folder]\n\
    plays the .dlg files of the folder (or of the folder of a file) named, else of \
    dialogue/ beside the program";

/// The folder the command line `args` (without the program's name) names,
/// with `beside` the program's own folder: a folder named, the folder of a
/// file named, else `dialogue/` beside the program if it is there. `None`
/// when there is none.
#[cfg(not(target_arch = "wasm32"))]
fn folder_of(args: &[String], beside: Option<&Path>) -> Result<Option<PathBuf>, String> {
    match args {
        [] => Ok(beside
            .map(|dir| dir.join(DIALOGUE))
            .filter(|dir| dir.is_dir())),
        [arg] if arg.starts_with('-') => Err(USAGE.to_owned()),
        [path] => {
            let path = PathBuf::from(path);
            if path.is_file() {
                Ok(path.parent().map(Path::to_path_buf))
            } else {
                Ok(Some(path))
            }
        }
        _ => Err(USAGE.to_owned()),
    }
}

/// The folder this program reads, watched in every build, or why it
/// can't be read: an empty one when the command line names none and none
/// is beside the program.
#[cfg(not(target_arch = "wasm32"))]
fn folder() -> Result<Layer, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let exe = std::env::current_exe().ok();
    let beside = exe.as_deref().and_then(Path::parent);
    let limit = u64::try_from(shuyi::assets::format::DEFAULT_MAX_BYTES).unwrap_or(u64::MAX);
    match folder_of(&args, beside)? {
        Some(path) => shuyi_app::read_watched_folder(&path, DIALOGUE, limit)
            .map_err(|e| format!("{e}\n{USAGE}")),
        None => no_folder(),
    }
}

/// On the web, which has no folders (dropped files are 0767's): none.
#[cfg(target_arch = "wasm32")]
fn folder() -> Result<Layer, String> {
    no_folder()
}

/// An empty folder: the preview starts on no scenes and says how to start
/// it.
fn no_folder() -> Result<Layer, String> {
    Layer::owned(DIALOGUE.to_owned(), std::collections::BTreeMap::new()).map_err(|e| e.to_string())
}

/// What the platform shell needs to know: the game's window, files, font,
/// music and settings (the key layout chosen in the game is used here).
fn config() -> AppConfig {
    let mut config = AppConfig::new(
        WINDOW,
        APP.storage,
        bundle::bytes,
        font::ATLAS_PNG_PATH,
        trpg_content::audio::files(None),
    );
    config.desktop_folder = DesktopFolder::MyGames("Visions of Shuyi");
    config
}

fn window_conf() -> Conf {
    shuyi_app::window_conf(&WINDOW)
}

/// The folder, once a packed build has its files: the pack beside the
/// program, laid in the bundle before anything reads a file, as the game's
/// (`main.rs`, ADR-0120). A pack that is missing or broken is the start's
/// error, shown in the window.
fn startup() -> Result<Layer, String> {
    #[cfg(feature = "packed")]
    {
        let pack = shuyi_app::pack_beside_program(env!("CARGO_PKG_NAME"), bundle::PACK_STAMP)
            .map_err(|e| e.to_string())?;
        bundle::use_pack(pack.into_layer(bundle::DISPLAY_ROOT));
    }
    folder()
}

#[macroquad::main(window_conf)]
async fn main() {
    let start = trpg_ui::story_preview::start(WINDOW.title, startup());
    shuyi_app::run(config(), start).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(args: &[&str]) -> Vec<String> {
        args.iter().map(|&a| a.to_owned()).collect()
    }

    #[test]
    fn the_folder_named_is_read_else_dialogue_beside_the_program() {
        let dir = tempfile::tempdir().unwrap();
        let scripts = dir.path().join(DIALOGUE);
        std::fs::create_dir(&scripts).unwrap();
        let file = scripts.join("ch01.dlg");
        std::fs::write(&file, "").unwrap();
        let named = scripts.to_string_lossy().into_owned();
        assert_eq!(folder_of(&args(&[&named]), None), Ok(Some(scripts.clone())));
        // A file dragged onto the program: its folder.
        let dragged = file.to_string_lossy().into_owned();
        assert_eq!(
            folder_of(&args(&[&dragged]), None),
            Ok(Some(scripts.clone()))
        );
        // Nothing named: dialogue/ beside the program, if it is there.
        assert_eq!(folder_of(&[], Some(dir.path())), Ok(Some(scripts)));
        assert_eq!(folder_of(&[], Some(&dir.path().join("x"))), Ok(None));
        assert_eq!(folder_of(&[], None), Ok(None));
    }

    #[test]
    fn a_wrong_command_line_says_how_to_start_it() {
        assert_eq!(folder_of(&args(&["--help"]), None), Err(USAGE.to_owned()));
        assert_eq!(folder_of(&args(&["a", "b"]), None), Err(USAGE.to_owned()));
        assert!(USAGE.starts_with("usage: story-preview [folder]"));
    }
}
