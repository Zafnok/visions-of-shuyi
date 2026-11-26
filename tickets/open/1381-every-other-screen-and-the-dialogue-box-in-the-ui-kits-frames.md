---
id: "1381"
title: "Every other screen and the dialogue box in the UI kit's frames"
type: feature
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["1380"]
nick_input: none
completed:
---

# 1381 — Every other screen and the dialogue box in the UI kit's frames

## Context

1380 builds the picture frames (engine nine-slice frame, `cargo xtask
ui-import`, `assets/ui/frames.ron`, the frame name → `PanelLook`
helper) and puts the battle's windows on them. The look is in
`docs/design/look-and-feel.md`, *Windows (the picture look)*, decided
by 1379. This ticket puts every other window of the PC game on the same
frames. The game will then have one look outside the map.

## Nick input

None (1379 decided the look). Pictures in the PR from `frame-png` with
`private-assets`, kept local; anything left to judge by playing goes on
the Act 1 playtest's checklist (1064).

## Scope

**In:** each screen's boxes on 1380's helper, with the frame 1379 names
for it:
- **Dialogue:** the text box and the speaker's name plate, the two
  portrait frames and their name lines (`screens/dialogue/glyph*`), the
  log (`dialogue/log/glyph.rs`), the guide's tip and lesson box.
- **Menus:** title menu, options (the engine's `screens/options.rs`
  skin: if it can't take a frame name, that is an engine-ergonomics
  item for the `engine-friction` skill), key bindings, files/save/save
  import, language list, mode select, voices question, profile, credits.
- **Between battles:** preparations (its lists and shop), camp, town,
  results, class change, familiarity, lead select, chapter card, game
  over, world map's windows.
- Frames 1379 named that 1380 didn't need, added to
  `assets-src/ui/` and `frames.ron` (re-run `ui-import`, move the pin).

**Out (do not do):**
- Battle windows (1380); the cursor, gauges, icons (1382); the font.
- The Playdate's skins; the title's logo and backdrop (ADR-0059);
  the world map's place icons (1082).
- Any change to a window's contents, size or place.

## Implementation steps

1. Estimate the lines (ADR-0092): about 600 (each screen's look moved
   onto the helper: a few lines per box, about 30 boxes, plus tests).
   Measure with `cargo xtask code-lines`. If past about 2,000 changed
   lines, split by the groups above (`write-ticket`) before coding.
2. Screen by screen, the `glyph.rs` skin's `panel` calls take the
   helper's `PanelLook`. Screens without a box keep none.
3. `frame-png` each screen in both looks and check by eye: text inside
   frames on its cells, nothing cut off, the name plate fits the
   longest name in each language pack (`test` pack included).

## Lesser targets

Playdate: unchanged. `reduced`/`minimal` tiers: as 1380 decided.

## Acceptance criteria

- [ ] Every screen in *In* draws the picture frame in the picture look
      (a Harness test per screen reads its sprite items).
- [ ] Glyph-look and no-art snapshots unchanged.
- [ ] The longest speaker name of every language pack fits its plate
      (a test).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot / integration: one Harness test per screen for the frame's
  sprite items; existing snapshots unchanged in the glyph look.
- Unit: the name plate's width for the longest name per pack.

## Completion notes

*(Filled in by the session that completes the ticket.)*
