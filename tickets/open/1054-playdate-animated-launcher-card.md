---
id: "1054"
title: "Playdate: an animated launcher card"
type: feature
milestone: Post–Chapter 1
model: reasoning
effort: medium
status: todo
blocked_by: ["1052"]
nick_input: decision
completed:
---

# 1054 — An animated launcher card

## Context

The Playdate's launcher shows each game as a 350 × 155 card. While a game
is selected it can play a loop of frames instead (the `.pdx`'s
`launcher/card-highlighted/1.png`, `2.png`, …, with an optional
`animation.txt` giving the order, how long each frame shows and how many
loops; *Inside Playdate*, `pdxinfo`'s `imagePath`). There is also
`card-pressed.png`, shown while A is held, and `icon-highlighted/` for the
list view's 32 × 32 icon.

0062 decided a still card first: the dithered title backdrop (Fields B at
dusk) with the logo in white (`docs/design/playdate.md`, *One-bit
pictures*, *The launcher card and icon*). While choosing it Nick said
(2026-10-15): "seems the cards view lets you use animated gif or
something so we should do that too at some point". 1052 ships the still
card, icon and launch image in the release builds.

## Nick input

**Decision first** (`ask-nick` skill, the question pop-up): what moves.
Show each option as a real loop at the device's size and ×2 (an animated
GIF from a scratch folder), and on his Playdate if he can sideload (a
scratch `.pdx` as in 0062). Cards made from the bought backdrop are
never committed (ADR-0040). Options to show, e.g.:
- the title screen's sparks drifting up behind the logo (as on the PC,
  `look-and-feel.md`, *Title screen*);
- a light sweep across the logo's letters;
- the backdrop going from dusk to night (the bundle has both pictures);
- or describe his own.

Plus whether `card-pressed.png` and an animated list icon are wanted.
No check after merge (Nick, 2026-10-21); the card in his launcher
goes on 1064's checklist.

## Scope

**In:**
- The decision, recorded in `docs/design/playdate.md` under *The
  launcher card and icon* (replacing the starting rule "the card and
  the icon don't animate").
- The frames and `animation.txt`, made by 1040's converter or a small
  `xtask` step at build time from the bought backdrop, the logo and
  whatever the answer needs, into the `.pdx`'s `launcher/` folder the
  way 1052 puts the still card there. Made from bought art means made
  at build time or kept in the private repository, never committed.
- The frame count stays small: check the `.pdx` size before and after
  and put both in the PR.

**Out (do not do):**
- Changing the still card's look, the launch image or the game's own
  title screen.
- Wrapping paper (still none).

## Implementation steps

1. Read `docs/design/playdate.md` (*One-bit pictures*), 1052's launcher
   step, *Inside Playdate*'s `imagePath` section
   (`PlaydateSDK/Inside Playdate.html`).
2. Make the option loops in a scratch folder; ask Nick; record the answer.
3. Generate the frames and `animation.txt` in the build; add them to the
   `launcher/` folder 1052 writes.
4. Simulator (its launcher shows the card), then ask Nick to sideload.

## Acceptance criteria

- [ ] `docs/design/playdate.md` records Nick's answer in his words.
- [ ] A Playdate build's `.pdx` has `launcher/card-highlighted/` frames
      (and whatever else Nick chose); the Simulator's launcher animates
      the card.
- [ ] No picture made from the bought art is committed.
- [ ] The `.pdx` size before and after is in the PR.
- [ ] Anything left to judge on the device is in the Completion notes
      (1064's checklist).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the frame generator writes the expected number of frames at
  350 × 155 and a valid `animation.txt` (from a placeholder backdrop).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
