---
id: "1220"
title: "Battle, world map and pool files written back with the engine's comment-keeping RON editor"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#2244"]
nick_input: none
completed:
---

# 1220 — Tuning files edited with the engine's RON editor

## Context

From the game/engine boundary audit of 2026-11-03.
`crates/content/src/battle/tune.rs` (0509) writes the autobalance's
changed enemy list back into a battle file keeping its comments, with a
private RON scanner: `code` and its `In` state machine (blank strings and
comments), `field_at`, `is_name`, `indent`, `enemy_spans`, `add_to_list`
and `set_field`, about 140 production lines that name no game concept.
0509's friction row (`docs/engine/friction.md`) accepted it with "a
second game file written back this way would make it an engine (assets)
piece". 0510 (#520) met that: `crates/bots/src/skirmish.rs` calls
`tune::add_to_list` to add a place to a world map's `nodes` and `roads`
(`add_place`) and a battle to a pool file's `pool` (`add_to_pool`). No
friction row says so.

The engine had the same parser privately (`shuyi new`'s
`template/ron_merge.rs`).
[engine#2244](https://github.com/Zafnok/shuyi-engine/blob/main/tickets/open/2244-a-public-comment-keeping-ron-editor.md)
(split out of engine#2517's format upgrades, which reuse it)
makes it public as `shuyi_assets::ron_text` with the two edits the game
needs: append an item to the list at a field path, and set or add a field.

## Nick input

None. No file the game ships changes, and the autobalance's and the
skirmish generator's written files stay the same text.

## Scope

**In:**
- Move the pin past engine#2244's merge (`engine-change` skill §7, the
  `engine-pin` skill), unless an open `engine-pin` PR already has.
- `tune.rs`: `BattleText::parse` takes the enemy entries' spans from
  `ron_text::parse(source)?.at(&["enemies"])`'s items; `foe_text` uses
  `ron_text::set_field(&text, &[], key, value)`; `text` uses the engine's
  indentation helper or keeps a three-line `indent`. Delete `code`, `In`,
  `field_at`, `is_name`, `enemy_spans`, `add_to_list` and `set_field`.
- `crates/bots/src/skirmish.rs`: `add_place` and `add_to_pool` call
  `ron_text::append_to_list(source, &["nodes"], &node)` (and `roads`,
  `pool`), mapping `EditError` to their `String` errors; add
  `shuyi-assets.workspace = true` to `crates/bots/Cargo.toml` (the
  engine, already in the workspace: no new license).
- `crates/content/src/battle/tune/tests.rs`: keep the `BattleText` tests;
  delete the scanner's own (`strings_and_comments_are_blanked_…`,
  `field_at_finds_a_whole_name_at_its_depth`,
  `set_field_replaces_the_whole_value_…`,
  `set_field_finds_the_field_by_name_not_inside_strings`,
  `add_to_list_keeps_the_text_…`), which engine#2244 covers.
- `docs/engine/friction.md`: a row for 0510 (the trigger of 0509's row
  met, outcome engine feature, engine#2244) and this ticket's measured
  row.

**Out (do not do):**
- Changing what the autobalance changes or how the skirmish generator
  picks a place (0522 owns the generator's layouts).
- Edits the engine doesn't give (removing an item): `BattleText::text`
  keeps building the list from the entries' spans as it does now.

**Either order with 1174** (engine friction you can check): if 1174 is
done, write the 0510 friction row with its Area (`bundle`) and Net
columns; if not, write it as the rows are today, and 1174 backfills it.

## Implementation steps

1. **Pin** (as above).
2. Read engine#2244's `ron_text` docs, `tune.rs`, `tune/tests.rs`,
   `bots/src/skirmish.rs` (`add_place`, `add_to_pool`) and
   `bots/src/skirmish/tests.rs`.
3. `tune.rs` on `ron_text`, its `BattleText` tests green; then the
   bots' two writers, their tests green. If the engine's indentation of
   an appended item differs from the 8 spaces the bots pass today, update
   the expected text in `skirmish/tests.rs` and say why in the notes.
4. Delete the scanner and its tests; write the friction rows.
5. **Measure** with `cargo xtask code-lines`: estimate about 20
   production lines added, about 140 removed (and about 90 test lines).

## Acceptance criteria

- [ ] `shuyi-engine.rev` is at or past the commit that closed
      engine#2244.
- [ ] `grep -n "fn code\|enum In\|fn field_at\|fn add_to_list"
      crates/content/src/battle/tune.rs` finds nothing, and
      `grep -rn "tune::add_to_list" crates` finds nothing.
- [ ] `tune/tests.rs`'s `BattleText` tests (those that call
      `BattleText`, not the scanner) pass with their expected text
      unchanged.
- [ ] The bots' and xtask's skirmish tests pass.
- [ ] `docs/engine/friction.md` has the 0510 row and this ticket's row.
- [ ] All gates in the `run-gates` skill pass, including
      `cargo xtask ticket-lint --check-engine`.

## Tests required

- Unit: the existing `BattleText` and skirmish write-back tests.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
