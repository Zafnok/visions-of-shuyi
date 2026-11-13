---
id: "0166"
title: Comments outside the tests no longer name the story's people or tell who dies or joins
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["0164"]
nick_input: none
completed: 2026-11-12
---

# 0166 — Comments outside the tests no longer name the story's people or tell who dies or joins

## Context

Found by 0164 (the UI integration tests' half of ADR-0118). ADR-0118 §2
says a public file may name a content id but never **describe the plot**
(who dies, who learns what, what a scene reveals); the leak check
(`cargo xtask check-leaks`) finds quoted lines and map rows only, so plot
told in comments is held by review. 0162, 0163, 0164 and 0165 reworded
the comments of the tests they moved, but on 2026-11-12 a grep for the
story's display names still found 62 places in `crates/` outside
`crates/ui/tests/it/`, some of them telling the plot, e.g.:

- `crates/content/src/dialogue/deaths.rs` (module docs: whose death the
  `@dies` directive was made for, and whose join `@joins`);
- `crates/content/src/dialogue.rs` (`Stage::Dies`, `Stage::Joins` docs);
- `crates/content/src/dialogue/presence.rs`,
  `crates/content/src/dialogue/tests/presence.rs` (who joins before which
  chapter's battle);
- `crates/core/src/campaign.rs`, `crates/core/src/campaign/tests.rs` (a
  scripted death and a recruit, by name).

The rest name a character only (`crates/xtask/src/voice/tests.rs`,
`crates/content/src/dialogue/tests/names.rs`,
`crates/content/src/character.rs`, `crates/xtask/src/battler_import.rs`,
…). The grep:

```bash
grep -rnwE "Hollis|Tamsin|Aske|Maud|Joss|Hedda|Harl|Dace|Hendry|Gil|Tull|Silas" crates --include=*.rs
```

(and the other display names in `assets/data/names.ron`). 0160 moves the
story out of the public repository, so these must go first.

## Nick input

None.

## Scope

**In:**
- Every comment, doc comment, test name and message under `crates/` that
  names one of the story's people or places by display name, or tells
  who dies, joins, leaves or betrays: reworded to ids (`retainer`,
  `defector`, `shieldbearer`, …) and to what the code does ("a scripted
  death", "a recruit who joins in a scene"), as 0165's notes did.
- A test whose expected value is a display name reads it from the
  content's names table at run time (as `title_cinematic.rs`'s `name_of`
  does, 0164) or is guarded (ADR-0118 §1 (a)) if it checks the real
  table.

**Out (do not do):**
- `crates/ui/tests/it/` (0164 did it).
- Tickets, ADRs and design documents (ADR-0118 §2: outside the rule).
- The public names table and data themselves (0160 moves them).
- Changing what any test checks.

## Implementation steps

1. Run the grep above with every display name of `assets/data/names.ron`
   (people and places) and list each hit, marked "plot" or "name only".
2. Reword each to ids; keep each comment saying what the code does.
3. Where a test asserts a display name of the real content, read it
   from `content.names` or guard the test.
4. Run the grep again: the only hits left are in data files that 0160
   moves and in the `lead`'s default names (public).

## Acceptance criteria

- [x] The grep finds no display name of the story in `crates/**/*.rs`
      outside data that 0160 moves; the completion notes list what was
      left and why.
- [x] No comment under `crates/` tells who dies, joins or leaves.
- [x] `cargo xtask check-leaks` passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None new: comments and names only; the existing tests pass.

## Completion notes

- Grepped every capitalised display name of `assets/data/names.ron`
  (people, places, factions, gods) in `crates/**/*.rs` outside
  `crates/ui/tests/it/`: 142 hits once the game's title was set aside.
  Plot: `dialogue/deaths.rs`, `dialogue.rs`, `dialogue/presence.rs`,
  `dialogue/tests/presence.rs`, `core/src/campaign.rs`,
  `core/src/campaign/tests.rs`, `bots/src/arrival/tests.rs` (who dies,
  who joins and when): reworded to what the code does. Name only: the
  rest, reworded to ids or roles (`the poacher`, `the keeper`, `ch03`).
- Tests of a mechanism that used story names as fixtures now use
  made-up ones: `content/src/dialogue/tests/names.rs` has its own
  invented names table (ADR-0118 §1 (b)), and `character.rs`,
  `ui/src/dialogue/tests.rs`, the Playdate tests, `xtask`'s voice and
  playtest tests use invented names. `check_script.rs`'s draft writes
  the lead's public default name instead of the king's, so the same
  "name written out" check fires through its lead branch.
- Tests of the real table are guarded (ADR-0118 §1 (a)) and read the
  names at run time: `names/tests.rs` (the king and the crown exist, the
  family and nickname short forms match the full names),
  `lang/data_tests.rs` (the retainer's first name is the table's), the
  README example in `dialogue/tests/names.rs`. These now check the
  shape of the names rather than their spelling.
- Left on purpose: `Veyne` (the lead's public family name in
  `trpg_core::lead`, and the lead-select tests of `Ellery Veyne`);
  `Vix` (the tutorial guide, outside the story; its tip box is in public
  snapshots); item and spell display names in the guarded tests of the
  real data (`Tor's Bow`, `Fordwarden's Spear`, `Vigil Light`), which
  0160 moves with the data; `Haltet das Tor` (German, not a name); and
  the `rook` fixture id (a placeholder unit, not the sergeant).
- No follow-up tickets.
