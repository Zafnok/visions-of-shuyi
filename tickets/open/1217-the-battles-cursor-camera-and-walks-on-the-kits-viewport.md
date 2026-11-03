---
id: "1217"
title: "The battle's cursor, camera and walks on the kit's Viewport and walk helpers, and one UnitView gait helper"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#3206"]
nick_input: none
completed:
---

# 1217 — The battle's cursor, camera and walks on the kit's Viewport

## Context

Found by the game/engine boundary audit of 2026-11-03. The battle
screen wires the tactics kit's `Cursor` and `Camera` together by hand,
and five places re-derive walk progress from `walk::steps`:

- `BattleScreen` keeps `camera`, `view` and `cursor` as three fields
  (`crates/ui/src/screens/battle/mod.rs`, lines 340 to 344), with
  `follow` (1456), `camera_in` (1465), `fit_camera` (2649 to 2661),
  step-then-follow in `move_cursor` (1947 to 1957), jump-then-follow at
  1835, 1938 and 1991, and `cycle`'s reading-order search (1981 to
  1994). `cursor::dir` (`battle/cursor.rs`, lines 17 to 25) matches
  the cursor actions onto `Dir`, which `Action::ui` already maps
  (`crates/content/src/keymap.rs`, line 287).
- `mode.rs`'s `walk_steps`, `walk_done`, `walk_pos` (755 to 772) and
  `tiles_entered` (697 to 712); `ai_phase.rs`'s `steps_at`,
  `tiles_entered` and `walker_pos` (196 to 233); `flight.rs`'s
  `Flight::views` (64 to 85); `cinematic/action.rs`'s `walk_on` (119
  to 138).
- Three hand copies of a `Gait`'s facing, frame and offset onto a
  `UnitView`: `animate` (`mod.rs`, 2631 to 2644), `flight.rs` (77 to
  82), `walk_on` (135 to 137).

Engine 3206
([link](https://github.com/Zafnok/shuyi-engine/blob/main/tickets/open/3206-a-grid-viewport-reading-order-cycling-and-walk-progress.md))
adds `grid::Viewport`, `grid::cycle`, `walk::{tile_at, arrived,
entered}` and `UiAction::dir`. The engine's `MapUnit` has no facing, so
the gait helper stays here.

## Nick input

None. Behavior is unchanged; the battle, AI-phase, flight and
cinematic tests are the check.

## Scope

**In:**
- `BattleScreen` holds one `Viewport` for `camera`, `view` and
  `cursor`; `follow`, `camera_in` and `fit_camera` go;
  `move_cursor`, `cycle`, the jump-then-follow pairs and `refit`'s
  camera half call it. `refit` keeps `player_view` and the AI pan
  (`followed` gives the pan's end camera, as in `apply_ai`).
- `cycle` calls `grid::cycle(&self.ready_units(), ..)`; `ready_units`
  stays (which units are ready is this game's).
- `cursor::dir` goes: `move_cursor` uses `action.ui().and_then(UiAction::dir)`;
  `battle/cursor.rs` keeps its re-exports.
- `mode.rs`, `ai_phase.rs`, `flight.rs` and `cinematic/action.rs` use
  `tile_at`, `arrived` and `entered`. Each keeps its own clock rules:
  Moving's hold skip, the AI's played-out guard and `walk_start`, the
  cinematic's milliseconds and its arrival filter.
- `UnitView::with_gait(self, Gait) -> Self`
  (`crates/ui/src/map_view/scene.rs`) for the three gait copies.

**Out (do not do):**
- The AI pan's lerp (`AiAction::camera`): 1228.
- `map_view/place.rs`'s `centred_on`, a one-off picture.
- Any change to walk speeds, pacing or what the camera shows.

**Either order with 1228:** both edit `ai_phase.rs`, in different
functions. If 1228 is done, keep its `camera`; if not, nothing to do
(1228 leaves the walk functions to this ticket).

**Either order with 1230:** if 1230 is done, `screens/battle/camera.rs`
is gone (the battle imports `shuyi_ui::grid::Camera` directly) and
`cursor.rs` re-exports only `BLINK_MIN` and `Cursor`: build on that; if
not, nothing to do (1230 deletes what this ticket leaves).

**Either order with 1233:** if 1233 is done, its row for
`screens/battle/{camera,cursor,path,walk}.rs` (`docs/engine/boundary.md:206`)
says the cursor, camera and walks move with 1217: write them as moved;
if not, change that row's words here, and 1233 leaves them.

## Implementation steps

1. Move the pin past engine 3206's merge (`engine-change` §7, the
   `engine-pin` skill), unless an open `engine-pin` PR has done so.
2. Replace the three fields with a `Viewport` in `BattleScreen::new`
   (line 524), then each site above. `BattleScreen::cursor()` and
   `camera()` read the viewport's.
3. Move `cycle` and `move_cursor`; delete `cursor::dir` and its test
   (the engine tests `UiAction::dir`; `keymap.rs` tests `Action::ui`).
4. The walk helpers in `mode.rs`, `ai_phase.rs`, `flight.rs` and
   `walk_on`.
5. `UnitView::with_gait` with a unit test; use it in the three places.
6. `cargo xtask code-lines` before and after. Expect about 50
   production lines removed and 15 added; record it in the completion
   notes.

## Acceptance criteria

- [ ] `cargo xtask ticket-lint --check-engine` passes with the pin past
      engine 3206.
- [ ] `fit_camera`, `camera_in`, `BattleScreen::follow`, `cursor::dir`,
      `walk_steps`, `walk_done` and `walk_pos` are gone (`grep`).
- [ ] No production file under `crates/ui/src` calls `walk::steps`
      outside `ai_phase.rs`'s played-out guard.
- [ ] The facing, frame and offset of a `Gait` are copied in one place,
      `UnitView::with_gait`.
- [ ] Every battle, AI-phase, flight and cinematic test and snapshot
      passes unchanged (`cargo test --no-fail-fast`).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `UnitView::with_gait`.
- Integration: the existing battle-screen tests of cursor, camera,
  next/previous unit, skin change (`refit`) and AI phase, unchanged.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
