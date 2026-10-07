---
id: "1025"
title: "Parked: Playdate port (glyph look, crank rewinds)"
type: research
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: blocked
blocked_by: ["0903"]
nick_input: decision
completed:
---

# 1025 — Parked: Playdate port

## Context

Nick, 2026-10-07: "could we make a Playdate console port? […] as far as
controls limitations we have solutions like chording A+B if we need to or
otherwise have A/B off of a unit act differently like it already does and
the rewind mechanic can use the winding mechanism on the side. this is for
hella after launch but if we just reuse the retro glyph stuff it could
totally scale down to Playdate I think".

This ticket keeps the idea and what a first look found on record.
**Do not start it** until the game has launched (0903) and Nick asks for it.

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

**Decision:** when Nick wants it, run a `00xx` decision ticket (`ask-nick`
skill, with rendered 400 × 240 black-and-white mockups) on: the buttons and
the crank; how the battle screen is cut down; how sides and states are
told apart without colour; what is left out (portraits, voices, Japanese
and Chinese text); where it is sold.

## Scope

**In:** when Nick revives the idea: a short feasibility spike (does `core`
build for the Playdate, how long does an enemy turn take on the device or
simulator), an ADR for the Playdate front end, the `00xx` decision ticket,
and the implementation tickets.

**Out (do not do):** any Playdate code, layouts or dependencies before
launch and before Nick's decisions. No change to the desktop or web game
for the Playdate's sake.

## Implementation steps

1. Wait for 0903 and for Nick to ask for the port.
2. Re-check the facts in *Context* against the current Playdate SDK and
   its licence (ADR-0013), and the licences of the Rust bindings.
3. Do the feasibility spike; record the result in an ADR (`write-adr`).
4. Write the `00xx` decision ticket and the implementation tickets
   (`write-ticket`), and close this one as superseded.

## Acceptance criteria

- [ ] Nick asked for it after launch.
- [ ] An ADR records whether and how the port is built.
- [ ] A `00xx` decision ticket and the implementation tickets exist.

## Tests required

- None.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
