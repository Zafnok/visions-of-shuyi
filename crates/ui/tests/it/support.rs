//! The drivers the integration tests share (ticket 1237).

use trpg_core::BattleState;
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;
use trpg_ui::screens::battle::BattleScreen;

/// The title screen, with Nick's right-handed keys.
pub fn title() -> Harness {
    Harness::with_layout(RIGHT_HANDED)
}

/// The debug Quick Battle from the title: Quick Battle, then
/// Preparations (Left wraps to `Fight!`), then the phase banner.
pub fn quick_battle() -> Harness {
    let mut h = title();
    h.keys("Down f Left f f f");
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    h
}

/// Plays through the chapter title card and the scenes before or after
/// a battle.
pub fn skip_scenes(h: &mut Harness) {
    for _ in 0..12 {
        // A chapter's title card first (0812): Confirm closes it.
        if h.top_screen() == "chapter_card" {
            h.keys("f");
            continue;
        }
        if h.top_screen() != "dialogue" {
            return;
        }
        h.keys("d f");
    }
    panic!("the scenes never end: {:?}", h.screens());
}

/// The battle screen on the stack.
pub fn battle(h: &Harness) -> &BattleScreen {
    h.battle()
        .unwrap_or_else(|| panic!("no battle: {:?}", h.screens()))
}

/// The battle's state, cloned.
pub fn state(h: &Harness) -> BattleState {
    battle(h).state().clone()
}
