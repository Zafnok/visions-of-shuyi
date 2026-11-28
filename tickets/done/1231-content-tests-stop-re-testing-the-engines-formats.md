---
id: "1231"
title: "Content tests stop re-testing the engine's formats: the .dlg syntax, line ids and language-pack mechanics"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-28
---

# 1231 — Content tests stop re-testing the engine's formats

## Context

From the game/engine boundary audit of 2026-11-03. Since 1105 the `.dlg`
format (parser, choices, `@if`, line ids, `Scene::resolved`, printing) is
`shuyi_assets::dialogue` and the pack mechanics (status, orphans, lookups,
loading) are `shuyi_assets::lang`; `crates/content/src/dialogue.rs:1-30`
says the game adds only its `Stage` directives and its `SceneRules`. Yet
`crates/content/src/dialogue/tests.rs` and `tests/{choice,presence,line_ids}.rs`
still test the format case for case, by the engine's test names (engine
`crates/assets/src/dialogue/tests.rs`), and `lang/data_tests.rs` copies
the engine's `lang/pack_tests.rs` scaffold and its tests. The game's
copies parse with `Stage` and check with the game's rules, so a few
error lists also hold game errors: those lines stay, in a game test.

The rule: **a game test goes only when an engine test checks the same
case.** A case the engine lacks stays here (porting it to the engine is a
possible engine ticket, not this one). Known gaps the audit found:
`scenes_without_an_end_get_ids_too`,
`inserting_or_moving_a_line_leaves_the_other_ids_unchanged`,
`changing_one_word_changes_that_lines_id_only`, the property
`inserting_a_line_never_changes_another_id`, and
`blocks_nest_and_stand_in_reactions` (`@if` inside a reaction).

**Expected size:** 0 production lines; about 1,400 test lines removed
(about 1,150 in `dialogue/`, about 250 in `lang/data_tests.rs`), about 40
added (a round trip of the game's directives).

## Nick input

None.

## Scope

**In:**
- Delete the content tests that copy an engine test (steps 2-5), and the
  helpers and `arb_*` generators only they use.
- Keep: the game's directives (`@place`, `@opening`, `@dies`, `@joins`)
  and scene rules (cast, portraits, the lead's lines, names written out,
  music cues), `Cast`/`check_presence` and its property, the embedded
  scripts, the README examples, the game's `PackRules` (data text, tips,
  lessons, terrain and battle notes, `m`/`f` lines) and the test pack.

**Out (do not do):**
- Production code. **Either order with 1230:** if 1230 is done,
  `dialogue::load`, `lang::pack_from_sources`, `Lang::new`, `load_ron`
  and `enums.rs` are gone with their tests, and the kept tests call
  `load_scripts(..).map(|s| s.table)`; if not, leave them (1230 deletes
  them). Both ways the test helpers `lang::pack_from_files` and
  `dialogue::from_sources` stay, even if this ticket deletes some of
  their callers.
- **Either order with 1223:** if 1223 is done, the kept
  `lang/data_tests.rs` tests expect its messages (unchanged by it);
  delete as listed; if not, nothing to do (1223 keeps what is left).
- **Either order with 1237:** if 1237 is done, a content test this
  ticket deletes may build its `ClassDef` from `trpg_core::testing`:
  delete it as it is; if not, nothing to do (1237 skips the files
  already gone).
- `crates/ui/src/dialogue/tests.rs` (1218) and the other `crates/ui`
  copies (1232).
- Adding the cases the engine lacks to the engine's tests.

## Implementation steps

1. For each test below, open the engine test named beside it (engine
   `crates/assets/src/dialogue/tests.rs`, `crates/assets/src/lang/pack_tests.rs`
   at the pin) and confirm it checks the same case. Where the game's copy
   also asserts a game error (an unknown character, an expression), move
   that assert into a kept game test before deleting.
2. `dialogue/tests.rs`: delete `parses_every_line_type`,
   `records_the_line_of_every_step`, `continuation_lines_join_with_one_space`,
   `several_scenes_per_file_and_crlf`, `unknown_directive`,
   `directive_arguments`, `invalid_ids`, `unrecognised_lines`, `empty_text`,
   `misplaced_indented_lines`, `lines_outside_scenes`, `missing_end`,
   `non_ascii_text`, `files_that_are_not_utf8`,
   `duplicate_scene_ids_across_files`, `every_error_is_reported_in_line_order`,
   `side_names`, `step_text`, and the music parse/print tests
   (`music_lines_parse_anywhere_in_a_scene`, `music_needs_exactly_one_argument`,
   `music_line_arguments`, `music_prints_as_written`) that the engine's
   `sides_steps_and_music` covers. Replace `print_parse_round_trip` and its
   `arb_*` generators (`:874-985`) with a round trip of the game's `Stage`
   lines only (`Stage::Setting` with `PlaceLine::{Map, Battle, None}`,
   `Stage::{Opening, Dies, Joins}`), about 40 lines: the engine's own
   round trip covers every other step.
3. `tests/choice.rs`: delete `parses_choice_blocks`, `choice_syntax_errors`,
   `option_count`, `option_text_limit`, `reaction_limit`, `prints_choices`,
   `choice_step_has_no_text` (engine `choices_parse_with_their_reactions`,
   `choice_structure_errors`, `option_limits`, `print_writes_every_step_back`).
