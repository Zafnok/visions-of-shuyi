//! The map menu (ticket 0405), opened by Cancel with nothing to cancel or
//! Confirm on an empty tile (`docs/design/controls.md`): `Units` (jump the
//! cursor to one), `Objective` (what to do and the turn), `Guides` (0458:
//! every lesson and tip, to read again), `Options` (0805:
//! opens the Options screen), `Suspend` (0802: saves the battle and goes back
//! to the title, after a confirm), `Restart Battle` (0801, with a confirm)
//! and `End Turn`;
//! and the end-turn prompt
//! (`docs/design/turn-structure.md`).
//!
//! On a platform with no keyboard (the Playdate, ticket 1035,
//! `docs/design/playdate.md`) the map menu also holds the shortcuts that
//! have no button there: `Danger Zone`, `Auto-end` and `Unit Info`, after
//! `Objective` ([`MapEntry::SHORTCUTS`], [`shortcuts`]).

use crate::prelude::*;
use core::fmt::Display;

use trpg_core::{BattleState, Faction, Objective, Phase, Pos, Turn, UnitId};

use crate::input::{Action, Keymap};
use crate::screen::Keyboard;
use crate::widgets::menu::{Menu, MenuItem};
use crate::words::Words;

/// One entry of the map menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapEntry {
    /// The player's units: choosing one jumps the cursor to it.
    Units,
    /// The objective and the turn.
    Objective,
    /// Every lesson and tip, to read again (ticket 0458).
    Guides,
    /// Opens the Options screen (ticket 0805).
    Options,
    /// Save the whole battle to the one-time suspend save and go back to
    /// the title (`death-and-difficulty.md`), after a confirm.
    Suspend,
    /// Start the battle again from its first turn, with every rewind
    /// charge back (`death-and-difficulty.md`), after a confirm.
    Restart,
    /// End the player phase.
    EndTurn,
    /// A shortcut with no button (ticket 1035): toggles the enemy danger
    /// zone, as the Danger zone key does.
    DangerZone,
    /// A shortcut with no button: turns auto-end on or off, as the
    /// Auto-end key does.
    AutoEnd,
    /// A shortcut with no button: the info screen of the unit under the
    /// cursor, as the Unit info key does.
    UnitInfo,
}

impl MapEntry {
    /// The PC's entries (every platform has them), in menu order.
    pub const ALL: [MapEntry; 7] = [
        MapEntry::Units,
        MapEntry::Objective,
        MapEntry::Guides,
        MapEntry::Options,
        MapEntry::Suspend,
        MapEntry::Restart,
        MapEntry::EndTurn,
    ];

    /// The shortcuts a platform without a keyboard lists when they have
    /// no button ([`shortcuts`]), in menu order, after `Objective`.
    pub const SHORTCUTS: [MapEntry; 3] =
        [MapEntry::DangerZone, MapEntry::AutoEnd, MapEntry::UnitInfo];

    /// The action a shortcut entry does, as if its key were pressed; `None`
    /// for the other entries.
    pub fn action(self) -> Option<Action> {
        match self {
            MapEntry::DangerZone => Some(Action::DangerZone),
            MapEntry::AutoEnd => Some(Action::ToggleAutoEnd),
            MapEntry::UnitInfo => Some(Action::Info),
            MapEntry::Units
            | MapEntry::Objective
            | MapEntry::Guides
            | MapEntry::Options
            | MapEntry::Suspend
            | MapEntry::Restart
            | MapEntry::EndTurn => None,
        }
    }

    /// The menu item, disabled if not `enabled`: its text in the player's
    /// language when it is painted.
    pub fn item(self, enabled: bool) -> MenuItem<MapEntry> {
        let key = match self {
            MapEntry::Units => "battle.map_menu.units",
            MapEntry::Objective => "battle.map_menu.objective",
            MapEntry::Guides => "battle.map_menu.guides",
            MapEntry::Options => "battle.map_menu.options",
            MapEntry::Suspend => "battle.map_menu.suspend",
            MapEntry::Restart => "battle.map_menu.restart",
            MapEntry::EndTurn => "battle.map_menu.end_turn",
            MapEntry::DangerZone => "battle.map_menu.danger_zone",
            MapEntry::AutoEnd => "battle.map_menu.auto_end",
            MapEntry::UnitInfo => "battle.map_menu.unit_info",
        };
        MenuItem::keyed(key).with_value(self).with_enabled(enabled)
    }

