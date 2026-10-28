//! What the message line says after a chest is opened or a house
//! evacuated (ticket 0409): `Got Potion.`, `Got 500 gold.`,
//! `House saved.`; and the village's line on the objective page,
//! `Save at least 3 of 5 houses (1 saved, 1 lost)`
//! (`docs/design/turn-structure.md`, *Save the village*).

use crate::prelude::*;

use trpg_core::{BattleState, Event, Loot};

use super::shop::gold_text;
use crate::words::Words;

/// The line for what `events` gave a player: the last chest opened or
/// house evacuated among them, if any.
pub fn line(state: &BattleState, words: Words<'_>, events: &[Event]) -> Option<String> {
    events.iter().rev().find_map(|e| match e {
        Event::ChestOpened { loot, .. } => Some(got(state, words, loot)),
        Event::HouseSaved { unit: Some(_), .. } => {
            Some(words.text("battle.loot.house_saved").to_owned())
        }
        _ => None,
    })
}

/// `Got Potion.` or `Got 500 gold.`
fn got(state: &BattleState, words: Words<'_>, loot: &Loot) -> String {
    match loot {
        Loot::Gold(gold) => words.text_with("battle.loot.gold", &[("gold", &gold_text(*gold))]),
        Loot::Item(item) => words.text_with(
            "battle.loot.item",
            &[("item", &words.item(item, state.items()))],
        ),
    }
}

/// The village's line, in a battle with a village.
pub fn village_text(state: &BattleState, words: Words<'_>) -> Option<String> {
    let v = state.village()?;
    Some(words.text_with(
        "battle.objective.village",
        &[
            ("minimum", &v.minimum()),
            ("total", &v.total()),
            ("saved", &v.saved()),
            ("lost", &v.lost()),
        ],
    ))
}
