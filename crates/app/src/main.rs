//! The game's binary: macroquad's entry point, which must be here, and what
//! the engine's platform shell (`shuyi-app`) needs to know about the game.
//! See ADR-0004.

// A release build on Windows is a GUI program: no console window (ticket 0902).
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app_info;
mod args;
#[cfg(test)]
mod tests;

use macroquad::window::Conf;
use shuyi_app::storage::DesktopFolder;
use shuyi_app::{AppConfig, CrashReports, VoiceFiles, Window};
use trpg_content::{bundle, font, voice};

use crate::app_info::APP;

// A build that ships has none of the test files (ADR-0057): only a debug
// build and a `debug-tools` build embed them. Features are shared across a
// build, so a release build of the whole workspace (or of this crate's
// tests) would bring them in through `xtask` or the test Harness.
#[cfg(not(any(debug_assertions, feature = "debug-tools")))]
const _: () = assert!(
    !trpg_content::bundle::TEST_FILES,
    "this release build would embed the test files (assets-test/): build the game \
     alone, with `-p trpg-app` (ADR-0057)"
);

/// The window: the console at 2x (1600x1024) plus a small margin, since the
/// framebuffer can come out a pixel smaller than asked, which would drop
/// to 1x. Its title and icon are `shuyi.ron`'s `app` (tickets 1199, 1201).
const WINDOW: Window = Window::new(APP.name, 1640, 1064).with_icon(APP.window_icon);

/// The storage's name: never change it, or every player's saves are lost
/// (`localStorage` keys under `tactical-rpg/` on the web, the folder
/// `tactical-rpg` in the OS's data folder on macOS and Linux, and the
/// Windows folder the saves move out of, `%APPDATA%/tactical-rpg`).
const STORAGE_NAME: &str = APP.storage;

/// Where a desktop keeps saves and settings: `Documents\My Games\Visions of
/// Shuyi` on Windows, the old `%APPDATA%` folder moved there on the first
/// launch (ticket 0875, ADR-0116). A literal, not the window's title: a
/// rename of the game must not move the saves again.
const DESKTOP_FOLDER: DesktopFolder = DesktopFolder::MyGames("Visions of Shuyi");

/// What the platform shell needs to know about the game.
fn config() -> AppConfig {
    let mut config = AppConfig::new(
        WINDOW,
        STORAGE_NAME,
        bundle::bytes,
        font::ATLAS_PNG_PATH,
        trpg_content::audio::files(None),
    );
    config.desktop_folder = DESKTOP_FOLDER;
    config.voices = Some(VoiceFiles::new(
        voice::VOICE_DIR,
        voice::SOURCE_LANG,
        voice::MANIFEST_FILE,
    ));
    // Each crash report names the game's version and holds the open
    // battle's record, in a release build too (ticket 1245; engine #331
    // keeps it to debug builds unless asked).
    config.crash = CrashReports::new(APP.version).with_record_in_release();
    config
}

fn window_conf() -> Conf {
    shuyi_app::window_conf(&WINDOW)
}

/// Where the game starts (the command line's), once a packed build has
/// its files: the pack beside the program, laid in the bundle before
/// anything reads a file (ADR-0120). A pack that is missing, of another
/// version or broken is the start's error, which the game shows in place
/// of its title, saying to bake it again.
fn startup() -> Result<args::Startup, String> {
    #[cfg(feature = "packed")]
    {
        let pack = shuyi_app::pack_beside_program(env!("CARGO_PKG_NAME"), bundle::PACK_STAMP)
            .map_err(|e| e.to_string())?;
        bundle::use_pack(pack.into_layer(bundle::DISPLAY_ROOT));
    }
    args::startup()
}

#[macroquad::main(window_conf)]
async fn main() {
    // The game's name in white on black while it loads, as on the
    // Playdate (ticket 1036).
    let start = trpg_ui::start::start(WINDOW.title, startup());
    shuyi_app::run(config(), start).await;
}