    /// Whether it can be chosen in `state`: `Suspend` while the battle is
    /// still going, `End Turn` only in the player phase of a battle still
    /// going, the others (`Restart Battle` too) always. `Unit Info` also
    /// needs a unit under the cursor ([`available`](Self::available)).
    pub fn enabled(self, state: &BattleState) -> bool {
        match self {
            MapEntry::Units
            | MapEntry::Objective
            | MapEntry::Guides
            | MapEntry::Options
            | MapEntry::Restart
            | MapEntry::DangerZone
            | MapEntry::AutoEnd
            | MapEntry::UnitInfo => true,
            MapEntry::Suspend => state.outcome().is_none(),
            MapEntry::EndTurn => state.phase() == Phase::Player && state.outcome().is_none(),
        }
    }

    /// Whether it can be chosen in `state` with the cursor at `cursor`:
    /// [`enabled`](Self::enabled), and for `Unit Info` a unit there.
    pub fn available(self, state: &BattleState, cursor: Pos) -> bool {
        match self {
            MapEntry::UnitInfo => state.units().iter().any(|u| u.pos == cursor),
            _ => self.enabled(state),
        }
    }
}

/// The shortcuts the map menu lists ([`MapEntry::SHORTCUTS`]): on a
/// platform with no keyboard, whose only device is its buttons, those whose
/// action has no button or button chord in `keymap` (the player's
/// bindings); none with a keyboard, so the PC's map menu is the same as
/// ever.
pub fn shortcuts(keyboard: Keyboard, keymap: &Keymap) -> Vec<MapEntry> {
    if keyboard == Keyboard::Present {
        return Vec::new();
    }
    MapEntry::SHORTCUTS
        .into_iter()
        .filter(|e| e.action().is_some_and(|a| keymap.buttons_for(a).is_empty()))
        .collect()
}

/// The map menu for `state`, focused on `focus` (if enabled).
pub fn map_menu(state: &BattleState, focus: MapEntry) -> Menu<MapEntry> {
    map_menu_with(state, focus, &[], Pos::new(0, 0))
}

/// [`map_menu`] with the `shortcuts` the platform lists (after
/// `Objective`), the cursor at `cursor`.
pub fn map_menu_with(
    state: &BattleState,
    focus: MapEntry,
    shortcuts: &[MapEntry],
    cursor: Pos,
) -> Menu<MapEntry> {
    let mut entries = MapEntry::ALL.to_vec();
    let after = entries
        .iter()
        .position(|&e| e == MapEntry::Objective)
        .map_or(0, |i| i + 1);
    entries.splice(after..after, shortcuts.iter().copied());
    let index = entries.iter().position(|&e| e == focus).unwrap_or(0);
    let items = entries
        .into_iter()
        .map(|e| e.item(e.available(state, cursor)));
    Menu::new(items.collect()).focused(index)
}

/// How many player units can still act this phase.
pub fn ready_players(state: &BattleState) -> usize {
    state
        .units()
        .iter()
        .filter(|u| u.faction == Faction::Player && Phase::of(u.faction) == state.phase())
        .filter(|u| !u.acted)
        .count()
}

/// The `Units` list: every player unit, as `name  HP hp/max  ready` (or
/// `acted`), in the battle's unit order, each standing for its unit.
pub fn unit_list(state: &BattleState, words: Words<'_>) -> Menu<UnitId> {
    let players: Vec<_> = state
        .units()
        .iter()
        .filter(|u| u.faction == Faction::Player)
        .collect();
    let name_w = players
        .iter()
        .map(|u| words.unit(u).chars().count())
        .max()
        .unwrap_or(0);
    let items = players
        .iter()
        .map(|u| {
            let status = words.text(if u.acted {
                "battle.units.acted"
            } else {
                "battle.units.ready"
            });
            let name = format!("{:name_w$}", words.unit(u));
            let hp = format!("{:>5}", format!("{}/{}", u.hp, u.stats.hp));
            let line = [
                ("name", &name as &dyn Display),
                ("hp", &hp),
                ("status", &status),
            ];
            MenuItem::new(words.text_with("battle.units.line", &line)).with_value(u.id)
        })
        .collect();
    Menu::new(items)
}

