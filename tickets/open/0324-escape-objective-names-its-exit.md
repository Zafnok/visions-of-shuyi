---
id: "0324"
title: "An escape map names its exit on the objective line"
type: feature
milestone: M2 Core rules
model: sonnet-5
effort: medium
status: todo
blocked_by: ["0319"]
nick_input: none
completed:
---

# 0324 — An escape map names its exit on the objective line

## Context

0319 built the escape objective (`Objective::Escape { area }`,
`docs/design/turn-structure.md`, *Escape*). The design's objective line
names the exit: `Get everyone to the Brennish lines (2/6 out)`. Maps
don't carry their own text yet, so 0319 shipped the starting wording
`Get everyone to the exit (2/6 out)` (`battle.objective.escape` in
`assets/lang/en/ui.ron`, built by `objective_text` in
`crates/ui/src/screens/battle/map_menu.rs`). Chapter 6 (0852) needs the
place's name: the Ashfields' exit is the Brennish lines.

`Objective` is `Copy` and lives in `core`, which knows nothing about the
look or the words (ADR-0004, ADR-0038), so the name belongs in the battle
file and the UI, not in the objective.

## Nick input

None. The name of each exit comes from the chapter's beat sheet
(`docs/story/beats.md`); the chapter ticket writes it.

## Scope

**In:**
- An optional battle-file field naming the exit, e.g. `exit_name: "the
  Brennish lines"` (written bare), carried on `BattleDef` and handed to
  the battle screen (as `BattleDef::battle_notes` are, or beside the
  battle's look), translatable the way battle notes are (by their English
  text, `Words::battle_note` / `trpg_content::lang`).
- `objective_text`: with a name, `Get everyone to {place} ({out}/{total}
  out)` (a new key, e.g. `battle.objective.escape_to`); without, the
  current `battle.objective.escape` line. Both in `en/ui.ron` and the
  `test` pack.
- Content checks: a name only on an escape map; a non-empty one line.
- `assets/battles/README.md`: the field.
- `assets-test/battles/test_escape.ron` gets a name.

**Out (do not do):**
- Chapter 6's battle itself (0852).
- Naming any other objective (Seize keeps its terrain name).
- Changing how escaping works (0319).

## Implementation steps

1. `crates/content/src/battle.rs`: `RawBattle` gets `exit_name:
   Option<String>` (`#[serde(default, deserialize_with = "bare")]`); check
   it in `Checker::objective` (error if the objective isn't `Escape`, or the
   text is empty or more than one line); carry it on `BattleDef` (new field
   in `crates/core/src/campaign.rs`, plain data) and from there to where the
   battle screen gets its battle file's data.
2. `crates/ui/src/screens/battle/map_menu.rs`, `objective_text`: use the
   name when there is one, through the language pack.
3. Strings, README, the test battle.
4. Tests below.

Expected size: about 60 production lines (one optional field, one check,
one text branch), like `light` in the battle file.

## Acceptance criteria

- [ ] `test_escape`'s objective line reads `Get everyone to <its name>
      (0/3 out)`; a battle without a name keeps `Get everyone to the exit`.
- [ ] A name on a non-escape map, or an empty one, is a content error.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `objective_text` with and without a name, in English and in the
  `test` pack; the content errors.
- Snapshot / integration: none needed beyond the objective text.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
