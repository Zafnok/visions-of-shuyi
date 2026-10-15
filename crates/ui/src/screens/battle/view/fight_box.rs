//! What the fight box shows (ticket 0404, ADR-0054): a fight playing back
//! with combat animations off, both combatants and the current strike's
//! result. Built by [`playback::box_view`](super::super::playback::box_view).

use trpg_core::{Faction, StatValue};

/// The fight box.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FightBoxView {
    /// The name of the art or active used, if any.
    pub banner: Option<String>,
    /// The name of the HP stat.
    pub hp_name: String,
    /// The attacker, then the defender.
    pub sides: [BoxFighterView; 2],
}

/// One combatant in the fight box.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxFighterView {
    /// Its name.
    pub name: String,
    /// Its faction.
    pub faction: Faction,
    /// Whether its name is lit this frame: it flashes before it strikes.
    pub flashing: bool,
    /// Its HP as shown now (draining while a result is up).
    pub hp: StatValue,
    /// Its max HP.
    pub max: StatValue,
    /// The result of the strike at it, while it is up (`HIT -8`).
    pub result: Option<String>,
}
