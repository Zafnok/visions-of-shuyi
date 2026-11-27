//! The one-time tips (ticket 0406) on the shipped Chapter 1 at `Full`
//! from a fresh profile, while the lessons (0457) run (ticket 0467): a
//! tip whose trigger fires while a lesson is open doesn't show then, and
//! shows the next time its trigger fires with no lesson up (ticket 1292);
//! a tip counts as seen only once it has been on screen.
//!
//! Tests of the game's content (ADR-0118 §1 (a)): the game's own tips and
//! lessons in its first battle. Each starts with the guard, and skips
//! without the game's content; the tips' mechanism is tested on the test
//! files (`lessons.rs`, `battle.rs`, `crates/ui/src/tips.rs`).

use trpg_content::TipTrigger;
use trpg_content::bundle::skip_without_the_game;
use trpg_core::{Pos, TutorialLevel};
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;
use trpg_ui::screens::battle::mode::Mode;
use crate::support::{battle, skip_scenes};

// Chapter 1's party where it starts (`assets-private/content/battles/ch01.ron`).
const LEAD: Pos = Pos::new(4, 12);
/// The `heretic`, a Mage: its action menu has Magic.
const MAGE: Pos = Pos::new(4, 13);
/// A forest tile on Chapter 1's map (`maps/ch01.map`).
const FOREST: Pos = Pos::new(2, 10);

/// Chapter 1 from New Game in Classic at `Full` on a fresh profile, with
/// the tips and lessons on: turn 1, its scenes skipped and its banners
/// closed, the cursor on the lead and the cursor lesson open, with no tip
/// before it (1304).
fn chapter_1() -> Harness {
    let mut h = Harness::with_layout(RIGHT_HANDED).with_shipped_story();
    h.with_lessons().with_tips();
    h.keys("f f f Up f");
    skip_scenes(&mut h);
    // The objective's banner, then `PLAYER PHASE`.
    h.keys("f f");
    skip_scenes(&mut h);
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    h.wait(0.1);
    let campaign = h
        .flow()
        .and_then(|f| f.campaign_now(h.game().ctx()))
        .map(|c| c.tutorial);
    assert_eq!(campaign, Some(TutorialLevel::Full));
    assert_eq!(close_tips(&mut h), []);
    assert_eq!(h.cursor_tile(), Some(LEAD));
    assert_eq!(battle(&h).shown_lesson(), Some("cursor"));
    h
}

/// Moves the cursor from where the map shows it to `to` with the arrow
/// keys (which, with the cursor lesson open, tick its directions).
fn go(h: &mut Harness, to: Pos) {
    let at = h.cursor_tile().unwrap_or_else(|| panic!("no cursor"));
    let (dx, dy) = (to.x - at.x, to.y - at.y);
    for _ in 0..dx.abs() {
        h.keys(if dx < 0 { "Left" } else { "Right" });
    }
    for _ in 0..dy.abs() {
        h.keys(if dy < 0 { "Up" } else { "Down" });
    }
}

/// Closes every tip that comes on screen, in turn, returning their
/// triggers. The next in its moment comes up as one closes (1292).
fn close_tips(h: &mut Harness) -> Vec<TipTrigger> {
    let mut seen = Vec::new();
    for _ in 0..20 {
        h.wait(0.1);
        let Some(tip) = battle(h).shown_tip() else {
            return seen;
        };
        seen.push(tip);
        h.keys("f");
    }
    panic!("the tips never ran out: {seen:?}");
}

/// Backs out to the map with Cancel, closing each tip that comes up on
/// the way, and returns their triggers.
fn back_to_map(h: &mut Harness) -> Vec<TipTrigger> {
    let mut seen = Vec::new();
    for _ in 0..8 {
        seen.extend(close_tips(h));
        if matches!(battle(h).mode(), Mode::Idle { .. }) {
            return seen;
        }
        h.keys("d").wait(0.1);
    }
    panic!("never back on the map: {:?}", battle(h).mode());
}

/// Finishes the cursor lesson: the four directions, the cursor ending
/// back where it was.
fn finish_cursor_lesson(h: &mut Harness) {
    if battle(h).shown_lesson() == Some("cursor") {
        h.keys("Up Down Left Right");
    }
    assert_ne!(battle(h).shown_lesson(), Some("cursor"));
}

