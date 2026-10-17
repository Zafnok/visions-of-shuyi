//! Scripted test of the Guides list (ticket 0458) through the real game:
//! New Game on the test chapter at `None` (no tips, no lessons), Guides
//! from the battle's map menu, stepping through the topics, and back.

use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;
use trpg_ui::screens::battle::BattleScreen;
use trpg_ui::{Ctx, Game, MemoryStorage, Storage};

fn battle(h: &Harness) -> &BattleScreen {
    let battle = h.flow().and_then(|f| f.battle());
    battle.unwrap_or_else(|| panic!("no battle"))
}

/// Acceptance: at `None` the map menu's Guides lists every lesson and
/// tip; moving through it shows each one's text with the keys; Cancel
/// goes back to the map menu.
#[test]
fn guides_opens_from_the_map_menu_at_none_and_steps_through_the_topics() {
    let mut storage: Box<dyn Storage> = Box::new(MemoryStorage::new());
    if let Err(e) = storage.write("layout", RIGHT_HANDED.name()) {
        panic!("saving the layout: {e}");
    }
    let mut ctx = Ctx::embedded()
        .unwrap_or_else(|e| panic!("{e}"))
        .with_storage(storage);
    ctx.content.new_game = trpg_content::NewGameDef::test_chapter();
    let mut h = Harness::from_game(Game::start(ctx));
    h.with_lessons();
    // New Game in Classic, the familiarity answer two rows below "Not
    // familiar" (None), the default lead, the intro scene skipped, the
    // battle notes and turn 1's banner closed.
    h.keys("f f Down Down f Up f");
    assert_eq!(h.top_screen(), "dialogue");
    h.keys("d f");
    assert_eq!(h.screens(), ["title", "battle"]);
    h.keys("f").wait(0.1).keys("f").wait(0.1);
    let level = h.flow().and_then(|f| f.campaign()).map(|c| c.tutorial);
    assert_eq!(level, Some(trpg_core::TutorialLevel::None));
    assert_eq!(battle(&h).shown_lesson(), None);
    // The map menu: Units, Objective, Guides.
    h.keys("d Down Down f");
    assert!(battle(&h).guides().is_some());
    let screen = h.snapshot();
    for text in [
        "Guides",
        "Moving the cursor",
        "That's your cursor. It picks tiles and units.",
        "Move it up: Up",
        "Danger zone",
        "Terrain",
        "arrows choose · d close",
    ] {
        assert!(screen.contains(text), "{text}:\n{screen}");
    }
    h.keys("Down");
    assert!(
        h.snapshot()
            .contains("No mouse here; it's all keys. And every key")
    );
    h.keys("Up Up");
    let screen = h.snapshot();
    assert!(screen.contains("Mind the ground."), "{screen}");
    // Back to the map menu, then to the map.
    h.keys("d");
    assert!(battle(&h).guides().is_none());
    assert!(h.snapshot().contains("Restart Battle"));
    h.keys("d");
    assert!(!h.snapshot().contains("Restart Battle"));
}
