---
id: "0491"
title: "Chapter 4's doorways look like doors"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0491 — Chapter 4's doorways look like doors

## Context

Nick's playtest of the Act 1 build (2026-11-08):

> "We should make the door tiles on ch4 more apparent -- do we need [to] buy
> more tilesets? I can do that."

Chapter 4's battle map (`assets/maps/ch04.map`, comments ~:2–14) has two
doors that matter, and neither is a door tile: each is a plain floor tile
(`_`, `floor`: Mov 1 for everyone, Def 0, Avoid 0, `terrain.ron` ~:33) in
a gap of the wall (`#`):

- the hall's **front door** at (13, 5), which a Hound holds
  (`assets/battles/ch04.ron` ~:40);
- the **laundry door** at (21, 10), in the long east wall: Hollis's way
  in (the battle note "Hollis's way in: the laundry door in the long wall,
  east, and the passage up", `ch04.ron` ~:126; the script, `ch04.dlg`
  ~:335: "It never locked, it only looked locked").

The map's other openings, (20, 5) where the servants' passage meets the
kitchen and the stair at (16, 3), are not doors in the story.

The terrains that look like doors today are both **impassable** and kept
for scene maps: `door` (`▐▌`, picture `dungeons-1/Set_A_Fortress1` tile 27,
`assets-src/tilesets/tiny_tales.ron` ~:128) and `house_door` (Small
Village tile 216 with a lintel on the tile above, ~:161–200). The gate at
(13, 12) is `gate` (Def +3, Avoid +20, the Fortress archway, tile 26).

**What the bought bundle has.** The Tiny Tales library
(`assets-private/library/tiny-tales/`, `cargo shuyi private-assets
--library`) has door sheets with an opening animation, closed to open:
`tilesets/dungeons-1/animated-objects/!$Dungeon_Doors1.png` and
`!$Dungeons_DoubleDoors1.png`, the same in `dungeons-2/`, `overworld/`
(`!$Overworld_Doors1_A.png`…) and `tower/`. A door frame is 16 × 32 (a
tile and the wall tile above it, like `house_door`'s lintel), three frames
across. These sheets have no Tiled `.tsx` file, and `cargo xtask
tileset-import` reads tiles only from `.tsx` sheets
(`tiny_tales.ron` header). So a new tile may need the importer to cut a
cell from a bare PNG sheet. Nothing needs to be bought if one of them
fits.

## Nick input

None, if the bundle has a fitting door (it seems to: above). Verified by
`cargo shuyi frame-png` pictures of the two doors in the glyph look and
the picture look in the PR; the pick goes on the Act 1 playtest's
checklist (1064). **If nothing in the bundle fits**, stop: add doors to
0471's research (Nick buys art there; he offered), and set this ticket
`blocked_by: ["0471"]`.

## Scope

**In:**
- A **passable door terrain**, e.g. `doorway` ("Doorway"), with the floor's
  rules exactly (Mov 1 for every movement type, Def 0, Avoid 0): a look,
  not a new rule (*the agent's starting rule*; `docs/design/terrain.md`
  gets a line saying so). Not `decoration`.
- Its **glyph look**: two glyphs that read as a way through a wall and
  differ from `door`'s `▐▌`, the wall and the floor (`ascii-art` skill;
  glyphs in the font, `assets/fonts/README.md`; colours from the palette).
- Its **picture look**: a door from the bundle in `tiny_tales.ron`, open
  (*the agent's starting rule*: a door that reads as a way in; the story's
  "only looked locked" is the characters' view). With the lintel on the
  wall tile above if the door is 16 × 32, as `house_door` does. If the
  importer must read a PNG sheet without a `.tsx`, add that to
  `cargo xtask tileset-import` with its test. Re-import, push the private
  repository, `cargo shuyi private-assets --pin`, and commit
  `assets-private.rev` in this PR (AGENTS.md, *Bought art*).
- The Playdate's one-bit look shows it (it uses the glyph skin's glyphs:
  check they read in one bit).
- `ch04.map`: `(13, 5)` and `(21, 10)` become the doorway (a legend
  letter), and the comments name it.

**Out (do not do):**
- Doors in other maps, or turning scene maps' `door` passable.
- Buying art (only through 0471, above).
- Doors that open or lock as a rule.
- The throne and the gate's art (0471; the seize mark is 0488).

## Implementation steps

Expected size: about 40–80 production lines (a terrain entry, a legend
letter, tileset lines, maybe an importer branch and its test), plus the
private repository's tile.

1. Read ADR-0038, ADR-0052 (terrain pictures in layers) and the
   `ascii-art` skill; look at the door sheets
   (`private-assets --library`); never commit the art or a crop of it.
2. Add the terrain and its glyphs; the map letters.
3. The picture tile (and the importer, if needed); re-import; pin.
4. Tests and pictures.

## Acceptance criteria

- [ ] `doorway`'s rules equal `floor`'s (content test).
- [ ] Chapter 4's map has the doorway at (13, 5) and (21, 10)
      (content test on the loaded map).
- [ ] Chapter 4 is still won by its recorded commands
      (`crates/ui/tests/it/ch04_winnable.rs`), and the bots' Chapter 4
      numbers don't move (no rule changed); goldens updated only for the
      terrain's id.
- [ ] The doorway has its own tile in `tiny_tales.ron`'s `terrain` table
      and its door layer; the picture-look map of Chapter 4 draws it
      (a look test reading the sprite items, or the sprite skin's
      snapshot).
- [ ] `frame-png` pictures of both doors in the glyph look, the picture
      look (`--features private-assets`) and the Playdate skin are in the
      PR.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit / content: the terrain's rules and the map's tiles.
- Integration: Chapter 4's winnable test.
- Snapshot: the Chapter 4 map snapshots that show the doors.

## Completion notes

*(Filled in by the session that completes the ticket.)*
