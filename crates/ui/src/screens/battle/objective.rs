//! The objective's lines (ticket 0481, `docs/design/turn-structure.md`,
//! *The objective at a battle's start*): what wins the battle and what
//! loses it, `Win: Defeat Hendry` / `Lose: 3 of 5 houses burn` /
//! `or Ellery falls`, and in a village with rewards
//! `Bonus: Save 4 or 5 houses for rewards`. Built once, by [`lines`], for the banner shown when a
//! battle starts and for the map menu's Objective page, so the two never
//! disagree.

use crate::prelude::*;

use trpg_core::{BattleState, Faction};

use super::map_menu::objective_text;
use super::view::ObjectiveLine;
use crate::words::Words;
use shuyi_ui::text::ToArg;

/// The objective's lines for `state`: `Win:` and what wins it, then
/// `Lose:` and the first thing that loses it, then `or` and each other
/// (the village, the turn limit, the lead), then `Bonus:` and the
/// village's rewards, if it has any. An escape names `exit`, the
/// battle file's name for its exit, if it has one.
pub fn lines(state: &BattleState, exit: Option<&str>, words: Words<'_>) -> Vec<ObjectiveLine> {
    let line = |key: &str, text: String| ObjectiveLine {
        label: words.text(key).to_owned(),
        text,
    };
    let mut out = vec![line(
        "battle.objective.win_label",
        objective_text(state, exit, words),
    )];
    for (i, lose) in losses(state, words).into_iter().enumerate() {
        let key = if i == 0 {
            "battle.objective.lose_label"
        } else {
            "battle.objective.or"
        };
        out.push(line(key, lose));
    }
    if let Some(bonus) = bonus(state, words) {
        out.push(line("battle.objective.bonus_label", bonus));
    }
    out
}

/// The village's rewards: `Save 4 houses for a reward`, `Save 4 or 5
/// houses for rewards`, `Save 4 to 6 houses for rewards`.
fn bonus(state: &BattleState, words: Words<'_>) -> Option<String> {
    let v = state.village()?;
    let rewards = v.rewards_on_offer();
    if rewards == 0 {
        return None;
    }
    let (from, to) = (v.minimum() + 1, v.minimum() + rewards);
    let args = [
        ("rewards", &rewards as &dyn ToArg),
        ("from", &from),
        ("to", &to),
    ];
    Some(words.text_with("battle.objective.bonus", &args))
}

/// What loses the battle, each its own part: `3 of 5 houses burn`,
/// `Turn 8 ends`, `Ellery falls`.
fn losses(state: &BattleState, words: Words<'_>) -> Vec<String> {
    let mut lose = Vec::new();
    if let Some(v) = state.village() {
        // Lost once fewer than the minimum can still stand.
        let burn = v.total().saturating_sub(v.minimum()) + 1;
        lose.push(words.text_with(
            "battle.objective.lose_houses",
            &[("burn", &burn), ("total", &v.total())],
        ));
    }
    if let Some(limit) = state.objective().turn_limit() {
        lose.push(words.text_with("battle.objective.lose_turn", &[("limit", &limit)]));
    }
    let lead = state
        .units()
        .iter()
        .find(|u| u.faction == Faction::Player && u.is_lord);
    if let Some(lead) = lead {
        lose.push(words.text_with("battle.objective.lose_lead", &[("name", &words.unit(lead))]));
    }
    lose
}

/// The village's houses so far, `Houses: 1 saved, 1 lost`, in a battle
/// with a village (the Objective page shows it under the lines).
pub fn village_count(state: &BattleState, words: Words<'_>) -> Option<String> {
    let v = state.village()?;
    Some(words.text_with(
        "battle.objective.village",
        &[("saved", &v.saved()), ("lost", &v.lost())],
    ))
}

/// `lines` as text, each label padded to the widest so the parts line up
/// (`Win:  Defeat Hendry` / `Lose: 3 of 5 houses burn`).
pub fn padded(lines: &[ObjectiveLine]) -> Vec<String> {
    let w = label_width(lines);
    lines
        .iter()
        .map(|l| format!("{:<w$}{}", l.label, l.text))
        .collect()
}

/// The labels' column width: the widest label and a space.
pub fn label_width(lines: &[ObjectiveLine]) -> usize {
    lines
        .iter()
        .map(|l| l.label.chars().count())
        .max()
        .map_or(0, |w| w + 1)
}

#[cfg(test)]
mod tests;
