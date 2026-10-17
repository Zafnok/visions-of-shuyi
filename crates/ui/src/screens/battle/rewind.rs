//! The rewind screen (ticket 0307, `docs/design/death-and-difficulty.md`):
//! an overlay in the side panel listing the battle's past actions, newest
//! first. The map behind shows the battle as it was just before the
//! highlighted action; confirming (after a "Use 1 of N charges?" prompt)
//! goes back there for one charge.
//!
//! It lives inside the battle screen rather than on the screen stack: it
//! shows the battle screen's map with another state and hands the chosen
//! point back, which a pushed screen can't. [`RewindScreen::view`] says
//! what it shows; the battle skin paints it (ADR-0054).

use crate::prelude::*;
use core::fmt::Display;

use trpg_core::{
    BattleHistory, BattleState, Command, Event, Phase, Replayed, Side, UnitAction, UnitId,
};

use super::view::RewindView;
use crate::input::Action;
use crate::words::Words;

/// One past action.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// What happened, e.g. `Turn 2 · Ana attacked Brigand (hit, 7 dmg)`.
    pub line: String,
    /// The point a rewind to it goes back to (the command's index).
    pub point: usize,
    /// The battle just before it.
    pub before: BattleState,
}

/// What the battle screen should do after an input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewindEffect {
    /// Nothing more.
    None,
    /// Close the screen, battle unchanged.
    Close,
    /// Rewind to this point (the player confirmed).
    Rewind(usize),
}

/// The open rewind screen.
#[derive(Debug, Clone, PartialEq)]
pub struct RewindScreen {
    entries: Vec<Entry>,
    focus: usize,
    confirming: bool,
    charges_left: u8,
    charges: u8,
}

impl RewindScreen {
    /// The screen for `history`, focused on the newest action.
    pub fn new(history: &BattleHistory, charges: u8, words: Words<'_>) -> Self {
        let mut entries: Vec<Entry> = history
            .replay()
            .into_iter()
            .enumerate()
            .map(|(point, r)| Entry {
                line: describe(&r, words),
                point,
                before: r.before,
            })
            .collect();
        entries.reverse();
        Self {
            entries,
            focus: 0,
            confirming: false,
            charges_left: history.charges_left(),
            charges,
        }
    }

    /// The past actions, newest first.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// The highlighted entry, if there are any.
    pub fn focused(&self) -> Option<&Entry> {
        self.entries.get(self.focus)
    }

    /// Whether the "Use 1 of N charges?" prompt is up.
    pub fn is_confirming(&self) -> bool {
        self.confirming
    }

    /// Whether Confirm can rewind: an action to go back to and a charge.
    pub fn can_rewind(&self) -> bool {
        self.charges_left > 0 && !self.entries.is_empty()
    }

    /// Handles one action: up/down choose (not wrapping), and so do the
    /// crank's notches (ticket 1035: Rewind back is down, further back;
    /// Rewind forward is up, towards the present), Confirm asks and then
    /// rewinds, Cancel backs out of the prompt or closes, Rewind closes.
    pub fn step(&mut self, action: Action) -> RewindEffect {
        match action {
            Action::Cancel if self.confirming => self.confirming = false,
            Action::Cancel | Action::Rewind => return RewindEffect::Close,
            Action::Confirm if self.confirming => {
                if let Some(e) = self.focused() {
                    return RewindEffect::Rewind(e.point);
                }
            }
            Action::Confirm => self.confirming = self.can_rewind(),
            Action::CursorDown | Action::RewindBack if !self.confirming => {
                self.focus = (self.focus + 1).min(self.entries.len().saturating_sub(1));
            }
            Action::CursorUp | Action::RewindForward if !self.confirming => {
                self.focus = self.focus.saturating_sub(1);
            }
            _ => {}
        }
        RewindEffect::None
    }

    /// What the screen shows: title, charges, the list, the highlighted
    /// entry and the question while it is asked.
    pub fn view(&self, words: Words<'_>) -> RewindView {
        let spent = self.charges_left == 0;
        let charges = if spent {
            words.text("battle.rewind.no_charges").to_owned()
        } else {
            let key = if self.charges == 1 {
                "battle.rewind.charges.one"
            } else {
                "battle.rewind.charges.many"
            };
            let counts = [
                ("left", &self.charges_left as &dyn Display),
                ("charges", &self.charges),
            ];
            words.text_with(key, &counts)
        };
        let prompt = self.confirming.then(|| {
            let key = if self.charges_left == 1 {
                "battle.rewind.use.one"
            } else {
                "battle.rewind.use.many"
            };
            words.text_with(key, &[("left", &self.charges_left)])
        });
        RewindView {
            title: words.text("battle.rewind.title").to_owned(),
            charges,
            spent,
            entries: self.entries.iter().map(|e| e.line.clone()).collect(),
            focus: self.focus,
            nothing: self
                .entries
                .is_empty()
                .then(|| words.text("battle.rewind.nothing").to_owned()),
            prompt,
        }
    }
}

