---
id: "0470"
title: "The forecast's kill mark is a sprite from an asset file, not rectangles drawn pixel by pixel"
type: bug
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-21
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

- [x] `grep -rn "SKULL" crates/ui/src` finds nothing outside tests.
- [x] The forecast snapshot with a killing strike is unchanged (or, if
      sprites render differently in snapshots, a test checks the sprite
      item's place and size, and the PR says why the snapshot moved).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot or a scene test of the forecast with a kill.

## Completion notes

- The skull is `assets/images/kill_mark.png` (`KILL_MARK_PATH`), 8×9,
  white where the old `SKULL` pattern had an `X` (46 pixels) and
  see-through elsewhere. Our own picture, made from the code's pattern:
  no `THIRD_PARTY_ASSETS.md` row.
- `forecast::kill_mark` returns the sprite item: the whole picture,
  `Paint::Solid(color)` (a silhouette in `hp_low`, as the rectangles
  were), `Layer::Over`, 4 pixels down its cell as before
  (`KILL_MARK_TOP` replaces `SKULL_TOP`: it is the place, not the
  pattern). `SKULL` and the rectangle loop are gone.
- **The one-bit skin drew the same pattern** (it called `skull` too), so
  it gets the same fix, as the ticket's scope says. Its renderer draws a
  sprite only from a one-bit picture in the Playdate's bundle (else a grey
  box), so `playdate_images` gains a `mark` kind for the paths in
  `MARKS` (only the kill mark: the other `images/` pictures are the sprite
  map skins' and the hints', which the Playdate doesn't show). A test in
  `trpg-playdate` draws it through the real one-bit pictures and checks
  the screen inks exactly the skull's 46 pixels.
- Snapshots: the three forecast snapshots with a kill changed only in
  their item list: the 16 `over … hp_low` rectangles became one `over
  sprite 632,y 8x9 images/kill_mark.png solid=hp_low` line at the
  rectangles' top-left corner. No cell changed. Tests check the sprite's
  place, size and paint in both skins.
- No gameplay rule decided. Nothing for the Act 1 playtest beyond the
  skull looking as it did.
- Every pack's file lists in `shuyi.ron` name the new picture (the
  disclosure test fails on a file it doesn't list).
- Friction: estimate 40 production lines, measured 38: under 2×.
