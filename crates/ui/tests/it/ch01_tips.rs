//! The one-time tips (ticket 0406) on the shipped Chapter 1 at `Full`
//! from a fresh profile, while the lessons (0457) run (ticket 0467): a
//! tip whose trigger fires while a lesson is open shows once the lesson
//! is done, and a tip counts as seen only once it has been on screen.
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
use trpg_ui::screens::battle::BattleScreen;
use trpg_ui::screens::battle::mode::Mode;

// Chapter 1's party where it starts (`assets/battles/ch01.ron`).
const LEAD: Pos = Pos::new(4, 12);
/// The `heretic`, a Mage: its action menu has Magic.
const MAGE: Pos = Pos::new(4, 13);
/// A forest tile on Chapter 1's map (`maps/ch01.map`).
const FOREST: Pos = Pos::new(2, 10);

/// Skips every scene on screen: Cancel, then Confirm on "Skip scene?".
fn skip_scenes(h: &mut Harness) {
    for _ in 0..8 {
        if h.top_screen() != "dialogue" {
            return;
        }
        h.keys("d f");
    }
    panic!("the scenes never end: {:?}", h.screens());
}

/// Chapter 1 from New Game in Classic at `Full` on a fresh profile, with
/// the tips and lessons on: turn 1, its scenes skipped and its banner
/// closed, the start tip closed, the cursor on the lead and the cursor
/// lesson open.
fn chapter_1() -> Harness {
    let mut h = Harness::with_layout(RIGHT_HANDED).with_shipped_story();
    h.with_lessons().with_tips();
    h.keys("f f f Up f");
    skip_scenes(&mut h);
    h.keys("f");
    skip_scenes(&mut h);
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    h.wait(0.1);
    let campaign = h
        .flow()
        .and_then(|f| f.campaign_now(h.game().ctx()))
        .map(|c| c.tutorial);
    assert_eq!(campaign, Some(TutorialLevel::Full));
    assert_eq!(close_tips(&mut h), [TipTrigger::FirstBattleStart]);
    assert_eq!(h.cursor_tile(), Some(LEAD));
    assert_eq!(battle(&h).shown_lesson(), Some("cursor"));
    h
}

fn battle(h: &Harness) -> &BattleScreen {
    h.battle().unwrap_or_else(|| panic!("no battle"))
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
/// triggers. Each waits for a press on the map ([`nudge`]).
fn close_tips(h: &mut Harness) -> Vec<TipTrigger> {
    let mut seen = Vec::new();
    for _ in 0..20 {
        nudge(h);
        h.wait(0.1);
        let Some(tip) = battle(h).shown_tip() else {
            return seen;
        };
        seen.push(tip);
        h.keys("f");
    }
    panic!("the tips never ran out: {seen:?}");
}

/// A press on the map that changes nothing (Zoom, with glyphs, which
/// don't zoom): a box waiting for the player to do something comes up
/// after it (0495).
fn nudge(h: &mut Harness) {
    h.keys("q");
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
/// selecting) and backs out to the map, the cursor on the lead. A tip
/// that fired meanwhile waits for them all (0495: a lesson done goes
/// straight to the next, never to a tip between them).
fn finish_turn_1_lessons(h: &mut Harness) {
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
    assert_eq!(battle(h).shown_tip(), None, "no tip over the menu");
    h.keys("d d").wait(0.1);
    assert!(matches!(battle(h).mode(), Mode::Idle { .. }));
}

/// Ticket 0466: on a map that can zoom, the zoom tip stays away through
/// the turn-1 lessons (cursor, keys, next unit, selecting) and the action
/// menu the last one ends in, and shows back on the map.
#[test]
fn the_zoom_tip_shows_on_turn_one_after_the_lessons() {
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
    // Back out of the menu and the move, to the map.
    h.keys("d d").wait(0.1);
    assert!(matches!(battle(&h).mode(), Mode::Idle { .. }));
    assert_eq!(battle(&h).state().turn(), 1);
    assert!(close_tips(&mut h).contains(&TipTrigger::ZoomAvailable));
}

/// (a) The mage's action menu opens while a lesson is open: the Magic tip
/// waits for it, then shows once turn 1's lessons are done (0495).
#[test]
fn the_magic_tip_shows_after_the_lesson_open_when_it_fired() {
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
    // Back to the map, and the lesson finished.
    h.keys("d d").wait(0.1);
    assert!(matches!(battle(&h).mode(), Mode::Idle { .. }));
    go(&mut h, LEAD);
    finish_turn_1_lessons(&mut h);
    assert!(
        close_tips(&mut h).contains(&TipTrigger::FirstCaster),
        "the Magic tip never showed"
    );
}

/// (b) The cursor rests on a forest with nothing selected while the
/// turn-1 lessons run: the Terrain tip shows once they are done (0495).
#[test]
fn the_terrain_tip_shows_after_the_lesson_open_when_it_fired() {
    if skip_without_the_game() {
        return;
    }
    let mut h = chapter_1();
    go(&mut h, FOREST);
    h.wait(0.5);
    assert_eq!(battle(&h).shown_tip(), None, "the lesson goes first");
    go(&mut h, LEAD);
    finish_turn_1_lessons(&mut h);
    assert!(
        close_tips(&mut h).contains(&TipTrigger::FirstTerrain),
        "the Terrain tip never showed"
    );
}

/// Ticket 0495 (Nick: "D both closes options and her tutorial"): on turn
/// 1 at Full, a tip that fires during the keys lesson (picking a unit up
/// and putting it back) waits through Options, the map menu the player
/// comes back to and the lessons after; it comes up only on the map,
/// after a press, stays until a press made while it is up, and the next
/// tip waits for another press on the map.
#[test]
fn leaving_options_never_closes_a_tip_unread() {
    if skip_without_the_game() {
        return;
    }
    let mut h = chapter_1();
    finish_cursor_lesson(&mut h);
    assert_eq!(battle(&h).shown_lesson(), Some("keys"));
    // A unit picked up and put back: its tip waits for the lesson.
    h.keys("f").wait(0.3);
    h.keys("d").wait(0.3);
    assert_eq!(battle(&h).shown_tip(), None);
    h.keys("Right Left");
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
    // The next lessons, to the action menu and back to the map.
    h.keys("s");
    go(&mut h, LEAD);
    h.keys("e d");
    assert_eq!(battle(&h).shown_lesson(), Some("moving"));
    h.keys("f Up f").wait(1.0);
    assert!(matches!(battle(&h).mode(), Mode::ActionMenu { .. }));
    assert_eq!(battle(&h).shown_tip(), None, "no tip over the action menu");
    h.keys("d d").wait(0.3);
    assert!(matches!(battle(&h).mode(), Mode::Idle { .. }));
    // Back on the map after a press: the tip that waited is up, and stays.
    let first = battle(&h).shown_tip();
    assert_eq!(first, Some(TipTrigger::FirstUnitSelected));
    h.wait(1.0);
    assert_eq!(battle(&h).shown_tip(), first);
    // A fresh press closes it; the next waits for a press on the map.
    h.keys("d").wait(0.3);
    assert_eq!(battle(&h).shown_tip(), None);
    assert!(matches!(battle(&h).mode(), Mode::Idle { .. }));
    nudge(&mut h);
    assert!(
        battle(&h).shown_tip().is_some(),
        "the next tip after a press"
    );
}