/// What the objective asks, e.g. `Rout the enemy` or `Seize the Fort`;
/// an escape names `exit`, the battle file's name for its exit, if it has
/// one. *Claude's starting wording* until maps carry their own text.
pub fn objective_text(state: &BattleState, exit: Option<&str>, words: Words<'_>) -> String {
    match state.objective() {
        Objective::Rout { .. } => words.text("battle.objective.rout").to_owned(),
        Objective::DefeatUnit { unit, .. } => {
            let boss = state
                .unit(unit)
                .or_else(|| state.fallen().iter().find(|u| u.id == unit));
            match boss {
                Some(u) => words.text_with("battle.objective.defeat", &[("name", &words.unit(u))]),
                None => words.text("battle.objective.defeat_boss").to_owned(),
            }
        }
        Objective::Seize { pos, .. } => {
            let place = state
                .map()
                .tiles
                .get(pos)
                .and_then(|&id| state.terrain().get(id));
            match place {
                Some(t) => {
                    words.text_with("battle.objective.seize", &[("place", &words.terrain(t))])
                }
                None => words.text("battle.objective.seize_objective").to_owned(),
            }
        }
        Objective::Survive { turns } => {
            words.text_with("battle.objective.survive", &[("turns", &turns)])
        }
        Objective::Escape { .. } => {
            // Those out of those still standing: on the map or out.
            let out = state.escaped().len();
            let on_map = state.units().iter();
            let total = out + on_map.filter(|u| u.faction == Faction::Player).count();
            match exit {
                Some(place) => words.text_with(
                    "battle.objective.escape_to",
                    &[
                        ("place", &words.exit_name(place)),
                        ("out", &out),
                        ("total", &total),
                    ],
                ),
                None => words.text_with(
                    "battle.objective.escape",
                    &[("out", &out), ("total", &total)],
                ),
            }
        }
    }
}

/// The turn, with the map's limit if it has one: `Turn 3` or `Turn 3/8`
/// (for Survive, the turn that wins).
pub fn turn_text(state: &BattleState, words: Words<'_>) -> String {
    let turn = state.turn();
    match shown_limit(state) {
        Some(limit) => words.text_with("battle.turn_of", &[("turn", &turn), ("limit", &limit)]),
        None => words.text_with("battle.turn", &[("turn", &turn)]),
    }
}

/// The turn shown after the current one as its limit: the objective's
/// turn limit, or for Survive the turn that wins.
pub fn shown_limit(state: &BattleState) -> Option<Turn> {
    match state.objective() {
        Objective::Survive { turns } => Some(turns),
        other => other.turn_limit(),
    }
}

