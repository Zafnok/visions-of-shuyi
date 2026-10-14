---
id: "0457"
title: The lessons - basic controls and what's different here
type: content
milestone: M3 Battle UI
model: opus-5.5
effort: medium
status: todo
blocked_by: ["0456"]
nick_input: sign-off
completed:
---

# 0457 — The lessons

## Context

Ticket 0057 (`docs/design/tutorial.md`, *The lessons*) decided which
lessons exist; ticket 0456 builds the checklist machinery
(`assets/data/lessons.ron`). This ticket writes the lessons, adds the
triggers and `done_when` conditions they need, and checks that each can
happen in Chapter 1 (except terrain magic, which waits for a battle where
it can happen).

**Basic controls** (level `Full`):

1. Moving the cursor: up, down, left, right (turn 1).
2. Choosing a unit, moving it, and its action menu.
3. Attacking: the forecast, then confirming (the first time an enemy is
   in reach).
4. The next unit and a unit's details.
5. Ending the turn.

**Different here** (level `Short`, so it shows at `Full` and `Short`):

6. The keyboard controls: the cursor on keys, and every key can be changed
   in Options (turn 1).
7. Rewind: undoing turns, with a limited number of charges per map. The
   lesson opens the rewind screen and closes it; it never spends a charge
   (0057).
8. Threat arrows: which enemies can reach the tile you're moving to (the
   first time they show).
9. Combat Arts: their own menu, and some cost uses per battle (the first
   time a unit with an art opens its action menu).
10. Terrain magic: a spell can change a tile (the first time a spell that
    can change a tile is in a unit's Magic menu).

## Nick input

**Sign-off:** after merge, play the Chapter 1 opening at `Full` and at
`Short`. The items and timings below that the design doc doesn't fix are
*Claude's starting rules*; list them in the PR for Nick to veto.

## Scope

**In:**
- The ten lessons in `assets/data/lessons.ron`: items, triggers, levels.
  Words are Claude's, in plain American English, ≤ 60 characters a line;
  every key through `{Action}` placeholders (`keyboard-input` skill). The
  guide's voice comes later (0731); write them plainly now.
- At `Full`, lessons 1 and 6 both start on turn 1: merge what 6 teaches
  into 1 when the level is `Full`, or run 6 after 1. Pick one and say which.
- New triggers and conditions the lessons need (in 0456's format), with
  tests.
- Lesson 7's timing: suggested first player phase of turn 2 (something has
  happened that can be undone). Lesson 5's: when no unit is left to act on
  the first turn (before auto-end would fire; auto-end waits for it).
- Check against Chapter 1's data (`assets/data/` and the battle file): if a
  lesson's moment can't happen there (no unit with an art, say), report it
  in the completion notes and the PR. Don't change the map or the units.

**Out (do not do):**
- New tips and the Guides list (0458).
- The guide's name and voice (0731).
- Changing Chapter 1's map, units or script.

## Implementation steps

1. Write the lessons; add triggers and conditions as needed.
2. A scripted Harness test per lesson on the shipped Chapter 1 (or the test
   chapter where Chapter 1 can't reach it), at the level it belongs to,
   doing its items in a shuffled order.
3. A test that at `Short` none of lessons 1–5 opens, and at `None` none
   opens.
4. Render the turn-1 lesson box(es) with `cargo xtask frame-png
   --features private-assets` and look at them.

## Acceptance criteria

- [ ] Each lesson opens at its moment at the right levels and closes when
      its items are done.
- [ ] Rewind's lesson leaves the rewind charges unchanged.
- [ ] Every key in the lessons' text comes from the keymap
      (`cargo xtask check-keys` or the content check passes; no letter
      written in).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot / integration: one scripted run per lesson; the level gating;
  the rewind charge count.

## Completion notes

*(Filled in by the session that completes the ticket.)*
