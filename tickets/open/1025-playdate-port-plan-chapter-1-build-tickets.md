---
id: "1025"
title: "Playdate port: plan the build tickets for Chapter 1"
type: research
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: blocked
blocked_by: ["1026", "0061"]
nick_input: none
completed:
---

# 1025 — Playdate port: plan the build tickets for Chapter 1

## Context

Nick, 2026-10-07: "could we make a Playdate console port? […] as far as
controls limitations we have solutions like chording A+B if we need to or
otherwise have A/B off of a unit act differently like it already does and
the rewind mechanic can use the winding mechanism on the side. this is for
hella after launch but if we just reuse the retro glyph stuff it could
totally scale down to Playdate I think".

Nick, 2026-10-15, unparking it: "I wanna rush a playdate port!!! how can
we get ch1 on playdate asap?!" He chose to run the technical test first
and not wait for the engine carve (1103–1131). The work is split:

- **1026**: the technical test. Do `core`, `content` and `ui` build for
  the device without `std`? How long does an enemy turn take? Which
  bindings and toolchain? It writes the Playdate front-end ADR.
- **0061**: Nick's decisions (screen size and layout, telling things
  apart without colour, buttons and crank, what is left out, where it is
  sold), recorded in `docs/design/playdate.md`.
- **This ticket**: once both are done, write the build tickets that put
  Chapter 1 on the Playdate.

The engine carve goes on as planned. Carve tickets 1108 (console size)
and 1110 (platform shell) must keep a Playdate front end building if one
exists by then (noted in those tickets).

What the first look found (2026-10-07; check again when revived, the
numbers come from memory of the Playdate documentation, not a fresh read):

- **Screen.** The game draws a 100 × 32 grid of 8 × 16 px cells
  (`crates/ui/src/console.rs`): 800 × 512 px. The Playdate screen is
  400 × 240 px, black and white only (no greys, no colour). With today's
  font that is 50 × 15 cells, under a quarter of the room. So it is not a
  scale-down: every screen needs its own smaller layout, and the battle
  screen's 70-cell map plus 30-cell side panel (ADR-0018) cannot both stay
  on screen. A smaller font (6 × 8 px gives 66 × 30 cells) buys room at the
  cost of reading comfort on a 2.7-inch screen.
- **No colour.** Sides, ready/acted units, danger zones, terrain and
  weapon types are told apart by colour today (`docs/design/look-and-feel.md`).
  On one-bit they need another sign: inverted cells, dither patterns,
  letter case, blinking. Portraits and the bought map art are colour pixel
  art; the glyph map skin (ADR-0049) is the natural Playdate look.
- **Buttons.** D-pad, A, B, the crank, and the system Menu button (the
  system menu can carry up to three of the game's own entries). The game
  has 14 actions (`docs/design/controls.md`), but only cursor, Confirm,
  Cancel and End turn are required, and the map menu already opens from
  Cancel with nothing to cancel or Confirm on an empty tile.
- **Crank.** Rewind goes back to any earlier action in the battle
  (`docs/design/death-and-difficulty.md`), which suits winding the crank
  backwards through the history.
- **What carries over.** `core`, `content` and `ui` do not use macroquad
  (ADR-0004), screens build views that a skin paints (ADR-0054), and
  graphics are a skin (ADR-0038). What does not: `app` (macroquad, gilrs,
  quad-snd) is replaced by a Playdate front end; Playdate Rust has no
  standard library, so `core`, `content` and `ui` would have to build
  without it; the CPU is a 168 MHz ARM chip with 16 MB of memory, so the
  enemy AI's thinking time and the embedded data's size need measuring;
  music and voice need converting to formats the device plays.
- **Selling it.** A Playdate build can be sold on itch.io beside the
  Windows one (players sideload it); Panic's own Catalog store is curated
  and needs an application.

## Nick input

`None.` His answers come from 0061.

## Scope

**In:** the build tickets (`write-ticket` skill) that take Chapter 1, as
0061 decided it, to a Playdate build that can be sold where 0061 says.
Each ticket is one PR's worth of work. Expect roughly:
- making `core`, `content` and `ui` build without `std` behind a default
  `std` feature, one ticket per crate, using 1026's findings;
- the Playdate front-end crate (beside `app`, per 1026's ADR): run loop,
  buttons and crank to `Action`s through a keymap (`keyboard-input`
  skill), saves, and a CI job that builds it;
- the 1-bit font and the Playdate skins: the map scene and each Chapter 1
  screen's view painted at 400 × 240 (ADR-0038, ADR-0054), screen by
  screen, limited to what Chapter 1 uses;
- sound and music in formats the device plays (0061 keeps them), and the
  voices;
- one-bit portraits made from the bought art (check ADR-0040's license
  notes first) and a simple one-bit combat picture (0061 keeps both);
- the title cinematic on the Playdate (0061 keeps it);
- one build per language (English, Japanese, Chinese), not all languages
  in one build (0061);
- packaging and the store page: itch.io first, one purchase giving every
  language's build, then the Catalog application (0061; next to 0901).

**Out (do not do):** the build work itself; any change to the desktop or
web game's look or controls for the Playdate's sake; the engine carve.

## Implementation steps

1. Read 1026's ADR, `docs/design/playdate.md` (0061) and this ticket's
   *Context*.
2. Write the build tickets. Check each one's dependency chain both ways
   (`write-ticket`). For each ticket, say what happens if carve tickets
   1108/1110 land first, and add a matching line to 1108/1110 if they are
   still open.
3. Add the tickets to the Playdate bullet in `docs/ROADMAP.md`.
4. Run `cargo xtask ticket-lint`. Archive this ticket.

## Acceptance criteria

- [ ] Build tickets exist for every part of 0061's answers and 1026's
      ADR, each with `blocked_by` filled in.
- [ ] `docs/ROADMAP.md` lists them in order.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

- None (planning).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