/// The end-turn question for `ready` units still ready.
pub fn end_turn_question(words: Words<'_>, ready: usize) -> String {
    let key = if ready == 1 {
        "battle.end_turn_question.one"
    } else {
        "battle.end_turn_question.many"
    };
    words.text_with(key, &[("ready", &ready)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen::ShuyiCtx;
    use crate::screen::tests::ctx;
    use crate::screens::battle::quick_battle;
    use crate::screens::battle::testing::{battle_with, wait};
    use crate::widgets::menu::{english, text_of};

    fn labels(menu: &Menu<MapEntry>) -> Vec<(String, bool)> {
        menu.items()
            .iter()
            .map(|i| (english(&i.label), i.enabled))
            .collect()
    }

    /// What each item of `menu` is.
    fn entries(menu: &Menu<MapEntry>) -> Vec<MapEntry> {
        menu.items().iter().map(|i| i.value).collect()
    }

    #[test]
    fn end_turn_needs_the_player_phase() {
        let mut s = quick_battle(&ctx().game.content).unwrap();
        let menu = map_menu(&s, MapEntry::Units);
        assert_eq!(entries(&menu), MapEntry::ALL);
        assert_eq!(
            labels(&menu),
            [
                ("Units", true),
                ("Objective", true),
                ("Guides", true),
                ("Options", true),
                ("Suspend", true),
                ("Restart Battle", true),
                ("End Turn", true),
            ]
            .map(|(l, e)| (l.to_owned(), e))
        );
        assert_eq!(menu.focus(), 0);
        assert_eq!(map_menu(&s, MapEntry::Guides).focus(), 2);
        assert_eq!(map_menu(&s, MapEntry::Restart).focus(), 5);
        assert_eq!(map_menu(&s, MapEntry::EndTurn).focus(), 6);
        assert_eq!(map_menu(&s, MapEntry::Options).focus(), 3);
        assert_eq!(map_menu(&s, MapEntry::Suspend).focus(), 4);
        // In the enemy phase, End Turn is disabled.
        s.apply(&trpg_core::Command::EndPhase).unwrap();
        assert_eq!(s.phase(), Phase::Enemy);
        assert!(!MapEntry::EndTurn.enabled(&s));
        // A disabled entry can't take the focus.
        assert_eq!(map_menu(&s, MapEntry::EndTurn).focus(), 0);
        assert!(MapEntry::Options.enabled(&s));
        assert!(MapEntry::Guides.enabled(&s));
        // Nobody acted, but it isn't the player's phase: none ready.
        assert_eq!(ready_players(&s), 0);
        assert!(MapEntry::Units.enabled(&s));
        assert!(MapEntry::Restart.enabled(&s));
        assert!(MapEntry::Suspend.enabled(&s));
    }

    /// Ticket 1035: with no keyboard, the shortcuts that have no button
    /// in the bindings; with one, none.
    #[test]
    fn the_shortcuts_are_those_with_no_button_on_a_platform_without_a_keyboard() {
        let mut c = ctx();
        let pc = c.keymap.clone();
        assert_eq!(shortcuts(Keyboard::Present, &pc), []);
        // The PC's pad has a button for each.
        assert_eq!(shortcuts(Keyboard::Absent, &pc), []);
        c.use_playdate_buttons();
        assert_eq!(shortcuts(Keyboard::Absent, &c.keymap), MapEntry::SHORTCUTS);
        assert_eq!(shortcuts(Keyboard::Present, &c.keymap), []);
        let actions = MapEntry::SHORTCUTS.map(MapEntry::action);
        let expected = [Action::DangerZone, Action::ToggleAutoEnd, Action::Info].map(Some);
        assert_eq!(actions, expected);
        assert!(MapEntry::ALL.iter().all(|e| e.action().is_none()));
    }

    /// Ticket 1035: the shortcuts go after Objective; Unit Info needs a
    /// unit under the cursor.
    #[test]
    fn the_shortcuts_follow_objective_and_unit_info_needs_a_unit() {
        let s = quick_battle(&ctx().game.content).unwrap();
        let on = s.units()[0].pos;
        let menu = map_menu_with(&s, MapEntry::UnitInfo, &MapEntry::SHORTCUTS, on);
        assert_eq!(
            labels(&menu)[..6],
            [
                ("Units", true),
                ("Objective", true),
                ("Danger Zone", true),
                ("Auto-End", true),
                ("Unit Info", true),
                ("Guides", true),
            ]
            .map(|(l, e)| (l.to_owned(), e))
        );
        assert_eq!(entries(&menu).len(), 10);
        assert_eq!(menu.focus(), 4);
        let empty = Pos::new(0, 0);
        assert!(s.units().iter().all(|u| u.pos != empty));
        let menu = map_menu_with(&s, MapEntry::UnitInfo, &[MapEntry::UnitInfo], empty);
        assert_eq!(labels(&menu)[2], ("Unit Info".to_owned(), false));
        assert_eq!(menu.focus(), 0, "a dimmed entry takes no focus");
    }

    #[test]
    fn end_turn_is_disabled_once_the_battle_is_over() {
        let c = ctx();
        let q = quick_battle(&c.game.content).unwrap();
        let players: Vec<_> = q
            .units()
            .iter()
            .filter(|u| u.faction == Faction::Player)
            .cloned()
            .collect();
        let rout = Objective::Rout { turn_limit: None };
        // No enemies: won from the start.
        let s = battle_with(&c, q.map().clone(), players, rout);
        assert!(s.outcome().is_some());
        assert!(!MapEntry::EndTurn.enabled(&s));
        // Nor is there a battle left to suspend.
        assert!(!MapEntry::Suspend.enabled(&s));
    }

    #[test]
    fn the_unit_list_shows_hp_and_who_is_ready() {
        let mut s = quick_battle(&ctx().game.content).unwrap();
        wait(&mut s, 2);
        assert_eq!(ready_players(&s), 3);
        let menu = unit_list(&s, Words::ENGLISH);
        let ids: Vec<UnitId> = menu.items().iter().map(|i| i.value).collect();
        assert_eq!(ids, [UnitId(1), UnitId(2), UnitId(3), UnitId(8)]);
        let rows: Vec<&str> = menu.items().iter().map(|i| text_of(&i.label)).collect();
        assert_eq!(
            rows,
            [
                "Test Lord    HP 19/19  ready",
                "Test Knight  HP 20/20  ready",
                "Test Archer  HP 17/17  acted",
                "Test Mage    HP 16/16  ready",
            ]
        );
    }

    #[test]
    fn objective_and_turn_texts() {
        let c = ctx();
        let q = quick_battle(&c.game.content).unwrap();
        assert_eq!(objective_text(&q, None, Words::ENGLISH), "Rout the enemy");
        assert_eq!(turn_text(&q, Words::ENGLISH), "Turn 1");
        let with = |o| battle_with(&c, q.map().clone(), q.units().to_vec(), o);
        let s = with(Objective::Rout {
            turn_limit: Some(8),
        });
        assert_eq!(turn_text(&s, Words::ENGLISH), "Turn 1/8");
        let s = with(Objective::DefeatUnit {
            unit: UnitId(6),
            turn_limit: None,
        });
        assert_eq!(objective_text(&s, None, Words::ENGLISH), "Defeat Raider");
        let s = with(Objective::DefeatUnit {
            unit: UnitId(99),
            turn_limit: None,
        });
        assert_eq!(objective_text(&s, None, Words::ENGLISH), "Defeat the boss");
        // A boss that has fallen keeps its name.
        let mut s = battle_with(
            &c,
            q.map().clone(),
            q.units().to_vec(),
            Objective::DefeatUnit {
                unit: UnitId(4),
                turn_limit: None,
            },
        );
        let mut units = s.units().to_vec();
        units[0].pos = Pos::new(7, 2);
        units[3].hp = 1;
        s = battle_with(&c, q.map().clone(), units, s.objective());
        s.apply(&trpg_core::Command::Act {
            unit: UnitId(1),
            dest: Pos::new(7, 2),
            action: trpg_core::UnitAction::Attack {
                target: UnitId(4),
                slot: 0,
                active: None,
                art: None,
            },
        })
        .unwrap();
        assert!(s.unit(UnitId(4)).is_none());
        assert_eq!(objective_text(&s, None, Words::ENGLISH), "Defeat Brigand");
        let s = with(Objective::Seize {
            pos: Pos::new(5, 5),
            by_lord: true,
            turn_limit: Some(3),
        });
        assert_eq!(
            (
                objective_text(&s, None, Words::ENGLISH),
                turn_text(&s, Words::ENGLISH)
            ),
            ("Seize the Fort".into(), "Turn 1/3".into())
        );
        let s = with(Objective::Seize {
            pos: Pos::new(99, 99),
            by_lord: true,
            turn_limit: None,
        });
        assert_eq!(
            objective_text(&s, None, Words::ENGLISH),
            "Seize the objective"
        );
        let s = with(Objective::Survive { turns: 5 });
        assert_eq!(
            (
                objective_text(&s, None, Words::ENGLISH),
                turn_text(&s, Words::ENGLISH)
            ),
            ("Survive 5 turns".into(), "Turn 1/5".into())
        );
    }

    #[test]
    fn the_question_counts_units() {
        assert_eq!(
            end_turn_question(Words::ENGLISH, 1),
            "End turn with 1 unit ready?"
        );
        assert_eq!(
            end_turn_question(Words::ENGLISH, 3),
            "End turn with 3 units ready?"
        );
    }
}
