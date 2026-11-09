---
id: "0528"
title: "Enemies close in instead of waiting, except on good ground"
type: tuning
milestone: M4 Enemy AI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0528 — Enemies close in instead of waiting

## Context

Nick played the Act 1 build (Chapters 1 to 4) on 2026-11-08:

> "Too many AI placements and behaviors don't make much sense like the
> random far away archer in ch3 and a lot like to stand still even when
> it doesn't make sense. I can give a pass for when they want to stand in
> advantageous terrain but overall they should advance toward the player
> more often"

**What the AI does today** (`crates/core/src/ai.rs`, rules in the module
doc ~:1–180; `AiBehavior` ~:218; `decide` ~:465–540):

- `Aggressive` (the default): attacks the best target it can reach this
  turn, else walks towards the nearest by flow distance.
- `Guard`: attacks only a target it can reach **this turn** (its move
  plus its weapon's range), else **holds its tile** (~:538), all battle.
- `Stationary`: attacks only from its own tile (bosses on forts, tower
  archers).
- `Healer`, and the raider and escape rules, as written there.

Nearly every enemy in Chapters 2 to 4 is `Guard`; under it a unit on open
ground waits until someone walks into its reach. Chapter 3's
`warden_archer` at (22, 12), the courtyard's far corner, and the one in
the reliquary at (25, 3) (`assets/battles/ch03.ron`) can wait all battle.
The terrain under each `Guard` today (from the maps):

| Chapter | On good ground (Def or Avoid > 0) | On open ground (plain, road, floor) |
| ------- | --------------------------------- | ----------------------------------- |
| 2 | raider (23, 5) forest | archer (18, 10); guards (8, 6), (10, 6) and archer (11, 5) beside Hendry; rider (16, 5); swordsmen (5, 3), (1, 4) |
| 3 | Ned (15, 7) and the warden (15, 8) on the gate | the landing's raider (10, 6) and swordsman (10, 9); courtyard guard (18, 7), raider (20, 9), rider (24, 7), archer (22, 12); the two at the stair's top (17, 3), (18, 4); reliquary archer (25, 3) |
| 4 | the gate's guard (13, 12) | yard raider (9, 8), swordsman (16, 8), rider (11, 6), the front door's guard (13, 6); sentries (23, 8), (24, 12); kitchen raider (18, 3); Dace (5, 3, boss) and his guard (7, 3) and archer (9, 2) |

Chapter 1 and the two side quests built so far
(`side_company_ledger.ron`, `side_letters_home.ron`) use `Guard` too.

Nick gave the direction, so this is not a decision ticket: the rule
below is **the agent's starting rule** (`tickets/README.md`), written
down with an example, judged by the bots here and by Nick in the Act 1
playtest (1064).

**Either order with 0084:** 0084 decides the behaviours of Chapter 3's
own units. If 0084 is done, keep what it set and apply this ticket's
rule only to units it left alone; if not, apply the rule as written and
0084's answer (built by 0868) sets Chapter 3's units afterwards.

## Nick input

None (Nick gave the direction above). Verified by the AI's tests and the
bots' bands; the starting rule and the units it moved go in the
Completion notes and on the Act 1 playtest's checklist (1064).

## Scope

**In:**

- **The agent's starting rule for `Guard`:**
  - A `Guard` **on good ground** (its tile gives Def or Avoid: forest,
    mountain, fort, gate, throne) holds it as today.
  - A `Guard` **on open ground** holds only until a hostile unit comes
    within **two of its moves**: the flow distance from its tile (the
    walking cost to a tile next to a target, the same field `decide`
    already uses) is at most 2 × its Mov. From then on it plays as
    `Aggressive`: it attacks if it can, else walks at the nearest target.
  - A **boss** keeps its authored behaviour (Hendry and the prior stay
    `Stationary`, Dace `Guard` as he holds the lord's end today): bosses
    hold in the games this one follows, and Chapter 4's bands were found
    with Dace as he is (`docs/playtesting.md`, Chapter 4).
  - Plain example, for the module doc and the Completion notes: *Chapter
    4's sentry swordsman at (23, 8) stands on the meadow's grass with Mov
    5. Today he waits there until someone walks within 6 tiles of him.
    With the rule, once anyone of the party is 10 tiles' walk away he
    comes at them. Ned Tolley on Greywater's gate (+3 Def, +20 Avoid)
    still holds the gate.*
- **A behaviour for bodyguards:** a new `AiBehavior::Hold` that is
  today's `Guard` (attacks what it can reach this turn, else holds,
  wherever it stands), for a unit whose place in the story is next to
  someone. The danger zone, the threat arrows (`movement.rs` ~:508–531)
  and the persona bots (`crates/bots/src/persona.rs` ~:403) treat `Hold`
  as they treat `Guard` (only `Stationary` is special there).
- **A pass over every battle's `ai` fields** (Chapters 1 to 4, the two
  side quests). The starting proposal, to check against the bots:
  - `Hold`: Chapter 2's two guards and the archer beside Hendry ((8, 6),
    (10, 6), (11, 5): his Pack Hunt counts allies next to the target,
    `ch02.ron` header); Chapter 3's two at the top of the stair ((17, 3),
    (18, 4)); Chapter 4's guard and archer with Dace ((7, 3), (9, 2)).
  - The rest keep `Guard` and so close in under the new rule.
  - Chapter 1: unchanged data first; its numbers were approved over four
    rounds of Nick's play, so if it leaves its bands the first fix is
    `Hold` on the Chapter 1 units that moved it, not other changes.
