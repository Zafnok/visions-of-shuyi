//! Threat arrows (ticket 0448, `docs/design/turn-structure.md`): the scene
//! names the enemies that could attack the tile a unit of the player's
//! stands on or is about to move to. What they look like is each map
//! skin's (`crates/ui/src/map_view/threat.rs`).

use trpg_core::ai::AiBehavior;
use trpg_core::{Phase, Pos};
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;
use trpg_ui::map_view::ThreatView;

fn p(x: i32, y: i32) -> Pos {
    Pos::new(x, y)
}

/// The tile the scene's arrows point at and where their enemies stand.
fn threat(h: &Harness) -> Option<(Pos, Vec<Pos>)> {
    let scene = h.map_scene().unwrap_or_else(|| panic!("no battle"));
    scene
        .threat
        .map(|ThreatView { tile, enemies }| (tile, enemies))
}

/// Arrows at `tile` from the enemies on `enemies`, for comparing with
/// [`threat`].
fn arrows(tile: Pos, enemies: &[Pos]) -> (Pos, Vec<Pos>) {
    (tile, enemies.to_vec())
}

/// Moves the cursor from where the map shows it to `to` with the arrow keys.
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

/// Chapter 1's battle from New Game, on turn 1 with its scenes skipped and
/// its banner closed: the cursor on the lead at (4, 12).
fn chapter_1() -> Harness {
    let mut h = Harness::with_layout(RIGHT_HANDED).with_shipped_story();
    // New Game, Classic, Not familiar, the default lead.
    h.keys("f f f Up f");
    skip_scenes(&mut h);
    h.keys("f");
    skip_scenes(&mut h);
    assert_eq!(h.screens(), ["title", "battle"]);
    assert_eq!(h.cursor_tile(), Some(p(4, 12)));
    h
}

/// The Quick Battle on turn 1, its banner closed: the cursor on the lord
/// at (3, 5).
fn quick_battle() -> Harness {
    let mut h = Harness::with_layout(RIGHT_HANDED);
    h.keys("Down f Left f f");
    assert_eq!(h.screens(), ["title", "battle"]);
    assert_eq!(h.cursor_tile(), Some(p(3, 5)));
    h
}

/// The example of `turn-structure.md`: on turn 1 of Chapter 1 the Sergeant
/// proposes the road tile 5 east of him. The Brigand beside it, the
/// Brigand to the east, the Archer and the Raider north of the road
/// bridge, who stands guard, could each attack him there.
#[test]
fn chapter_1_names_who_can_attack_the_tile_the_sergeant_proposes() {
    let mut h = chapter_1();
    let (brigand, east, archer, raider) = (p(10, 10), p(16, 11), p(12, 9), p(11, 5));
    // Nobody reaches the lead where he starts; the nearest Brigand reaches
    // the Sergeant.
    assert_eq!(threat(&h), None);
    go(&mut h, p(6, 11));
    assert_eq!(h.unit_at(p(6, 11)).map(|u| u.label), Some("Ta".to_owned()));
    assert_eq!(threat(&h), Some(arrows(p(6, 11), &[brigand])));
    // Selected, on his own tile: the same.
    h.keys("f");
    assert_eq!(h.path(), [p(6, 11)]);
    assert_eq!(threat(&h), Some(arrows(p(6, 11), &[brigand])));
    h.keys("Right Right Right Right Right");
    assert_eq!(h.path().last(), Some(&p(11, 11)));
    let four = [brigand, east, archer, raider];
    assert_eq!(threat(&h), Some(arrows(p(11, 11), &four)));
    // The Raider waits for someone to come within reach: it still counts.
    let battle = h.battle().unwrap_or_else(|| panic!("no battle"));
    let guard = battle.state().units().iter().find(|u| u.pos == raider);
    assert_eq!(guard.map(|u| u.ai), Some(AiBehavior::Guard));
    // One tile back, the bridge's Raider no longer reaches.
    h.keys("Left");
    assert_eq!(
        threat(&h),
        Some(arrows(p(10, 11), &[brigand, east, archer]))
    );
    // Back on his own tile.
    h.keys("Left Left Left Left");
    assert_eq!(h.path(), [p(6, 11)]);
    assert_eq!(threat(&h), Some(arrows(p(6, 11), &[brigand])));
}

