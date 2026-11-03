---
id: "1044"
title: "Playdate: the title screen and its intro cinematic in one bit"
type: feature
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["1040", "1041"]
nick_input: none
completed: 2026-11-03
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
  through the system menu, so *the agent's starting
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

- [x] The cinematic plays to the title menu in the Simulator; the menu
      works.
- [ ] Device frame time in the PR.
- [x] No PC snapshot changes.
- [x] Starting rules and anything left to judge by playing are in the
      Completion notes (they go on 1064's checklist).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit (look, with the skin): one background shot, the menu.
- Integration (harness, Playdate skin set): the cinematic reaches the
  menu; reads the view.

## Completion notes

**What was done.**

- **The logo and the Game Over words are one-bit letterings**
  (`crates/content/src/playdate_images.rs`, `LETTERING`, `lettering`):
  made at build time like the other one-bit pictures, white letters, the
  shadow (the PNG's dark pixels) black, and a black rim a block wide all
  round, 3 pixels a block. The rim needs a block more on every side, so
  the picture is 162 × 99 for the 52 × 31 logo while its source size
  stays the PNG's: it is drawn whole. The title backdrops were already
  converted by 1040 (lightened, Bayer 4 × 4, at the screen's width); the
  bundle (1036) holds the logo's new picture at the PNG's path, nothing
  to list.
- **The title's Playdate paint** (`crates/ui/src/screens/title/playdate.rs`,
  chosen by the title's `SKINS`): the backdrop over all but the bottom
  row, the logo at the top, the menu (or "Press any button") in a window
  whose bottom is on the row over the help line. A sprite draws over the
  cells' backgrounds, so the backdrop and the logo are clipped around the
  window (`around`). The menu has no Quit on the Playdate already
  (`can_quit`, 1200).
- **The cinematic in the Playdate's look:** its logo shot through
  `title::paint_logo` (the skin set's), its fights through the stage's
  new `SKINS` (1041's combat picture), and its map shots at
  `console_zoom`: the file's zoom scaled to the console's width, so 3×
  and 4× become 2× and the 2× whole field 1× (the whole map fits).
- **Options shows Intro on the Playdate** now that it has the cinematic
  (`PC_ONLY` lost it, as its comment asked; `title-screen.md` updated).
  With it the Playdate's rows are one more than the window: Restore
  defaults scrolls into view (1045's scrolling).
- **"Press any button"** where there is no keyboard (`title.press_any_button`).
- **Tests:** look tests with the skin (`title/playdate/tests.rs`: the
  sizes the converter makes, one background shot through the one-bit
  renderer (the logo's white block and black rim, the white bottom row),
  the menu's window clear of the backdrop, the prompt, the half-way cuts,
  the cinematic's shots in the Playdate look); the lettering's own test;
  `console_zoom`'s. Integration (`tests/it/playdate.rs`): on the
  Playdate's skins with no music, the prompt, the menu, the march at 2×,
  a fight on the combat picture, then the logo with the menu, whose New
  Game opens the game mode screen; frames drawn one-bit as the front end
  draws them.
- **Pictures:** `docs/design/playdate/1044-*.png` (frame-png, the public
  placeholders) and two Simulator screenshots with the bought art.

**Simulator** (`cargo xtask playdate --release --private-assets --both`,
driven by window messages to the Simulator's window): A at the prompt,
the menu over the bought backdrop with the song; left alone, the
cinematic took over and played in time with the song (the march, the
brigand's attack on the combat picture, the whole field), reached the
logo at 117 s with the menu back, and A there opened the game mode
screen.

**Device frame time:** see the PR (Nick sideloads the build).

**Deviations.** The talk shots and the faces shot are not given a
Playdate look here: they draw the dialogue screen's PC look, which is off
the 50 × 15 screen (blank talk shots, two frame edges for the faces).
The Playdate's dialogue look is 1043's, so 1043 now paints them too (its
scope says so) and waits for this ticket.

**Engine friction:** estimate 230 production lines (one paint of an
existing view plus a converter step, judged by 1041's Playdate fight
paint, 317 lines), measured 238 (1.0×). No signals.

**Follow-up tickets:** none new; 1043's scope gained the cinematic's
talk and faces shots.

***the agent's starting rules*** (in `docs/design/playdate.md`, *The
title cinematic*; for 1064's checklist, Nick can veto):
- The logo 3 pixels a block (the card's 4 leaves no room for the menu),
  at the top; the backdrop's middle rows over all but the bottom row.
- The menu in a window above the help line, on the backdrop; a notice
  takes the help line's place. No Quit.
- "Press any button" in a window in the menu's place (no keyboard, no
  pictures of a keyboard and pad).
- No sparks: dots are lost in the backdrop's dither.
- No fades: the menu and the cinematic each cut in halfway through the
  PC's fade.
- The map shots at half the file's zoom, rounded up (3× and 4× → 2×,
  2× → 1×), so a shot shows about as much of the map as the PC's.

**Left to judge by playing (1064):** whether the logo at 3 pixels a
block reads on the device; whether the menu window over the dithered
backdrop is comfortable; whether the cuts in place of fades feel abrupt;
whether the 2× map shots are too big or too small on the small screen;
whether the cinematic keeps time with the song on the device.
