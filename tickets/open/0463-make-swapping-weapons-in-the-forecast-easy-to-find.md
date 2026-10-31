---
id: "0463"
title: "Make swapping weapons in the forecast easy to find"
type: feature
milestone: M3 Battle UI
model: reasoning
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0463 — Make swapping weapons in the forecast easy to find

## Context

Since 0442 there is no weapon list: Attack opens the forecast with the
equipped weapon, and ← / → swap weapons there (`docs/design/
combat-arts.md`, *The arts menu and the forecast*). Chapter 1 playtest
round 3 (0804, 2026-10-16), on the Arts menu:

> "so swapping weapons on forecast menu feels quite hidden as a feature it
> should be more obvious idk how also the guide should probably mention
> it"

Nothing on the forecast says ← / → do anything, and no lesson or tip
mentions it.

**Waits for the engine carve and the Playdate port** (Nick, 2026-10-16),
so `blocked_by` has 1118 and 1050.

**Re-gated 2026-10-21 (0129):** the carve is done and 1050 was folded
into the Act 1 playtest (1064), so neither 1118 nor 1050 is in
`blocked_by` any more.

## Nick input

**Decision** (`ask-nick`) on real frames of the forecast with the bought
art (a unit with two weapons, e.g. the lead with a sword and a lance if
one exists in Chapter 1, or the debug Quick Battle). Options, from real
games (check each before showing it):

- A. Arrows ◄ ► either side of the weapon's name, as Fire Emblem's
  forecasts do on the weapon row (Three Houses, Engage).
- B. A key hint on the forecast: "← → weapon" with the player's keys
  (follow 0250's look for key names if it's done).
- C. A line in the Attacking lesson: "Swap weapons: {CursorLeft} {CursorRight}"
  (only when the unit has 2+ weapons).
- D. A tip, the first time a unit with 2+ usable weapons opens a forecast.
- E. Combinations, or describe your own.

Nick said "the guide should probably mention it", so a lesson line or tip
(C or D) is likely part of the answer; ask which.

## Scope

**In:** the question; building the chosen look in the forecast view and
skins (ADR-0054, ADR-0038: the forecast is a screen view, not the map);
the lesson line or tip; `combat-arts.md` and `tutorial.md` updated.

**Out (do not do):** changing what ← / → do.

## Implementation steps

1. Mockups from the real forecast view; ask Nick; record the answer.
2. Build it; keys through the keymap (keyboard-input skill: never a
   hard-coded key).
3. Tests.

## Acceptance criteria

- [ ] Nick picked; recorded in `combat-arts.md`.
- [ ] The forecast shows the swap affordance only when there's more than
      one weapon to swap to (test on the view).
- [ ] The lesson line / tip exists if picked (test).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the forecast view's swap flag.
- Snapshot: the forecast with and without a second weapon.

## Completion notes

*(Filled in by the session that completes the ticket.)*
