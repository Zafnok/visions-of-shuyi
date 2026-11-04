---
id: "1218"
title: "The dialogue tests keep only the game's hooks: the engine's player tests take the mechanics"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#3207"]
nick_input: none
completed:
---

# 1218 — The dialogue tests keep only the game's hooks

## Context

From the game/engine boundary audit of 2026-11-03. Since 1112,
`crates/ui/src/dialogue.rs` is a newtype over the engine's
`shuyi_ui::dialogue::DialoguePlayer<Hooks>`: the game's own logic is
`Hooks::fill` (the lead's and the story's name tokens), `Hooks::direct`
(`@place`, `@opening`), `take_opening` and flattening `view.stage` into
`View`; the other methods forward one line each. Yet
`crates/ui/src/dialogue/tests.rs` (1,184 lines) still tests the engine's
state machine: choices, skipping, music, `@if` parts and three property
tests. `docs/engine/boundary.md` (1112) kept those tests "unchanged" for
the carve; that was a carve-time choice.

Engine ticket
[engine#3207](https://github.com/Zafnok/shuyi-engine/blob/main/tickets/open/3207-the-dialogue-players-tests-cover-what-the-games-copy-tests.md)
ports the property tests and the cases the engine's player tests lack
onto its toy hooks, so deleting the game's copies loses no coverage.
`docs/engine/friction.md` row 16 accepted keeping the newtype itself; this
ticket keeps it.

**Expected size:** 0 production lines; about 650 test lines removed
(1,184 to about 530), nothing added but a few lines of doc comment.

## Nick input

None.

## Scope

**In:**
- Move the engine pin past engine#3207's merge (unless an open
  `engine-pin` PR has done so).
- Delete from `crates/ui/src/dialogue/tests.rs` the tests of the engine's
  machine (step 3's list) and the helpers only they use.
- Keep the tests of this game's hooks and content (step 4's list).

**Out (do not do):**
- Turning the newtype into a type alias (friction row 16: "cheaper
  kept").
- The dialogue screen's log and its `History`
  (`screens/dialogue/history.rs`), which parked engine#2766 may take (its note).
- The other `crates/ui` test files that re-test the engine (1232).

**Either order with 1233:** if 1233 is done, `docs/engine/boundary.md`'s
E-tests note says the dialogue player's copies go with 1218: change
those words to what this ticket deleted; if not, nothing in
`boundary.md` (1233 writes the note).

## Implementation steps

1. Move the engine pin to the engine's `main` (`cargo xtask engine-pin`,
   the `engine-pin` skill), past engine#3207's merge, unless an open
   `engine-pin` PR has already moved it; review what the pin passes.
2. Open engine#3207's completion notes: check that each case it lists by
   the game's test name is in the engine's
   `crates/ui/src/dialogue/player/tests.rs`. Delete only a test whose
   engine case is there.
3. Delete: `every_reply_rejoins_the_same_way`,
   `advancing_a_finished_scene_does_nothing`,
   `a_scene_without_text_starts_finished`,
   `expression_changes_and_offscreen_speakers`,
   `a_choice_waits_for_a_reply`, `a_choice_without_replies_is_passed_over`,
   `the_reaction_leaves_the_portraits_as_they_are`, the three music tests,
   `skipping_stops_at_choices`, the first `proptest!` block (its three
   tests with `arb_step`, `arb_step_or_choice`, `last_music`,
   `expected_texts`), `a_block_plays_the_part_for_who_is_there`,
   `the_part_not_played_leaves_no_trace`, `skipping_follows_the_parts_played`,
   `a_scene_with_nothing_to_say_for_those_there_starts_finished`, the
   `BLOCKS` scene and `texts`. Then any helper left unused (the compiler
   says which: `option`, `music`, `cue`, `parsed`, `says`, `narrates`
   are candidates).
4. Keep: `walks_the_test_scene` (the embedded `assets-test` scene with the
   lead's token), `tokens_follow_the_lead`, `name_tokens_are_filled_in`,
   `no_embedded_scene_shows_someone_who_is_gone`,
   `the_line_on_screen_and_each_reply_have_their_ids` (it is also the
   smoke test that `line`, `reply_line`, `pending_music`, `take_music` and
   `choose` forward), `the_place_lasts_until_the_next_place_line`,
   `an_opening_is_due_once_and_lasts_until_the_place_changes`.
5. Check by reading that every method of `DialoguePlayer` and the arms of
   `Hooks::direct` are still reached by a kept test (`skip_to_choice` by
   the opening test; `lead` only by `screens/dialogue.rs` and `log.rs`, so
   check the dialogue screen's tests reach it); add an assert to a kept test for any that isn't, so CI's
   mutation job finds no new survivor. Don't run `cargo mutants` locally.
6. Update the file's module doc: it tests this game's hooks and content;
   the player's mechanics are the engine's tests.

## Acceptance criteria

- [ ] `shuyi-engine.rev` is at or past engine#3207's merge commit.
- [ ] `crates/ui/src/dialogue/tests.rs` is about 550 lines or fewer and
      has none of step 3's tests; step 4's tests pass unchanged.
- [ ] The only files changed are `crates/ui/src/dialogue/tests.rs`, the
      pin (`shuyi-engine.rev`, `Cargo.lock`) and this ticket.
- [ ] `cargo xtask pr-size` reports the PR well under 3,000 lines.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the kept tests of step 4 (no new tests unless step 5 finds a
  gap).
- Property: `no_embedded_scene_shows_someone_who_is_gone`, kept.
- Snapshot / integration: none changed.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
