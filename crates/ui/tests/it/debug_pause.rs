//! The developer's pause and step keys (ticket 1390, engine ADR-0153):
//! with debug tools on, the pause key holds the battle's clock, so a walk
//! stands still, the step key plays one frame of it, and the pause key
//! again lets it run on, to the same state as a straight run. With debug
//! tools off neither key does anything, and neither ever reaches a screen.

use trpg_core::Pos;
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;

/// What the scene shows of the walking lord: its tile and its offset
/// towards the next.
type Seen = Option<(Pos, (f32, f32))>;

/// The Quick Battle from the title with the debug tools on or off, the
/// lord picked and its path drawn three tiles right and one up.
fn before_the_walk(tools: bool) -> Harness {
    let mut h = Harness::with_layout(RIGHT_HANDED);
    h.keys("Down f Left f f f f Right Right Right Up");
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    h.ctx_mut().debug_tools = tools;
    h
}

/// The lord as the scene shows it.
fn lord(h: &Harness) -> Seen {
    let id = h.battle().map(|b| b.state().units()[0].id)?;
    let scene = h.map_scene()?;
    scene.unit(id).map(|u| (u.pos, u.offset))
}

/// The battle's rules state, hashed.
fn state_hash(h: &Harness) -> shuyi_core::StateHash {
    let battle = h.battle().unwrap_or_else(|| panic!("no battle"));
    shuyi_core::state_hash(battle.state()).unwrap_or_else(|e| panic!("the state doesn't hash: {e}"))
}

#[test]
fn the_pause_key_holds_a_walk_and_the_step_key_plays_one_frame() {
    let mut h = before_the_walk(true);
    h.keys("f").wait(0.1);
    let start = lord(&h);
    assert!(start.is_some());
    // Paused: however long it waits, the lord stands where it was.
    h.keys("F3");
    assert!(h.ctx().debug_view.paused());
    let held = lord(&h);
    h.wait(0.5);
    assert_eq!(lord(&h), held);
    // A step moves it on a frame, then it stands again.
    h.keys("F4");
    let stepped = lord(&h);
    assert_ne!(stepped, held);
    h.wait(0.5);
    assert_eq!(lord(&h), stepped);
    // Run again: the walk ends on the path's last tile, in the state a
    // walk never paused reaches.
    h.keys("F3").wait(3.0);
    assert!(!h.ctx().debug_view.paused());
    assert_eq!(lord(&h), Some((Pos::new(6, 4), (0.0, 0.0))));
    let mut straight = before_the_walk(true);
    straight.keys("f").wait(3.6);
    assert_eq!(state_hash(&h), state_hash(&straight));
}

#[test]
fn with_the_debug_tools_off_the_keys_do_nothing() {
    let (mut keyed, mut plain) = (before_the_walk(false), before_the_walk(false));
    keyed.keys("f").wait(0.1);
    plain.keys("f").wait(0.1);
    // Each key is two frames (pressed, let go), as the waits beside it.
    keyed.keys("F3 F4");
    plain.wait(4.0 / 60.0);
    assert!(!keyed.ctx().debug_view.paused());
    assert_eq!(lord(&keyed), lord(&plain));
    assert_eq!(keyed.screens(), plain.screens());
}

#[test]
fn neither_key_reaches_the_battle_screen() {
    for tools in [true, false] {
        let mut h = before_the_walk(tools);
        let (cursor, path, hash) = (h.cursor_tile(), h.path(), state_hash(&h));
        // Pause then run again, and a step in between: the map is as it
        // was, its path still drawn, no menu opened.
        h.keys("F3 F4 F3");
        assert!(!h.ctx().debug_view.paused(), "tools {tools}");
        assert_eq!(
            h.screens(),
            ["title", "game_flow", "battle"],
            "tools {tools}"
        );
        assert_eq!(h.cursor_tile(), cursor, "tools {tools}");
        assert_eq!(h.path(), path, "tools {tools}");
        assert_eq!(state_hash(&h), hash, "tools {tools}");
    }
}
