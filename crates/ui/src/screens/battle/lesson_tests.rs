//! Tests of the checklist lessons on the battle screen (ticket 0456): a
//! lesson opens at its trigger when the campaign's level shows it, ticks
//! its items as the player does them in any order and closes when all
//! are; while it is open the turn can't end; rewinding never un-ticks one.

use insta::assert_snapshot;
use proptest::prelude::*;
use trpg_content::{DoneWhen, Lesson, LessonItem, LessonLevel, TipTrigger};
use trpg_core::{Command, Phase, TutorialLevel, UnitAction};

use super::cursor::Cursor;
use super::glyph::{in_top_half, lesson_rect};
use super::layout::MAP_VIEW;
use super::mode::Selection;
use super::testing::{Stacked, skirmish_charged, vaulted, wait};
use super::*;
use crate::audio::AudioRequest;
use crate::glyph_buffer::Rect;
use crate::harness::Harness;
use crate::screen::tests::ctx;

/// The test lesson (`assets-test/data/lessons.ron`).
const TEST_LESSON: &str = "test_cursor";

/// The cursor's four directions, the test lesson's items in order.
const CURSORS: [Action; 4] = [
    Action::CursorUp,
    Action::CursorDown,
    Action::CursorLeft,
    Action::CursorRight,
];

/// A context with the lessons on, the test lesson the only one (the
/// story's lessons, 0457, start in any battle: they have their own tests).
fn lessoned() -> Ctx {
    let mut c = ctx();
    c.game.lessons = crate::screen::Lessons::On;
    c.game
        .content
        .lessons
        .lessons
        .retain(|l| l.id == TEST_LESSON);
    c
}

/// The Quick Battle at its start, as the game flow names it.
fn quick() -> BattleScreen {
    BattleScreen::new(quick_battle(&ctx().game.content).unwrap()).in_battle(QUICK_BATTLE)
}

/// One instant frame with `actions`.
fn press(s: &mut BattleScreen, c: &mut Ctx, actions: &[Action]) {
    s.update(c, &FrameInput::new(actions.to_vec(), 0.0, vec![]));
}

/// A lesson `id` of `level`, started by the first turn, done by `items`.
fn lesson(id: &str, level: LessonLevel, items: &[DoneWhen]) -> Lesson {
    Lesson {
        id: id.into(),
        level,
        trigger: TipTrigger::FirstBattleStart,
        battles: vec![],
        title: "Try it".into(),
        text: "Do these.".into(),
        items: items
            .iter()
            .map(|&done_when| LessonItem {
                label: format!("{done_when:?}"),
                done_when,
            })
            .collect(),
        face: None,
    }
}

/// The menu sound cues `c` heard since last asked.
fn sounds(c: &mut Ctx) -> Vec<String> {
    c.audio
        .take()
        .into_iter()
        .filter_map(|r| match r {
            AudioRequest::PlaySound { cue, .. } => Some(cue),
            _ => None,
        })
        .collect()
}

