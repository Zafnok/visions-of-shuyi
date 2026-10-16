//! Turn rewind (ticket 0307, `docs/design/death-and-difficulty.md`): a
//! battle's first state and every command applied since, so any earlier
//! point can be rebuilt by replaying.
//!
//! # Rules
//!
//! - **Points.** Point `n` is the battle after its first `n` commands
//!   ([`BattleHistory::state_at`]); point 0 is the state the battle started
//!   in. Rewinding to point `n` undoes command `n` and every one after it,
//!   the enemy's included.
//! - **Charges.** A battle starts with [`BattleState::rewind_charges`]
//!   charges (from the map's difficulty tier, 0801). A rewind costs one
//!   however far back it goes; with none left it is refused. Charges don't
//!   carry between battles, and restarting a battle starts a fresh history
//!   with every charge back.
//! - **Luck is part of the state.** The battle's [`SimRng`](crate::SimRng)
//!   is in [`BattleState`], so replaying rebuilds the exact same state:
//!   **rewinding and then doing the same thing again gives the same
//!   result** (like Echoes' Turnwheel). Doing something different consumes
//!   the RNG differently and so changes what follows naturally.
//! - **Saving.** The history is serialisable for the suspend save (0802).
//!   Like any saved [`BattleState`] (ADR-0020), the first state comes back
//!   without its content tables: the loader must call
//!   [`BattleHistory::restore_tables`] before replaying.
//!
//! The points, the replay and the rewind are the engine's
//! ([`shuyi_core::history`], over the battle's
//! [`TurnRules`](shuyi_core::TurnRules)); the charges are the game's.

use crate::prelude::*;

use core::fmt;

use serde::{Deserialize, Serialize};
use shuyi_core::history::{History, NoSuchPoint};

use crate::{BattleState, Command, GameTables};

/// A battle's first state, the commands applied since and the rewind
/// charges left.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "SavedHistory", into = "SavedHistory")]
pub struct BattleHistory {
    history: History<BattleState>,
    charges_left: u8,
}

/// A [`BattleHistory`] as a save holds it: its fields as they were saved
/// before the history moved to the engine, so saves keep their format and
/// version (ADR-0039).
#[derive(Serialize, Deserialize)]
#[serde(rename = "BattleHistory")]
struct SavedHistory {
    initial: BattleState,
    commands: Vec<Command>,
    charges_left: u8,
}

impl From<SavedHistory> for BattleHistory {
    fn from(saved: SavedHistory) -> Self {
        Self {
            history: History::from_parts(saved.initial, saved.commands),
            charges_left: saved.charges_left,
        }
    }
}

impl From<BattleHistory> for SavedHistory {
    fn from(battle: BattleHistory) -> Self {
        let (initial, commands) = battle.history.into_parts();
        Self {
            initial,
            commands,
            charges_left: battle.charges_left,
        }
    }
}

/// One past command, as shown by the rewind screen: the battle just before
/// it (the point a rewind to it goes back to), the command and what it did.
pub type Replayed = shuyi_core::history::Replayed<BattleState>;

/// Why a rewind was refused. Nothing changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewindError {
    /// No charges left.
    NoCharges,
    /// There is no earlier point `n` (it must be below the number of
    /// commands).
    NoSuchPoint(usize),
}

impl fmt::Display for RewindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoCharges => f.write_str("no rewind charges left"),
            Self::NoSuchPoint(n) => write!(f, "no earlier point {n} to rewind to"),
        }
    }
}

impl core::error::Error for RewindError {}

impl From<NoSuchPoint> for RewindError {
    fn from(NoSuchPoint(n): NoSuchPoint) -> Self {
        Self::NoSuchPoint(n)
    }
}

impl BattleHistory {
    /// A history starting at `initial` (a battle just started), with its
    /// [`rewind_charges`](BattleState::rewind_charges).
    pub fn new(initial: BattleState) -> Self {
        Self {
            charges_left: initial.rewind_charges(),
            history: History::new(initial),
        }
    }

    /// Records `cmd`, which was just applied to the battle successfully.
    /// (A refused command changes nothing, so recording one by mistake
    /// replays as nothing too.)
    pub fn push(&mut self, cmd: Command) {
        self.history.push(cmd);
    }

    /// The commands applied so far, oldest first.
    pub fn commands(&self) -> &[Command] {
        self.history.commands()
    }

    /// How many commands were applied (the latest point).
    pub fn len(&self) -> usize {
        self.history.len()
    }

    /// Whether no command was applied yet (nothing to rewind).
    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }

    /// Rewind charges left.
    pub fn charges_left(&self) -> u8 {
        self.charges_left
    }

    /// The battle after its first `n` commands (all of them if `n` is past
    /// the end).
    pub fn state_at(&self, n: usize) -> BattleState {
        self.history.state_at(n)
    }

    /// Every command replayed in order, with the battle just before it and
    /// its events: one replay for the whole list.
    pub fn replay(&self) -> Vec<Replayed> {
        self.history.replay()
    }

    /// Goes back to point `n` (before command `n`), forgetting that command
    /// and every later one, for one charge. Returns the battle at that
    /// point.
    pub fn rewind_to(&mut self, n: usize) -> Result<BattleState, RewindError> {
        // A point to go back to, but no charge to get there.
        if self.charges_left == 0 && n < self.history.len() {
            return Err(RewindError::NoCharges);
        }
        let state = self.history.rewind_to(n)?;
        self.charges_left -= 1;
        Ok(state)
    }

    /// Reattaches the content tables to the first state after
    /// deserialising ([`BattleState::restore_tables`]).
    pub fn restore_tables(&mut self, tables: &GameTables) {
        self.history.initial_mut().restore_tables(tables);
    }
}

#[cfg(test)]
mod tests;
