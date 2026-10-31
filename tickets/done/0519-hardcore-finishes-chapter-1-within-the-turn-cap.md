---
id: "0519"
title: "Find why Hardcore stalls in Chapter 1 until the turn cap"
type: bug
milestone: M4 Enemy AI
model: reasoning
effort: medium
status: done
blocked_by: ["0518"]
nick_input: none
completed: 2026-10-28
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

- [x] The cause is named in the Completion notes, with the fix (or, if it
      is a weight, the ticket is blocked on 0508 and says so).
- [x] No Hardcore try of `ch01` (seeds 1 to 20) ends at the turn cap.
- [x] `personas.ron` is unchanged.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the fixed count or order (`crates/bots/src/persona/tests.rs`).

## Completion notes

**The cause: heals filled every place Hardcore tries, so it never
weighed a move forward.** Not the danger count, a guard or a weight.
In seed 9 the lead is down to 2 HP after turn 3, with every enemy left
a guard out of reach. `order` lists the heals third, after the attacks
and the enemy AI's pick, "each once" per unit and action, but a pack
slot is part of the action: the same Potion in three pack slots was
three heals, and each of the five units in reach of the lead offered
them. With `candidates: 12`, the eleven places before ending the phase
all went to heals of the lead (each -200 or so: an item at -300 for 10
HP back). The advances (+70 for one) and holds were never tried, ending
the phase (0) was the best of what was, and the battle stood still for
57 turns. Seeds 10, 12, 15 and 17 were the same.

**The fix** (`order`, `crates/bots/src/persona.rs`): one heal per wounded
unit and remedy (a pack item kind or a heal spell), the first by the
same order as before (the most wounded first, a healer who needn't move
first). Who uses it, and from which slot, no longer counts twice. Test:
`order_lists_one_heal_per_wounded_unit_and_item` (two Potions and an
Elixir in the pack, three units in reach of two wounded ones: four
heals, not eighteen).

Chapter 1, `cargo xtask playtest ch01 --bot all --runs 20`, before ->
after:

| persona | succeed | won | turns median | items median |
| ------- | ------- | --- | ------------ | ------------ |
| casual | 5/20 (too hard) -> 7/20 (in band) | 5 -> 7 | 19 -> 17 | 3 -> 3 |
| normal | 13/20 -> 13/20 | 18 -> 16 | 10 -> 8 | 3 -> 3 |
| hardcore | 10/20 -> 16/20 (too easy, band 50-75%) | 14 -> 17 | 8 -> 8 | 1 -> 0 |

- No Hardcore try ends at the cap. The five stalls: seeds 9, 10, 12, 17
  won, seed 15 lost (the lead fell on turn 6). The other two defeats
  (13, 18) are new paths, each turn now weighing advances it used to
  skip.
- The order is shared, so Casual and Normal changed too: their 10 places
  had the same flood of heals whenever anyone was wounded. Casual
  is now in its band; Normal is faster (median 10 -> 8 turns).
- Two bands moved out: Hardcore succeeds 16/20, over its 50-75% ("too
  easy"), and its "fewer turns than Normal" check fails (8 against 8),
  because Normal got faster too. Both are the bands' and weights' to
  judge in 0508 (ADR-0095 §5); nothing here changes them.
- Hardcore uses fewer items (median 1 -> 0): it no longer drifts into a
  Potion as the best of a list with nothing else on it.
- No gameplay rule decided: the bots only.
- Engine friction: estimate 25 production lines (one key for a heal and
  a filter, like `once_per_action`), written once the cause was found,
  not before the trace; measured 36 (1.4x), under 2x.
