---
id: "0774"
title: "The web build draws the study's faces and text boxes dark"
type: bug
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0774 — The web build draws the study's faces and text boxes dark

## Context

Found by 0739 (2026-11-18). Nick played Chapter 1 in the **web build**
(the playtest page). He was shown real frames of the yard and the study
with today's dimming, and said the dimming itself is fine:

> "I have no problem with the listener face as it is today. the concern is
> that the ambient lighting bleeds into the dialogue boxes. like in the
> study"
>
> "the problem is the study has its dialogue boxes being a ridiculously low
> brightness." (asked where he saw it: "web build")

His round 3 note that 0739 came from ("why are the faces impacted? they
are foreground this is weird", 0804, 2026-10-16) is very likely the same
bug.

What 0739 checked:

- The study (`assets-private/content/maps/veyne_study.map`) has
  `look: (tiles: "indoor", light: "night")`. `night` in
  `assets/data/lighting.ron` is `grey: 0.45, tint: (0.40, 0.44, 0.58)`.
  Its scene's place line is `@place veyne_study 12,7 bright 60%` (0738).
- `cargo shuyi frame-png` (the CPU raster) draws the study with the faces,
  name plates and text box exactly as bright as over the yard, in the
  picture look (`--features private-assets`), the glyph look and the
  Playdate look. The night light reaches the floor and walls only. So the
  game's view and skin are right (`crates/ui/src/screens/dialogue/glyph.rs`
  paints the place, fades it, then the faces and the box on top).
- The difference must be in the **live renderer** the web build uses: the
  engine's `crates/app/src/render.rs` (macroquad). The night light is a
  `Paint::Shaded` sprite there, recoloured on the CPU and cached in
  `recoloured` by `(image, recolour)`; a place faded to 60% sets each
  sprite's `brightness`. Something there, or in WebGL's state between
  draws, darkens what is drawn after the shaded tiles. Not yet seen on the
  desktop window: check it too, it shares the renderer.

## Nick input

None. Verified by a test that fails before the fix and by a capture of
the web build's canvas; the study's look goes on the Act 1 playtest's
checklist (1064).

## Scope

**In:**
- Reproduce in the web build (`cargo shuyi web` with the bought art, the
  study scene `ch01_tbc`), and on the desktop window.
- Find the cause and fix it where it is: in the engine
  (`engine-change` skill: an engine PR, then this PR moves the pin) if it
  is the renderer, or in the game if it is not.
- A test that fails before the fix.

**Out (do not do):**
- Changing how much the listener or narration is dimmed, or the place's
  40% (0739: Nick keeps them as they are).
- The study's own brightness (60%, 0738).

## Implementation steps

1. Build the web build with the bought art and the debug tools and open
   it in the built-in browser pane (memory: hidden pane needs the
   `MessageChannel` frame loop; read the canvas with `gl.readPixels`).
   Play to `ch01_tbc` (the debug tools' scene jump, or New Game through
   Chapter 1), and capture a frame with the messenger speaking.
2. Compare it with `cargo run -p trpg-tools --features private-assets --
   shuyi frame-png out.png --scene ch01_tbc --wait 5 --act Confirm --wait 3
   --act Confirm --wait 4` (the same moment): the portraits' skin and the
   text box's background should match.
3. Run the desktop window on the same scene (`cargo run -p trpg-app
   --features private-assets -- --scene ch01_tbc`) and note whether it
   shows the bug too.
4. Read `render.rs`'s `draw_sprite` / `draw_painted` and the order items
   are drawn in; look for a tint, brightness or blend state that a shaded
   or faded sprite leaves for the next draw, and for the recolour cache
   handing a shaded texture to an unshaded sprite of the same image.
5. Fix it, with a test in the crate that has the bug (an engine test of
   the renderer's colours, or a game test), then re-capture step 1.

## Acceptance criteria

- [ ] In the web build, the study's portraits, name plates and text box
      are as bright as over the yard (captures before and after in the PR).
- [ ] A test fails before the fix and passes after.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the renderer's (or skin's) colour for a sprite drawn after a
  shaded, faded one.
- Snapshot / integration: none new unless the cause is in the game's skin.

## Completion notes

*(Filled in by the session that completes the ticket.)*