4. `tests/presence.rs` (the `@if` half, `:1-506`): delete `parses_if_blocks`,
   `if_block_syntax_errors`, `blocks_left_open_and_out_of_place`,
   `both_parts_leave_the_same_screen_and_caption`,
   `lines_in_both_parts_get_ids_in_script_order`, `blocks_print_as_written`,
   `present_is_everyone_or_only_some`, `a_resolved_scene_keeps_the_parts_that_hold`,
   `a_scene_with_nothing_left_to_say_has_no_text` (engine
   `if_blocks_parse_with_their_else`, `if_structure_errors`,
   `every_part_of_a_block_leaves_the_same_screen_caption_and_weather`,
   `every_line_gets_an_id_in_script_order`, `a_scene_resolves_for_those_present`).
   Keep everything from `cast` (`:507`) on.
5. `tests/line_ids.rs`: delete `the_hash_of_known_lines_is_fixed`,
   `speech_narration_replies_and_reactions_get_ids`, `an_id_shows_as_its_text`,
   `the_id_comes_from_the_scene_the_speaker_and_the_text_as_written`,
   `repeats_of_a_line_in_a_scene_are_numbered`,
   `different_lines_with_the_same_hash_are_an_error`,
   `a_scene_built_by_hand_gets_ids_when_asked`, `a_line_id_names_its_scene`;
   keep the gaps listed in *Context*. In `tests/names.rs`, delete only what
   the engine's `tokens_and_names` checks (`known_name_tokens_pass`,
   `unknown_name_ids`, `unterminated_name_token` if it does).
6. `lang/data_tests.rs`: delete
   `english_lines_replies_and_captions_have_their_kind_and_scene` (engine
   `:138`), `data_lookups_use_the_pack_while_its_source_is_todays_english`
   (`:430`), `a_name_token_counts_as_the_packs_longest_name` (`:324`),
   `status_lists_what_a_pack_lacks_of_each_kind` and
   `an_orphan_is_paired_with_the_nearest_untranslated_line_of_its_scene`
   (`status_lists_data_and_lines_and_pairs_orphans`, keep any orphan case
   it lacks), `a_packs_data_and_dialogue_files_are_loaded_and_checked`
   (`from_files_reads_data_and_dialogue`). Keep
   `a_dialogue_key_is_a_line_id_or_a_caption_key_used_once` (the engine
   checks one bad key, not its list).
7. Check by reading that each game function a deleted test reached
   (`load_files`, `lang::from_files`, `Stage::parse`/`print`) is still
   reached by a kept test; add an assert to one if not, so CI's mutation
   job finds no new survivor. Don't run `cargo mutants` locally.
8. Update each test file's module doc: the format's cases are the
   engine's tests; these test this game's directives, rules and data.

## Acceptance criteria

- [x] None of the tests named in steps 2-6 is left, except those kept by
      step 1's or step 6's rule (named in the completion notes).
- [x] The tests named as kept in *Scope* and *Context* pass unchanged.
- [x] Only test files under `crates/content/src/` and this ticket change.
- [x] `cargo xtask pr-size` reports the PR under 2,000 lines.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Property: the new round trip of the game's `Stage` lines.
- Unit: the kept tests, unchanged.

## Completion notes

Done against the engine at the pin (`230080d`), test by test. 1230 and
1223 are not done, so `dialogue::load`, `Lang::new` and the rest stay
as they are; 1237 is done and touched none of the deleted tests.

**Deleted** (each checked against its engine twin): every test named in
steps 2-6, except the ones below. `print_parse_round_trip` and its
`arb_*` generators gave way to `the_games_directives_print_and_parse_back`,
a property over every `Stage` line (`@place` map/battle/none, with or
without `bright`, `@ambience` bed/none, `@opening`, `@dies`, `@joins`).
`@ambience` (ticket 1383) came after this ticket was written; it is in
the round trip too. Helpers left with no caller went with them (`option`,
`CHOICE`, `BLOCKS`, `files`).

**Kept by step 1's rule** (the engine lacks the case):
- `non_ascii_text`: plain ASCII is this game's rule
  (`dialogue.rs:443`), not the format's.
- `the_id_comes_from_the_scene_the_speaker_and_the_text_as_written`: the
  engine doesn't check that the expression, a reply's tone and the tokens
  as written stay out of a line id.
- `known_name_tokens_pass`, `unknown_name_ids` (`names.rs`): the
  capitalised `{N:...}` form and the game's hint naming `names.ron`.
  Only `unterminated_name_token` went.
- The gaps the ticket listed, all kept. The line-id property got its own
  small scene generator in `line_ids.rs`, since the old one went with
  the round trip.
- `status_lists_what_a_pack_lacks_of_each_kind`, cut down and renamed
  `status_lists_screen_text_then_data_then_dialogue`: the engine checks
  what a status holds, not its order or the game's counts.
- `an_orphan_is_paired_with_the_nearest_untranslated_line_of_its_scene`,
  cut down to the cases the engine lacks: an orphan is not missing or
  stale, a translated line is no candidate, a line never pairs with a
  caption, a caption with no candidate, repeats (`_2`) and key order.

**Game asserts moved** before deleting (step 1): from `invalid_ids`, an
invalid id is still looked up as a character with its expression (into
`unknown_characters`); from `choice_syntax_errors`, a reply's text is
plain ASCII too (into `non_ascii_text`); from `if_block_syntax_errors`,
an `@if` on an invalid id or no id is still looked up (into
`a_blocks_character_must_be_known`).

**Reach (step 7):** `load_files` is still reached by
`the_embedded_scripts_are_checked_against_the_audio_manifest`;
`lang::from_files` by every test of the embedded languages (`load`);
`Stage::parse`/`print` by the new property; `Lang::item_name` and
`class_name` by `the_test_pack_translates_a_little_of_every_kind`.

No gameplay rule decided. Follow-up: none (porting the kept cases to the
engine is the engine's call, outside this ticket).
