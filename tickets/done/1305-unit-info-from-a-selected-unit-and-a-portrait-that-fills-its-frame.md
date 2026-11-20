---
id: "1305"
title: "Unit info opens from a selected unit, and the portrait fills its frame"
type: bug
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-19
---

# 1305 — Unit info opens from a selected unit, and the portrait fills its frame

## Context

Nick, 2026-11-20, playing the Chapter 1 web demo as a writer applicant
would (0766's `before-sending.md`):

- "when selecting a friendly unit E should still open unit info"
- "info screen portrait frame should be smaller OR the portrait itself
  bigger to fill the frame"

E is the Info action on the right-handed layout (`assets/data/keymap.ron`).
Today Info opens unit info from the map cursor, but not once a friendly
unit is selected (its move range showing). The unit info screen shows the
unit's portrait since 1295 (PR #756); its frame is larger than the
picture, leaving empty space.

This comes before the writer trial's first packets go out (0766).

## Nick input

None: Nick allowed either fix for the portrait ("frame smaller OR the
portrait bigger"). The agent picks the one that looks right at both
scales and in both looks (glyphs and pictures), shows the before and
after in the PR, and says which in the Completion notes.

## Scope

**In:**
- **Info with a unit selected:** pressing Info while a friendly unit is
  selected (move range up, and in its action menu if Info isn't used there
  for something else) opens that unit's info; backing out returns to the
  same selected state (the unit still selected, range still up).
- **The portrait and its frame:** the frame fits the portrait (or the
  portrait is drawn at the next size up to fill it), with no empty band, on
  every unit's info including unnamed enemies (1303's soldier face) and the
  lead (male and female). Keep the fix in the info screen's skin
  (`crates/ui/src/screens/battle/glyph/info.rs` and the picture skin), per
  ADR-0038 and ADR-0054.

**Out:**
- Other screens' portraits; the side panel.

## Implementation steps

1. Expected size: about 30-60 production lines.
2. Find where Info is handled on the battle screen (`Action::Info`) and why
   a selected unit ignores it; route it to the selected unit's info, and
   back to the selection on Cancel.
3. Measure the portrait's drawn size against its frame in the skin; pick
   the fix; check both looks at 1x and 2x with `cargo shuyi frame-png`
   (crop to the panel; no real map in a committed picture, ADR-0118).
4. Tests: a Harness test that Info with a unit selected opens its info and
   Cancel returns to the selection; a snapshot test of the info screen's
   portrait area.
5. Run the `run-gates` skill.

## Acceptance criteria

- [x] Info with a friendly unit selected opens its info; Cancel returns to
      the selection (test).
- [x] The portrait fills its frame with no empty band, in both looks
      (snapshot and frame-png pictures in the PR).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Harness integration test (input and state), snapshot of the info skin.

## Completion notes

- **Info with a unit selected:** `Mode::Info` now remembers where it was
  opened from (`back`). Info while a unit is selected (range up) or in its
  action menu opens that unit's info; Cancel (or Info again) goes back to
  exactly that selection or menu, the path and cursor as they were. Paging
  through units with Next/Previous keeps the way back. From the map cursor
  it works as before. The menu sound treats it as one level deeper.
- **Portrait:** the frame shrank (Nick allowed either) from 24 × 12 to
  18 × 10 cells. Its inside, 16 × 8 cells, is 128 × 128 console pixels,
  which every portrait fills exactly at a whole scale: the bought faces and
  the soldier face (64 × 64) at 2×, the public placeholders (32 × 32) at 4×.
  The bigger-portrait option was rejected: the next whole scale (3×, a
  26 × 14 frame) would push the left column (Mov, ranks, affinities,
  supports) two rows down where the supports already stop at the box. The
  left column now starts two rows higher instead. The lead's female face is
  the same 64 × 64 size; frame-png has no option for it, the size test
  covers it.
- The faces' own transparent margin (part of the bought art) still shows as
  a little space above the hair; that is inside the picture, not the frame.
- The help bar while a unit is selected still reads `move here · cancel`
  without naming Info (not asked for; no new UI added).
- Starting rules: none.
