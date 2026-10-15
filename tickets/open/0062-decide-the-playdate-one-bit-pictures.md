---
id: "0062"
title: "Decide: how the Playdate's one-bit pictures look (portraits, the combat picture, the launcher card and icon)"
type: design-decision
milestone: Design decisions
model: opus-5.5
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
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

- [ ] `docs/design/playdate.md` has the *One-bit pictures* section with
      Nick's words and rules.
- [ ] No picture made from the bought art is committed.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

- None (decision).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