/// Acceptance: at `Full` the test lesson opens on the first turn, and
/// pressing the four cursor directions in any order ticks them and
/// closes it; the campaign keeps it as done.
#[test]
fn the_test_lesson_opens_on_the_first_turn_and_the_four_directions_close_it() {
    let mut c = lessoned();
    let mut s = quick();
    assert_eq!(s.shown_lesson(), None, "nothing before the first frame");
    press(&mut s, &mut c, &[]);
    assert_eq!(s.shown_lesson(), Some(TEST_LESSON));
    assert!(s.lesson_open());
    let view = s.view(&c).lesson.unwrap();
    assert_eq!(
        (view.title.as_str(), view.heading.as_str()),
        ("Vix", "Moving the cursor")
    );
    assert_eq!(
        view.lines[0],
        "Move the cursor with arrows, one tile each way."
    );
    let labels: Vec<&str> = view.items.iter().map(|i| i.label.as_str()).collect();
    assert_eq!(
        labels,
        [
            "Move it up: Up",
            "Move it down: Down",
            "Move it left: Left",
            "Move it right: Right"
        ]
    );
    assert_eq!(view.progress, "0 of 4 done");
    // Right, then left: ticked in the order of the list, not the presses.
    press(&mut s, &mut c, &[Action::CursorRight]);
    press(&mut s, &mut c, &[Action::CursorLeft, Action::CursorRight]);
    assert_eq!(s.lesson_ticks(), [false, false, true, true]);
    let view = s.view(&c).lesson.unwrap();
    assert_eq!(view.progress, "2 of 4 done");
    let done: Vec<bool> = view.items.iter().map(|i| i.done).collect();
    assert_eq!(done, [false, false, true, true]);
    // The map works under it: the cursor moved.
    assert_eq!(s.cursor().pos, Pos::new(4, 5));
    press(&mut s, &mut c, &[Action::CursorDown]);
    assert!(s.lesson_open());
    press(&mut s, &mut c, &[Action::CursorUp]);
    assert_eq!(s.shown_lesson(), None);
    assert!(!s.lesson_open());
    assert!(c.game.campaign_lessons.contains(TEST_LESSON));
    assert_eq!(s.view(&c).lesson, None);
    // The first turn goes on: it doesn't come back.
    press(&mut s, &mut c, &[Action::CursorUp]);
    assert_eq!(s.shown_lesson(), None);
    // Nor in another battle of the campaign.
    let mut again = quick();
    press(&mut again, &mut c, &[]);
    assert_eq!(again.shown_lesson(), None);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// Pressing the cursor directions in any order, with repeats and
    /// other keys in between, closes the test lesson exactly when the
    /// last of the four was pressed.
    #[test]
    fn any_order_closes_the_lesson_exactly_when_all_four_are_pressed(
        order in prop::collection::vec(0usize..6, 0..16),
    ) {
        let mut c = lessoned();
        let mut s = quick();
        press(&mut s, &mut c, &[]);
        let mut seen = std::collections::BTreeSet::new();
        for i in order {
            // Info and Cancel come and go; neither is on the list.
            let action = match i {
                0..4 => CURSORS[i],
                4 => Action::Info,
                _ => Action::Cancel,
            };
            press(&mut s, &mut c, &[action]);
            if i < 4 {
                seen.insert(i);
            }
            prop_assert_eq!(s.shown_lesson().is_none(), seen.len() == 4);
            prop_assert_eq!(c.game.campaign_lessons.contains(TEST_LESSON), seen.len() == 4);
            // Back to browsing for the next key.
            press(&mut s, &mut c, &[Action::Cancel, Action::Cancel]);
        }
    }
}

