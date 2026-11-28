---
id: "1384"
title: "A fire in Harrowby's yard and a candle in Veyne's study"
type: content
milestone: Post–Chapter 1
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
blocks: ["1064"]
nick_input: none
completed: 2026-11-26
---

# 1384 — A fire in Harrowby's yard and a candle in Veyne's study

## Context

1324 (2026-11-26) gave two of Chapter 1's scene places a fire's crackle
in their beds, but neither map draws a fire: the yard behind the rented
house (`harrowby_yard`, where the script puts Hollis "by the fire") and
Veyne Hall's study (`veyne_study`). Nick wants the picture to match the
sound: "probably a fire in the yard makes sense and the study should be
more like some light source like a candle or something. but dont let it
block the rest of the ambience." His full words are in the private
repository's `story/design/audio.md`, *Chapter 1's ambience*.

He prefers drawn animation from the bought packs over faked motion
(0096, `look-and-feel.md`): a flame that moves comes from a pack's own
frames.

## Nick input

None: what to add is decided (1324). Verified by `cargo shuyi frame-png`
pictures (kept out of the repository: bought art, ADR-0040); what's left
to judge goes on the Act 1 playtest's checklist (1064).

## Scope

**In:**
- `harrowby_yard`: a small open fire (a fire pit or campfire) near where
  Hollis sits, from the bought packs, animated if a pack has the frames.
- `veyne_study`: a candle or other small light on or near the desk, not a
  hearth, from the bought packs, animated if a pack has the frames.
- Both through the `content-change` skill (the maps are private content)
  and the ADR-0038 rules (sprite items from asset files, the map scene).

**Out (do not do):**
- Sound (the beds are 1335).
- Light on the map (glow, flicker of the room's light): not decided.
- Any other map; script lines.

## Implementation steps

1. Find a fire and a candle in the bought packs (`assets-private/game/`,
   `cargo shuyi private-assets --library` for the sorted packs), and
   credit any new file (ADR-0051).
2. Add them to the two maps; render each scene with `frame-png`
   (`--scene ch01_intro`, `--scene ch01_tbc`, picture look).
3. Content pull request first, then this one moves `assets-private.rev`.

## Acceptance criteria

- [x] `frame-png` of `ch01_intro` shows a fire in the yard; of `ch01_tbc`
      a candle or small light in the study.
- [x] Content tests pass (every map loads; every bought file credited).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Content: the two maps load with their new items.

## Lesser targets

The Playdate shows the still frame (1314).

## Completion notes

**Done (2026-11-26).**

- **The art:** the fire is Rogue Adventure's village fire pit
  (`RA_Village_Animations.png`, (6, 1), its 4 drawn frames 150 ms each),
  the candle Electric Lemon's candle in a dish (*Dungeons*,
  `classic_dungeons_animated_candles.png`, (0, 12), its first 4 frames of
  5, 150 ms; the fifth is a guttering flame that jumps out of the loop).
  Both credited (`credits.ron`: a new Classic Dungeons row, the ELV row's
  comment; `THIRD_PARTY_ASSETS.md`).
- **Where:** two decoration terrains, `campfire` on the yard's earth at
  (15, 8), two tiles right of Hollis's mark in the sparring opening, off
  the lane he and the lead spar in; `candle` on the desk at (14, 5), the
  far end from the report. The glyph look draws `^^` in the fire colour
  and `╤i`. Both block movement, like the rest of the furniture.
- **Frames down a column:** the fire pit's frames run down its sheet, so
  the importer's `moving` step may now be whole rows (`Strip`, the
  mapping's header); a frame below the image is an error as before.
- **The conversation's place now keeps the clock:** a place behind the
  talk was drawn at clock 0, so no moving tile moved there (1315's river
  neither). It takes the battle map's clock (`screen::loop_clock_ms`,
  0 with idle motion off, so tests' frames don't change).
- **Stacked on 1356** (PR #859): 1356's re-packed tileset was already on
  the private repository's `main`, so this branch merges #859's and
  re-packs on it; it merges after #859.
- **Content:** merged as Zafnok/visions-of-shuyi-assets#41;
  `assets-private.rev` is at its merge on `main` (`41d6283e`).
- **Checked:** `frame-png` of `ch01_intro` (picture look, after the
  captions) shows the fire pit beside Hollis; of `ch01_tbc` the candle on
  the desk; both in the glyph look too. The pictures stay local (bought
  art, ADR-0040; a real map, ADR-0118).
- **Tests:** the importer's strip down a column (and past the bottom);
  the dialogue's place keeps the clock with idle motion on, 0 with it
  off; the private places tests read the fire and the candle on their
  tiles; the two places' snapshots show the new glyphs; the public test
  tilesets regenerated for the two terrains.
- **Friction:** estimate 15 production Rust lines (and about 30 of
  data), by 1318's `moving` strip and the battle map's clock; measured
  16 (1.1×), under 2×.

### For the Act 1 playtest (1064)

**No gameplay rule decided.** Left to judge in play (1064): where the
fire sits in the yard and the candle on the desk; the candle reads small
and dim in the study's night light; light on the map (a glow) stays
undecided, as the ticket says.
