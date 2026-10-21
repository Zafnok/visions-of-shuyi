---
id: "1044"
title: "Playdate: the title screen and its intro cinematic in one bit"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: ["1040", "1041"]
nick_input: none
completed:
---

# 1044 — The Playdate title screen and cinematic

## Context

Nick kept **the title cinematic** on the Playdate
(`docs/design/playdate.md`, Q5). The title screen (`crates/ui/src/screens/title*`,
a view and a glyph skin) plays the intro cinematic
(`assets/cinematics/title.ron`, `crates/ui/src/cinematic*`;
`docs/design/title-screen.md`) in time with the title song: background
shots from `assets/title/` (fields and meadows at several times of day,
the logo), map shots of the Chapter 1 battle (painted by the map skin:
1037's on the Playdate) and combat shots (1041's combat picture), then
the title menu.

0062 decided how the one-bit backgrounds and logo look; 1040 converts
them.

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> Nick watches the cinematic and the title menu in the
> Simulator or on his Playdate and comments.

## Scope

**In:**
- One-bit versions of `assets/title/`'s pictures with 1040's tool and
  0062's method (`playdate.md`, *One-bit pictures*, *The title
  cinematic*: backdrops lightened and Bayer-dithered, the logo with white
  letters and a black shadow and rim); the Playdate bundle (1036) lists them.
- A Playdate paint of the title view (chosen by `SkinSet`): the
  cinematic's shots framed for 400 × 240 (the cinema framing, Shuyi
  Engine's since 1108 and re-exported as `crates/ui/src/cinema.rs`,
  reads the console size since 1030), the logo, the title menu (New
  Game, Continue/Load, Options, …, as on the PC; the PC's Quit entry
  only if the Playdate allows quitting from a game, which it does
  through the system menu, so *Claude's starting
  rule (Nick can veto)*: no Quit entry on the Playdate).
- Light effects the PC cinematic uses (`assets/data/lighting.ron`,
  times of day) become the different one-bit pictures (day, dusk,
  night) rather than tints: one-bit has no tints.
- Without music (1047 not done yet) the cinematic runs on its own clock
  as it does on the PC with the sound off; with music, in time with it.
- Measure on the device: frame time during map and combat shots.

**Out (do not do):**
- Changing the cinematic's shots, timing or the PC title.
- The chapter title card and screen transitions (0812, 0813): if they
  have landed, give them a Playdate paint here too (small); if not,
  they are not this ticket's job.

**Carve (1113):** either order works. If 1113 moved the cinematic
timeline to the engine, the shot kinds and their paints stay the game's.

## Implementation steps

1. Read `docs/design/title-screen.md`, `title.ron`, the title screen's
   view and skin, `cinema.rs`, 0062's answers.
2. Convert the pictures; the Playdate paint; tests.
3. Simulator; ask Nick to sideload for the frame time.

## Measured on Nick's Playdate

Frame time during the map shots and combat shots, and whether the
cinematic keeps time with the song (once 1047 is in).

## Acceptance criteria

- [ ] The cinematic plays to the title menu in the Simulator; the menu
      works.
- [ ] Device frame time in the PR.
- [ ] No PC snapshot changes.
- [ ] Starting rules and anything left to judge by playing are in the
      Completion notes (they go on 1064's checklist).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (look, with the skin): one background shot, the menu.
- Integration (harness, Playdate skin set): the cinematic reaches the
  menu; reads the view.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
