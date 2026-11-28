---
id: "1333"
title: "Hits throw sparks, dust and embers in the fight"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["1323", "1314"]
blocks: ["1064"]
nick_input: answer-first
completed:
---

# 1333 — Hits throw sparks, dust and embers in the fight

## Context

1323 decided (2026-11-27) which burst each strike throws on the combat
stage: `docs/design/look-and-feel.md`, *Particles and small touches*,
*Hits in the fight* (its table names each effect's file in the pack).
In short, over today's flash, shake and result text: a dense shower of
sparks for a sword, spear or axe; an arrow that flies from the bow to the
target, then a small star; a dust puff off the target and dust at its
feet for a gauntlet; a small explosion for fire; dust at the dodger's
feet on a miss; and a crit's own big four-point star under the kind's
burst. No blood. The Playdate gets none of it.

The art is ByteWright's *Pixel VFX & SFX Mega Pack* (bought 2026-11-24,
`assets-private/library/bytewright/effects/pixel-vfx-sfx-mega-pack/`).
Each effect is a sheet drawn by the artist: one row per take or
direction, its frame size, frame times and anchor in the `.json` beside
it, played once from its start. So these are flipbooks keyed by time, not
particle emitters: this ticket does not need 1316's engine particles
(2026-11-27, 1323). A sheet's frame at a time is the `shuyi-animate`
skill's sprites cut from a sheet.

1314 (done, ADR-0131) gives the tier: `ctx.platform.graphics`; the
Playdate is `Reduced`. Each burst carries a `trpg_content::motion::Motion`
(`lowest: Full`, nothing below), checked by the content test
`every_moving_look_names_its_stills`.

The stage today: `crates/ui/src/screens/battle/stage.rs`,
`stage/motion.rs` (the flash, shake, lunge, fade; *tunable*),
`stage/view.rs` (the view), `stage/glyph.rs` (the skin), and the
Playdate's `screens/battle/playdate/stage.rs`. The strike's kind and its
result come from the fight's events, which are already read for sounds
(`event_sounds.rs`), so the burst and the sound start at the same
moment.

## Nick input

None: 1323 answered it. Nothing to check after merge: verified by tests
and `frame-png` strips; what's left to judge in play (1323's starting
rules: ice, no-element and Absorb strikes) goes on 1064's checklist.

## Scope

**In:**
- The burst sheets 1323 picked, copied from the library into
  `assets-private/game/` (bought art, ADR-0040 and ADR-0051: the path in
  ByteWright's credit's `private` list in `assets/data/credits.ron`, with
  ByteWright credited; `cargo shuyi private-assets --pin`), and an effect
  table in data: each strike kind (weapon kind, element, crit, miss) to
  its sheet, row, size and anchor, so a new kind is a row. This table
  and its loader are what 1334 reuses for the map's effects.
- The stage's view lists the bursts alive at its time, each placed at
  the target's near edge where the blow lands (data the tests read); the
  picture skin paints them as sprite items at the stage's 2×; the glyph
  skin paints 1323's starting rule (the font's `*` `+` `•` `°`, snapped to
  the grid and darkening, like the title's embers).
- The bow's arrow in flight from the archer's bow to the target, landing
  at the impact moment.
- Each burst starts at the strike's impact moment in `motion.rs`; the
  crit's star plays under the kind's burst, with today's stage flash.
- Nothing below `Full` (the Playdate keeps its flash and shake).

**Out (do not do):** anything on the map (1334: the heal, the level up,
the touches under feet, the map hit with animations off, the creatures);
drifting effects (1316); new sounds; blood.

## Implementation steps

1. Write the expected size (about 250 production lines: an effect table
   and loader, a view field, the arrow's flight, two skins' painting) and
   measure with `cargo xtask code-lines` (ADR-0092).
2. Copy the sheets into `assets-private/game/` with their credit
   (`content-change` skill for the private side), then the effect table.
3. Map each strike result to its burst (the table); add `bursts` to the
   stage's view at a given time; paint in `stage/glyph.rs` and the
   picture skin.
4. `frame-png` strips of a sword hit, an arrow, a gauntlet crit and a
   miss, and a fire spell (crop to the stage, ADR-0118).

## Acceptance criteria

- [ ] A test: a crit's view at its impact time holds the crit's star and
      its kind's burst.
- [ ] A test: a bow strike's view holds the arrow before the impact and
      the star after it.
- [ ] A test: the same fight at the same time gives the same bursts.
- [ ] A test: the `Reduced` tier's view has no bursts.
- [ ] `frame-png` strips in the PR.
- [ ] `cargo shuyi bench` within budget on desktop and web.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the strike-to-burst table; the view at fixed times.
- Look: a snapshot of each skin at one burst's peak.

## Lesser targets

The Playdate (`Reduced`): none (1323, Nick: "None"); GBA: no target.

## Completion notes
