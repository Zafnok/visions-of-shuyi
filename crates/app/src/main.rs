//! The game's binary: macroquad's entry point, which must be here, and what
//! the engine's platform shell (`shuyi-app`) needs to know about the game.
//! See ADR-0004.

// A release build on Windows is a GUI program: no console window (ticket 0902).
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod args;
#[cfg(test)]
mod tests;

use macroquad::window::Conf;
use shuyi_app::{AppConfig, Icon, Platform, VoiceFiles, Window};
use trpg_content::{FontAtlasDef, ImageTable, bundle, font, voice};
use trpg_ui::screen::{Lessons, Shuyi};
use trpg_ui::{ConsoleSizes, ConsoleSpec, Ctx, Game, ShuyiCtx};

use crate::args::Startup;

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
/// to 1x. The icon is from `assets/icon/` (ticket 0902, `cargo xtask icon`).
const WINDOW: Window = Window {
    title: "Visions of Shuyi",
    width: 1640,
    height: 1064,
    icon: Some(Icon {
        small: include_bytes!("../../../assets/icon/icon-16.png"),
        medium: include_bytes!("../../../assets/icon/icon-32.png"),
        big: include_bytes!("../../../assets/icon/icon-64.png"),
    }),
};

/// Where saves and settings are kept: never change it, or every player's
/// saves are lost (`%APPDATA%/tactical-rpg`, `localStorage` keys under
/// `tactical-rpg/`).
const STORAGE_NAME: &str = "tactical-rpg";

/// What the platform shell needs to know about the game.
fn config() -> AppConfig<Shuyi> {
    AppConfig {
        window: WINDOW,
        storage_name: STORAGE_NAME,
        files: bundle::bytes,
        font: font_atlas,
        font_png: font::ATLAS_PNG_PATH,
        images,
        sounds: trpg_content::audio::files(None),
        voices: Some(VoiceFiles {
            folder: voice::VOICE_DIR,
            lang: voice::SOURCE_LANG,
            manifest: voice::MANIFEST_FILE,
        }),
    }
}

fn font_atlas(game: &Shuyi) -> &FontAtlasDef {
    &game.content.font
}

fn images(game: &Shuyi) -> &ImageTable {
    &game.content.images
}

/// The game on `platform`: at the title, or on the scene the command line
/// asks for (ticket 0723).
fn start(platform: &Platform) -> Result<Game, String> {
    let startup = args::startup()?;
    let mut ctx = Ctx::embedded().map_err(|e| e.to_string())?;
    ctx.console = ConsoleSpec::PC;
    ctx.game.tips_enabled = true;
    ctx.game.lessons = Lessons::On;
    let ctx = platform.setup(ctx);
    match startup {
        Startup::Title => Ok(trpg_ui::game::start(ctx)),
        Startup::Scene { id, gender } => trpg_ui::game::start_on_scene(ctx, &id, gender),
    }
}

fn window_conf() -> Conf {
    shuyi_app::window_conf(&WINDOW)
}

#[macroquad::main(window_conf)]
async fn main() {
    shuyi_app::run(config(), start).await;
}
