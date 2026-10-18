---
id: "0066"
title: "Decide: how the guide's tip and lesson box fits the Playdate's screen"
type: design-decision
milestone: Design decisions
model: opus-5.5
effort: medium
status: done
blocked_by: []
nick_input: decision
completed: 2026-10-18
---

# 0066 — Decide: the guide's box on the Playdate

## Context

On the PC the guide (Vix) speaks every tip and lesson in a box over the
map: a double-line box with the guide's name as its title, the face on
the left at twice the size of a 64 × 64 bust, the text to its right, and
for a lesson a checklist under the text (`[ ]` / `[x]` items, then
"0 of 4 done") (`docs/design/tutorial.md`; the look is
`crates/ui/src/tips/glyph.rs`). It is sized for the PC's 100 × 32 cell
console.

Nick played the 1055 build on his Playdate (2026-10-16): in Chapter 1's
first battle, the "Moving the cursor" lesson box covered the top of the
50 × 15 screen and ran off its right edge, and the map showed in two rows
under it. The Playdate's battle map will fill the whole screen with small
corner pop-ups (`docs/design/playdate.md`, *Layout*, 0061 Q2 A; built by
1037 and 1038), but no decision says where the guide's box goes there,
how big the face is, or how a lesson's text and checklist fit 50 × 15
cells (400 × 240 pixels). Tips and lessons stay on for the Playdate
(1033 turns them on, as on the PC). 1058 builds the answer.

## Nick input

**Decision** (`ask-nick` skill). Show real mockups at the device's size
(and scaled ×2): Chapter 1's first battle on the Playdate's map skin
(1037, or a hand-made frame in its look if 1037 isn't done) with the
"Moving the cursor" lesson and one ordinary tip, through `cargo xtask
frame-png --playdate` or a throwaway test. Several rounds are fine
(Nick's visual reviews). Check each game comparison against pictures of
the game before showing Nick.

Questions:

1. **Where the box sits:** e.g. a strip along the bottom of the screen
   with the map above (as GBA Fire Emblem's tutorial text is shown), a
   window on the side of the screen away from the cursor (as 1038's
   menus are), or a full page over the map that the player closes before
   acting. Plus "describe your own".
2. **The guide's face:** the one-bit portrait of 0062 / 1042 beside the
   text (as the dialogue box of 1043 shows faces), a smaller face, or no
   face and the guide's name only.
3. **A lesson's text and checklist:** all at once (the text pages with A
   like dialogue when it doesn't fit), or the text first and then only a
   small checklist window while the player does the steps (e.g. "Move
   the cursor: 1 of 4"), or something else.

## Scope

**In:**
- Run the decision with `ask-nick`; record the answers in
  `docs/design/playdate.md` (a *Guide's box* section under *Layout*,
  Nick's words and the rules) with the chosen mockup in
  `docs/design/playdate/` (no bought art in a committed picture,
  ADR-0040: a face is a grey box or the guide's name only).
- Update 1058 with the answers (its steps and acceptance criteria).

**Out (do not do):**
- Building it (1058).
- What the guide says, which lessons exist, when they show
  (`docs/design/tutorial.md` stands).
- The dialogue box (1043) and the battle's other windows (1038).

## Implementation steps

1. Read `docs/design/tutorial.md`, `docs/design/playdate.md`,
   `crates/ui/src/tips/` (the views and the glyph paint), and 1037, 1038,
   1043.
2. Make the mockups (one per option of each question), show them with
   `ask-nick`, and run as many rounds as Nick wants.
3. Write the design doc section and update 1058.

## Acceptance criteria

- [x] `docs/design/playdate.md` has the guide's box section with Nick's
      words and the rules, and the chosen mockup is committed.
- [x] 1058's text matches the answers.
- [x] `cargo xtask ticket-lint` passes.

## Tests required

- None (a decision).

## Completion notes

- Mockups: Chapter 1's first battle rendered with `cargo xtask frame-png
  --playdate --map` (the 1037 map skin; 1038 isn't built, so no corner
  pop-ups), with the boxes drawn over it at the device's size in the
  game's font, and today's box (`frame-png --playdate --tips`) for
  comparison. The face is a grey box (ADR-0040). Shown at ×2 in one
  round: Nick chose **B side window, B 1× face, A all at once** (the
  recommended set).
- Recorded in `docs/design/playdate.md`, *The guide's box (0066)*,
  with the pictures `docs/design/playdate/0066-*.png`; the design
  index row updated; 1058's scope, steps and criteria rewritten to the
  answers (and set to `todo`: 0066 and 1037 are done).
- Counted every Chapter 1 tip and lesson at 22 letters a line: all fit
  the window at once, given that long checklist items wrap and the blank
  row before the checklist goes when there's no room.
- **Claude's starting rules** (Nick can veto): the window is 24 cells
  wide and the screen's full height, on the right when the cursor is in
  the left half; double black frame, the guide's name in its top; the
  progress and the close hint in its bottom frame; text wrapped by words
  at 22 letters; every lesson fits at once (if one doesn't, its text is
  shortened, never paged, since A is one of its steps); a tip too long
  pages with A ("A more"), none in Chapter 1 does; with no face the
  title takes its place; the window goes over the corner pop-ups on its
  side.
- Follow-up: **1062** the guide's words that are untrue on the Playdate
  ("Blue is where it can go", "red tile", the "Played on the keys"
  lesson about a mouse and keys); needs one question to Nick. Added to
  1050's `blocked_by` and the roadmap.
