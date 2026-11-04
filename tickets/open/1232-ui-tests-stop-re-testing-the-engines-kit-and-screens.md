---
id: "1232"
title: "UI tests stop re-testing the engine's kit and screens: map-skin copies, the key-binding, picker and profile screens, the scene camera"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1232 — UI tests stop re-testing the engine's kit and screens

## Context

From the game/engine boundary audit of 2026-11-03. Three carves left
copies of the engine's tests behind, under the engine's test names:
`map_view/{sprite.rs,glyph/tests.rs,scene/tests.rs}` repeat the grid
kit's `grid/{sprite/tests.rs,glyph.rs,scene.rs}` (1115, 1167);
`tests/it/{key_bindings,key_bindings_screen,rebind_buttons,layout_picker,profiles}.rs`
test the behavior of screens that are now the engine's (1112, 1164;
engine `screens/{key_bindings,profiles}/tests.rs`,
`tests/it/{key_bindings,layout_picker}.rs`), which
`docs/engine/boundary.md:226-235` kept "until those screens move"; and
`debug/scene_camera.rs`, about 32 production lines since 1154, keeps
about 200 lines of tests of the engine's `SceneCameraScreen` (engine
`crates/ui/src/debug/camera/tests.rs`).

The rule: **a test goes only when an engine test checks the same case
and it reaches no game code the kept tests don't.** Whatever paints the
game's units (`SpriteUnits`, `GlyphUnits`: HP bars, effect marks,
facing, edge arrows), `UnitView`'s fields and text, Nick's default keys,
the game's font, its settings migration and its per-profile storage stay.

**Expected size:** 0 production lines; about 1,350 test lines removed
(about 480 in `sprite.rs`, 180 in `glyph/tests.rs`, 25 in
`scene/tests.rs`, 180 in `scene_camera.rs`, 480 in `tests/it` and the
profile snapshots), nothing added.

## Nick input

None.

## Scope

**In:** the deletions of steps 1-4, the helpers only they use, their
snapshot files, and the boundary note of step 5.

**Out (do not do):**
- Production code, and making any test faster. **Either order with
  0128:** if 0128 is done, delete `tiles_lie_inside_the_area_edge_to_edge`
  as it is now; if not, 0128 has one slow test fewer (its *Out* says so).
- `crates/ui/src/dialogue/tests.rs` (1218) and the `crates/content` tests (1231).
- **Either order with 1237:** if 1237 is done, the files deleted here
  may use `tests/it/support.rs`'s shared drivers: delete them and keep
  `support.rs`; if not, nothing to do (1237 skips the files already gone).
- The edge-arrow tests (`an_edge_arrow_sits_on_its_middle_tile_above_the_hp_bar_and_fades`,
  `on_glyph_terrain_the_edge_arrows_are_the_glyph_skins`,
  `an_arrow_in_the_cursors_colour_on_each_edge_the_map_goes_on_past`).
  **Either order with 1213:** if 1213 is done, they are already the
  engine's or changed; if not, leave them for 1213.
- **Either order with 1214:** if 1214 is done, the kept tests already
  read frames through the engine's readers; if not, delete as they are,
  and 1214 has fewer copies to change.

## Implementation steps

