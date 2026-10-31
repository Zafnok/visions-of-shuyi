---
id: "0062"
title: "Decide: how the Playdate's one-bit pictures look (portraits, the combat picture, the launcher card and icon)"
type: design-decision
milestone: Design decisions
model: reasoning
effort: medium
status: done
blocked_by: []
nick_input: decision
completed: 2026-10-15
---

# 0062 — Decide: the Playdate's one-bit pictures

## Context

Nick decided the Playdate version keeps **one-bit (dithered) portraits**
of the bought faces in dialogue, **a simple one-bit combat picture**, and
the title cinematic (`docs/design/playdate.md`, *What the first version
keeps*, ticket 0061). He did not see what "one-bit" or "simple" look like
for those pictures: 0061's mockups were the map only. The build tickets
need the answer: 1041 (combat picture), 1042 (portraits), 1044 (title
cinematic's backgrounds) and 1052 (the Playdate launcher's card and icon,
which every Playdate game shows in the system's game list).

The Playdate screen is 400 × 240, black or white only, reflective (no
backlight), 2.7 inches; dither patterns stand in for greys. The bought
art (ADR-0032, ADR-0040) may be modified (its licence: "Assets can be
modified freely to fit the needs of your game"), and the result stays in
the private repository like the originals.

## Nick input

**Decision** (`ask-nick` skill). Show real pictures at the device's
pixel size (and scaled ×2 so he can see the pixels), made from Chapter
1's real faces and battlers (`assets-private/game/`), never committed
(ADR-0040: no picture that hands out the art; show them to Nick from a
scratch folder). Several rounds are fine (Nick's visual reviews).

Questions:

1. **Portraits:** the same face in several one-bit styles, e.g. an
   ordered (Bayer-pattern) dither, the regular cross-hatch look of
   *Return of the Obra Dinn*; error diffusion (Floyd–Steinberg, or
   Atkinson as in the early Macintosh's HyperCard pictures); a threshold
   with hand-cleaned lines (flat black and white, closer to Playdate
   games like *Mars After Midnight*); and at which size (today's
   portrait size vs. smaller to leave room for the text box). Check each
   game comparison against pictures of it before showing Nick. Plus
   "describe your own".
2. **The combat picture ("simple"):** e.g. the battlers dithered as the
   portraits, black silhouettes on white, outlined figures, or just the
   two units' map tiles large with the strike numbers (closest to the
   fight box). Shown at the device's size over a plain or dithered
   background.
3. **The launcher card (350 × 155) and icon (32 × 32):** e.g. the title
   logo on white, a dithered title background with the logo, a
   character's dithered face; the icon as the logo's first letter, a
   unit tile, a face.

## Scope

**In:** the questions above; the answers recorded in
`docs/design/playdate.md` (a new *One-bit pictures* section), with
glyph-only or non-bought example pictures committed under
`docs/design/playdate/` where they help (pictures made from the bought
art are not committed).

**Out (do not do):** building the converter or any game code (1040–1042,
1044, 1052).

## Implementation steps

1. Read `docs/design/playdate.md`, `docs/design/look-and-feel.md`
   (*Portraits*, *Combat screen*), ADR-0040's rules on pictures of the
   bought art.
2. Make the option pictures with a scratch script (Python + Pillow is
   fine; nothing ships).
3. Ask with `ask-nick` (the question pop-up); record the answers.
4. Update 1041, 1042, 1044 and 1052 if an answer changes their scope.

## Acceptance criteria

- [x] `docs/design/playdate.md` has the *One-bit pictures* section with
      Nick's words and rules.
- [x] No picture made from the bought art is committed.
- [x] `cargo xtask ticket-lint` passes.

## Tests required

- None (decision).

## Completion notes

- Three rounds with Nick, the second also on his Playdate: a scratch
  slideshow `.pdx` (16 slides, built with `pdc`, sideloaded from the
  Simulator). The pictures and the slideshow stayed in the session's
  scratch folder; none made from bought art are committed.
- **Decided** (`docs/design/playdate.md`, *One-bit pictures*):
  - Portraits: style **A** (Bayer 4 × 4) by default, plus a
    Playdate-only Options setting with A, A+ lines, F+ lines and G.
  - Portraits at **2×** in today's dialogue layout.
  - Combat: fighters in style A on white ground, the dithered backdrop
    only above a horizon line.
  - Launcher card: the dithered title backdrop with a white logo.
    Icon: the block S of `SHUYI`.
- **Found while making them:** a fighter drawn at 1.25× (the Brigand)
  had its face broken by pixel-doubling before the dither. It is now
  enlarged smoothly first (a rule in the doc; 1040 does it).
- **the agent's starting rules** (Nick can veto):
  - The listener is faded by a 50% checker.
  - The portrait style setting doesn't change the combat fighters (A
    only).
  - The setting is called *Portrait style*.
  - The launcher card's backdrop is Fields B at dusk, fixed, not
    animated (1054 animates it later).
  - The launch image is the card's look at 400 × 240.
  - No wrapping paper.
  - A build without the bought art shows the logo over a checker.
  - The cinematic's backdrops and logo are converted like the card's.
- **Ticket created:** 1054, an animated launcher card (Nick asked for
  one later).
- **Tickets updated:** 1040 (the methods, one portrait set per style),
  1041 (the horizon stage), 1042 (2×, the style setting is in its
  scope), 1044 (the backdrops and logo), 1052 (card, icon, launch
  image).
- Committed: `docs/design/playdate/0062-icon.png` (×4 too) and
  `0062-card-logo.png`, made from the game's own logo only.
