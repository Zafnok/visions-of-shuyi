---
id: "1058"
title: "Playdate: the guide's tip and lesson box fits the 50 × 15 screen"
type: feature
milestone: Post–Chapter 1
model: reasoning
effort: high
status: todo
blocked_by: ["0066", "1037"]
nick_input: none
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
the Playdate (1033), and Nick plays the tutorial there in the Act 1 playtest (1064),
so the box must fit. **0066** decided how it looks
(`docs/design/playdate.md`, *The guide's box*): a window down the side
of the screen away from the cursor, 24 cells wide and the screen's full
height, the guide's face at 1× (64 × 64) in its top corner with the
title beside it, the text under the face wrapped at 22 letters, and a
lesson's checklist under its text, all at once as on the PC. The chosen
mockups are `docs/design/playdate/0066-lesson.png` and `0066-tip.png`.

The view is already split from the look (ADR-0054): `TipView` and
`LessonView` (`crates/ui/src/tips/view.rs`) say what the guide says, the
glyph paint decides the look. The Playdate's skins are chosen by
`SkinSet` (1037), and the battle's other windows are 1038's.

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> In the Simulator or on his Playdate, play Chapter 1's first
> battle with tips and lessons on: the lessons and a tip read in full, the
> checklist ticks, nothing runs off the screen.

## Scope

**In:**
- A Playdate paint of `TipView` and `LessonView`, chosen by `SkinSet`,
  as 0066 decided: the side window (24 × 15 cells, double black frame,
  white inside, black letters, the guide's name in the top of the
  frame), the 1× face (8 × 4 cells) in its top corner and the title
  beside it, the text under it.
- What the PC writes under the text goes in the bottom of the frame: a
  lesson's progress (`0 of 4 done`), a tip's close hint.
- Word wrapping at the window's width (22 letters), the text's own line
  breaks ignored; a checklist item too long for a line goes on under its
  own words, past the `[ ] `; the blank row between the text and the
  checklist only when there is room.
- A tip too long for the window turns pages with A ("A more" in the
  frame until the last page): the view gains the page, for both skins,
  the PC's look unchanged. A lesson never pages (A is one of its steps).
- No face (no guide portrait, or no picture yet): the title takes the
  face's place and the text moves up.
- The battle's choice of where the box goes (`lesson_at_bottom` in
  `screens/battle/glyph.rs`) follows the Playdate skin's rule: the
  window on the right when the cursor is in the left half of the
  screen, on the left otherwise, for tips and lessons alike. It goes
  over 1038's corner pop-ups on its side.

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
   `onebit::render`) for a tip, a lesson with its checklist, a long
   checklist item that wraps, a tip that pages, no face, and the window
   on each side of the cursor.
3. Behaviour tests read the view: a lesson's items tick the same on both
   skins.
4. `frame-png --playdate` pictures of a tip and a lesson for the PR.
5. Simulator: Chapter 1's first battle with lessons on, every lesson
   shown and completed.

## Acceptance criteria

- [ ] On `ConsoleSpec::PLAYDATE` with the Playdate skins, every tip and
      lesson of Chapter 1 fits inside 50 × 15 cells (a test walks them).
- [ ] The look matches 0066's chosen mockups (`0066-lesson.png`,
      `0066-tip.png`): the side window away from the cursor, the 1×
      face, a lesson's text and checklist at once.
- [ ] Every lesson of Chapter 1 shows whole without paging; a tip too
      long for the window pages with A.
- [ ] No PC snapshot changes.
- [ ] All gates in the `run-gates` skill pass, and the CI `playdate` job.

## Tests required

- Unit (look, with the skin): a tip, a lesson, a long text.
- Integration: Chapter 1's first battle on the Playdate's console with
  lessons on; each lesson box inside the screen and completable.
- Manual: none (left-overs go to 1064).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
