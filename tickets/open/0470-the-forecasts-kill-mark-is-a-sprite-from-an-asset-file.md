---
id: "0470"
title: "The forecast's kill mark is a sprite from an asset file, not rectangles drawn pixel by pixel"
type: bug
milestone: M3 Battle UI
model: implementation
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0470 — The forecast's kill mark is a sprite from an asset file

## Context

Found by the first engine-friction sweep (1153). `CLAUDE.md` rule 8 and
ADR-0038: "A picture is a sprite item from an asset file, never
per-pixel rectangles or cells." The forecast's skull, which marks a
strike that kills, breaks it: `crates/ui/src/screens/battle/glyph/forecast.rs`
draws `SKULL`, an 8 × 9 pattern of `X`s in the code, as one `Overlay`
rectangle per run of pixels in a row (`fn skull`). Swapping the look
for a drawn sprite later would mean editing code, not a file.

The effect marks (`EFFECT_MARKS_PATH`) already show the right way: a
small picture in an asset file, drawn as a sprite item.

## Nick input

None. The skull must look exactly as it does now.

## Scope

**In:**
- The skull as a PNG in `assets/` beside the effect marks (or a frame
  of the same sheet, if that sheet is ours and has room), drawn
  through the same sprite path the effect marks use, in `color`, at
  the same place in its cell.
- `SKULL`, `SKULL_TOP` and `fn skull`'s rectangle loop go.
- If the PNG is new, it is ours (drawn from the pattern in the code):
  no `THIRD_PARTY_ASSETS.md` row; say so in the PR.

**Out (do not do):**
- A new look for the mark (that is Nick's).
- The one-bit skin, unless it draws the same pattern by hand too; then
  the same fix there.

## Implementation steps

1. Read how `EFFECT_MARKS_PATH` is loaded and drawn.
2. Make the PNG from `SKULL` (a pixel per `X`, transparent elsewhere).
3. Draw it the same way; delete the pattern and the loop.

## Acceptance criteria

- [ ] `grep -rn "SKULL" crates/ui/src` finds nothing outside tests.
- [ ] The forecast snapshot with a killing strike is unchanged (or, if
      sprites render differently in snapshots, a test checks the sprite
      item's place and size, and the PR says why the snapshot moved).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot or a scene test of the forecast with a kill.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
