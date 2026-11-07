//! The game's record in a crash report (ticket 1245, engine ADR-0079): the
//! open battle as its first state, the commands applied since and the one
//! the rules were applying, so `cargo shuyi replay` plays a crash in a
//! battle to the command it crashed on. Nothing about the look (ADR-0038
//! §5), and nothing the player typed.

use serde::{Deserialize, Serialize};
use trpg_core::{BattleHistory, Command};

use crate::game::Game;
use crate::screens::BattleScreen;

/// The [`BattleRecord`] format's version.
pub const VERSION: u32 = 1;

/// A battle as a crash report holds it. RON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattleRecord {
    /// The format's version ([`VERSION`]).
    pub version: u32,
    /// The battle's first state and the commands applied since. Its
    /// content tables are left out: a reader restores them
    /// ([`BattleHistory::restore_tables`]).
    pub history: BattleHistory,
    /// The command the rules were applying when they didn't return.
    pub applying: Option<Command>,
}

impl BattleRecord {
    /// The commands to replay from the first state: those applied, then
    /// the one being applied.
    pub fn commands(&self) -> Vec<Command> {
        let mut commands = self.history.commands().to_vec();
        commands.extend(self.applying.clone());
        commands
    }
}

/// The open battle's [`BattleRecord`] as RON, for a crash report
/// ([`shuyi_platform::Start::crash_record`]): `None` outside a battle.
pub fn crash_record(game: &Game) -> Option<String> {
    let battle = game.screen::<BattleScreen>()?;
    let record = BattleRecord {
        version: VERSION,
        history: battle.history().clone(),
        applying: battle.applying().cloned(),
    };
    ron::to_string(&record).ok()
}