/// Finishes turn 1's run of lessons (the cursor, the keys, the next unit,
/// selecting) and backs out to the map, the cursor on the lead, returning
/// the tips that came up after the last lesson (in their moment, 1292). A
/// lesson done goes straight to the next, never to a tip between them.
fn finish_turn_1_lessons(h: &mut Harness) -> Vec<TipTrigger> {
    finish_cursor_lesson(h);
    assert_eq!(battle(h).shown_lesson(), Some("keys"));
    h.keys("Right Left");
    // Options from the map menu, and back to the map.
    h.keys("d Down Down Down f d d");
    assert_eq!(battle(h).shown_tip(), None);
    assert_eq!(battle(h).shown_lesson(), Some("next_unit"));
    h.keys("s");
    go(h, LEAD);
    h.keys("e d");
    assert_eq!(battle(h).shown_lesson(), Some("moving"));
    h.keys("f Up f").wait(1.0);
    assert_eq!(battle(h).shown_lesson(), None);
    back_to_map(h)
}

/// Tickets 0466 and 1304: on a map that can zoom, the zoom lesson stays
/// away through the turn-1 lessons (cursor, keys, next unit, moving) and
/// the action menu the last one ends in, opens back on the map, and zooming
/// out and in does it.
#[test]
fn the_zoom_lesson_opens_on_turn_one_after_the_lessons() {
    if skip_without_the_game() {
        return;
    }
    let mut h = chapter_1();
    h.with_map_skin("sprite");
    let mut seen = Vec::new();
    for keys in ["Up Down Left Right", "Right Left"] {
        h.keys(keys);
        seen.extend(close_tips(&mut h));
    }
    // Options from the map menu, and back to the map.
    h.keys("d Down Down Down f d d");
    seen.extend(close_tips(&mut h));
    assert_eq!(battle(&h).shown_lesson(), Some("next_unit"));
    h.keys("s");
    go(&mut h, LEAD);
    h.keys("e d");
    seen.extend(close_tips(&mut h));
    assert_eq!(battle(&h).shown_lesson(), Some("moving"));
    h.keys("f Up f").wait(1.0);
    assert!(
        matches!(battle(&h).mode(), Mode::ActionMenu { .. }),
        "{:?}",
        battle(&h).mode()
    );
    assert_eq!(battle(&h).shown_lesson(), None);
    seen.extend(close_tips(&mut h));
    assert!(!seen.contains(&TipTrigger::ZoomAvailable), "{seen:?}");
    // Back out of the menu and the move, to the map (closing the tips of
    // the unit picked up, in their moment on the way, 1292).
    let back = back_to_map(&mut h);
    assert!(!back.contains(&TipTrigger::ZoomAvailable), "a lesson now");
    h.wait(0.1);
    close_tips(&mut h);
    assert_eq!(battle(&h).state().turn(), 1);
    assert_eq!(battle(&h).shown_lesson(), Some("zoom"));
    h.keys("q");
    assert_eq!(battle(&h).lesson_ticks(), [true, false]);
    h.keys("q").wait(0.1);
    assert_ne!(battle(&h).shown_lesson(), Some("zoom"));
}

/// (a) The mage's action menu opens while a lesson is open: no Magic tip
/// then, nor after the lessons; it shows the next time the mage's action
/// menu opens (1292).
#[test]
fn the_magic_tip_waits_for_the_mages_menu_after_the_lessons() {
    if skip_without_the_game() {
        return;
    }
    let mut h = chapter_1();
    go(&mut h, MAGE);
    h.keys("f f").wait(0.5);
    assert!(
        matches!(battle(&h).mode(), Mode::ActionMenu { .. }),
        "{:?}",
        battle(&h).mode()
    );
    assert_eq!(battle(&h).shown_tip(), None, "the lesson goes first");
    // Back to the map, and the lessons finished: no Magic tip.
    h.keys("d d").wait(0.1);
    assert!(matches!(battle(&h).mode(), Mode::Idle { .. }));
    go(&mut h, LEAD);
    let after = finish_turn_1_lessons(&mut h);
    assert!(!after.contains(&TipTrigger::FirstCaster), "{after:?}");
    // The mage's action menu again: now it shows.
    go(&mut h, MAGE);
    h.keys("f f").wait(0.5);
    assert!(
        close_tips(&mut h).contains(&TipTrigger::FirstCaster),
        "the Magic tip never showed"
    );
}

