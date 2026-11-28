//! How the game starts on a device (engine ADR-0072, ticket 1210): its
//! loading card, its load and its look. The binary runs it, and so do the
//! tools (`cargo shuyi run --session`) and the tests that play a kept
//! session record, so a record a debug build made plays back from the same
//! start. Each frame's hash holds the battle's rules state
//! ([`BattleScreen`](crate::screens::BattleScreen)'s `state_hash`), and a
//! crash report holds the open battle's record ([`crash_record`]); the start
//! after a crash says so on the title (ticket 1245).

use crate::prelude::*;
use shuyi_platform::Progress;
use shuyi_platform::prelude::{Card, Look, Setup, Start};
use trpg_core::LeadGender;

use crate::console::{ConsoleSizes, ConsoleSpec};
use crate::crash::crash_record;
use crate::game::{self, Game};
use crate::hot::HotContent;
use crate::screen::{Ctx, Lessons, Shuyi};
use crate::screens::TitleScreen;

/// Where the game starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Startup {
    /// At the title screen.
    Title,
    /// On a dialogue scene, with the default lead of this gender.
    Scene {
        /// The scene's id.
        id: String,
        /// The lead's gender.
        gender: LeadGender,
    },
}

/// The game as a device starts it: `title` on its loading card, then the
/// load, on the scene `startup` asks for (the command line's; an error is
/// the load's).
pub fn start(title: &'static str, startup: Result<Startup, String>) -> Start<Shuyi> {
    when_files_arrived(Card::text(title), look, move |setup| load(setup, startup))
        .state_hash(Game::state_hash)
        .crash_record(crash_record)
        .on_system(crate::save_files::hear)
}

/// What the game's frames are painted with: its font atlas and pictures.
fn look(game: &Shuyi) -> Look<'_> {
    Look::new(&game.content.font, &game.content.images)
}

/// The game on the platform `setup` describes: at the title, which says so
/// if the game crashed last time, or on the scene `startup` asks for
/// (ticket 0723).
fn load(setup: Setup, startup: Result<Startup, String>) -> Result<Game, String> {
    let startup = startup?;
    let crashed = setup.last_crash.is_some();
    let mut ctx = Ctx::embedded().map_err(|e| e.to_string())?;
    ctx.console = ConsoleSpec::PC;
    ctx.game.tips_enabled = true;
    ctx.game.lessons = Lessons::On;
    let ctx = setup.ctx(ctx);
    let game = match startup {
        Startup::Title => {
            let mut game = game::start(ctx);
            // Once, under the menu (`docs/design/crash-reports.md`).
            if crashed {
                let notice = game.ctx().text("title.crashed").to_owned();
                if let Some(title) = game.screen_mut::<TitleScreen>() {
                    title.show_notice(notice);
                }
            }
            game
        }
        Startup::Scene { id, gender } => game::start_on_scene(ctx, &id, gender)?,
    };
    // Its palettes and scenes reload as their files change (ticket 1209).
    HotContent::install(game)
}

/// A start that shows `card` and runs `load` once every file of the game's
/// bundle can be read ([`trpg_content::bundle::arrived`]): at once in a
/// build that embeds its files or reads its pack whole; on the web, a frame
/// at a time behind the card while the page streams the pack in (engine
/// #509), so the load reads every file as it did when the page fetched the
/// whole pack before the game started. The first load reads every folder
/// and the page has the pack long before its `.wasm` (ticket 1298 measured
/// it), so the start waits for the whole pack, not for folders laid first.
pub(crate) fn when_files_arrived(
    card: Card,
    look: fn(&Shuyi) -> Look<'_>,
    load: impl FnOnce(Setup) -> Result<Game, String> + 'static,
) -> Start<Shuyi> {
    Start::in_steps(card, look, move |setup| {
        let mut load = Some((load, setup));
        move || {
            if !trpg_content::bundle::arrived() {
                return Progress::Wait;
            }
            // The platform calls no step after `Done`, so it's taken once.
            let Some((load, setup)) = load.take() else {
                return Progress::Wait;
            };
            Progress::done(load(setup))
        }
    })
}
