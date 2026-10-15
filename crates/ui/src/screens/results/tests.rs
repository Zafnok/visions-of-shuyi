//! Tests of the results screen on rewards made by hand over the Quick
//! Battle's units: the lord (90 EXP) levels up from a 21 EXP bonus, the
//! knight (10 EXP) doesn't, the archer gets none.

use insta::assert_snapshot;
use trpg_core::{BattleState, Event, Faction, StatGains, Unit, UnitId};

use super::*;
use crate::harness::{FRAME_DT, Harness};
use crate::screen::tests::ctx;
use crate::screens::battle::progress::Page;
use crate::screens::battle::quick_battle;
use crate::screens::battle::view::ProgressView;

pub(super) fn state() -> BattleState {
    quick_battle(&ctx().content).unwrap()
}

/// The Quick Battle's first three units: the lord with 90 EXP, the knight with
/// 10 and the archer.
pub(super) fn deployed(s: &BattleState) -> Vec<Unit> {
    let players = s.units().iter().filter(|u| u.faction == Faction::Player);
    let mut units: Vec<Unit> = players.take(3).cloned().collect();
    units[0].exp = 90;
    units[1].exp = 10;
    units
}

/// 500 gold and 3 unused charges: the lord levels up, the knight gains,
/// the archer gets nothing.
pub(super) fn rewards(s: &BattleState) -> BattleRewards {
    let exp = |id, amount| Event::ExpGained {
        unit: UnitId(id),
        amount,
    };
    BattleRewards {
        clear_gold: 500,
        unused_charges: 3,
        bonus_exp: 21,
        deployed: deployed(s),
        events: vec![
            exp(1, 21),
            Event::LeveledUp {
                unit: UnitId(1),
                level: 2,
                gains: StatGains([1, 1, 0, 0, 2, 0, 0]),
            },
            exp(2, 21),
        ],
        ..BattleRewards::default()
    }
}

pub(super) fn screen() -> ResultsScreen {
    let s = state();
    ResultsScreen::new(&rewards(&s), (&s, Words::ENGLISH), 3, 1500)
}

/// One frame of `dt` seconds with `actions` pressed and `held` held.
pub(super) fn frame(s: &mut ResultsScreen, actions: &[Action], held: &[Action], dt: f32) -> bool {
    let input = FrameInput::new(actions.to_vec(), dt, held.to_vec());
    matches!(s.update(&mut ctx(), &input), Transition::Pop)
}

pub(super) fn press(s: &mut ResultsScreen, action: Action) -> bool {
    frame(s, &[action], &[], 0.0)
}

pub(super) fn wait(s: &mut ResultsScreen, dt: f32) -> bool {
    frame(s, &[], &[], dt)
}

/// The EXP each row shows.
fn shown(s: &ResultsScreen) -> Vec<u32> {
    s.rows().iter().map(|r| s.exp_shown(r)).collect()
}

/// The tests (and the game flow) reach the screen through `as_any`.
#[test]
fn it_can_be_found_by_its_type() {
    let s = screen();
    let any = Screen::as_any(&s).expect("opts into as_any");
    assert!(any.downcast_ref::<ResultsScreen>().is_some());
}

#[test]
fn rows_are_the_deployed_units_before_the_bonus() {
    let s = screen();
    assert_eq!(s.name(), "results");
    let rows: Vec<_> = s
        .rows()
        .iter()
        .map(|r| (r.name.as_str(), r.class.as_str(), r.level, r.exp, r.gained))
        .collect();
    assert_eq!(
        rows,
        [
            ("Test Lord", "Exile", 1, 90, 21),
            ("Test Knight", "Guard", 1, 10, 21),
            ("Test Archer", "Archer", 1, 0, 0),
        ]
    );
    // The level up, without the EXP bars the screen shows itself.
    let pages = s.pages().unwrap().pages();
    assert!(matches!(pages, [Page::LevelUp(p)] if p.unit == UnitId(1)));
}

#[test]
fn the_bars_wait_then_fill_over_their_time() {
    let mut s = screen();
    assert_eq!(shown(&s), [90, 10, 0]);
    assert!(!wait(&mut s, INTRO_S));
    assert_eq!(shown(&s), [90, 10, 0]);
    assert!(!s.filled());
    wait(&mut s, FILL_S / 3.0);
    assert_eq!(shown(&s), [97, 17, 0]);
    wait(&mut s, FILL_S / 3.0);
    assert_eq!(shown(&s), [104, 24, 0]);
    assert!(!s.filled());
    wait(&mut s, FILL_S / 3.0 + 0.01);
    assert_eq!(shown(&s), [111, 31, 0]);
    assert!(s.filled());
    // Full bars wait for a press.
    assert!(!wait(&mut s, 10.0));
    assert!(!s.paging());
    // Bad frame times count as nothing.
    let mut s = screen();
    wait(&mut s, f32::NAN);
    wait(&mut s, -1.0);
    wait(&mut s, INTRO_S + FILL_S / 3.0);
    assert_eq!(shown(&s), [97, 17, 0]);
}

