---
id: "0096"
title: "Decide: the tileset overhaul, moving tiles and drifting effects"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: []
blocks: ["1064"]
nick_input: decision
completed: 2026-11-24
---

# 0096 — Decide: the tileset overhaul, moving tiles and drifting effects

## Context

Nick, after watching a new player play Chapter 1 (2026-11-22):

> "I want a tileset overhaul in general. I think Tiny Tales is ok for
> character sprites but the tilesets are lacking or we're not making full
> use of it. I want animations like swaying trees, flowing water, and
> that effect Stardew Valley has for like petals flowing on screen, we
> should have some effects similar."

What we use today (`assets-src/tilesets/tiny_tales.ron`, packed by
`cargo xtask tileset-import`, ticket 0437, ADR-0052): parts of the Tiny
Tales world-map, dungeons-1 and overworld sets. Many terrains share one
ground tile (forest, thicket, mountain, peak, fort and burning all sit on
the same grass tile with a layer on top). What the bundle has that we
don't use (`assets-private/library/tiny-tales/INDEX.md`):

- five 16 px sets (world-map, overworld, dungeons-1, dungeons-2, tower)
  and 48 px sets in a different style (terrains-plus, trees-forest,
  **trees-cherry-blossoms**);
- **animated tiles already drawn**: 15 Tiled `.tsx` files with frame
  loops — beaches and coasts (`Set_A_Beach*`, `Set_A_*_Coast`; we use
  `Standard_Coast` for water, but still), hills, dark woods, desert,
  snow, mines;
- `animated-objects/` folders: braziers, torches, doors, chests,
  hearths, smoke.

No sway animation for trees is known to be in the bundle (check); a tree
sway made by the skin (a small time-based shear or offset, no new art)
is an option. Drifting petals or leaves would be particles: the engine
has none yet (parked engine tickets 2103 particles, 2370 weather
ambiences; 2359/2360 animated tiles).

The weaker targets matter here (Nick: "deprio playdate / gba in general
but not forget they exist later"): every moving look has a still
fallback (1314). 0093 asks whether players can stop looping motion;
moving tiles and petals fall under its answer (either order works: if
0093 is answered first, include its switch in the mockups).

## Nick input

**Decision** (`ask-nick`, several rounds expected, per the memory on
visual reviews). Show real game frames and short loops (an animated GIF
or a page made from `frame-png` frames) of Chapter 1's map:

1. **Tileset direction:** A. stay with Tiny Tales 16 px, using far more
   of it (distinct ground per terrain, the coast and beach animations,
   the overworld's details). B. the 48 px "different style" sets mixed
   in. C. look for another bought tileset pack in the same chibi,
   hand-made style (memory: art candidates Nick accepts) — only with
   Nick's OK to buy. D. your own.
2. **Moving tiles:** which of: flowing river and sea water, waves on
   coasts, swaying trees, flickering torches and braziers, smoke from
   houses. How fast (two speeds to compare).
3. **Drifting effects** (Stardew Valley's falling leaves and petals; also
   check Octopath Traveler, Triangle Strategy's weather, Fire Emblem
   Three Houses' snow): which effects (petals, leaves, snow, fireflies,
   dust), per map or per season, how dense, and on which screens (battle
   map only, or the world map and talk scenes too).
4. Whether any of it should differ for the Playdate (a still map, or a
   few moving tiles) — the default is still, per 1314.

Answers go to `docs/design/look-and-feel.md` (a new *Moving tiles and
effects* part) and shape 1315, 1316 and 1318.

**A palette for the chapter passes** (Nick, 2026-11-23, "lets take it
chapter by chapter"): this answer, with 1322 (ambient sound) and 1323
(hit particles, map touches and creatures), is the set of kinds that
1324–1331 pick from, one chapter each. So ask it as kinds (petals,
leaves, ash, mist, snow…) that a chapter can name, not only for
Chapter 1.

## Scope

**In:** the questions, pictures and loops; the answers recorded; 1315,
1316 and 1318 adjusted.

**Out (do not do):** building any of it; buying art without Nick's OK.

## Implementation steps

1. Survey the bundle for what can be shown without code (its `.tsx`
   animations played as frame strips in a mockup page made from game
   frames; never the raw sheets, ADR-0040).
2. Mock a sway and a petal drift (a throwaway mock skin through an env
   var, as in the memory on staging fights).
3. Ask Nick; record; adjust the build tickets.

## Acceptance criteria

- [x] `look-and-feel.md` records the tileset direction, the moving tiles
      (with speeds), the drifting effects (where, how dense) and the
      Playdate's version.
- [x] 1315, 1316 and 1318 match.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None (a design document).

## Completion notes

Asked over about sixteen rounds of mockups (2026-11-23 to 2026-11-24):
store previews with our units on them, then Chapter 1's own map, units
and starting places drawn from the real sheets by a throwaway script
(kept out of the repository: it draws bought art). Recorded in
`docs/design/look-and-feel.md`, *Moving tiles and effects*, *Which
tileset* and the new *Chapter 1's map*.

- **Direction:** a new tileset, not more Tiny Tales (C). Nick bought
  ELV Games' *Game Assets Mega Bundle*, Electric Lemon's bundle and
  ByteWright's *Everything So Far*; filed in the private repository's
  `library/` with the originals as private releases, and recorded in
  `THIRD_PARTY_ASSETS.md`. Chapter 1 is **Rogue Adventure**; the other
  chapters' sets are not decided.
- **Moving tiles:** drawn art only, never made by the skin (round 1's
  code sway and colour cycling were rejected). Water, swaying trees by
  footprint (with half-size trees for small woods), grass tufts, lamps;
  speeds are the art's own timing.
- **Drifting effects:** petals, leaves, snow, ash, fireflies, from
  ByteWright's *Weather Overlays*; battle map only; density per chapter
  later. Ambient critters on open plain.
- **Playdate:** still.
- **Chapter 1's look:** rounded raised hills (Lonesome Forest's cliff kit,
  recoloured sandy brown), Harl's fort as Rogue Adventure's stone keep.
- 1315, 1316 and 1318 carry the answer; 1318 must be split before coding.
