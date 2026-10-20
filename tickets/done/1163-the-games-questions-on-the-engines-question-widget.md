---
id: "1163"
title: "The game's yes/no questions on the engine's question widget"
type: feature
milestone: "Engine: carve"
model: sonnet-5
effort: medium
status: done
blocked_by: ["engine#2309"]
nick_input: none
completed: 2026-10-19
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
      one-bit kit's. **Not met, on purpose:** it finds the class
      change screen's, which stays (see below).
- [x] Every snapshot is unchanged.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The existing tests and snapshots of the four screens, unchanged.

## Completion notes

Estimate (step 1): about 120 lines removed and 30 added, from the ticket.

- **The pin:** this branch first moved it to `939f023` itself and
  patched the game for engine 2308's menu change. Then 1162 and 1164
  landed on `main` with the same pin and the real 2308 work, so the
  merge kept `main`'s side and dropped the patch. The pin step was
  already done on `main`.
- **Save** (overwrite a slot): holds an `Ask::yes_no` and paints it with
  the engine's `ask::glyph`, the same box as before. `OverwriteView` and
  the screen's `paint_question` are gone.
- **Class change** (confirm): holds an `Ask::yes_no`; its view is the
  widget's view plus the cost. Its skin keeps its own `paint_question`
  because the box has a third line (the seal it uses) between the
  question and its keys, which the engine's two-line box doesn't have.
- **Dialogue** (skip): holds an `Ask::yes_no`; the question is still
  painted by the engine's text box (its `SkipView`, inside the box), fed
  from the widget's view.
- **Preparations** (leave): holds an `Ask::yes_no`; the skin still shows
  the question in the message row and its keys on the help line.
- **One-bit skins:** `screens/playdate.rs`'s `ask_view` (added by 1164)
  feeds its `ask` from the widget's view. The engine's options and Key
  bindings screens' Playdate skins now use it too, instead of their own
  copies. None of the four screens has a one-bit skin.

Friction: estimate 30, measured 154 added (178 removed), 5.1×, accept
(row in `docs/engine/friction.md`). Most added lines are rewritten
ones, such as the dialogue's skip block moving out one level. Three of
the four questions keep their look, so only one painter could go.

Nothing the player sees changed; no gameplay rules decided.
