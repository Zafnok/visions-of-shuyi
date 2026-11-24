---
id: "1323"
title: "Decide: hit particles in the fight and small touches on the map"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 1323 — Decide: hit particles in the fight and small touches on the map

## Context

Nick (2026-11-23), on Stardew Valley: "I see they have particle effect
for hits [...] we need to take some inspiration from here for giving the
game more life even if its not a direct 1:1 copy". The full quote is in
1322.

One of three palette decisions the chapter passes (1324–1331) pick from:
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

- [ ] `look-and-feel.md` has a *Particles and small touches* part
      answering questions 1–7, Nick's words quoted.
- [ ] The art picked for each effect is named with its license.
- [ ] 1333 and 1334 match the answer (or are closed as not needed, with
      the reason, if Nick picks none for them).
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

None (a decision).

## Lesser targets

Question 7 decides the Playdate's; GBA: no target (1314).

## Completion notes
