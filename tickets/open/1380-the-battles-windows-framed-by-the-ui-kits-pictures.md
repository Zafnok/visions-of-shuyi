---
id: "1380"
title: "The battle's windows framed by the UI kit's pictures in the picture look"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["1379", "1238"]
nick_input: none
completed:
---

# 1380 — The battle's windows framed by the UI kit's pictures

## Context

1379 picks the art for the picture look's windows (Nick, 2026-11-26:
"rework UI elements using assets from itch … from scratch like frames
and such … keep the glyph look as an option but I want it to be more
professional"). Its answers are in `docs/design/look-and-feel.md`,
*Windows (the picture look)*. This ticket builds the first part: the
picture frames themselves, and the battle's windows drawn with them.
1381 does every other screen, 1382 the cursor, gauges and icons.

Today every box is a `GlyphBuffer::panel` with the engine's
`PanelLook` (`style: BoxStyle`, border, title, text and fill
colours). The engine's `crates/ui/src/frame/draw.rs` holds it, and the
lines are drawn from the font. 1238 (blocked_by) moves the last
hand-built boxes onto `panel`, so a frame drawn from a picture lands in
one place. The buffer already carries sprite items
(`GlyphBuffer::add_sprite`, `Sprite`) drawn at pixel positions over the
cells (ADR-0038: a picture is a sprite item from an asset file, never
per-pixel rectangles).

**The engine has no picture frame** (no nine-slice: searched
2026-11-26 at the pin `96eb097c`). The game must never wait on the
engine (ADR-0092), so follow the `engine-change` skill: an engine
ticket and PR first (Nick's OK before the engine PR, memory), then this
game PR moving the pin. If the engine work is still waiting, build the
frame game-side behind a marked workaround as `engine-change` says.

## Nick input

None: 1379 decides the look. Pictures of each window in the PR come
from `frame-png` with `private-assets`, kept local; only a cropped
screenshot of the running game may go in the PR (ADR-0040, ADR-0118:
no real map). What is left to judge by playing goes on the Act 1
playtest's checklist (1064).

## Scope

**In:**
- **Engine** (its own ticket and PR, `engine-change`): a frame drawn
  from a picture's nine slices: four corners, four edges (tiled or
  stretched, the picture says which), the inside (tiled, stretched or
  left clear), and a title or name plate (left, middle and right pieces
  around the title). Each piece is a sprite item, laid on the cell grid
  so text inside stays on its cells. Chosen through `PanelLook` (a
  frame by picture in place of `BoxStyle`); a buffer without the
  picture falls back to the box lines. Tests in the engine (pixels of
  each piece, a 1-cell and a wide panel, the fallback).
- **Game:** the chosen pack's pieces cut into the private repository's
  `game/ui/` by an importer (`cargo xtask ui-import`, its source list in
  `assets-src/ui/`, like `tileset-import`), and a public data file
  (`assets/ui/frames.ron`, its format in `assets/ui/README.md`) naming
  each frame (`panel`, `menu`, `plate`, `forecast` or as 1379 names
  them) with its picture's path and slice sizes. Missing pictures mean
  glyph frames: a public build looks as today.
- The **switch** as 1379 decided (starting rule: the look option
  switches the map and the windows; the entry renamed in every pack).
- The battle's windows on the frames: the side panel, the unit menu
  and its sub-menus (Arts, Skill, Item, Equip), the forecast, the map
  menu, unit info (`glyph/info.rs`), the progress, guides, rewind and
  shop boxes, and the fight box (`glyph/fight_box.rs`).
- Credits: the pack's line in `THIRD_PARTY_ASSETS.md` names what the
  game uses and where (1379 adds the purchase), and the credits screen's
  data if the licence asks for credit.

**Out (do not do):**
- Screens outside the battle and the dialogue box (1381).
- The menu cursor, gauges and icons (1382); the font (its own ticket if
  1379 picked one).
- The Playdate (its one-bit skins stay; 1184, 1294).
- Any change to a window's contents, size or place: only its frame and
  fill change.

## Implementation steps

1. Read *Windows (the picture look)* in `docs/design/look-and-feel.md`.
   Estimate the lines (ADR-0092): about 350 engine lines (the frame and
   its tests) and about 900 game lines (importer, data file and its
   loading, the switch, the battle skins' looks), measured with `cargo
   xtask code-lines`.
2. Engine ticket and PR (`engine-change`), then move the pin in this
   branch (`engine-change` §7).
3. `assets-src/ui/` source list and `cargo xtask ui-import`; run it,
   push the pictures to the private repository, `cargo shuyi
   private-assets --pin` (the art only, so no `content-change`).
4. `assets/ui/frames.ron` + its loader in `trpg-content` (validated:
   every name the skins ask for exists, slice sizes fit the picture).
5. One helper in `crates/ui` that turns a frame name into a
   `PanelLook` (picture when the look option says pictures and the
   picture loaded, else today's look), and every battle box listed
   above on it.
6. `frame-png` pictures of each window, both looks, checked by eye.

## Lesser targets

Playdate: unchanged (one-bit skins). The `reduced` and `minimal` tiers
(ADR-0131): the picture frame if it costs no more than the box lines
(it is a handful of sprites); otherwise the glyph frame, said in the
completion notes.

## Acceptance criteria

- [ ] The engine's picture frame merged, the pin moved past it.
- [ ] `cargo xtask ui-import` writes the pieces; the private repository
      holds them and `assets-private.rev` names that commit.
- [ ] `assets/ui/frames.ron` loads and validates; a content test fails
      on a missing frame name.
- [ ] With the bought art and the picture look, every battle window in
      *In* draws the picture frame (a test reads the buffer's sprite
      items per window); with the glyph look or without the art, the
      snapshots are unchanged.
- [ ] The look option switches the windows as 1379 decided.
- [ ] `THIRD_PARTY_ASSETS.md` names the pieces used.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the frame name → `PanelLook` helper (picture, glyph look,
  picture missing).
- Snapshot / integration: the battle screens' existing snapshots stay
  as they are in the glyph look; a Harness test per window checks
  the picture frame's sprite items in the picture look (by name and
  rect, not pixels).
- Content: `frames.ron` validation.

## Completion notes

*(Filled in by the session that completes the ticket.)*
