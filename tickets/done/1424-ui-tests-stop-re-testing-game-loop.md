---
id: "1424"
title: "UI tests stop re-testing the engine's Game loop, Ctx and Options screen: game.rs, screen.rs and options/tests.rs"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-28
---

# 1424 — UI tests stop re-testing the engine's Game loop, Ctx and Options screen

## Context

This is a sibling of 1232, under the same rule, found by the engine's
redundancy sweep of 2026-11-28: **a test goes only when an engine test
checks the same case and it reaches no game code the kept tests don't.**
1232 names the map-skin copies, the key-binding, picker and profile
screens and the scene camera. It does not name these three files:

- `crates/ui/src/game.rs`: 1,083 lines, its tests from line 170. 18
  test names match the engine's `crates/ui/src/game/tests.rs` (some are
  helper functions). The exact-name matches are about 212 lines.
- `crates/ui/src/screen.rs`: 1,296 lines, its tests from line 504. 25
  names match the engine's `crates/ui/src/ctx/tests.rs`, about 242
  lines.
- `crates/ui/src/screens/options/tests.rs`: 725 lines. 6 tests repeat
  the engine's `crates/ui/src/screens/options/tests.rs`, about 160
  lines.

More near-name matches (pad events, the music fade, fullscreen,
`play_voice`, preload and a failed save) may also go.

**Expected size:** 0 production lines. About 600–700 test lines removed,
nothing added.

## Nick input

None.

## Scope

**In:**
- In each file, delete the tests that check the same case as an engine
  test and reach no game code.
- Keep every test that runs through the game's own keymap, palette,
  voice manifest, settings migration or data.
- Keep one smoke test per kind (the loop, the context, the options
  screen), so the mutation job finds no new survivor.

**Out (do not do):**
- 1232's files (either order: the two tickets touch different files).
- Changing a production line.

## Implementation steps

1. For each test in the three files, find the engine test of the same
   case at the pin. Use the names first, then read the bodies.
2. Delete those that pass the rule. List the deleted tests and their
   engine twins in the completion notes.
3. Run the ui crate's tests and the diff-mutation job's local check on
   the three files.

## Acceptance criteria

- [x] Every deleted test is listed in the completion notes beside the
      engine test that checks the same case.
- [x] No production line changes (`git diff --stat` touches test code
      only).
- [x] The ui crate's tests pass, and the PR's mutation job reports no new
      survivor in `game.rs`, `screen.rs` or the options screen.

## Tests required

- None new. The kept tests and the engine's tests are the regression
  suite.

## Completion notes

Done. 0 production lines; 949 test lines removed and 15 written (two
tests cut down to their game part, and the imports), more than the
600–700 the ticket expected because `screen.rs` had more twins than its
exact names showed. Tests: `game.rs` 41 → 20, `screen.rs` 35 → 16, the
Options tests 18 → 15. The engine tests are at the pin `230080d`, in
`crates/ui/src/game/tests.rs` (the loop), `ctx/tests.rs` (the context)
and `screens/options/tests.rs` (the Options screen).