/// With nothing selected, the cursor resting on one of the player's units
/// shows who could attack it where it stands; any other tile shows nothing.
#[test]
fn the_cursor_on_a_unit_of_the_players_shows_who_can_attack_it() {
    let mut h = quick_battle();
    // The lord at (3, 5): the brigand at (7, 4) reaches him.
    assert_eq!(threat(&h), Some(arrows(p(3, 5), &[p(7, 4)])));
    // An empty tile.
    h.keys("Up");
    assert_eq!(threat(&h), None);
    // The mage at (3, 6), whom nobody reaches.
    h.keys("Down Down");
    assert_eq!(h.unit_at(p(3, 6)).map(|u| u.label), Some("Ma".to_owned()));
    assert_eq!(threat(&h), None);
    // An enemy: its own range is another key's business.
    go(&mut h, p(7, 4));
    assert_eq!(h.unit_at(p(7, 4)).map(|u| u.label), Some("Br".to_owned()));
    assert_eq!(threat(&h), None);
    // A unit that has acted shows them too: the lord waits where he is.
    go(&mut h, p(3, 5));
    h.keys("f f").wait(0.5).keys("f");
    assert_eq!(h.unit_at(p(3, 5)).map(|u| u.acted), Some(true));
    assert_eq!(h.cursor_tile(), Some(p(3, 5)));
    assert_eq!(threat(&h), Some(arrows(p(3, 5), &[p(7, 4)])));
}

/// A selected unit: the arrows follow the tile the cursor proposes, stay
/// through the walk and the action menu, and go when the forecast opens.
#[test]
fn the_arrows_follow_the_proposed_tile_and_stay_until_the_forecast() {
    let mut h = quick_battle();
    h.keys("f");
    assert_eq!(threat(&h), Some(arrows(p(3, 5), &[p(7, 4)])));
    // The mage's tile below him: he can pass through it, not stop there.
    h.keys("Down");
    assert_eq!(h.path(), [p(3, 5), p(3, 6)]);
    assert_eq!(threat(&h), None);
    h.keys("Up");
    h.keys("Right");
    assert_eq!(threat(&h), Some(arrows(p(4, 5), &[p(7, 4)])));
    // Further in, the other brigand and the raider reach him too. The
    // frost elemental at (8, 7) never moves: its spell doesn't reach.
    let all = [p(8, 2), p(7, 4), p(7, 1)];
    h.keys("Right Right Right");
    assert_eq!(h.path().last(), Some(&p(7, 5)));
    assert_eq!(threat(&h), Some(arrows(p(7, 5), &all)));
    // A tile he can't move to: nothing.
    h.keys("Right");
    assert_eq!(h.cursor_tile(), Some(p(8, 5)));
    assert_eq!(h.path().last(), Some(&p(7, 5)));
    assert_eq!(threat(&h), None);
    // Back on the proposed tile; Confirm walks him there.
    h.keys("Left");
    assert_eq!(threat(&h), Some(arrows(p(7, 5), &all)));
    h.keys("f");
    assert_eq!(threat(&h), Some(arrows(p(7, 5), &all)));
    h.wait(1.0);
    assert!(h.snapshot().contains("Attack"), "{}", h.snapshot());
    assert_eq!(threat(&h), Some(arrows(p(7, 5), &all)));
    // `Attack`: the forecast of the brigand above him. No arrows.
    h.keys("f");
    assert!(h.snapshot().contains("Forecast"), "{}", h.snapshot());
    assert_eq!(threat(&h), None);
    // Cancel: the menu again, and the arrows.
    h.keys("d");
    assert_eq!(threat(&h), Some(arrows(p(7, 5), &all)));
}

/// Pointing at an enemy the selected unit can attack aims the path at the
/// tile it will attack from: the arrows are for that tile.
#[test]
fn pointing_at_an_enemy_shows_the_arrows_of_the_tile_attacked_from() {
    let mut h = quick_battle();
    h.keys("f");
    go(&mut h, p(7, 4));
    let path = h.path();
    let from = *path.last().unwrap_or_else(|| panic!("no path"));
    assert_ne!(from, p(7, 4));
    assert_eq!(Pos::manhattan(from, p(7, 4)), 1);
    let Some((tile, enemies)) = threat(&h) else {
        panic!("no arrows");
    };
    assert_eq!(tile, from);
    assert!(enemies.contains(&p(7, 4)), "{enemies:?}");
}

/// Nothing shows in the enemy's phase.
#[test]
fn no_arrows_outside_the_players_phase() {
    let mut h = quick_battle();
    assert!(threat(&h).is_some());
    // End turn, twice to confirm with units ready.
    h.keys("Space Space");
    let battle = h.battle().unwrap_or_else(|| panic!("no battle"));
    assert_ne!(battle.state().phase(), Phase::Player);
    assert_eq!(threat(&h), None);
}
