//! Playtest bots (ADR-0033, tickets 0505 and 0506): players made of code,
//! which play a battle through `core`'s [`Command`](trpg_core::Command)s,
//! and the measures of how a try went. Dev tooling: used by `cargo xtask
//! playtest` and tests, never by the game.
//!
//! Pure like `core` (ADR-0004): no I/O and no clock. The caller loads the
//! battle and the data files, times the tries and writes the report. Bots
//! know nothing about the look of the game (ADR-0038).
//!
//! The bot trait, the play loop, the turn planner and the playtest runner
//! are the engine's (`shuyi_core::bots`, tickets 1116, engine#2125,
//! engine#2126 and engine#2128); this crate holds the game's parts:
//!
//! - [`PlayerBot`]: what plays the Player phase, any engine
//!   `Bot<BattleState>`. [`BaselineBot`] is the enemy AI on the player's
//!   side; [`PersonaBot`] is a persona (Casual, Normal, Hardcore) on the
//!   engine's turn planner, with its [`order`] of candidates and its
//!   scoring ([`Tally`], [`Weights`]), from `personas.ron`.
//! - [`BattleEntrant`]: one named player of a battle for the engine's
//!   runner, and [`BattleMeasure`]: what happened in a try, as the
//!   engine's named measures, from [`BattleMeasures`].
//! - [`Targets`]: what the personas should achieve, from `targets.ron`,
//!   as the engine's checks for a battle's tier.

mod bot;
mod measures;
mod persona;
mod playtest;

pub use bot::{BaselineBot, PlayerBot};
pub use measures::BattleMeasures;
pub use persona::{
    Persona, PersonaBot, PersonaOrder, Personas, Scoring, Tally, Weights, order, personas_from_ron,
};
pub use playtest::{BattleEntrant, BattleMeasure, BattleStage, Faster, Player, Targets};

#[cfg(test)]
mod testkit;