**Deleted, `crates/ui/src/game.rs`** (twin in the engine's `game/tests.rs`):

| Deleted | Engine twin |
| --- | --- |
| `screens_see_actions_dt_and_held_keys` | `screens_see_actions_dt_and_held_keys` |
| `screens_see_the_keys_pressed_and_the_text_typed` | `screens_see_the_keys_pressed_and_the_text_typed` |
| `screens_see_unbound_and_shifted_presses_as_chords` | `screens_see_the_keys_pressed_and_the_text_typed` (the shifted, unbound chord) |
| `screens_see_the_buttons_that_went_down_and_up` | `screens_see_the_buttons_that_went_down_and_up` |
| `screens_see_held_buttons` | `screens_see_the_buttons_that_went_down_and_up` (Confirm held, then let go) |
| `the_clock_adds_up_frame_times` | `the_clock_adds_up_frame_times` |
| `rebinding_keys_reaches_the_input_before_the_next_press` | `rebinding_keys_reaches_the_input_before_the_next_press` |
| `popping_the_last_screen_quits` | `popping_the_last_screen_quits` |
| `quit_stops_further_frames` | `a_toy_game_of_two_screens_pushes_pops_and_quits` (the frames after quitting) |
| `a_key_press_moves_the_key_prompt_on` | `a_press_moves_the_key_prompt_on` |
| `a_pad_button_moves_the_key_prompt_on` | `a_press_moves_the_key_prompt_on` |
| `the_key_that_opens_the_picker_counts_as_the_keyboard` | `with_no_layout_the_first_key_opens_the_first_key_screen` (the device is the keyboard) |
| `debug_key_does_nothing_without_debug_screens` | `the_debug_key_opens_the_debug_screen_once` (`with_debug_screens(false)`) |
| `toggling_fullscreen_flips_the_setting_and_tells_app` | `frames_carry_the_gains_and_fullscreen` |
| `pad_changes_become_pad_events_with_the_pressing_pads_kind` | `pad_changes_become_events_of_each_pad_with_its_id` |
| `frames_carry_the_audio_screens_asked_for` | `frames_carry_the_audio_screens_asked_for` |
| `frames_carry_the_music_fade` | `frames_carry_the_music_fade_of_the_manifest` |
| `the_fade_length_comes_from_the_manifest` | `frames_carry_the_music_fade_of_the_manifest` (the one-second fade) |
| `the_quitting_frame_still_sounds_and_later_ones_are_silent` | `the_quitting_frame_still_sounds_and_later_ones_are_silent` |
| `an_unknown_cue_panics_in_debug_builds` | `an_unknown_cue_panics_in_debug_builds` |
| `the_music_app_reports_becomes_the_clock` | `the_music_the_platform_reports_becomes_the_clock` |

Their helpers went with them: `Noisy`, `noisy`, `beep`, `music`, `load`,
`begin`, `stop`, and `screen::tests::ctx_with_cues`, which only they
used. **Cut down:** `the_context_knows_the_device_pressed_last` is now
`picking_a_layout_keeps_the_device_pressed_last`. Its first part is the
engine's `the_context_knows_the_device_pressed_last`, but no engine test
checks that a new layout keeps the device.

**Deleted, `crates/ui/src/screen.rs`** (twin in the engine's `ctx/tests.rs`):

| Deleted | Engine twin |
| --- | --- |
| `an_unknown_text_key_panics_in_debug_builds` | `an_unknown_text_key_panics_in_debug_builds` |
| `text_comes_from_the_language_in_use` | `text_comes_from_the_language_in_use` |
| `a_new_ctx_has_no_layout_and_the_picker_keys` | `a_new_ctx_starts_on_the_games_palette_console_and_picker_keys` |
| `use_layout_switches_without_saving` | `use_layout_switches_without_saving` |
| `choose_layout_saves_and_saved_layout_reads_it_back` | `choose_layout_saves_it_in_the_settings_and_saved_layout_reads_it_back` |
| `a_bad_saved_layout_counts_as_none` | `a_saved_layout_the_keymap_doesnt_list_is_ignored` (an unknown old key, spaces around it) |
| `settings_are_saved_on_change_and_loaded_with_the_storage` | `settings_are_saved_on_change_and_loaded_with_the_storage` |
| `previewed_settings_are_saved_only_when_kept` | `previewed_settings_are_saved_only_when_kept` |
| `change_settings_cant_change_the_layout` | `change_settings_cant_change_the_layout` |
| `unreadable_settings_give_the_defaults_and_a_warning` | `unreadable_settings_give_the_defaults_and_a_warning` |
| `unreadable_storage_gives_default_keys_and_a_warning` | `a_failing_storage_leaves_the_defaults_and_a_warning_for_each` |
| `a_failed_save_still_changes_the_settings` | `a_failing_storage_leaves_the_defaults_and_a_warning_for_each` |
| `a_failed_save_still_switches_layout` | `a_failing_storage_leaves_the_defaults_and_a_warning_for_each` |
| `set_layout_bindings_saves_and_switches_the_layout_in_use` | `set_layout_bindings_saves_and_switches_the_layout_in_use` |
| `saved_keys_load_with_the_storage` | `set_layout_bindings_saves_and_switches_the_layout_in_use` (loaded with the storage) |
| `a_failed_save_still_switches_keys` | `set_layout_bindings_saves_and_switches_the_layout_in_use` (a failed save) |
| `set_pad_bindings_saves_and_applies_in_every_layout_and_before_one_is_chosen` | `set_pad_bindings_applies_in_every_layout_and_before_one_is_chosen` |
| `repaired_keys_leave_a_warning` | `repaired_keys_leave_a_warning` |
| `preload_lists_the_playable_clips_in_order` | `preload_lists_the_playable_clips_in_script_order` |

Their helpers went with them: `info_on_g`, `info_on_right_trigger` and
the `Failing` storage.

**Deleted, `crates/ui/src/screens/options/tests.rs`** (twin in the
engine's `screens/options/tests.rs`):

| Deleted | Engine twin |
| --- | --- |
| `restore_defaults_asks_first_and_keeps_the_layout` | `restore_defaults_asks_first_and_keeps_the_layout` |
| `discard_puts_every_setting_back_and_saves_nothing` | `discard_puts_every_setting_back_and_saves_nothing` |
| `help_names_what_confirm_does_on_each_kind_of_row` | `the_layout_and_key_bindings_rows_open_their_screens` and `a_row_that_isnt_a_setting_is_kept_or_put_back_with_the_settings` (the help for each kind of row) |

**Cut down:** `a_failed_save_keeps_the_change_says_so_and_stays` is now
`a_tips_reset_that_cant_be_saved_says_so`. A change that can't be kept
is the engine's test of that name, but a Reset tips that can't be saved
is this game's row (`options.rs`, `NOT_SAVED_MESSAGE`).

**Kept**, because they reach the game's own code or data. In `game.rs`:
the start functions (`start`, `start_on_scene`, the saved layout), the
game's layout picker and title through its keymap
(`events_reach_the_top_screen` and `pad_buttons_reach_the_top_screen`
are the loop's smoke tests), the Playdate's buttons, the debug menu's
screens, the Harness's typing and the title music after a battle. In
`screen.rs`: the voice manifest's checks
(`play_voice_asks_only_for_lines_with_a_playable_clip` is the voices'
smoke test), the old `layout` key and the version-1 settings file, the
game's palette, keymap and text (`ctx_uses_the_content_palette_and_keymap`
is the context's smoke test), the map skins, Classic to Casual and
`LoadError`. In the Options tests: this game's rows
(`rows_show_their_values` is the screen's smoke test). Other tests cover
the game parts of deleted cases: the settings round trip and migration in
`settings.rs`, the theme palettes and Restore defaults in
`tests/it/options.rs`, the Playdate's Options in
`options/playdate/tests.rs`.

**Checks:** `cargo test -p trpg-ui --no-fail-fast` passes (2,066, 534
and 2). `cargo mutants --in-diff --list` finds no mutants in the diff,
since it only deletes test code. A kept test still reaches each
production item a deleted test reached: `start` and `use_saved_layout`
through `a_saved_layout_skips_the_picker` and
`a_layout_in_use_beats_the_saved_one`, `hooks` through the picker and
debug-menu tests, `old_layout_key` through
`a_layout_saved_before_the_settings_is_still_found`, `debug_lang`
through `help_keys_name_keys_with_the_languages_words`, and `voices`
through the voice tests. No gameplay rules were decided, and nothing is
left for the Act 1 playtest.
