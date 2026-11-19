---
id: "0089"
title: "Decide: how a rider on foot looks (the glyph mark, the combat picture)"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0089 — Decide: how a rider on foot looks

## Context

Ticket 0083 (2026-11-17) decided that riders can get off their horse and
back on (`terrain.md`, *Getting off and back on a horse*; built by 0334).
Nick deferred two parts of the look: "defer glyphs. dont need to block
mechanic on it." So 0334 ships without them, and until this ticket is
done a rider on foot looks like a mounted one in the glyph look and in
combat.

1. **The glyph look's mark.** A unit is two letters in its side's colour
   (`look-and-feel.md`, *Units on the map (the glyph look)*), and
   nothing tells a rider from a foot soldier. Nick saw these mockups in
   0083 and rejected them all ("all look like shit but if you c an make a
   real horseshoe it might work and dont collide"):
   - a dot in the corner of a rider on foot;
   - a bar over a mounted unit's letters;
   - four "horse legs" under the letters;
   - a horseshoe in the top corner (it hit the "i" of `Ri`);
   - capitals when mounted (`RI` / `Ri`);
   - a horseshoe behind the letters (thick, it read as a blob; thin, it
     still crossed the letters);
   - a 5×4-pixel ∩ horseshoe in the strip under the letters, beside a
     shortened HP bar (the last round; he answered "defer").
   What's left to try: a real horseshoe that doesn't touch the letters.
   The letters fill the 16×16 tile but its bottom 4 rows (the HP bar is
   rows 14–15). Possible ideas: a horseshoe-shaped HP bar end, a
   horseshoe glyph in the font for a third cell on wide tiles, or no
   mark plus a line on the side panel.
2. **The combat picture on foot.** Riders fight as Pixel Flag's Paladin
   on a horse (`look-and-feel.md`, *Who uses what*). A rider on foot
   needs a side-on knight on foot. Nick picked in 0083: "Horses on map,
   knight on foot", meaning a knight from art already owned, recoloured
   in Tamsin's red (her tabard and saddle cloth are "a clear red",
   0446), shown as game screenshots (never the raw files, ADR-0040). If
   nothing fits, he may buy one (a purchase ticket 0334's follow-up
   waits on).

## Nick input

**Decision** (`ask-nick`), with `frame-png` mockups of the glyph look
(cropped to a few tiles, and close up) and the candidates' combat
pictures at their game size facing a Hound. Expect several rounds
(memory: visual reviews with Nick).

## Scope

**In:** the questions, the mockups (throwaway, never committed: a `zz_`
battle file and an env-var mock in the skin, then reverted), Nick's
words in `look-and-feel.md` (*Units on the map*, *Who uses what*) and
`terrain.md`'s *The look*; a build ticket (04xx) for whatever he picks,
and a purchase ticket if he wants one.

**Out (do not do):** building it; the map sprites (0499); the rules
(0334).

## Implementation steps

1. Read `terrain.md` (*Getting off and back on a horse*), `look-and-feel.md`
   (glyph units, *Who uses what*, the bought-art rules), 0083's
   completion notes, and the glyph skin (`crates/ui/src/map_view/glyph/units.rs`).
2. Make new glyph mockups that don't touch the letters; ask.
3. Find side-on foot knights in the owned art (the private library's
   indexes, e.g. `pixel-flag/INDEX.md`, Shironejiya, Tiny Tales),
   render each as Tamsin's combat picture; ask.
4. Record; write the build ticket(s).

## Acceptance criteria

- [ ] Nick's answers and words are in `look-and-feel.md`;
      `docs/design/README.md`'s row for 0083 names this ticket's result.
- [ ] A build ticket exists for what he picked (and a purchase ticket if
      needed).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

None (a decision); `cargo xtask ticket-lint`.

## Completion notes

*(Filled in by the session that completes the ticket.)*
