//! What the combat stage shows this frame (ADR-0054, ADR-0056): plain
//! data, in the player's language. The skin ([`super::glyph`]) decides
//! where and how.

use trpg_content::{Battler, ImageId};
use trpg_core::{Faction, StatValue};

use super::motion::Pose;

/// The stage: two fighters in front of a backdrop.
#[derive(Debug, Clone, PartialEq)]
pub struct StageView {
    /// The picture behind the fighters.
    pub backdrop: Option<ImageId>,
    /// The name of the art or active used, if any.
    pub banner: Option<String>,
    /// The attacker, then the defender.
    pub fighters: [FighterView; 2],
    /// Which of [`fighters`](Self::fighters) stands on the left.
    pub left: usize,
    /// Whether they fight hand to hand: a striker runs up to its target.
    /// Otherwise it only steps forward.
    pub melee: bool,
    /// Which of them is striking now, if one is.
    pub striker: Option<usize>,
    /// The current strike's result and which fighter it is shown over.
    pub result: Option<(usize, String)>,
    /// Whether the whole stage flashes (a critical just landed).
    pub veil: bool,
}

/// One fighter on the stage.
#[derive(Debug, Clone, PartialEq)]
pub struct FighterView {
    /// Its name.
    pub name: String,
    /// Its faction.
    pub faction: Faction,
    /// Its HP as shown now (draining while a result is up).
    pub hp: StatValue,
    /// Its max HP.
    pub max: StatValue,
    /// Whether its name is lit: it is about to strike.
    pub lit: bool,
    /// Its numbers for this combat; `None` if it can't strike back.
    pub numbers: Option<Numbers>,
    /// Its picture; `None` if the game has none at all.
    pub picture: Option<Battler>,
    /// How its picture stands this frame.
    pub pose: Pose,
}

/// A fighter's numbers for the combat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Numbers {
    /// Damage of a strike (HP healed, if [`heals`](Self::heals)).
    pub damage: StatValue,
    /// Whether its strikes heal the target instead (Absorb).
    pub heals: bool,
    /// Hit chance, 0 to 100.
    pub hit: u8,
    /// Critical chance, 0 to 100.
    pub crit: u8,
}
