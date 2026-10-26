---
id: "0519"
title: "Find why Hardcore stalls in Chapter 1 until the turn cap"
type: bug
milestone: M4 Enemy AI
model: opus-5.5
effort: medium
status: todo
blocked_by: ["0518"]
nick_input: none
completed:
---

# 0519 — Find why Hardcore stalls in Chapter 1 until the turn cap

## Context

Seen in 0516's Chapter 1 runs and again in 0518's: 5 of Hardcore's 20
tries of `ch01` (seeds 9, 10, 12, 15, 17) stop unfinished at the 60-turn
cap, with nobody fallen and no item used. 0518 fixed two counting bugs in
the persona bots' danger (a boss's combat active wasn't seen; a lone
lord's danger was dropped in an escape) and the stalls stayed exactly
the same, seed for seed, so the cause is something else.

Reproduce: `cargo xtask playtest ch01 --bot hardcore --runs 20`.

Suspects, none checked: the danger count (`crates/bots/src/persona.rs`,
`danger`, `lord_at_risk` at -5000) finding every advance into Red Harl's
company unsafe, so holding out of reach (`order`'s holds) always scores
better than the fight and the battle never moves; a guard who never moves
(`Stationary`) the plan never gets past; `turns: -200` too small against
the danger to make it go.

## Nick input

None, unless the cause is a setting or a weight: then stop, say so in
the ticket, and it waits for 0508 (ADR-0095 §5).

## Scope

**In:**
- Follow a stalled try's commands (seed 9) to the turns where nothing
  changes, and find what Hardcore scored the moves forward against what
  it did.
- Fix it in the bots' scoring or order, with a test.

**Out (do not do):**
- Any weight, band, floor or persona setting (ADR-0095 §5).
- Chapter 1's own units or map.

## Implementation steps

1. Trace seed 9 (a throwaway test with `play_observed`, as 0518 did).
2. Fix the cause; write the test first.
3. Chapter 1's playtest (`cargo xtask playtest ch01 --bot all --runs
   20`) before and after, in the PR; any change explained.

## Acceptance criteria

- [ ] The cause is named in the Completion notes, with the fix (or, if it
      is a weight, the ticket is blocked on 0508 and says so).
- [ ] No Hardcore try of `ch01` (seeds 1 to 20) ends at the turn cap.
- [ ] `personas.ron` is unchanged.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the fixed count or order (`crates/bots/src/persona/tests.rs`).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
