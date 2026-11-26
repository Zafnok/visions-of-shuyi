---
id: "1378"
title: "The lamps at Harl's gate and the road bridge in Rogue Adventure"
type: content
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: blocked
blocked_by: ["engine#3247"]
blocks: ["1064"]
nick_input: none
completed:
---

# 1378 — The lamps at Harl's gate and the road bridge in Rogue Adventure

## Context

Split from 1360 (2026-11-26, Nick: "Split"), the tileset overhaul 0096
decided (`docs/design/look-and-feel.md`, *Chapter 1's map*: "Lamps burn
at the fort's gate and the bridges"). 1360 drew Harl's fort as the stone
keep and Harrowby as a round tower with today's tileset format; the
lamps couldn't be: they stand at places no terrain names (two on the
grass under the keep's south gate, two on the river beside the road
bridge), and a map file names only terrain.

**engine#3247** adds what they need: a map's `look` may place named
pictures on tiles (`look: (tiles: "rogue_adventure", props: { (19, 5):
"lamp" })`, `shuyi_assets::grid_map::MapLook::props`), a tileset's look
says what each name is drawn as (`shuyi_assets::tileset::Look::props`, a
picture at its own size), and the sprite skin's ground paints each one
standing on its tile, its bottom on the tile's bottom, after the look's
layers. It doesn't animate them: the game swaps a picture's frames by
its clock as it does for its tiles (`crates/ui/src/map_view/sprite/moving.rs`,
`trpg_content::tileset::TileFrames`, ADR-0131).

**How 0096's mockups drew the lamps:** `RA_Jungle_Animation.png`
(`assets-private/library/elv-games/tilesets/rogue-adventure/`), 16 × 32,
four frames 32 pixels apart from pixel (0, 256), 150 ms each, standing
on (19, 5) and (21, 5) beside the keep's south gate and (10, 6) and
(12, 6) beside the road bridge (Chapter 1's map), reaching a tile above.

## Nick input

None: 0096 decided it. Nothing to check after merge: `frame-png`
pictures of the gate and the bridge before and after in the PR
(cropped, ADR-0118); what's left to judge in play goes on 1064's
checklist.

## Scope

**In:**
- The pin moved past engine#3247 (its first step, `engine-change` §7,
  or the open `engine-pin` PR's merge).
- The tileset format (`trpg_content::tileset`, `look.rs`'s `LookFile`):
  a look's `props`, by name, each a picture of the image (its size in
  pixels, inside the image), checked; `assets/tilesets/README.md`; an
  ADR if the format's shape changes (`write-adr`).
- The maps: a `props` name a map's look places must be one its tileset's
  look has (the content checks, with the map's line); `assets/maps/README.md`.
- Moving props: a placed picture whose cell has `frames` moves as a tile
  does, its still the first frame on a lesser tier (ADR-0131).
- The importer (`crates/tools/src/tileset_import.rs`): a look's props in
  the mapping (`assets-src/tilesets/tiny_tales.ron`), packed two cells
  tall, their frames from a strip (the lamp's four frames, 2 tiles
  apart, 150 ms).
- The four lamps placed in Chapter 1's map, in the private repository
  (`content-change`); the art re-packed, pushed, `assets-private.rev`
  moved.

**Out (do not do):** the keep and the tower (1360, done); hills (1359,
1361); trees (1356); other chapters' lamps or props; a lamp's light on
the ground around it.

## Implementation steps

1. Move the pin past engine#3247 (`engine-change` §7; the `engine-pin`
   skill's review for the engine PRs it passes).
2. `props` in the tileset format and its checks; the map check of the
   names placed; READMEs.
3. Frames for a prop's picture in `moving.rs` (the look's `props`
   swapped as its tiles are).
4. The importer's props and the mapping's lamp; re-pack.
5. Chapter 1's map places the four lamps (private pull request), then
   `frame-png` before and after.

## Acceptance criteria

- [ ] Four lamps stand at Harl's south gate and beside the road bridge,
      reaching a tile up, burning (moving) on the PC and still on the
      Playdate's tier.
- [ ] A map placing a name its tileset's look lacks fails the content
      check with its line.
- [ ] Every other map looks as before.
- [ ] Before/after pictures in the PR.
- [ ] The tileset import's tests pass; `assets-private.rev` moved.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- `trpg_content::tileset`: a look's props read and checked; a map's
  placed names checked.
- The sprite skin: a placed lamp moves through its frames by the clock
  and stands still on a lesser tier (with the skin, ADR-0038).
- The importer: a prop packed two cells tall with its frames.

## Lesser targets

The Playdate map uses the glyph look (0061) unless 0067 changes that;
the lamps' stills are their first frame. GBA: no target.

## Completion notes

