//! How the game starts on a device (engine ADR-0072, ticket 1210): its
//! loading card, its load and its look. The binary runs it, and so do the
//! tools (`cargo shuyi run --session`) and the tests that play a kept
//! session record, so a record a debug build made plays back from the same
//! start. Each frame's hash holds the battle's rules state
//! ([`BattleScreen`](crate::screens::BattleScreen)'s `state_hash`).

use crate::prelude::*;
use shuyi_platform::{Card, Look, Setup, Start};
use trpg_core::LeadGender;

use crate::console::{ConsoleSizes, ConsoleSpec};
use crate::game::{self, Game};
use crate::screen::{Ctx, Lessons, Shuyi};

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
    Start::new(Card::text(title), look, move |setup| load(setup, startup))
        .state_hash(Game::state_hash)
}

/// What the game's frames are painted with: its font atlas and pictures.
fn look(game: &Shuyi) -> Look<'_> {
    Look::new(&game.content.font, &game.content.images)
}

/// The game on the platform `setup` describes: at the title, or on the
/// scene `startup` asks for (ticket 0723).
fn load(setup: Setup, startup: Result<Startup, String>) -> Result<Game, String> {
    let startup = startup?;
    let mut ctx = Ctx::embedded().map_err(|e| e.to_string())?;
    ctx.console = ConsoleSpec::PC;
    ctx.game.tips_enabled = true;
    ctx.game.lessons = Lessons::On;
    let ctx = setup.ctx(ctx);
    match startup {
        Startup::Title => Ok(game::start(ctx)),
        Startup::Scene { id, gender } => game::start_on_scene(ctx, &id, gender),
    }
}
