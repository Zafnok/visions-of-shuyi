---
id: "1163"
title: "The game's yes/no questions on the engine's question widget"
type: feature
milestone: "Engine: carve"
model: sonnet-5
effort: medium
status: in-progress
blocked_by: ["engine#2309"]
nick_input: none
completed:
---

# 1163 — The game's questions on the engine's question widget

## Context

The first engine-friction sweep (1153, `docs/engine/friction.md`) found
the same pop-up question (a line of text, two or three answers, a focus,
what Cancel does) written in four of the game's screens, each with its
own logic (about 15 lines), view (about 10) and glyph skin (about 20):

- `crates/ui/src/screens/preparations.rs` (the leave prompt), its view
  and `preparations/glyph.rs`;
- `crates/ui/src/screens/save.rs` (overwrite a slot: `OverwriteView`
  and its own `paint_question`);
- `crates/ui/src/screens/class_change.rs` (the confirm stage);
- `crates/ui/src/screens/dialogue.rs` (the skip prompt).

Engine ticket 2309 (`Zafnok/shuyi-engine`) adds a question widget with
a view and a glyph skin, and moves the engine's own options and Key
bindings screens onto it.

## Nick input

None.

## Scope

**In:**
- Move the pin past 2309 (`engine-change` skill, step 7).
- The four questions above hold the engine's widget and paint it with
  its skin. Where a screen's question looks different on purpose (its
  own colors or place), keep the look with the widget's view and a
  few lines of the screen's skin, or keep that screen's skin and say
  why in the Completion notes.
- The one-bit skins keep `screens/playdate.rs`'s `ask` (the game's
  Playdate look), fed from the widget's view.

**Out (do not do):**
- The profile screen's questions (1164 replaces the screen whole).
- Any change the player sees.

## Implementation steps

1. Write the estimate: about 120 lines removed, 30 added.
2. One screen at a time; snapshots unchanged after each.

## Acceptance criteria

- [ ] `grep -rn "fn paint_question" crates/ui/src` finds at most the
      one-bit kit's.
- [ ] Every snapshot is unchanged.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The existing tests and snapshots of the four screens, unchanged.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
