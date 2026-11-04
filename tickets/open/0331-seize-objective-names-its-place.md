---
id: "0331"
title: "Seize objective names its place"
type: feature
milestone: Post–Chapter 1
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["0849"]
nick_input: none
completed:
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

- [ ] Chapter 3's objective line reads `Seize the scriptorium`; a Seize
      battle without a name keeps `Seize the <terrain>`.
- [ ] A name on a battle whose objective isn't Seize, or an empty one,
      is a content error.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the content checks; `objective_text` with and without a name.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