1. `crates/ui/src/map_view/sprite.rs` tests: delete
   `threat_arrows_and_marks_are_painted_last_on_the_skins_tiles`,
   `view_tiles_and_the_margin_follow_the_tile_size`,
   `the_zoom_multiplies_the_tile_and_divides_the_view`,
   `at_twice_the_zoom_everything_is_twice_the_size`,
   `a_tile_is_its_terrain_sprite`, `the_cursor_is_dark_edged_corner_marks_inside_its_tile`,
   `the_path_runs_under_units_and_its_arrowhead_over_them`,
   `the_ring_and_the_look_are_painted_by_a_tileset_that_has_layers_and_looks`,
   `the_light_is_painted_by_a_tileset_that_has_terrain_tiles` and the
   proptest `tiles_lie_inside_the_area_edge_to_edge` (kit geometry:
   `view_tiles`, `tile_px`). Where one also asserts on a unit's picture or
   HP bar, move that assert into a units test first. Keep
   `every_scene_feature_is_painted` (it lists `UnitView`'s fields),
   `a_tileset_without_terrain_paints_units_on_the_glyph_skins_ground`
   (`SpriteUnits` clears a unit's glyphs), the snapshots and the
   proptests `sprite_skin_paints_only_the_area` and
   `sprite_units_skin_paints_only_the_area` (they paint `SpriteUnits`).
2. `map_view/glyph/tests.rs`: delete
   `threat_arrows_and_marks_are_painted_last_over_everything`,
   `paints_terrain_then_tints_then_units_path_and_cursor`,
   `a_changed_tile_flashes_and_a_tile_a_spell_would_change_shows_what_it_becomes`,
   `units_and_the_cursor_off_the_view_are_not_drawn` (engine `grid/glyph.rs`
   tests). Keep `glyph_skin_paints_only_the_area` (it paints `GlyphUnits`)
   and its `any_*` generators. `map_view/scene/tests.rs`: delete
   `tints_go_on_visible_tiles_in_the_order_laid_on` and
   `only_units_on_visible_tiles_are_kept`; keep the `to_text` tests (the
   game's `unit_text`, `scene.rs:236`).
3. `crates/ui/src/debug/scene_camera.rs`: delete
   `confirm_steps_the_zoom_through_the_whole_steps_and_round`,
   `held_cursor_keys_pan_by_the_pixel_and_stop_at_the_edges`,
   `zooming_keeps_the_point_where_the_window_can_show_it`,
   `cancel_closes_with_the_cancel_sound`,
   `the_scene_shows_only_through_the_windows_see_through_cells` and the
   helpers left unused (`zoom_to`, `pan`, `frame`, `near`). Keep
   `the_sizes_are_the_windows_and_the_buffers_pixels`,
   `the_scene_is_the_whole_test_map_as_the_skin_paints_it`,
   `says_so_when_the_map_is_missing`, and `tests/it/scene_camera.rs`.
4. `crates/ui/tests/it`: delete from `key_bindings.rs`
   `a_rebound_key_works_at_once_and_the_old_one_doesnt`,
   `escape_cancels_in_both_layouts`, `escape_cancels_even_with_cancel_unbound`,
   `shift_escape_can_be_bound_and_plain_escape_still_cancels`,
   `custom_keys_survive_a_restart`, `each_layout_keeps_its_own_keys`; from
   `key_bindings_screen.rs` `a_key_bound_twice_moves_and_the_loser_shows_not_mapped`,
   `leaving_is_blocked_until_confirm_has_a_key_again`,
   `escape_backs_out_of_capture_and_delete_empties_a_slot`,
   `edits_survive_a_restart_and_stay_with_their_layout`,
   `restore_defaults_leaves_the_other_layout_alone`; from
   `rebind_buttons.rs` `a_button_moved_off_confirm_blocks_leaving_until_it_has_one_again`,
   `a_hold_backs_out_a_tap_binds_and_clear_empties`,
   `switching_layout_leaves_the_buttons_unchanged`; from
   `layout_picker.rs` `picker_works_with_either_hand`,
   `picker_cannot_be_cancelled`, `the_choice_persists_across_restart`; from
   `profiles.rs` the three name-box tests (`cancel_in_the_name_box_…`,
   `a_name_another_profile_has_is_refused_and_the_box_stays`) and, from
   `deleting_on_the_screen_asks_first_and_takes_the_saves`, the question
   (keep that the saves go). In `screens/profile/tests.rs`, keep
   `profile_screen_snapshot` and delete the other two with their `.snap`
   files. Keep Nick's keys (`left_handed_wasd_moves_and_j_confirms`…), the
   split keys, the debug key, `every_glyph_drawn_is_in_the_font`,
   `a_rebound_button_plays_a_battle`, the version-1 config and corrupt-keys
   tests and the per-profile storage tests.
5. `docs/engine/boundary.md` (the E-tests note, `:210-235`). **Either
   order with 1233:** if 1233 is done, change its "go with 1232" words to
   what this ticket deleted, with the new line count; if not, leave the
   note to 1233.
6. Check by reading that each game function a deleted test reached is
   still reached by a kept one (the rows `KEY_ROWS`, `scene_camera()`,
   `map_picture()`); add an assert if not, so CI's mutation job finds no
   new survivor. Don't run `cargo mutants` locally.

## Acceptance criteria

- [ ] None of the tests named for deletion in steps 1-4 is left.
- [ ] The tests named as kept pass unchanged.
- [ ] Only test files and snapshots under `crates/ui/`, `docs/engine/boundary.md`
      and this ticket change.
- [ ] `cargo xtask pr-size` reports the PR under 2,000 lines.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit, property and integration: the kept tests, unchanged.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
