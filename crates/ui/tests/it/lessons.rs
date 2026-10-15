//! Scripted tests of the checklist lessons (ticket 0456) through the real
//! game: New Game on the test chapter at each familiarity answer, the test
//! lesson (`assets-test/data/lessons.ron`: the cursor's four directions on
//! the first turn), the turn waiting for it, and a done lesson staying
//! done after the battle is suspended and continued.

use trpg_core::{Phase, TutorialLevel};
use trpg_ui::harness::Harness;
use trpg_ui::input::Layout;
use trpg_ui::save::SUSPEND_KEY;
use trpg_ui::screens::battle::BattleScreen;
use trpg_ui::{Ctx, Game, MemoryStorage, Storage};

/// The test lesson.
const TEST_LESSON: &str = "test_cursor";

/// A launch on `storage` with the right-handed layout picked, New Game on
/// the test chapter, and the lessons on (as the game has them).
fn launch(storage: Box<dyn Storage>) -> Harness {
    let mut storage = storage;
    if let Err(e) = storage.write("layout", Layout::RightHanded.name()) {
        panic!("saving the layout: {e}");
    }
    let mut ctx = Ctx::embedded()
        .unwrap_or_else(|e| panic!("{e}"))
        .with_storage(storage);
    ctx.content.new_game = trpg_content::NewGameDef::test_chapter();
    let mut h = Harness::from_game(Game::start(ctx));
    h.with_lessons();
    h
}

/// From the title: New Game in Classic, the familiarity answer `downs`
/// rows below "Not familiar", the default lead, the intro scene skipped,
/// the battle notes and turn 1's banner closed.
fn to_battle(h: &mut Harness, downs: &str) {
    h.keys("f f").keys(downs).keys("f Up f");
    assert_eq!(h.top_screen(), "dialogue");
    h.keys("d f");
    assert_eq!(h.screens(), ["title", "battle"]);
    h.keys("f").wait(0.1).keys("f").wait(0.1);
}

fn battle(h: &Harness) -> &BattleScreen {
    let battle = h.flow().and_then(|f| f.battle());
    battle.unwrap_or_else(|| panic!("no battle"))
}

fn lessons_done(h: &Harness) -> Vec<String> {
    let campaign = h.flow().and_then(|f| f.campaign());
    let campaign = campaign.unwrap_or_else(|| panic!("no campaign"));
    campaign.lessons_done.iter().cloned().collect()
}

/// Acceptance: at `Full` the test lesson opens on the first turn; the
/// four directions, in any order, close it, and the campaign keeps it.
/// While it is open End Turn is refused; after, it works.
#[test]
fn at_full_the_lesson_opens_holds_the_turn_and_closes() {
    let mut h = launch(Box::new(MemoryStorage::new()));
    to_battle(&mut h, "");
    assert_eq!(battle(&h).shown_lesson(), Some(TEST_LESSON));
    assert!(h.snapshot().contains("Moving the cursor"));
    // End turn: nothing, and no prompt.
    h.keys("Space");
    assert!(!h.snapshot().contains("End turn"), "no End turn prompt");
    assert_eq!(battle(&h).state().phase(), Phase::Player);
    h.keys("Down Right Left");
    assert!(battle(&h).lesson_open());
    assert!(lessons_done(&h).is_empty());
    h.keys("Up");
    assert_eq!(battle(&h).shown_lesson(), None);
    assert!(!h.snapshot().contains("Moving the cursor"));
    assert_eq!(lessons_done(&h), [TEST_LESSON]);
    // Now End turn asks, and yes ends the phase.
    h.keys("Space f");
    assert_ne!(battle(&h).state().phase(), Phase::Player);
}

/// Acceptance: at `None` the lesson never opens, nor at `Short` (it is a
/// `Full` lesson); the turn ends as usual.
#[test]
fn at_short_and_none_the_full_lesson_never_opens() {
    for (downs, level) in [
        ("Down", TutorialLevel::Short),
        ("Down Down", TutorialLevel::None),
    ] {
        let mut h = launch(Box::new(MemoryStorage::new()));
        to_battle(&mut h, downs);
        let campaign = h.flow().and_then(|f| f.campaign()).map(|c| c.tutorial);
        assert_eq!(campaign, Some(level));
        h.wait(1.0);
        assert_eq!(battle(&h).shown_lesson(), None, "{level:?}");
        assert!(!battle(&h).lesson_open());
        h.keys("Space f");
        assert_ne!(battle(&h).state().phase(), Phase::Player, "{level:?}");
    }
}

/// Acceptance: a done lesson doesn't reopen after saving and loading: the
/// battle suspended (the map menu's `Suspend`) and continued in the next
/// launch, back on its first turn's player phase.
#[test]
fn a_done_lesson_stays_done_after_suspend_and_continue() {
    let mut h = launch(Box::new(MemoryStorage::new()));
    to_battle(&mut h, "");
    h.keys("Up Down Left Right");
    assert_eq!(lessons_done(&h), [TEST_LESSON]);
    // The map menu's Suspend, then yes.
    h.keys("d Down Down Down f f");
    assert_eq!(h.screens(), ["title"]);
    let saved = h.game().ctx().storage.read(SUSPEND_KEY);
    let saved = saved
        .ok()
        .flatten()
        .unwrap_or_else(|| panic!("no suspend save"));
    assert!(saved.contains("lessons_done:[\"test_cursor\"]"), "{saved}");
    // The next launch: Continue.
    let mut h = launch(h.into_storage());
    h.keys("f");
    assert_eq!(h.screens(), ["title", "battle"]);
    h.wait(1.0);
    assert_eq!(battle(&h).state().turn(), 1);
    assert_eq!(battle(&h).shown_lesson(), None);
    assert_eq!(lessons_done(&h), [TEST_LESSON]);
}

/// Not done when suspended: the lesson opens again on Continue, from
/// scratch (its ticks belong to the battle screen that showed it).
#[test]
fn a_lesson_left_open_opens_again_after_continue() {
    let mut h = launch(Box::new(MemoryStorage::new()));
    to_battle(&mut h, "");
    h.keys("Up Down");
    h.keys("d Down Down Down f f");
    assert_eq!(h.screens(), ["title"]);
    let mut h = launch(h.into_storage());
    h.keys("f").wait(0.1);
    assert_eq!(battle(&h).shown_lesson(), Some(TEST_LESSON));
    assert_eq!(battle(&h).lesson_ticks(), [false; 4]);
    assert!(lessons_done(&h).is_empty());
}