/// Acceptance: while the lesson is open, End Turn is refused with the
/// "denied" sound, from the key and from the map menu; once it closes, End
/// Turn works.
#[test]
fn the_turn_waits_for_the_open_lesson() {
    let mut c = lessoned();
    let mut s = quick();
    press(&mut s, &mut c, &[]);
    sounds(&mut c);
    press(&mut s, &mut c, &[Action::EndTurn]);
    assert_eq!(s.mode(), &Mode::default(), "no End turn prompt");
    assert_eq!(sounds(&mut c), [crate::audio::AUDIO.cue(MenuSound::Denied)]);
    // The map menu's End turn: the menu stays open.
    let menu = map_menu::map_menu(&s.state, map_menu::MapEntry::EndTurn);
    s.mode = Mode::MapMenu { menu };
    press(&mut s, &mut c, &[Action::Confirm]);
    assert!(matches!(s.mode(), Mode::MapMenu { .. }), "{:?}", s.mode());
    assert_eq!(sounds(&mut c), [crate::audio::AUDIO.cue(MenuSound::Denied)]);
    press(&mut s, &mut c, &[Action::Cancel]);
    assert_eq!(s.state().phase(), Phase::Player);
    // Done: End turn asks (units are ready), and yes ends the phase.
    press(&mut s, &mut c, &CURSORS);
    assert!(!s.lesson_open());
    press(&mut s, &mut c, &[Action::EndTurn]);
    assert!(matches!(s.mode(), Mode::EndTurnPrompt { .. }));
    press(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(s.state().phase(), Phase::Enemy);
}

/// While the lesson is open, auto-end waits; it ends the phase once the
/// lesson closes. A lesson not yet open (here: before its first frame)
/// holds nothing.
#[test]
fn auto_end_waits_for_the_open_lesson() {
    let mut c = lessoned();
    let _ = c.change_settings(|s| s.game.auto_end_turn = true);
    let mut s = quick();
    press(&mut s, &mut c, &[]);
    let ready: Vec<Unit> = s
        .state()
        .units()
        .iter()
        .filter(|u| u.faction == Faction::Player && !u.acted)
        .cloned()
        .collect();
    for u in &ready {
        let action = UnitAction::Wait;
        s.send(&Command::Act {
            unit: u.id,
            dest: u.pos,
            action,
        });
    }
    for _ in 0..5 {
        press(&mut s, &mut c, &[]);
    }
    assert_eq!(s.state().phase(), Phase::Player, "auto-end waits");
    press(&mut s, &mut c, &CURSORS);
    assert_ne!(s.state().phase(), Phase::Player, "and then ends the phase");
}

/// Acceptance: at `None` no lesson opens, and a `Full` lesson doesn't
/// open at `Short` (a `Short` one does). Nor does one in a battle it isn't
/// for, or with the lessons off.
#[test]
fn levels_battles_and_the_switch_gate_the_lessons() {
    /// The lesson `s` shows after its first frame.
    fn shown(c: &mut Ctx, mut s: BattleScreen) -> Option<String> {
        press(&mut s, c, &[]);
        s.shown_lesson().map(str::to_owned)
    }
    let mut c = lessoned();
    for (level, test) in [
        (TutorialLevel::Full, true),
        (TutorialLevel::Short, false),
        (TutorialLevel::None, false),
    ] {
        c.game.campaign_tutorial = Some(level);
        let open = shown(&mut c, quick());
        assert_eq!(open.as_deref() == Some(TEST_LESSON), test, "{level:?}");
    }
    // A `Short` lesson (no battle named: any) at Short and Full, not None.
    let short = lesson("short", LessonLevel::Short, &[DoneWhen::Attacked]);
    c.game.content.lessons.lessons.insert(0, short);
    for (level, open) in [
        (TutorialLevel::Full, Some("short")),
        (TutorialLevel::Short, Some("short")),
        (TutorialLevel::None, None),
    ] {
        c.game.campaign_tutorial = Some(level);
        assert_eq!(shown(&mut c, quick()).as_deref(), open, "{level:?}");
    }
    c.game.content.lessons.lessons.remove(0);
    // The test lesson is the test and Quick battles' only.
    c.game.campaign_tutorial = Some(TutorialLevel::Full);
    let plain = BattleScreen::new(quick_battle(&c.game.content).unwrap());
    assert_eq!(shown(&mut c, plain), None);
    let other = quick().in_battle("ch01");
    assert_eq!(shown(&mut c, other), None);
    c.game.lessons = crate::screen::Lessons::Off;
    assert_eq!(shown(&mut c, quick()), None);
}

/// A screen on `state` whose only lesson, `only`, asks for `items`, open
/// after its first frame.
fn only(c: &mut Ctx, state: BattleState, items: &[DoneWhen]) -> BattleScreen {
    c.game.content.lessons.lessons = vec![lesson("only", LessonLevel::Full, items)];
    let mut s = BattleScreen::new(state);
    press(&mut s, c, &[]);
    assert!(s.lesson_open());
    s
}

/// Moving a unit after its action (Vault) counts as moving it.
#[test]
fn moving_after_an_action_ticks_the_move() {
    let mut c = lessoned();
    let state = vaulted(&c);
    let mut s = only(&mut c, state, &[DoneWhen::UnitMoved, DoneWhen::Attacked]);
    let Mode::MoveAfter { unit, tiles } = s.mode().clone() else {
        panic!("not moving after: {:?}", s.mode());
    };
    let here = s.state().unit(unit).map(|u| u.pos);
    let to = tiles.into_iter().find(|&t| Some(t) != here);
    s.send(&Command::MoveAfter { unit, to });
    press(&mut s, &mut c, &[]);
    assert_eq!(s.lesson_ticks(), [true, false]);
}

/// Selecting ticks when a unit becomes selected: one already selected
/// when the lesson opens doesn't count until it is selected again.
#[test]
fn a_selection_from_before_the_lesson_doesnt_count() {
    let mut c = lessoned();
    let mut s = quick();
    let lord = s.state().units()[0].id;
    s.mode = Mode::Selected(Selection::new(s.state(), lord).expect("the lord can act"));
    c.game.content.lessons.lessons =
        vec![lesson("only", LessonLevel::Full, &[DoneWhen::UnitSelected])];
    press(&mut s, &mut c, &[]);
    assert!(s.lesson_open());
    press(&mut s, &mut c, &[]);
    press(&mut s, &mut c, &[Action::CursorRight]);
    assert!(matches!(s.mode(), Mode::Selected(_)));
    assert_eq!(s.lesson_ticks(), [false]);
    // Backed out, then selected again.
    press(&mut s, &mut c, &[Action::CursorLeft, Action::Cancel]);
    assert_eq!(s.mode(), &Mode::default());
    press(&mut s, &mut c, &[Action::Confirm]);
    assert!(!s.lesson_open(), "done: {:?}", s.lesson_ticks());
}

/// A lesson that starts at the forecast doesn't count the forecast that
/// started it; opening it again does.
#[test]
fn the_forecast_that_starts_a_lesson_doesnt_count() {
    let state = skirmish_charged(&ctx(), 30, 0);
    let mut h = Harness::with_screen(Box::new(BattleScreen::new(state)));
    h.with_lessons();
    let mut at = lesson("forecast", LessonLevel::Full, &[DoneWhen::ForecastOpened]);
    at.trigger = TipTrigger::FirstForecast;
    h.ctx_mut().game.content.lessons.lessons = vec![at];
    let battle = |h: &Harness| h.battle().expect("a battle").clone();
    // The lord moved one step right, `Attack`: the forecast.
    h.keys("f Right f").wait(0.5).keys("f").wait(0.2);
    assert!(matches!(battle(&h).mode(), Mode::Targeting(_)));
    assert!(battle(&h).lesson_open());
    assert_eq!(battle(&h).lesson_ticks(), [false]);
    // Back to the menu, and the forecast again.
    h.keys("d").wait(0.1).keys("f").wait(0.1);
    assert!(matches!(battle(&h).mode(), Mode::Targeting(_)));
    assert!(!battle(&h).lesson_open());
}

/// The test battle as the game flow starts it: its notes, then turn 1's
/// banner, then the lesson. A lesson waits for each, as tips do, and
/// opens once the map is the player's.
#[test]
fn the_lesson_waits_for_the_notes_and_the_banner() {
    let mut c = lessoned();
    let def = &c.game.content.battles["test"];
    let lead = LeadProfile::new(DEFAULT_NAME, LeadGender::Male);
    let campaign = battle_campaign(&c.game.content, def, GameMode::Classic, lead);
    let setup = campaign.battle_setup(def, &c.game.content.tables());
    let (state, events) = BattleState::new(setup);
    let mut s = BattleScreen::start(state, &events).in_battle("test");
    press(&mut s, &mut c, &[]);
    assert!(s.notes_open());
    assert_eq!(s.shown_lesson(), None);
    assert!(!s.lesson_open());
    press(&mut s, &mut c, &[Action::Confirm]);
    assert!(!s.notes_open() && s.banner().is_some());
    assert_eq!(s.shown_lesson(), None);
    assert!(!s.lesson_open());
    // Keys while waiting tick nothing.
    press(&mut s, &mut c, &[Action::CursorUp]);
    press(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(s.banner(), None);
    assert_eq!(s.shown_lesson(), Some(TEST_LESSON));
    assert!(s.lesson_open());
    assert_eq!(s.lesson_ticks(), [false; 4]);
}

/// A tip of the same moment shows first; the lesson opens once it is
/// closed, and from then on tips wait while the lesson is on screen.
#[test]
fn a_tip_comes_first_then_tips_wait_for_the_open_lesson() {
    let mut c = lessoned();
    c.game.tips_enabled = true;
    let mut s = quick();
    press(&mut s, &mut c, &[]);
    assert_eq!(s.shown_tip(), Some(TipTrigger::FirstBattleStart));
    assert_eq!(s.shown_lesson(), None);
    assert!(!s.lesson_open());
    // The tip takes the keys: the lesson isn't open, nothing ticks.
    press(&mut s, &mut c, &[Action::CursorUp]);
    press(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(s.shown_tip(), None);
    assert_eq!(s.shown_lesson(), Some(TEST_LESSON));
    assert!(s.lesson_open());
    assert_eq!(s.lesson_ticks(), [false; 4]);
    // Selecting the lord fires the "Moving a unit" tip: it waits.
    press(&mut s, &mut c, &[Action::Confirm]);
    assert!(matches!(s.mode(), Mode::Selected(_)));
    assert_eq!(s.shown_tip(), None);
    assert_eq!(s.view(&c).tip, None);
    // Keys go to the map, not the tip.
    press(&mut s, &mut c, &CURSORS);
    assert_eq!(s.shown_lesson(), None);
    assert_eq!(s.shown_tip(), Some(TipTrigger::FirstUnitSelected));
}

/// Every condition a lesson can ask for is ticked where the screen sees
/// it happen: selecting a unit, picking where it goes, the forecast, an
/// attack, the rewind screen (where the lesson stays on screen), and
/// ending the turn, which a lesson with that item doesn't hold.
#[test]
fn each_condition_ticks_where_it_happens() {
    let items = [
        DoneWhen::UnitSelected,
        DoneWhen::UnitMoved,
        DoneWhen::ForecastOpened,
        DoneWhen::Attacked,
        DoneWhen::RewindOpened,
        DoneWhen::TurnEnded,
    ];
    let mut h = Harness::with_screen(Box::new(BattleScreen::new(skirmish_charged(&ctx(), 30, 2))));
    h.with_lessons();
    h.ctx_mut().game.content.lessons.lessons = vec![lesson("all", LessonLevel::Full, &items)];
    let ticks = |h: &Harness| {
        h.battle()
            .map(|b| b.lesson_ticks().to_vec())
            .unwrap_or_default()
    };
    h.wait(0.1);
    assert_eq!(h.battle().and_then(BattleScreen::shown_lesson), Some("all"));
    // The lord selected, moved one step right, its action menu open.
    h.keys("f");
    assert_eq!(ticks(&h), [true, false, false, false, false, false]);
    h.keys("Right f").wait(0.5);
    assert_eq!(ticks(&h), [true, true, false, false, false, false]);
    // `Attack`: the forecast; then the attack, played out.
    h.keys("f");
    assert_eq!(ticks(&h), [true, true, true, false, false, false]);
    h.keys("f")
        .wait(6.0)
        .keys("f")
        .wait(2.0)
        .keys("f")
        .wait(1.0);
    assert!(
        h.battle()
            .is_some_and(|b| matches!(b.mode(), Mode::Idle { .. }))
    );
    assert_eq!(ticks(&h), [true, true, true, true, false, false]);
    // The rewind screen: the lesson stays over the map there.
    h.keys("r");
    let rewind = h.screen::<RewindScreen>().expect("the rewind screen");
    assert!(rewind.view(h.game().ctx()).lesson.is_some());
    assert_eq!(ticks(&h), [true, true, true, true, true, false]);
    h.keys("d");
    // End turn isn't held, and ends the lesson.
    h.keys("Space f");
    assert_eq!(h.battle().map(|b| b.state().phase()), Some(Phase::Enemy));
    assert!(h.game().ctx().game.campaign_lessons.contains("all"));
}

/// Rewinding never un-ticks an item nor reopens a lesson that is done.
#[test]
fn rewinding_keeps_the_ticks_and_the_lessons_done() {
    let mut c = lessoned();
    let state = skirmish_charged(&c, 30, 3);
    let mut s = Stacked::new(BattleScreen::new(state).in_battle(QUICK_BATTLE));
    s.frame(&mut c, &[], 0.0);
    s.frame(&mut c, &[Action::CursorUp, Action::CursorLeft], 0.0);
    let mut waited = s.state().clone();
    wait(&mut waited, 1);
    let knight = &s.state().units()[1];
    let (unit, dest) = (knight.id, knight.pos);
    s.battle_mut().send(&Command::Act {
        unit,
        dest,
        action: UnitAction::Wait,
    });
    assert_eq!(s.state(), &waited);
    // Rewind, the one point, yes.
    s.press(&mut c, &[Action::Rewind, Action::Confirm, Action::Confirm]);
    assert!(s.rewind().is_none());
    assert_eq!(s.history().len(), 0);
    assert_eq!(s.lesson_ticks(), [true, false, true, false]);
    assert!(s.lesson_open());
    s.frame(&mut c, &[Action::CursorDown, Action::CursorRight], 0.0);
    assert!(c.game.campaign_lessons.contains(TEST_LESSON));
    // Done, then a rewind back to the first turn: it stays done.
    s.battle_mut().send(&Command::Act {
        unit,
        dest,
        action: UnitAction::Wait,
    });
    s.press(&mut c, &[Action::Rewind]);
    s.frame(&mut c, &[Action::Confirm, Action::Confirm], 0.0);
    assert_eq!(s.history().len(), 0);
    s.frame(&mut c, &[], 0.0);
    assert_eq!(s.shown_lesson(), None);
}

/// The box goes against the map's top edge, or its bottom edge while the
/// cursor is in the top half of the map: it never hides the cursor's tile,
/// wherever on the map that is.
#[test]
fn the_box_never_hides_the_cursor() {
    let mut c = lessoned();
    let mut s = quick();
    press(&mut s, &mut c, &[]);
    let tiles = &s.state().map().tiles;
    let (w, h) = (i32::from(tiles.width()), i32::from(tiles.height()));
    let mut places = std::collections::BTreeSet::new();
    for y in 0..h {
        for x in 0..w {
            s.cursor = Cursor::new(Pos::new(x, y));
            s.follow(Pos::new(x, y));
            let view = s.view(&c);
            let rect = lesson_rect(&c, &view).expect("the lesson's box");
            let cell = c
                .game
                .map_skin
                .tile_cells(&view.scene, MAP_VIEW, Pos::new(x, y));
            let cell = cell.expect("the cursor's tile in view");
            let apart = cell.y + cell.h <= rect.y || rect.y + rect.h <= cell.y;
            assert!(apart, "({x}, {y}): {cell:?} under {rect:?}");
            // Down exactly while the cursor's row (a glyph tile is one) is
            // in the top half.
            assert_eq!(cell.h, 1);
            let middle = MAP_VIEW.y + MAP_VIEW.h / 2;
            let at_bottom = rect.y > MAP_VIEW.y;
            assert_eq!(at_bottom, cell.y < middle, "({x}, {y}): {cell:?}");
            places.insert(at_bottom);
        }
    }
    let middle = MAP_VIEW.y + MAP_VIEW.h / 2;
    let top_row = c
        .game
        .map_skin
        .tile_cells(&s.scene(&c), MAP_VIEW, Pos::new(0, 0))
        .map(|r| r.y);
    // The test map's first row is in the top half: the box went down.
    assert!(top_row.is_some_and(|y| y < middle) && places.contains(&true));
}

/// A tile is in the map's top half when its middle row is: with tiles
/// taller than a row (a sprite skin's), what counts is their middle.
#[test]
fn a_tile_is_high_by_its_middle_row() {
    let middle = MAP_VIEW.y + MAP_VIEW.h / 2;
    let tile = |y, h| Rect::new(0, y, 2, h);
    assert!(in_top_half(tile(middle - 1, 1)));
    assert!(!in_top_half(tile(middle, 1)));
    // Rows 13 and 14, middle 14: high. Rows 14 and 15, middle 15: low.
    assert!(in_top_half(tile(middle - 2, 2)));
    assert!(!in_top_half(tile(middle - 1, 2)));
    // Rows 12 to 15, middle 14: high; rows 13 to 16, middle 15: low.
    assert!(in_top_half(tile(middle - 3, 4)));
    assert!(!in_top_half(tile(middle - 2, 4)));
}

/// The lesson's box over the Quick Battle, two of its four items done.
#[test]
fn the_lesson_box_snapshot() {
    let mut h = Harness::with_screen(Box::new(quick()));
    h.with_lessons();
    h.ctx_mut().game.content.lessons = lessoned().game.content.lessons;
    // Up from its first frame, when the box comes up.
    h.wait(0.1).keys("Left Up");
    let battle = h.battle().expect("a battle");
    assert_eq!(battle.lesson_ticks(), [true, false, true, false]);
    assert_snapshot!(h.snapshot());
}

/// A Harness on `state` whose only lesson asks for `items`, open after a
/// short wait.
fn harness_only(state: BattleState, items: &[DoneWhen]) -> Harness {
    let mut h = Harness::with_screen(Box::new(BattleScreen::new(state)));
    h.with_lessons();
    h.ctx_mut().game.content.lessons.lessons = vec![lesson("only", LessonLevel::Full, items)];
    h.wait(0.1);
    assert!(h.battle().is_some_and(BattleScreen::lesson_open));
    h
}

fn harness_ticks(h: &Harness) -> Vec<bool> {
    h.battle()
        .map(|b| b.lesson_ticks().to_vec())
        .unwrap_or_default()
}

/// A heal aimed at an ally is no tile aimed at: only a spell on a tile
/// ticks `TileAimed` (0457).
#[test]
fn a_heal_on_an_ally_is_no_tile_aimed() {
    let c = ctx();
    let (map, mut units) = super::testing::quick_units(&c);
    // The archer two tiles left of the mage, hurt.
    units[2].pos = Pos::new(1, 6);
    units[2].hp = 5;
    let state = super::testing::battle_with(
        &c,
        map,
        units,
        trpg_core::Objective::Rout { turn_limit: None },
    );
    let mut h = harness_only(state, &[DoneWhen::TileAimed, DoneWhen::Attacked]);
    // The mage selected, pointed at the archer: the heal list, then the
    // heal's preview on her.
    h.keys("Down f Left Left f").wait(0.5).keys("f");
    let battle = h.battle().expect("a battle");
    assert!(
        matches!(battle.mode(), Mode::CastTarget(_)),
        "{:?}",
        battle.mode()
    );
    assert_eq!(harness_ticks(&h), [false, false]);
}

/// Picking an action from the menu is no move: only leaving the selection
/// for where the unit goes ticks `UnitMoved` (0457). The second lesson
/// opens over the action menu, then `Attack` opens the forecast.
#[test]
fn picking_an_action_after_the_move_is_no_move() {
    let mut h = Harness::with_screen(Box::new(BattleScreen::new(skirmish_charged(&ctx(), 30, 2))));
    h.with_lessons();
    let mut menu = lesson("menu", LessonLevel::Full, &[DoneWhen::ActionMenuOpened]);
    menu.title = "First".into();
    let moved = lesson(
        "moved",
        LessonLevel::Full,
        &[DoneWhen::UnitMoved, DoneWhen::Attacked],
    );
    h.ctx_mut().game.content.lessons.lessons = vec![menu, moved];
    h.wait(0.1);
    h.keys("f Right f").wait(0.5);
    assert_eq!(
        h.battle().and_then(BattleScreen::shown_lesson),
        Some("moved")
    );
    h.keys("f");
    let battle = h.battle().expect("a battle");
    assert!(
        matches!(battle.mode(), Mode::Targeting(_)),
        "{:?}",
        battle.mode()
    );
    assert_eq!(harness_ticks(&h), [false, false]);
}

/// The threat arrows on a unit the cursor rests on don't tick
/// `ThreatPointed`: only a selected unit's path does (0457).
#[test]
fn threat_arrows_while_browsing_dont_tick_the_path_items() {
    let mut c = lessoned();
    let state = quick_battle(&c.game.content).unwrap();
    let mut s = only(
        &mut c,
        state,
        &[DoneWhen::ThreatPointed, DoneWhen::SafePointed],
    );
    // The cursor on the lord, whom a brigand reaches: arrows, browsing.
    assert!(s.scene(&c).threat.is_some());
    press(&mut s, &mut c, &[]);
    assert_eq!(s.lesson_ticks(), [false, false]);
    // Selected, on his own tile: the path points at a tile with arrows.
    press(&mut s, &mut c, &[Action::Confirm]);
    press(&mut s, &mut c, &[]);
    assert_eq!(s.lesson_ticks(), [true, false]);
}

/// `SecondTurn` fires in turn 2's player phase, not in its enemy phase.
#[test]
fn the_second_turn_starts_in_its_player_phase_only() {
    let c = ctx();
    let mut state = quick_battle(&c.game.content).unwrap();
    let mut seen = Vec::new();
    while !(state.turn() == 2 && state.phase() == Phase::Enemy) {
        let fired = BattleScreen::new(state.clone()).lesson_triggers_now();
        seen.push((
            state.turn(),
            state.phase(),
            fired.contains(&TipTrigger::SecondTurn),
        ));
        state.apply(&Command::EndPhase).unwrap();
    }
    let fired = BattleScreen::new(state.clone()).lesson_triggers_now();
    assert!(!fired.contains(&TipTrigger::SecondTurn));
    let second: Vec<_> = seen.iter().filter(|(_, _, f)| *f).collect();
    assert_eq!(second, [&(2, Phase::Player, true)]);
}

/// Ticket 0464: lessons are settled only in the player's phase. In the
/// enemy's no unit of the player's is left to act, yet a lesson waiting
/// for a unit stays for the player's next phase.
#[test]
fn the_enemy_phase_keeps_a_lesson_waiting_for_a_unit() {
    let mut c = lessoned();
    c.game.content.lessons.lessons =
        vec![lesson("unit", LessonLevel::Full, &[DoneWhen::UnitSelected])];
    let mut s = quick();
    s.send(&Command::EndPhase);
    assert_eq!(s.state().phase(), Phase::Enemy);
    s.lessons.fire(TipTrigger::FirstBattleStart);
    s.lessons.absorb(&c, None, s.state().turn());
    s.settle_lessons();
    assert_eq!(s.lessons.current(), Some("unit"));
}
