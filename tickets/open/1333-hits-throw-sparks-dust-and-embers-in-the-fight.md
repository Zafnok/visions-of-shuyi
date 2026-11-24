---
id: "1333"
title: "Hits throw sparks, dust and embers in the fight"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: blocked
blocked_by: ["1323", "1316", "1314"]
blocks: ["1064"]
nick_input: answer-first
completed:
---

# 1333 — Hits throw sparks, dust and embers in the fight

## Context

1323 decides which bursts a hit, a crit, a miss, a spell, a heal and a
level up throw on the combat stage, and how busy they are. 1316 brings
the engine's particles (emitters as data, each particle a pure function
of seed and time) and their tier cap; 1314 gives each target a graphics
tier.

The stage today: `crates/ui/src/screens/battle/stage.rs`,
`stage/motion.rs` (the flash, shake, lunge, fade; *tunable*),
`stage/view.rs` (the view), `stage/glyph.rs` (the skin), and the
Playdate's `screens/battle/playdate/stage.rs`. The strike's kind and its
result come from the fight's events, which are already read for sounds
(`event_sounds.rs`), so the burst and the sound start at the same
moment.

## Nick input

**Answer first:** 1323. Nothing to check after merge: verified by tests
and `frame-png` strips; what's left to judge in play goes on 1064's
checklist.

## Scope

**In:**
- An effect file per burst 1323 picked (in `assets/`, the art from the
  bundle or a credited free pack, never drawn per pixel, ADR-0038).
- The stage's view lists the bursts alive at its time (data the tests
  read); both skins paint them as sprite items, the glyph look with its
  own glyphs.
- Each burst starts at the strike's impact moment in `motion.rs`, keyed
  by the strike's kind (weapon type, element, crit, miss, heal).
- The tier cap from 1314; with combat animations off, nothing (or 1323's
  map burst, which is 1334's).

**Out (do not do):** bursts on the map (1334); drifting effects (1316);
new sounds; blood unless 1323 says so.

## Implementation steps

1. Write the expected size (about 200 production lines: a burst table, a
   view field, two skins' painting) and measure with `cargo xtask
   code-lines` (ADR-0092).
2. Map each strike result to a burst id (a table, so a new weapon kind
   is a data row).
3. Add `bursts` to the stage's view at a given time; paint in
   `stage/glyph.rs` and the picture skin.
4. `frame-png` strips of a sword hit, a crit, a miss and a fire spell.

## Acceptance criteria

- [ ] A test: a crit's view at its impact time holds the crit burst.
- [ ] A test: the same fight at the same time gives the same bursts.
- [ ] A test: the `Reduced` tier's view has no bursts (or 1323's cheap
      set).
- [ ] `frame-png` strips in the PR.
- [ ] `cargo shuyi bench` within budget on desktop and web.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the strike-to-burst table; the view at fixed times.
- Look: a snapshot of each skin at one burst's peak.

## Lesser targets

As 1323 decides for the Playdate (default none, `Reduced`); GBA: no
target.

## Completion notes
