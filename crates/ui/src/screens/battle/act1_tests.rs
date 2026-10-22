//! The battle screen and Act 1's battle rules (ticket 0319): an escape
//! map's objective line, its `Escape` entry and its exit area on the map;
//! the forecast against a unit that can't be killed, and that unit drawn
//! down on its tile once beaten.

use insta::assert_snapshot;
use trpg_core::{BattleState, Command, Defeat, Objective, Pos, TileRect, UnitAction, UnitId};

use super::BattleScreen;
use super::map_menu::objective_text;
use super::mode::{MenuEntry, Mode};
use super::testing::{Values, battle_with};
use super::view::ForecastView;
use crate::harness::Harness;
use crate::map_view::scene::range;
use crate::screen::tests::ctx;
use crate::words::Words;

/// The Quick Battle as an escape map whose exit is the lord's own tile
/// (3, 5) and the one right of it.
fn escape() -> BattleState {
    let c = ctx();
    let quick = super::quick_battle(&c.game.content).unwrap_or_else(|e| panic!("{e}"));
    let area = TileRect {
        x: 3,
        y: 5,
        w: 2,
        h: 1,
    };
    let objective = Objective::Escape { area };
    battle_with(&c, quick.map().clone(), quick.units().to_vec(), objective)
}

#[test]
fn the_escape_objective_counts_who_is_out() {
    let mut s = escape();
    let text = |s: &BattleState| objective_text(s, Words::ENGLISH);
    assert_eq!(text(&s), "Get everyone to the exit (0/4 out)");
    let out = Command::Act {
        unit: UnitId(1),
        dest: Pos::new(3, 5),
        action: UnitAction::Escape,
    };
    s.apply(&out).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(text(&s), "Get everyone to the exit (1/4 out)");
}

#[test]
fn the_action_menu_offers_escape_in_the_exit_area_but_never_opens_on_it() {
    let mut h = Harness::with_screen(Box::new(BattleScreen::new(escape())));
    // The lord, staying on its tile of the exit.
    h.keys("f f").wait(0.5);
    let Some(Mode::ActionMenu { menu, .. }) = h.battle().map(BattleScreen::mode) else {
        panic!("no action menu");
    };
    assert!(
        menu.values().contains(&MenuEntry::Escape),
        "{:?}",
        menu.values()
    );
    assert_ne!(menu.value(menu.focus()), Some(&MenuEntry::Escape));
    // Off the exit: no Escape.
    let mut h = Harness::with_screen(Box::new(BattleScreen::new(escape())));
    h.keys("f Up f").wait(0.5);
    let Some(Mode::ActionMenu { menu, .. }) = h.battle().map(BattleScreen::mode) else {
        panic!("no action menu");
    };
    assert!(!menu.values().contains(&MenuEntry::Escape));
}

#[test]
fn the_exit_area_is_on_the_map() {
    let h = Harness::with_screen(Box::new(BattleScreen::new(escape())));
    assert!(h.tints_at(Pos::new(3, 5)).contains(&range::EXIT));
    assert!(h.tints_at(Pos::new(4, 5)).contains(&range::EXIT));
    assert!(!h.tints_at(Pos::new(5, 5)).contains(&range::EXIT));
}

/// The attack skirmish (the lord at (6, 2) next to a brigand at (8, 2)
/// with `brigand_hp` HP) with that brigand unkillable or not.
fn skirmish(brigand_hp: i32, defeat: Defeat) -> BattleState {
    let c = ctx();
    let quick = super::quick_battle(&c.game.content).unwrap_or_else(|e| panic!("{e}"));
    let mut units = quick.units().to_vec();
    units[0].pos = Pos::new(6, 2);
    units[2].pos = Pos::new(8, 4);
    units[3].hp = brigand_hp;
    units[3].defeat = defeat;
    battle_with(
        &c,
        quick.map().clone(),
        units,
        Objective::Rout { turn_limit: None },
    )
}

/// The lord at (7, 2) aiming at the brigand.
fn aiming(state: BattleState) -> Harness {
    let mut h = Harness::with_screen(Box::new(BattleScreen::new(state)));
    h.keys("f Right f").wait(0.5);
    h.keys("f Down");
    h
}

fn forecast(h: &Harness) -> ForecastView {
    let battle = h.battle().expect("a battle");
    battle.view(h.game().ctx()).forecast.expect("a forecast")
}

/// The forecast against a unit that can't be killed is the normal one: its
/// HP reaches 0 (Nick, 0069).
#[test]
fn the_forecast_against_a_unit_that_cant_be_killed_is_the_normal_one() {
    let unkillable = aiming(skirmish(5, Defeat::GoesDown));
    let plain = aiming(skirmish(5, Defeat::Falls));
    assert_eq!(forecast(&unkillable), forecast(&plain));
    assert_snapshot!(unkillable.snapshot());
}

#[test]
fn a_unit_that_cant_be_killed_is_drawn_down_on_its_tile() {
    let mut h = aiming(skirmish(5, Defeat::GoesDown));
    h.keys("f").wait(5.0);
    let s = h.battle().expect("a battle").state();
    assert_eq!(s.down().len(), 1, "{:?}", s.fallen());
    let at = s.down()[0].pos;
    let unit = h.unit_at(at).expect("still drawn");
    assert!(unit.is_down());
    assert_eq!(unit.hp.0, 0);
}
