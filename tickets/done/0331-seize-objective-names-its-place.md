---
id: "0331"
title: "Seize objective names its place"
type: feature
milestone: Post–Chapter 1
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["0849"]
nick_input: none
completed: 2026-11-21
---

# 0331 — Seize objective names its place

## Context

A Seize objective's line names the terrain of its tile: `Seize the
{place}` (`battle.objective.seize` in `assets/lang/en/ui.ron`, built by
`objective_text` in `crates/ui/src/screens/battle/map_menu.rs`), so
Chapter 3's reads `Seize the throne`. Its tile is a `throne` (0037: the
scriptorium's Seize point is a building tile, `gate` or `throne`), but
the place is Greywater's scriptorium: the prior's seat at the end of the
long room over the gate (`docs/story/chapters/ch03.md`). A player reads
"the throne" in an abbey and looks for a king's.

0324 did the same for Escape (`exit_name`, `Get everyone to the Brennish
lines`) and left Seize out ("Seize keeps its terrain name"). Chapter 3
(0849) is the first story battle with a Seize objective and needs the
name.

## Nick input

None. The name comes from the chapter's beat sheet (`docs/story/`);
the chapter ticket writes it. Verified by tests.

## Scope

**In:**
- An optional battle-file field naming the Seize point, as `exit_name`
  names the exit (e.g. `seize_name: "the scriptorium"`, written bare),
  carried on `BattleDef` to the battle screen and translated by its
  English text (`battles.<id>.…`, `assets/lang/README.md`).
- `objective_text`: with a name, `Seize {place}` with the name (a new
  key); without, the current terrain line. Both in `en/ui.ron` and the
  `test` pack.
- Content checks: a name only with `Seize`; a non-empty one line. Or one
  field for both objectives instead of two, if that reads better (the
  session picks; say why).
- `assets/battles/README.md`; `assets/battles/ch03.ron` gets
  `the scriptorium`.

**Out (do not do):**
- How seizing works; any other objective's line.

## Implementation steps

1. Read 0324 (`tickets/done/`) and its code: `exit_name` in
   `crates/content/src/battle.rs`, `BattleDef`, `objective_text`.
2. Add the field and its check, carry it, use it in `objective_text`.
3. Strings, README, Chapter 3's battle file.
4. Tests below.

Expected size: about 50 production lines (one optional field, one check,
one text branch), like 0324's.

## Acceptance criteria

- [x] Chapter 3's objective line reads `Seize the scriptorium`; a Seize
      battle without a name keeps `Seize the <terrain>`.
- [x] A name on a battle whose objective isn't Seize, or an empty one,
      is a content error.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the content checks; `objective_text` with and without a name.

## Completion notes

- **One field for both objectives** (the ticket left it open): 0324's
  `exit_name` became `place_name`, the place an objective names, read
  on an `Escape` (where the exit leads) or a `Seize` (the Seize point)
  battle. Escape and Seize never share a battle, so one field carries
  both through the same path (`BattleDef::place_name`,
  `BattleScreen::with_place_name`, `objective::lines`) instead of a
  second copy of each; no shipped battle used `exit_name` yet (Chapter
  6's comes with 0852), so only `test_escape` changed. Its translation
  key is now `battles.<id>.place` (was `.exit`); `Lang::battle_place` and
  `Words::battle_place` translate it (`place_name` was already the world
  map's).
- The loader's check: a `place_name` only with `Escape` or `Seize`, not
  empty, one line (`place_name: only an escape or seize map names its
  place`).
- `objective_text`: a Seize with a name reads `Seize {place}`
  (`battle.objective.seize_at`, in `en/ui.ron` and the `test` pack);
  without one, the terrain line as before (`Seize the Throne`).
- Chapter 3's battle file (private repository) names `the scriptorium`.
- Tests: the content check (accepts on Seize, rejects elsewhere, empty,
  two lines), `objective_text` with and without a name, and Chapter 3's
  line from its content (skips without the game's content).
- Friction: estimate 50, measured 36 added (0.7×).
- No gameplay rules decided.
