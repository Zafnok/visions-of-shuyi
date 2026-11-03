---
id: "1212"
title: "Use the engine's sprite-test screen, and delete the game's copy"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: low
status: blocked
blocked_by: ["engine#3202"]
nick_input: none
completed:
---

# 1212 — Use the engine's sprite-test screen

## Context

From the game/engine boundary audit of 2026-11-03. The "Sprite test"
debug tool (ticket 0231, ADR-0038) is `SpriteTestScreen` in
`crates/ui/src/debug/sprite_test.rs`: the test card drawn at 1×, 3× and
5×, flipped, cropped, under the glyphs, at half opacity and cut by a
box drawn after it. `docs/engine/boundary.md` (the
`debug/sprite_test.rs` row) says it stayed in the carve (1112) because
"it draws this game's test card"; but the card is the engine's
(`crates/xtask/src/test_card.rs` only writes `shuyi_cli::test_card`'s
pixels to `assets-test/images/test_card.png`), and the screen names no
game concept. It is the same kind of tool as the scene camera, which
moved to `shuyi_ui::debug` in 1154.

[engine#3202](https://github.com/Zafnok/shuyi-engine/blob/main/tickets/open/3202-a-sprite-test-debug-screen-scales-flips-crops-opacity-and-clipping.md)
adds `shuyi_ui::debug::SpriteTestScreen` beside `SceneCameraScreen`,
given an image id and its size. This ticket points the debug menu at it
and deletes the game's copy, as `debug/scene_camera.rs` does for the
camera.

## Nick input

None. A debug tool; nothing a player sees changes.

## Scope

**In:**
- Move the pin past engine#3202 (`engine-change` skill §7, `engine-pin`
  skill), unless an open `engine-pin` PR has already done so.
- The debug menu's "Sprite test" opens the engine's screen with the
  game's card; the game's screen and its tests go.
- `docs/engine/boundary.md`: the sprite test's row. **Either order with
  1233:** if 1233 is done, change the sprite test's words in the row it
  left; if not, change them here, and 1233 has nothing left for this row.

**Out (do not do):**
- `TEST_CARD_PATH`, `assets-test/images/test_card.png` and `cargo xtask
  test-card` stay: the world map's tests (`screens/world_map/tests.rs`)
  and the content image tests load the card.
- Moving or changing any other debug tool or the menu's order.

## Implementation steps

1. **Pin** (as above), then read the engine's `debug/sprite_test.rs`.
2. `crates/ui/src/debug.rs`: drop `mod sprite_test`; re-export
   `shuyi_ui::debug::SpriteTestScreen`; add `fn sprite_test(ctx: &Ctx) ->
   SpriteTestScreen` that looks `TEST_CARD_PATH` up in
   `ctx.game.content.images` (`id`, then `info`) and passes
   `format!("No {}", display_path(TEST_CARD_PATH))` as what to say
   without it; tool 4 pushes `sprite_test(ctx)`. Update the module doc's
   sprite-test sentence (the screen is the engine's, ticket 3202).
3. Delete `crates/ui/src/debug/sprite_test.rs` (the screen and its unit
   tests).
4. `crates/ui/src/screens/actions.rs`: delete `SPRITE_TEST` and its row
   in the game's own list; add
   `ScreenActions::declared(SpriteTestScreen::NAME, ui_actions(&[UiAction::Cancel]))`
   beside the glyph sampler's.
5. `crates/ui/tests/it/sprite_test.rs`: keep only the test that the
   debug menu opens it with nine sprites of the card and Cancel closes
   it; delete the others and
   `snapshots/it__sprite_test__every_sprite_is_one_snapshot_line.snap`
   (the engine's tests pin the layout).
6. `docs/engine/boundary.md` (step "In"'s clause).
7. **Measure** with `cargo xtask code-lines`: about 120 production lines
   removed and 10 added; about 170 test lines removed.

## Acceptance criteria

- [ ] `shuyi-engine.rev` is at or past the commit that closed
      engine#3202.
- [ ] `crates/ui/src/debug/sprite_test.rs` is gone, and `grep -rn
      "SPRITE_TEST\b" crates/ui/src` finds only `SPRITE_TEST_TOOL`.
- [ ] `the_debug_menu_opens_the_sprite_test_and_cancel_closes_it`
      passes on the engine's screen; the debug menu's tests still say
      `Push(sprite_test)` for tool 4.
- [ ] The boundary row no longer says the sprite test stays.
- [ ] All gates in the `run-gates` skill pass, including
      `cargo xtask ticket-lint --check-engine`.

## Tests required

- Integration: the remaining harness test of the sprite test; the debug
  menu's and `screens::actions`' existing tests.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