/// A unit's name in `state`, or `?` (never shown: a command only names
/// units on the map just before it).
fn name<'a>(state: &'a BattleState, words: Words<'a>, id: UnitId) -> &'a str {
    state.unit(id).map_or("?", |u| words.unit(u))
}

/// How a combat went for the attacker: `hit, 7 dmg`, `2 hits, 14 dmg` or
/// `missed`, plus `defeated` if the defender fell.
fn combat_summary(words: Words<'_>, events: &[Event]) -> Option<String> {
    let (outcome, defender_hp) = events.iter().find_map(|e| match e {
        Event::CombatResolved { outcome, .. } => Some((outcome, outcome.defender_hp)),
        _ => None,
    })?;
    let hits: Vec<_> = outcome
        .strikes
        .iter()
        .filter(|s| s.by == Side::Attacker && s.hit && !s.healed)
        .collect();
    let dmg: i32 = hits.iter().map(|s| s.damage).sum();
    let text = match hits.len() {
        0 => words.text("battle.rewind.missed").to_owned(),
        1 => words.text_with("battle.rewind.hit", &[("dmg", &dmg)]),
        n => words.text_with("battle.rewind.hits", &[("n", &n), ("dmg", &dmg)]),
    };
    Some(if defender_hp == 0 {
        words.text_with("battle.rewind.defeated", &[("result", &text)])
    } else {
        text
    })
}

/// A readable line for a past command, e.g. `Turn 2 · Ana attacked Brigand
/// (hit, 7 dmg)`.
pub fn describe(r: &Replayed, words: Words<'_>) -> String {
    let s = &r.before;
    let name = |s, id| name(s, words, id);
    let did = |key, who: &str| words.text_with(key, &[("who", &who)]);
    let did_to = |key, who: &str, what: &str, target: &str| {
        words.text_with(key, &[("who", &who), ("what", &what), ("target", &target)])
    };
    let what = match &r.command {
        Command::Act { unit, action, .. } => {
            let who = name(s, *unit);
            // A fight's result after what started it: `… (hit, 7 dmg)`.
            let fought = |what: String| match combat_summary(words, &r.events) {
                Some(result) => words.text_with(
                    "battle.rewind.with_result",
                    &[("what", &what), ("result", &result)],
                ),
                None => what,
            };
            match action {
                UnitAction::Wait => did("battle.rewind.waited", who),
                UnitAction::Attack { target, .. } => {
                    fought(did_to("battle.rewind.attacked", who, "", name(s, *target)))
                }
                UnitAction::UseItem { pack_index, target } => {
                    let item = s
                        .pack()
                        .items
                        .get(*pack_index)
                        .filter(|id| s.items().get(id).is_some())
                        .map_or(words.text("battle.rewind.an_item"), |id| {
                            words.item(id, s.items())
                        });
                    if target == unit {
                        did_to("battle.rewind.used", who, item, "")
                    } else {
                        did_to("battle.rewind.used_on", who, item, name(s, *target))
                    }
                }
                UnitAction::Seize => did("battle.rewind.seized", who),
                UnitAction::Shop { .. } => did("battle.rewind.shopped", who),
                UnitAction::Open => did("battle.rewind.opened", who),
                UnitAction::Cast { spell, target, .. } => {
                    let spell = s
                        .spells()
                        .get(spell)
                        .map_or(words.text("battle.rewind.a_spell"), |d| words.spell(d));
                    match target {
                        trpg_core::CastTarget::Unit(t) => {
                            fought(did_to("battle.rewind.cast_on", who, spell, name(s, *t)))
                        }
                        trpg_core::CastTarget::Tile(_) => {
                            did_to("battle.rewind.cast", who, spell, "")
                        }
                    }
                }
                UnitAction::UseSkill { skill, .. } => {
                    let skill = s
                        .skills()
                        .get(skill)
                        .map_or(words.text("battle.rewind.a_skill"), |d| words.skill(d));
                    did_to("battle.rewind.used", who, skill, "")
                }
            }
        }
        Command::Equip { unit, .. } => did("battle.rewind.equipped", name(s, *unit)),
        Command::MoveAfter { unit, to } => {
            let who = name(s, *unit);
            if to.is_some() {
                did("battle.rewind.moved_after", who)
            } else {
                did("battle.rewind.stayed_after", who)
            }
        }
        Command::Talk { unit, target, .. } => {
            did_to("battle.rewind.talked", name(s, *unit), "", name(s, *target))
        }
        Command::EndPhase => words.text(phase_ended(s.phase())).to_owned(),
    };
    words.text_with(
        "battle.rewind.line",
        &[("turn", &s.turn()), ("what", &what)],
    )
}

/// The key of the line for the end of `phase` (`Player phase ended`).
const fn phase_ended(phase: Phase) -> &'static str {
    match phase {
        Phase::Player => "battle.rewind.phase_ended.player",
        Phase::Enemy => "battle.rewind.phase_ended.enemy",
        // Ally and neutral units (FE calls it the ally phase).
        Phase::Other => "battle.rewind.phase_ended.other",
    }
}

#[cfg(test)]
mod tests;
