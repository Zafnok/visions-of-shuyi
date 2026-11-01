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
use shuyi_app::{AppConfig, Icon, VoiceFiles, Window};
use shuyi_platform::{Card, Look, Setup, Start};
use trpg_content::{bundle, font, voice};
use trpg_ui::screen::{Lessons, Shuyi};
use trpg_ui::{ConsoleSizes, ConsoleSpec, Ctx, Game, ShuyiCtx};

use crate::app_info::APP;
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
/// to 1x. Its title is `shuyi.ron`'s `app` (ticket 1199). Its icon is drawn
/// per size in `assets/icon/` (ticket 0902, `cargo xtask icon`), since the
/// engine's downscale from one picture blurs the glyph at 16 and 32 px.
// engine-friction: engine#3113
const WINDOW: Window = Window::new(APP.name, 1640, 1064).with_icon(Some(Icon::new(
    include_bytes!("../../../assets/icon/icon-16.png"),
    include_bytes!("../../../assets/icon/icon-32.png"),
    include_bytes!("../../../assets/icon/icon-64.png"),
)));

/// Where saves and settings are kept: never change it, or every player's
/// saves are lost (`%APPDATA%/tactical-rpg`, `localStorage` keys under
/// `tactical-rpg/`).
const STORAGE_NAME: &str = APP.storage;

/// What the platform shell needs to know about the game.
fn config() -> AppConfig {
    let mut config = AppConfig::new(
        WINDOW,
        STORAGE_NAME,
        bundle::bytes,
        font::ATLAS_PNG_PATH,
        trpg_content::audio::files(None),
    );
    config.voices = Some(VoiceFiles::new(
        voice::VOICE_DIR,
        voice::SOURCE_LANG,
        voice::MANIFEST_FILE,
    ));
    config
}

/// What the game's frames are painted with: its font atlas and pictures.
fn look(game: &Shuyi) -> Look<'_> {
    Look::new(&game.content.font, &game.content.images)
}

/// The game on the platform `setup` describes: at the title, or on the
/// scene the command line asks for (ticket 0723).
fn start(setup: Setup) -> Result<Game, String> {
    let startup = args::startup()?;
    let mut ctx = Ctx::embedded().map_err(|e| e.to_string())?;
    ctx.console = ConsoleSpec::PC;
    ctx.game.tips_enabled = true;
    ctx.game.lessons = Lessons::On;
    let ctx = setup.ctx(ctx);
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
    // The game's name in white on black while it loads, as on the
    // Playdate (ticket 1036).
    let start = Start::new(Card::text(WINDOW.title), look, start);
    shuyi_app::run(config(), start).await;
}