#[test]
fn other_timings_wait_and_fill_as_long_as_they_say() {
    let mut s = screen().with_timings(0.25, 4.0);
    wait(&mut s, 0.25);
    assert_eq!(shown(&s), [90, 10, 0]);
    wait(&mut s, 2.0);
    assert_eq!(shown(&s), [100, 20, 0]);
    assert!(!s.filled());
    wait(&mut s, 2.0);
    assert_eq!(shown(&s), [111, 31, 0]);
    assert!(s.filled());
    // No fill time: full once the wait is over.
    let mut s = screen().with_timings(0.25, 0.0);
    assert_eq!(shown(&s), [90, 10, 0]);
    assert!(!s.filled());
    wait(&mut s, 0.25);
    assert_eq!(shown(&s), [111, 31, 0]);
    assert!(s.filled());
}

#[test]
fn holding_confirm_fills_faster_and_a_press_fills_at_once() {
    let mut s = screen();
    frame(&mut s, &[], &[Action::Confirm], INTRO_S / 4.0);
    frame(&mut s, &[], &[Action::Confirm], FILL_S / 12.0);
    assert_eq!(shown(&s), [97, 17, 0]);
    // Holding Cancel doesn't.
    frame(&mut s, &[], &[Action::Cancel], FILL_S / 3.0);
    assert_eq!(shown(&s), [104, 24, 0]);
    assert!(!press(&mut s, Action::Cancel));
    assert_eq!(shown(&s), [111, 31, 0]);
    assert!(s.filled() && !s.paging());
}

#[test]
fn a_press_on_full_bars_shows_the_level_up_which_waits_for_its_own() {
    for key in [Action::Confirm, Action::Cancel] {
        let c = ctx();
        let mut s = screen();
        // Fill, then go on.
        assert!(!press(&mut s, key));
        assert!(!s.paging());
        assert!(matches!(s.view(&c).body, ResultsBody::Tally(_)));
        assert!(!press(&mut s, key));
        assert!(s.paging());
        // The page of the unit the bonus levelled; no stat is revealed yet.
        let ResultsBody::LevelUp(page) = s.view(&c).body else {
            panic!("the level-up page is up");
        };
        let Some(ProgressView::Gains(g)) = page.as_deref() else {
            panic!("{page:?}")
        };
        assert_eq!(
            (g.banner.as_str(), g.name.as_str()),
            ("LEVEL UP!", "Test Lord")
        );
        assert_eq!(stats_shown(page.as_deref()), 0);
        // The help is the page's: it is still revealing its stats.
        assert_eq!(s.view(&c).help, "f skip · hold f fast");
        // It reveals its stats and then waits, however long.
        assert!(!wait(&mut s, 30.0));
        let ResultsBody::LevelUp(page) = s.view(&c).body else {
            panic!("the level-up page is up");
        };
        assert_eq!(stats_shown(page.as_deref()), 7);
        assert_eq!(s.view(&c).help, "f continue");
        assert!(!press(&mut s, Action::CursorDown));
        assert!(press(&mut s, key), "{key:?} closes the last page");
    }
}

/// How many stat rows the level-up `page` shows.
fn stats_shown(page: Option<&ProgressView>) -> usize {
    match page {
        Some(ProgressView::Gains(g)) => g.stats.len(),
        other => panic!("not a level up: {other:?}"),
    }
}

#[test]
fn a_press_mid_reveal_shows_every_stat_before_closing() {
    let c = ctx();
    let mut s = screen();
    press(&mut s, Action::Confirm);
    press(&mut s, Action::Confirm);
    let stats = |s: &ResultsScreen| match s.view(&c).body {
        ResultsBody::LevelUp(page) => stats_shown(page.as_deref()),
        ResultsBody::Tally(_) => panic!("the level-up page is up"),
    };
    assert!(stats(&s) < 7);
    assert!(!press(&mut s, Action::Cancel));
    assert_eq!(stats(&s), 7);
    assert!(press(&mut s, Action::Cancel));
}

#[test]
fn without_a_level_up_the_press_on_full_bars_closes() {
    let s0 = state();
    let mut r = rewards(&s0);
    r.events.remove(1);
    r.deployed[0].exp = 20;
    let mut s = ResultsScreen::new(&r, (&s0, Words::ENGLISH), 3, 1500);
    assert_eq!(s.pages(), None);
    assert!(!press(&mut s, Action::Confirm));
    assert_eq!(shown(&s), [41, 31, 0]);
    assert!(press(&mut s, Action::Confirm));
}

/// The tally of the view, which the screen shows until the bars are
/// done with.
fn tally(s: &ResultsScreen) -> TallyView {
    match s.view(&ctx()).body {
        ResultsBody::Tally(tally) => tally,
        ResultsBody::LevelUp(_) => panic!("the tally is up"),
    }
}

