---
id: "0130"
title: "The playtest bots go engine first: 0506 and 0509 wait for the engine's rollout bot, runner and autobalance"
type: infra
milestone: M0 Foundation
model: opus-5.5
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-10-21
---

# 0130 — The playtest bots go engine first

## Context

Nick wants "the engine as capable as possible to support our game". The
playtest bots are Lane C of the Act 1 critical path
(`docs/ROADMAP.md`): 0506's persona bots, then 0509's autobalance.
0512 and every chapter battle ticket wait on them. A planning session
read the engine's bot tickets and found:

- engine#2112 (a breadth-first level solver) and engine#2113's minimax
  and full MCTS don't fit a wide tactics map. Both tickets say so.
- 0506's bot was already flat Monte Carlo, with redrawn luck
  (`reseed_luck`), a softmax and weighted persona scoring. So its
  generic parts belong in the engine:
  - `Determinize`, taken out of engine#2113;
  - a rollout bot;
  - a playtest runner with pass bands;
  - an autobalance search over knobs.
- The game keeps its personas, scoring, measures, bands and knobs.
- engine#2113 waits on engine#3009 (slicing), which waits on
  engine#3008 (benchmarks). The dev bots run on a PC and must not
  inherit that chain. engine#3009 matters for the enemy AI on the
  Playdate (1051), not for the bots.

This is a ticket-only re-gate, like 0129: no code.

## Nick input

None.

## Scope

**In:**
- Engine tickets in `Zafnok/shuyi-engine` (its own PR, merged first):
  - engine#2125, the rollout bot with `Determinize` and `Evaluate`
    (split out of engine#2113; no blockers);
  - engine#2126, the playtest runner with pass bands and
    `shuyi playtest` (no blockers);
  - engine#2127, the autobalance search (after engine#2126);
  - engine#2113 now also waits for engine#2125;
  - engine#3009 notes that this game's enemy AI (1051) is its first
    real user.
- 0506 rewritten to implement only the game's parts on engine#2125 and
  engine#2126, which are added to its `blocked_by`. Every design rule
  of `docs/design/playtest-bots.md` and the persona and band values
  stay.
- 0509 rewritten to supply the battle's knobs and the command on
  engine#2127's search, blocked by 0506 and engine#2127.
- 1051: a line on implementing engine#3009's `Think` when it lands,
  without waiting for it.
- `docs/ROADMAP.md`: Lane C (engine PR first, then the game PR), Lane
  F, the line saying that no engine ticket blocks Act 1, and the bots'
  line under *Later*.

**Out (do not do):**
- Any code, in either repository.
- Changing a persona's settings, a band or a design rule.

## Implementation steps

1. Write the engine tickets and edits in the engine's repository; open
   its PR.
2. Rewrite 0506, 0509 and 1051, and update the roadmap.
3. Check `blocked_by` both ways. 0507, 0508, 0510, 0512, 1021, 1064,
   0848–0854, 1070, 1071 and 1072 wait on 0506 or 0509 as before, so
   the chain now reaches the engine tickets through them.
4. Run `cargo xtask ticket-lint` (with `--check-engine` once the engine
   PR has merged), and the gates.

## Acceptance criteria

- [x] The engine PR holds engine#2125–2127 and the edits to
      engine#2113 and engine#3009.
- [x] 0506's `blocked_by` names engine#2125 and engine#2126, and 0509's
      names engine#2127.
- [x] `cargo xtask ticket-lint` passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

None (tickets and docs only); `ticket-lint` and the docs gates.

## Completion notes

Engine PR: Zafnok/shuyi-engine#42 (tickets only). What it does:

- adds engine#2125 (the rollout bot, with `Determinize` and `Evaluate`
  moved out of 2113), engine#2126 (the playtest runner with
  `play_tries` and `judge`, so the game's xtask keeps its threads) and
  engine#2127 (the autobalance search);
- edits engine#2113 (now blocked by 2112, 3009 and 2125), engine#3009
  (game#1051 is its first user, plus an either-order `impl Think for
  RolloutSearch`), and engine#2112, 2907 and 1134 (references to
  `Determinize` and the skill pack);
- adds the three tickets to the engine's roadmap under *Requests from
  the game*.

Game side:

- 0506 and 0509 are rewritten with a table of what is the engine's and
  what is the game's, and an expected size (ADR-0092). The persona and
  band values and every design rule are unchanged.
- 0506 adds the acceptance tests `planning_never_sees_pending_reinforcements`
  and same-report-at-any-thread-count.
- `Determinize` for `BattleState` goes in `trpg-core` (the orphan
  rule).
- Ticket numbers: Zafnok/visions-of-shuyi#382 (0449) adds 0072 and
  1178, which this re-gate doesn't use.

`cargo xtask ticket-lint`: OK. `--check-engine` reports engine#2125–2127
missing until the engine PR merges. Run it again after that merge.
