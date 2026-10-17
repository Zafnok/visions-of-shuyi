//! The one-time tips (ticket 0406) on the shipped Chapter 1 at `Full`
//! from a fresh profile, while the lessons (0457) run (ticket 0467): a
//! tip whose trigger fires while a lesson is open shows once the lesson
//! is done, and a tip counts as seen only once it has been on screen.

use trpg_content::TipTrigger;
use trpg_core::{Pos, TutorialLevel};
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;
use trpg_ui::screens::battle::BattleScreen;
use trpg_ui::screens::battle::mode::Mode;

// Chapter 1's party where it starts (`assets/battles/ch01.ron`).
const LEAD: Pos = Pos::new(4, 12);
/// Rue, a Mage: her action menu has Magic.
const RUE: Pos = Pos::new(4, 13);
/// A forest tile west of Harrowby (`assets/maps/ch01.map`).
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
    assert_eq!(h.screens(), ["title", "battle"]);
    h.wait(0.1);
    let campaign = h.flow().and_then(|f| f.campaign()).map(|c| c.tutorial);
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
/// triggers.
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

/// Finishes the cursor lesson: the four directions, the cursor ending
/// back where it was.
fn finish_cursor_lesson(h: &mut Harness) {
    if battle(h).shown_lesson() == Some("cursor") {
        h.keys("Up Down Left Right");
    }
    assert_ne!(battle(h).shown_lesson(), Some("cursor"));
}

/// (a) Rue's action menu opens while a lesson is open: the Magic tip
/// waits for it, then shows once the lesson is done.
#[test]
fn the_magic_tip_shows_after_the_lesson_open_when_it_fired() {
    let mut h = chapter_1();
    go(&mut h, RUE);
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
    finish_cursor_lesson(&mut h);
    assert!(
        close_tips(&mut h).contains(&TipTrigger::FirstCaster),
        "the Magic tip never showed"
    );
}

/// (b) The cursor rests on a forest with nothing selected while the
/// turn-1 lessons run: the Terrain tip shows once they are done.
#[test]
fn the_terrain_tip_shows_after_the_lesson_open_when_it_fired() {
    let mut h = chapter_1();
    go(&mut h, FOREST);
    h.wait(0.5);
    assert_eq!(battle(&h).shown_tip(), None, "the lesson goes first");
    go(&mut h, LEAD);
    finish_cursor_lesson(&mut h);
    assert!(
        close_tips(&mut h).contains(&TipTrigger::FirstTerrain),
        "the Terrain tip never showed"
    );
}
