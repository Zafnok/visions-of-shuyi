---
id: "1323"
title: "Decide: hit particles in the fight and small touches on the map"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: []
blocks: ["1064"]
nick_input: decision
completed: 2026-11-27
---

# 1323 — Decide: hit particles in the fight and small touches on the map

## Context

Nick (2026-11-23), on Stardew Valley: "I see they have particle effect
for hits [...] we need to take some inspiration from here for giving the
game more life even if its not a direct 1:1 copy". The full quote is in
1322.

One of three palette decisions the chapter passes pick from (1362–1369
for the maps; 1324–1331 pick only the sounds):
**0096** (moving tiles, drifting petals and leaves), **1322** (ambient
sound) and this one.

What a hit looks like today (`docs/design/look-and-feel.md`, *Combat
screen*, ticket 0413; `crates/ui/src/screens/battle/stage/motion.rs`):
the target flashes white and shakes, a crit flashes the whole stage and
shakes twice as hard, a miss steps aside, a fallen fighter fades and
sinks, and the result (`HIT -8`, `CRITICAL! -27`) pops up over the
target. Nothing flies off. On the map, units step in place, the cursor
pulses, and nothing reacts to the ground (no dust, no splash), and no
creatures live there.

The engine has no particles yet; 1316 brings them (engine parked ticket
2103: emitters as data, a pure function of seed and time). Whatever is
picked is a sprite item from an asset file, painted by the skins, never
per-pixel rectangles (ADR-0038).

The 00xx block is full (0099), so this decision takes a 13xx number.

## Nick input

**Decision** (`ask-nick`, several rounds expected). Show moving loops
(GIFs or a page of `frame-png` frames) in **both looks**, glyphs and
pictures (memory: mockups cover both looks), on Chapter 1's fight and
map, cropped per ADR-0118. Check each comparison against the game before
it's shown and drop one that doesn't hold.

1. **Hits in the fight** (the combat stage), per kind of strike:
   - A. **None** (today: the flash and shake only).
   - B. **Small bursts**: a few sparks for a sword or axe on armour, dust
     for a punch or a miss's footwork, a puff of the element for a spell
     (embers, frost shards), like the chips that fly in Stardew Valley
     when a rock or a slime is hit (check), or Wargroove's hit effects
     (check).
   - C. B plus a **crit's own burst**: bigger, with the stage flash.
   - D. Describe your own. No blood unless Nick asks (today there is
     none).
2. **A heal and a level up:** motes rising (like the title's embers),
   or none.
3. **The map, under units' feet:** a puff of dust as a unit steps, a
   splash in a ford or shallow water, leaves shaken as a unit passes
   through forest. Which, or none.
4. **Hits seen on the map** (when combat animations are off, Options):
   a small burst on the target's tile, or nothing beyond today's.
5. **Creatures on the map** (Stardew Valley's birds that fly off when you
   come near, butterflies, rabbits, crows, seagulls): none; a few that
   only pass by; or a few that scatter when a unit steps next to them.
   Never on a tile the rules use, never hiding a unit or the cursor.
6. **How busy:** two densities to compare for each picked effect.
7. **The Playdate** (`Reduced` tier, 1314): the default is none of it,
   or the fight's bursts only if cheap (1-bit, a few sprites).

0093 (a switch to stop looping motion) covers creatures and drifting
effects too; either order works (if 0093 is answered first, include its
switch in the loops).

## Scope

**In:** the questions and the loops (throwaway, never committed: a mock
skin through an env var, as in the memory on staging fights); the
answers in `docs/design/look-and-feel.md` (a new *Particles and small
touches* part, next to 0096's *Moving tiles and effects*) and a row in
`docs/design/README.md`; 1333 and 1334 adjusted.

**Out (do not do):** building any of it (1333, 1334); drifting effects
(0096); each chapter's choices (1324–1331); sounds (1322); anything that
changes the rules.

## Implementation steps

1. Read `look-and-feel.md` (*Combat screen*, *Units on the map*,
   *Title screen*'s embers), `stage/motion.rs`, the title's embers in
   `screens/title/glyph.rs`, and 1316.
2. Find art: the Tiny Tales bundle's effects
   (`assets-private/library/tiny-tales/INDEX.md`, `cargo shuyi
   private-assets --library`) and free packs under CC0 / CC-BY only
   (ADR-0013); never drawn per pixel. Note each one's license.
3. Mock each option as moving loops in both looks; ask; record; adjust
   1333 and 1334.

## Acceptance criteria

- [x] `look-and-feel.md` has a *Particles and small touches* part
      answering questions 1–7, Nick's words quoted.
- [x] The art picked for each effect is named with its license.
- [x] 1333 and 1334 match the answer (or are closed as not needed, with
      the reason, if Nick picks none for them).
- [x] `cargo xtask ticket-lint` passes.

## Tests required

None (a decision).

## Lesser targets

Question 7 decides the Playdate's; GBA: no target (1314).

## Completion notes

Asked over five rounds of moving mockups (2026-11-23 to 2026-11-27): real
Chapter 1 fights, walks and a fight with combat animations off, captured
with `frame-png` from staged throwaway battles (`zz_1323_*`, never
committed) and composited with the candidate effects by a throwaway
script. The mockups show bought art, so none is committed (ADR-0040).
Recorded in `docs/design/look-and-feel.md`, *Particles and small
touches*, and a row in `docs/design/README.md`.

- **Round 1** (Ninja Adventure, CC0, both looks): "look for other asset
  packs with these particle effects these look bad", "probably I want a
  high quality pack", and "dont need to render glyphs look option for
  now". Nick then bought ByteWright's *Everything So Far* (2026-11-24,
  for 0096 and this ticket).
- **Rounds 2–5** (ByteWright): fights C with "many bursts but crit gets
  the star"; the arrow's splinters and the fire's ember pop replaced (a
  flying arrow and a star; a small explosion, two notches bigger); a heal's
  crosses and a level up's rays on the unit's tile; light dust, leaves
  and splashes underfoot; with animations off, one drawn frame of a small
  star per landed blow; critters startle and run; none on the Playdate.
- **Art:** ByteWright's *Pixel VFX & SFX Mega Pack* and *Ambient
  Critters* (bought; any number of games, commercial or not; edits
  allowed; credit optional, and we credit ByteWright; no redistribution,
  no AI training), already in `THIRD_PARTY_ASSETS.md`, now with what 1323
  picked.
- **Downstream:** 1333 (the fight) and 1334 (the map, now with the heal,
  the level up and the map hit) rewritten to the answer. The effects are
  drawn flipbooks, not emitters, so both no longer wait on 1316's engine
  particles (1316's *Out* says so); 1334 reuses 1333's effect table and
  waits on it. 1333 is ready to work.

**Claude's starting rules** (not shown to Nick; judge them in the Act 1
playtest, 1064):

- The glyph look shows the same moments with the font's marks (`*` `+`
  `•` `°`), snapped to the grid and darkening, like the title's embers.
- An ice spell plays the fire's small explosion in the pack's ice
  colours; a spell with no element in its arcane colours.
- An Absorb strike plays its burst on the target and the heal's crosses
  on the striker; a strike that does no damage still plays its burst.
- Thicket counts as forest underfoot; bridge, floor, fort, hills, peaks
  and buildings have no touch.
- With combat animations off, a miss shows no star.