- The module doc's *A unit's decision* (rule 3) and `assets/data/ai.ron`'s
  header say the rule, with the example.
- The bots on every battle changed, both seed ranges, inside their bands
  (`cargo xtask playtest chNN --bot all`; `cargo shuyi autobalance
  --stage chNN` where a chapter leaves its band, which moves only the
  enemies' numbers). The recorded playthroughs that move are recorded
  again on purpose (each winnable test's header,
  `crates/ui/tests/it/ch01_winnable.rs` to `ch04_winnable.rs`), in
  order. Tables in `docs/playtesting.md`.

**Out (do not do):**
- Where Chapter 3's far archers stand: 0868 (Chapter 3's seize, after
  0084) may move or remove them. **Either order:** if 0868 is done first,
  this pass takes its placements as they are; if not, 0868's pass reads
  this ticket's rule.
- The AI's attack scoring (`assets/data/ai.ron` weights).
- 0450 (the danger zone of a `Stationary` enemy): either order; it
  doesn't touch `Guard` or `Hold`.
- 1051 (the AI plans once per phase). **Either order:** if 1051 is done
  first, the wake test (flow distance from the unit's tile) is part of
  the plan and must be invalidated when a unit moves, as 1051's other
  facts are; if not, 1051's equality tests include this rule.
- 0525 (Hardcore no faster at Veyne Hall). Either order; whichever lands
  second re-runs Chapter 4's bands with the other in.
- Chapter 3's seize (0868), Chapter 2's houses (0867), Joss (0870),
  armour (0333), EXP (0610): either order; whichever lands second
  re-records the playthroughs and re-runs the bands with the first in.
- Random skirmishes (`assets/skirmishes/`, the `act1_random_*` battles):
  they use no `Guard` today.

## Implementation steps

1. Read `ai.rs` (`decide`, `flow`, `stand`, the tests in
   `crates/core/src/ai/tests.rs`), 0523's done ticket (Hardcore stalling
   in front of a guarded chokepoint) and 0524's (bands after it).
2. Add `AiBehavior::Hold` (serde name `Hold` in battle files; the content
   loader and the AI soak test, `crates/content/tests/it/ai_soak.rs`,
   accept it).
3. Change `Guard` in `decide`: good ground from the tile's terrain
   (`terrain.defense > 0 || terrain.avoid > 0`), the wake test from the
   flow distance at the unit's tile against 2 × its Mov, a boss
   (`Role` / `boss`) keeping today's behaviour. Keep it a pure function of
   the state (ADR-0004).
4. Data: the `ai` pass above.
5. Re-record the playthroughs that moved (1 → 4), run the bots, adjust
   with `Hold` or the autobalancer, record the tables.

## Acceptance criteria

- [ ] A `Guard` on a plain with a hostile 2 × Mov away walks towards it
      (test); one on a gate or fort with the same hostile holds (test); a
      `Guard` boss holds (test); a `Hold` unit holds on a plain (test).
- [ ] A `Guard` on a plain with no hostile within 2 × Mov holds (test).
- [ ] The danger zone and threat arrows give `Hold` units the same tiles
      as `Guard` units (test).
- [ ] Each battle's `ai` fields match the Completion notes' list.
- [ ] The winnable tests pass; the bots are in their bands on every
      battle this changed (tables in `docs/playtesting.md` and the PR).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the cases above in `crates/core/src/ai/tests.rs`.
- Property: the AI stays deterministic (the same state gives the same
  command) and a `Guard`'s command is still one the battle accepts.
- Integration: the AI soak test; the winnable tests; the bots' runs.

## Completion notes

*(Filled in by the session that completes the ticket.)*
