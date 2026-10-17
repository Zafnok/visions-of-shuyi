---
id: "0067"
title: "Decide: the bought tiles and unit sprites on the Playdate's battle map, in one-bit"
type: design-decision
milestone: Design decisions
model: opus-5.5
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0067 — Decide: bought tiles and unit sprites on the Playdate map

## Context

On the PC the battle map is drawn with the bought Tiny Tales art: 16 × 16
terrain tiles and 16 × 20 unit sprites (`docs/design/look-and-feel.md`,
*Battle map: bought tiles and unit sprites*; ADR-0049, ADR-0052). For the
Playdate, 0061 decided the **glyph look** instead (`docs/design/playdate.md`,
*Size*: "The glyph look (ADR-0049) is the Playdate's look"), and 1037
builds it: letters on black, grey or outlined tiles.

Nick asked on 2026-10-17 to explore the bought tiles and sprites on the
Playdate too ("I want to explore using the 2D battlers rather than glyphs
and 2D tiles too on the playdate port"). It reopens 0061's choice for the
map, so it is his decision, shown with real mockups.

What is known before the mockups:

- **Size fits.** The bought tiles are 16 × 16 pixels, the same as a glyph
  tile (two 8 × 16 cells), so the map still shows 24–25 × 15 tiles on the
  400 × 240 screen; units stand 4 pixels taller, as on the PC.
- **The look is the risk.** 0062's one-bit rule is "enlarge first, then
  dither at the screen's pixels" (`playdate.md`, *One-bit pictures*). At 1×
  a map tile has no room to enlarge: each art pixel is one screen pixel,
  so a Bayer dither of small colourful pixel art can turn grass, water and
  faces into noise. Playdate games with sprites mostly use art drawn for
  one bit.
- **Telling sides apart** (0061 Q3 E: yours black, foes grey, acted
  outlined) was built around letters on tiles; with sprites it needs its
  own answer. The PC tells them apart with a 1-pixel outline in the side's
  colour and an acted unit grey and darker (ADR-0049 §3).
- **The combat picture is not this ticket's:** 0062 Q2 decided its one-bit
  fighters and 1041 builds it.
- **Bought art stays private** (ADR-0040): mockups made from it are shown
  to Nick from a scratch folder and on his Playdate, never committed.
  Converting the art to one-bit is a modification its licence allows
  (ADR-0032, as 0062 found).

## Nick input

**Decision** (`ask-nick` skill). Show real mockups of Chapter 1's first
battle (Harrowby Ford) at the device's size and scaled ×2, and in a later
round as a slideshow he sideloads on his Playdate, as 0062 did (one-bit
pictures look different on the real reflective screen). Several rounds
are fine (Nick's visual reviews).

Questions:

1. **The map's look on the Playdate** — e.g.
   - A. **Glyphs** as decided in 0061 (1037's look; the baseline, shown
     beside the others);
   - B. **Bought tiles and sprites, converted automatically**, in a few
     styles (0062's A Bayer, F/G flat levels with the art's own dark
     lines, a plain threshold for terrain);
   - C. **Bought unit sprites on glyph terrain** (the PC already has this
     mixed skin, `sprite_units`, ADR-0049 §2);
   - D. **Double size** (32 × 32 tiles, room to dither as 0062 likes, but
     only about 12 × 7 tiles on screen; Fire Emblem on the GBA showed
     15 × 10);
   - plus "describe your own".
2. **Only if B, C or D wins: telling sides apart and acted units** — e.g.
   yours with a black outline and foes with a dotted one, foes on a grey
   (checker) tile as in 0061, foes mirrored or with a mark over the head,
   acted units drawn as a 50% checker (the one-bit "grey and darker").
3. **Only if B, C or D wins: the range marks** — do 0061's light dots,
   diagonal lines and dashed danger edge still read over pictures, or do
   they change (e.g. the PC's 75 % tint over bought tiles becomes a checker)?
4. **Only if B, C or D wins: whether the glyph look stays as an Options
   choice** on the Playdate, as on the PC (`look-and-feel.md`: the glyph
   look as an option).

## Scope

**In:**
- Run the decision with `ask-nick`, with one mockup per option, each
  showing: browsing, a selected unit's ranges, the danger zone, an acted
  unit, a foe beside one of yours.
- Record the answers in `docs/design/playdate.md` (a *Map pictures*
  section after *Telling things apart*; Nick's words and the rules, as
  0061 and 0062 did). Commit only mockups without bought art (e.g. option
  A's, or a test tileset's); the bought-art ones stay in the scratch
  folder.
- **If B, C or D wins:** write the build ticket(s) with `write-ticket`: a
  Playdate sprite map skin painting the same `MapScene` (ADR-0038), the
  tiles and sprites converted ahead of time by 1040's converter (a new
  kind in its method table), blocked by 1037 and 1040, with the carve's
  map ticket (1115) as "either order works" as 1037 says. Builds without
  the bought art keep 1037's glyph skin, as the PC falls back.
  Update 1037, 1038, 1040 and 1050 where the answer changes them.
- **If A wins:** record it in the design doc; nothing else changes.

**Out (do not do):**
- Building any skin or converter (1037, 1040 and the build tickets this
  ticket writes).
- The combat picture (0062, 1041) and portraits (1042).
- The frame rate (1059).
- Drawing new art or touching up bought sprites by hand (Claude never
  draws character art; Nick vetoed commissions).

## Implementation steps

1. Read `docs/design/playdate.md` (all of it), the *Battle map* sections of
   `docs/design/look-and-feel.md`, ADR-0038, ADR-0040, ADR-0049, ADR-0052,
   ADR-0074, and tickets 1037 and 1040.
2. `cargo xtask private-assets` to get the bought tileset and sprites.
3. Make the mockups with a scratch script (Python + Pillow, as 0062 did):
   render Harrowby Ford's map with the PC's sprite skin at 1× (e.g.
   `cargo xtask frame-png` with the `private-assets` feature, without the
   side panel's marks), convert each picture style to one-bit at 400 × 240,
   then draw the Playdate marks (cursor, ranges, danger edge, HP lines) on
   top. Option A comes from `cargo xtask frame-png --playdate` (or 0061's
   `q3r2-e-grey.png` if 1037 isn't done).
4. Round 1 at ×2 on screen; round 2 as a Playdate slideshow (0062's
   scratch slideshow `.pdx`) for Nick to sideload.
5. Write the design doc section; write or update the tickets in *Scope*.
6. `cargo xtask ticket-lint`.

## Acceptance criteria

- [ ] `docs/design/playdate.md` has the *Map pictures* section with Nick's
      words and the rules.
- [ ] No committed picture contains bought art.
- [ ] If B, C or D won: the build ticket(s) exist with their `blocked_by`,
      and 1037, 1038, 1040 and 1050 match the answer.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

- None (a decision).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
