---
id: "1058"
title: "Playdate: the guide's tip and lesson box fits the 50 × 15 screen"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: blocked
blocked_by: ["0066", "1037"]
nick_input: sign-off
completed:
---

# 1058 — Playdate: the guide's tip and lesson box

## Context

On Nick's Playdate (2026-10-16, the 1055 build) the guide's "Moving the
cursor" lesson box covered the top of the 50 × 15 screen and ran off its
right edge, with the map in two rows under it: the box is the PC's,
sized for 100 × 32 cells (`crates/ui/src/tips/glyph.rs`: `paint`,
`paint_lesson`, `rect`, `lesson_rect`; called from
`crates/ui/src/screens/battle/glyph.rs`). Tips and lessons are on for
the Playdate (1033), and Nick plays Chapter 1's tutorial there (1050),
so the box must fit. **0066** decides how it looks; this ticket builds
it.

The view is already split from the look (ADR-0054): `TipView` and
`LessonView` (`crates/ui/src/tips/view.rs`) say what the guide says, the
glyph paint decides the look. The Playdate's skins are chosen by
`SkinSet` (1037), and the battle's other windows are 1038's.

## Nick input

**Sign-off:** in the Simulator or on his Playdate, play Chapter 1's first
battle with tips and lessons on: the lessons and a tip read in full, the
checklist ticks, nothing runs off the screen.

## Scope

**In:**
- A Playdate paint of `TipView` and `LessonView`, chosen by `SkinSet`,
  as 0066 decided (where the box sits, the face, how text and checklist
  fit), in the one-bit look (black frame, white inside, black text, as
  1038's windows).
- If 0066's answer needs something the views lack (a page of the text,
  a short checklist line), add it to the view for both skins, the PC's
  look unchanged.
- The battle's choice of where the lesson box goes (`lesson_at_bottom`
  in `screens/battle/glyph.rs`) follows the Playdate skin's rule.

**Out (do not do):**
- What the guide says or when (`docs/design/tutorial.md`).
- The battle's other windows (1038), the dialogue box (1043), the
  one-bit portraits themselves (1042: if it isn't done, the face is a
  sprite without a picture, a grey box).
- Any change to the PC's tip box.

## Implementation steps

1. Read 0066's section of `docs/design/playdate.md`, ADR-0054, ADR-0074,
   `crates/ui/src/tips/`, and 1037's `SkinSet`.
2. The Playdate paint, with look tests (cells and pixels through
   `onebit::render`) for a tip, a lesson with its checklist, and a long
   text.
3. Behaviour tests read the view: a lesson's items tick the same on both
   skins.
4. `frame-png --playdate` pictures of a tip and a lesson for the PR.
5. Simulator: Chapter 1's first battle with lessons on, every lesson
   shown and completed.

## Acceptance criteria

- [ ] On `ConsoleSpec::PLAYDATE` with the Playdate skins, every tip and
      lesson of Chapter 1 fits inside 50 × 15 cells (a test walks them).
- [ ] The look matches 0066's chosen mockup.
- [ ] No PC snapshot changes.
- [ ] All gates in the `run-gates` skill pass, and the CI `playdate` job.

## Tests required

- Unit (look, with the skin): a tip, a lesson, a long text.
- Integration: Chapter 1's first battle on the Playdate's console with
  lessons on; each lesson box inside the screen and completable.
- Manual: Nick's sign-off.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