#[test]
fn every_charge_used_shows_no_exp_line_and_closes_on_one_press() {
    let s0 = state();
    let r = BattleRewards {
        clear_gold: 500,
        deployed: deployed(&s0),
        ..BattleRewards::default()
    };
    let mut s = ResultsScreen::new(&r, (&s0, Words::ENGLISH), 3, 500);
    assert!(s.rows().is_empty());
    assert!(s.filled());
    let t = tally(&s);
    let gold = t.gold.unwrap();
    assert_eq!(
        (gold.label.as_str(), gold.gained, gold.now.as_str()),
        ("Gold for clearing the map", 500, "(now 500)")
    );
    assert_eq!(t.bonus, None);
    assert!(t.units.is_empty());
    assert_eq!(s.view(&ctx()).help, "f continue");
    assert!(press(&mut s, Action::Confirm));
}

#[test]
fn a_battle_without_clear_gold_shows_no_gold_line() {
    let s0 = state();
    let mut r = rewards(&s0);
    r.clear_gold = 0;
    let t = tally(&ResultsScreen::new(&r, (&s0, Words::ENGLISH), 3, 40));
    assert_eq!(t.gold, None);
    assert_eq!(t.rewinds.left, "3 of 3");
    let bonus = t.bonus.unwrap();
    assert_eq!(
        (bonus.label.as_str(), bonus.exp),
        ("Bonus EXP for each unit", 21)
    );
}

#[test]
fn the_tally_has_its_heading_and_every_line() {
    let t = tally(&screen());
    assert_eq!(t.heading, ctx().text(HEADING));
    let gold = t.gold.unwrap();
    assert_eq!(
        (gold.label.as_str(), gold.gained, gold.now.as_str()),
        ("Gold for clearing the map", 500, "(now 1500)")
    );
    assert_eq!(t.rewinds.label, ctx().text(REWINDS_LABEL));
    assert_eq!(t.rewinds.left, "3 of 3");
    assert_eq!(t.bonus.unwrap().exp, 21);
    assert_eq!(t.units.len(), 3);
}

#[test]
fn rows_show_the_level_reached_and_mark_the_level_up() {
    let mut s = screen();
    let lord = |s: &ResultsScreen| tally(s).units.remove(0);
    let first = lord(&s);
    assert_eq!(
        (
            first.name.as_str(),
            first.class.as_str(),
            first.level.as_str()
        ),
        ("Test Lord", "Exile", "Lv 1")
    );
    assert_eq!(
        (first.exp_label.as_str(), first.exp, first.exp_max),
        ("EXP", 90, 100)
    );
    assert_eq!(first.level_up, None);
    assert_eq!(s.view(&ctx()).help, "f skip · hold f fast");
    wait(&mut s, INTRO_S + FILL_S * 0.5);
    // 100 EXP exactly: the level is reached.
    assert_eq!(shown(&s)[0], 100);
    let reached = lord(&s);
    assert_eq!((reached.level.as_str(), reached.exp), ("Lv 2", 0));
    assert_eq!(reached.level_up.as_deref(), Some("LEVEL UP"));
    press(&mut s, Action::Confirm);
    let units = tally(&s).units;
    let lord = &units[0];
    assert_eq!((lord.level.as_str(), lord.exp), ("Lv 2", 11));
    assert!(lord.level_up.is_some());
    // The unit at the cap keeps its bar; none of the others levelled.
    assert_eq!((units[1].level.as_str(), units[1].exp), ("Lv 1", 31));
    assert_eq!((units[2].level.as_str(), units[2].exp), ("Lv 1", 0));
    assert!(units[1].level_up.is_none() && units[2].level_up.is_none());
    assert_eq!(s.view(&ctx()).help, "f continue");
}

#[test]
fn every_deployed_unit_gets_a_row() {
    let s0 = state();
    let unit = deployed(&s0).remove(1);
    for n in [7, 8, 40] {
        let mut r = rewards(&s0);
        r.events.clear();
        r.deployed = vec![unit.clone(); n];
        let s = ResultsScreen::new(&r, (&s0, Words::ENGLISH), 3, 0);
        assert_eq!(tally(&s).units.len(), n);
    }
}

#[test]
fn results_snapshot() {
    let mut h = Harness::with_screen(Box::new(screen()));
    h.wait(INTRO_S + FILL_S + FRAME_DT);
    assert_snapshot!(h.snapshot());
}

#[test]
fn results_level_up_snapshot() {
    let mut h = Harness::with_screen(Box::new(screen()));
    h.keys("f f").wait(5.0);
    assert_eq!(h.top_screen(), "results");
    assert_snapshot!(h.snapshot());
    h.keys("f");
    assert_eq!(h.top_screen(), "");
}
