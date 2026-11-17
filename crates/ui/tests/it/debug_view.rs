//! The debug view on the battle map (engine ADR-0085, ticket 1252): the
//! battle screen offers the grid kit's `grid` channel, which outlines the
//! map's cells, the cursor's cell and each unit's; nothing is built while
//! the channel or the debug tools are off; and looking changes nothing.

use shuyi_ui::debug::gizmo::{DebugView, GizmoRole};
use shuyi_ui::frame::{GizmoItem, GizmoShape};
use shuyi_ui::grid::debug::GRID;
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;
use trpg_ui::screens::battle::layout::MAP_VIEW;

/// The Quick Battle from the title, its banner closed and the cursor moved
/// a tile, with the `grid` channel on or off and, once there, the debug
/// tools on or off (Quick Battle is one of them).
fn quick_battle(channel: bool, tools: bool) -> Harness {
    let mut h = Harness::with_layout(RIGHT_HANDED);
    // Quick Battle, then Preparations: Left wraps to `Fight!`; the banner.
    h.keys("Down f Left f f").keys("f");
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    h.ctx_mut().debug_tools = tools;
    h.ctx_mut().debug_view = DebugView::default();
    h.ctx_mut().debug_view.set(GRID.id(), channel);
    h.keys("Right").wait(0.2);
    h
}

/// The gizmos of the frame drawn last.
fn gizmos(h: &Harness) -> Vec<GizmoItem> {
    h.game().buffer().gizmos()
}

/// The battle's rules state, hashed.
fn state_hash(h: &Harness) -> shuyi_core::StateHash {
    let battle = h.battle().unwrap_or_else(|| panic!("no battle"));
    shuyi_core::state_hash(battle.state()).unwrap_or_else(|e| panic!("the state doesn't hash: {e}"))
}

#[test]
fn the_grid_channel_outlines_the_cells_the_cursor_and_each_unit() {
    let h = quick_battle(true, true);
    assert!(h.ctx().debug_view.offered().contains(&GRID));
    let scene = h.map_scene().unwrap_or_else(|| panic!("no map"));
    let grid = h.ctx().game.map_skin.grid(&scene, MAP_VIEW);
    let drawn = gizmos(&h);
    assert!(drawn.iter().all(|g| g.channel == GRID.id()));
    // The cell lines and the map's bounds come first.
    let bounds = drawn.iter().take_while(|g| g.role == GizmoRole::Bounds);
    assert!(bounds.count() > 2);
    let notes: Vec<_> = drawn.iter().filter(|g| g.role == GizmoRole::Note).collect();
    let cell = |pos| GizmoShape::Rect {
        rect: grid
            .rect(pos)
            .unwrap_or_else(|| panic!("{pos:?} is off screen")),
    };
    // The cursor's cell, then each unit's cell and its index.
    let cursor = h.cursor_tile().unwrap_or_else(|| panic!("no cursor"));
    assert_eq!(notes[0].shape, cell(cursor));
    assert_ne!(scene.units, []);
    for (i, unit) in scene.units.iter().enumerate() {
        assert_eq!(notes[1 + 2 * i].shape, cell(unit.pos));
        let label = &notes[2 + 2 * i].shape;
        assert!(matches!(label, GizmoShape::Label { text, .. } if text.as_str() == i.to_string()));
    }
}

#[test]
fn with_the_channel_off_nothing_is_drawn_and_the_state_is_the_same() {
    let (on, off) = (quick_battle(true, true), quick_battle(false, true));
    assert_ne!(gizmos(&on), []);
    assert_eq!(gizmos(&off), []);
    // Still offered, so the debug menu's "Debug view" lists it.
    assert!(off.ctx().debug_view.offered().contains(&GRID));
    assert_eq!(state_hash(&on), state_hash(&off));
}

#[test]
fn with_the_debug_tools_off_nothing_is_built() {
    let h = quick_battle(true, false);
    assert_eq!(gizmos(&h), []);
    assert_eq!(h.ctx().debug_view.offered(), []);
}
