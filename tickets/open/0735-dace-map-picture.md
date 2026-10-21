---
id: "0735"
title: Give Dace a map picture of his own
type: content
milestone: M6 Story & dialogue
model: sonnet-5
effort: medium
status: todo
blocked_by: ["0734"]
nick_input: decision
completed:
---

# 0735 — Give Dace a map picture of his own

## Context

Ticket 0734 added conversation openings (`assets/openings/`, ADR-0076):
characters placed on a place and walking there, drawn by the map skins
with their map picture. `ch01_tbc` opens on Dace (character id `rival`)
pacing the study at Veyne Hall (`assets/openings/ch01_pacing.ron`).

Dace is a **speaker** (`assets/data/characters.ron`, `speakers`): he has
no class, and the private tileset (`assets-private/game/tilesets/
tiny_tales.ron`, `units` → `characters`) names no picture for him. The
sprite skin then draws the tileset's `fallback` picture
(`units/adventurer_m1.png`), the same one generic units get, so in the
picture look he looks like nobody in particular. In the glyph look he is
his label `Da`, which is fine.

The bought Tiny Tales bundle (ADR-0040, `assets-private/library/
tiny-tales/characters/`) has many walking sheets in the same style as the
units already used (`fighter_male.png`, `church_knight.png`…).

## Nick input

**Decision** (`ask-nick`, before building): pick Dace's map picture from three to five candidates out
of the bought bundle, shown as frames of the opening (`ch01_tbc`'s first
second, picture look), each with the sheet's name. "Describe your own"
is an option (another sheet from the bundle). Never a commission, never
drawn by Claude (feedback: no commissions; bought art only).

## Scope

**In:**
- Choosing candidates: walking sheets from `assets-private/library/
  tiny-tales/characters/` with the four facings the sprite skin reads
  (down, left, right, up rows; `walk: true`), for a young noble who took
  a hall that wasn't his (`docs/story/characters/` for Dace's look; his
  portrait is `rival`).
- Copying the picked sheet into `assets-private/game/units/`, naming it
  in `tiny_tales.ron`'s `characters` as `"rival": (image: …, frame: (1,
  0), walk: true)`, adding the file to its credit's `private` list
  (`assets/data/credits.ron`, ADR-0051), pushing the private repository
  and pinning it (`cargo xtask private-assets --pin`, commit
  `assets-private.rev`).
- A frame of `ch01_tbc`'s opening with the picture, looked at (not
  committed unless as a screenshot of the game).

**Out (do not do):**
- Pictures for other speakers (the messenger, the Master of Vows): only
  when an opening places them.
- Making Dace a unit (a class, stats): he never fights in Chapter 1.
- Changing the opening's walk or the study.

## Implementation steps

1. `cargo xtask private-assets --library`; list the walking sheets under
   `assets-private/library/tiny-tales/characters/` that have four facing
   rows like `units/fighter_male.png`.
2. Render each candidate as Dace: temporarily point `"rival"` at it in
   `assets-private/game/tilesets/tiny_tales.ron`, render with
   `cargo run -p xtask --features private-assets -- frame-png out.png`
   at the opening (the scene is reached from the debug tools or a
   temporary local hook; don't commit the hook). Show Nick with the
   `ask-nick` skill.
3. Make his pick permanent (step "In" above), credit, push, pin.

## Acceptance criteria

- [ ] `ch01_tbc`'s opening shows Dace with the picture Nick picked, in the
      picture look; the glyph look is unchanged.
- [ ] `cargo xtask private-assets --pin` accepts the new file (credited).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None new: the sprite skin's character lookup is tested; the all-assets
  test loads the tileset. Gates never read the private art.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