/// (b) The cursor rests on a forest with nothing selected while the
/// turn-1 lessons run: no Terrain tip then, nor after the lessons with
/// the cursor elsewhere; it shows the next time the cursor rests on
/// special ground (1292).
#[test]
fn the_terrain_tip_waits_for_the_forest_after_the_lessons() {
    if skip_without_the_game() {
        return;
    }
    let mut h = chapter_1();
    go(&mut h, FOREST);
    h.wait(0.5);
    assert_eq!(battle(&h).shown_tip(), None, "the lesson goes first");
    go(&mut h, LEAD);
    let after = finish_turn_1_lessons(&mut h);
    assert!(!after.contains(&TipTrigger::FirstTerrain), "{after:?}");
    go(&mut h, FOREST);
    assert!(
        close_tips(&mut h).contains(&TipTrigger::FirstTerrain),
        "the Terrain tip never showed"
    );
}

/// Tickets 0495 (Nick: "D both closes options and her tutorial") and 1292:
/// on turn 1 at Full, a tip that fires during the keys lesson (the cursor
/// resting on a forest) missed its moment. It doesn't come up over the map
/// menu back from Options, nor on the map after it; the D that closes the
/// menu closes the menu. The next time the cursor rests on the forest with
/// no lesson up, *Terrain* comes up, and stays until a press made while it
/// is up.
#[test]
fn leaving_options_never_closes_a_tip_unread() {
    if skip_without_the_game() {
        return;
    }
    let mut h = chapter_1();
    finish_cursor_lesson(&mut h);
    assert_eq!(battle(&h).shown_lesson(), Some("keys"));
    // The cursor on the forest and back: its tip doesn't show under the
    // lesson.
    go(&mut h, FOREST);
    h.wait(0.3);
    assert_eq!(battle(&h).shown_tip(), None);
    go(&mut h, LEAD);
    // Options from the map menu, and back out of it.
    h.keys("d Down Down Down f");
    assert_eq!(h.top_screen(), "options");
    h.keys("d").wait(0.3);
    assert_eq!(h.top_screen(), "battle");
    assert!(matches!(battle(&h).mode(), Mode::MapMenu { .. }));
    assert_eq!(battle(&h).shown_tip(), None, "no tip over the map menu");
    // Cancel closes the menu, not a tip; the next lesson is up.
    h.keys("d").wait(0.3);
    assert!(matches!(battle(&h).mode(), Mode::Idle { .. }));
    assert_eq!(battle(&h).shown_tip(), None);
    assert_eq!(battle(&h).shown_lesson(), Some("next_unit"));
    // The next lessons, to the action menu and back to the map: the tip
    // of the unit picked up under the lesson never came up.
    h.keys("s");
    go(&mut h, LEAD);
    h.keys("e d");
    assert_eq!(battle(&h).shown_lesson(), Some("moving"));
    h.keys("f Up f").wait(1.0);
    assert!(matches!(battle(&h).mode(), Mode::ActionMenu { .. }));
    assert_eq!(battle(&h).shown_lesson(), None);
    // Back to the map, and the cursor on the forest with no lesson up:
    // *Terrain* comes up, and stays.
    back_to_map(&mut h);
    go(&mut h, FOREST);
    h.wait(0.3);
    assert_eq!(battle(&h).shown_tip(), Some(TipTrigger::FirstTerrain));
    h.wait(1.0);
    assert_eq!(battle(&h).shown_tip(), Some(TipTrigger::FirstTerrain));
    // A press closes it; the map is back.
    h.keys("f").wait(0.3);
    assert_ne!(battle(&h).shown_tip(), Some(TipTrigger::FirstTerrain));
    assert!(matches!(battle(&h).mode(), Mode::Idle { .. }));
}
