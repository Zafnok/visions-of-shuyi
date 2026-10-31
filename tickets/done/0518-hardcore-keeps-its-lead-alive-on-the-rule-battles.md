---
id: "0518"
title: "Find why Hardcore loses its lead early on test_escape and test_retreat"
type: bug
milestone: M4 Enemy AI
model: reasoning
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-10-26
---

# 0518 — Find why Hardcore loses its lead early on test_escape and test_retreat

## Context

Found by 0516's persona smoke test over `assets-test/battles/`. The
Hardcore persona, the best player of the three (ADR-0095), loses its lead
(a Defeat) on turn 4 or 5 in every one of 3 tries of `test_escape`, and
on turn 1 in two of 3 tries of `test_retreat`. Normal wins most of the
same tries. Hardcore weighs a fallen lord at -10000, the loss at another
-10000 and a lord left where it could fall (`lord_at_risk`) at -5000
(`crates/bots/personas.ron`), so it should never walk its lead into
reach; something in what it sees or is offered is wrong.

Related, seen in 0516's Chapter 1 runs (the same before and after
0516): 5 of Hardcore's 20 tries of `ch01` (seeds 9, 10, 12, 15, 17) stop
unfinished at the 60-turn cap, with nobody fallen and no item used. If
the cause turns out to be the same, fix both; if not, file the Chapter 1
stall as a ticket of its own.

Reproduce: `cargo xtask playtest test_retreat --bot hardcore --runs 3`
and `cargo xtask playtest test_escape --bot hardcore --runs 3`.

Suspects to check, none confirmed: the danger count (`danger`,
`threats` in `crates/bots/src/persona.rs`) missing a threat these maps
have (reinforcements arriving next to the lead on turn 3 in
`test_escape`; a boss who retreats in `test_retreat`); Hardcore's
`turns: -200` pushing it to hurry; a test character that is simply too
weak for the map.

## Nick input

None, unless the cause is a setting or a weight: then stop, say so in
the ticket, and it waits for 0508 (ADR-0095 §5). If the cause is the
test battle itself (it is a test file, not a chapter), changing the test
file is fine.

## Scope

**In:**
- Find the cause by following the tries' commands, and fix it in the
  bots' scoring or order, or in the test file.
- A test that shows it.

**Out (do not do):**
- Any weight, band, floor or persona setting (ADR-0095 §5).
- Casual stalling on `test_escape` (0517).

## Implementation steps

1. Play the tries above and find the command that puts the lead where it
   falls, and what Hardcore scored it against the alternatives.
2. Fix the cause; write the test first.
3. Chapter 1's playtest (`cargo xtask playtest ch01 --bot all --runs
   20`) before and after, in the PR; any change explained.

## Acceptance criteria

- [x] The cause is named in the Completion notes, with the fix.
- [x] Hardcore wins at least 2 of 3 tries of each of `test_escape` and
      `test_retreat` (seeds 1 to 3).
- [x] `personas.ron` is unchanged.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the fixed count or order (`crates/bots/src/persona/tests.rs`).

## Completion notes

Two causes, one per battle; both were in what the bots count, not in
their weights.

- **`test_retreat`: a boss's combat active was invisible.** The danger
  count reads each enemy's forecast with `BattleState::threat_forecast`,
  which forecast the plain attack only. The boss (a Raider) has Heavy
  Blow (+5 might): 14 damage instead of 9. Hardcore walked the lead (19
  HP) to (6, 3), in reach of the boss and the brigand; by the plain
  forecasts they could do 17 at most, so `lord_at_risk` was 0, and with
  Heavy Blow they did 22. Fix: `threat_forecast` now takes, of the plain
  attack and each combat active and Combat Art the enemy may use and
  pay for (bosses only, among enemies), the one that hits hardest: the
  most damage if every strike hits, then the most expected. Hardcore now
  wins 3 of 3, in 2 turns.
- **`test_escape`: a lone lord's danger was dropped.** `danger` stops
  counting a lord's danger once it is the last player unit ("waiting
  can't win"), so once the knight and the archer escaped, the lead's own
  danger counted for nothing while `counter` still paid for striking
  back. It walked from (8, 2) back towards the arriving raiders and fell.
  In an escape (and a seize, a survive) walking or waiting does win, so
  the rule now applies only in a rout and a boss fight. Hardcore now
  wins 2 of 3.
- The third `test_escape` try (seed 3) is still lost: with 19 HP, the
  lone lead steps towards one raider because `counter` (+15 per HP struck
  back) beats `lord_exposed` (-10 per HP taken), then can't get clear
  next turn. That is the weights' trade, not a count, so it is left for
  0508 (ADR-0095 §5).
- Chapter 1, `cargo xtask playtest ch01 --bot all --runs 20`, before →
  after: Hardcore identical, try for try (10/20 succeed, 14/20 won, the
  same 5 stalls at the cap). Casual 6 → 5 succeed, Normal 19 → 18 won
  (13/20 succeed both): Red Harl's Heavy Blow now counts in their danger,
  which sends a few tries (Casual seeds 5, 6, 10, 15, 16, 19, 20; Normal
  14, 18, 19) down other paths; no band changed but Casual's (6/20 was
  just in, 5/20 just out of 30-55%).
- The Chapter 1 stalls didn't move, so they have another cause: ticket
  **0519**.
- Tests: `a_threat_forecast_counts_a_bosss_harder_active` (core),
  `danger_counts_the_threats_a_player_can_see` extended to the lone lord
  in an escape, a seize and a survive (bots), and
  `the_personas_finish_the_rule_battles` now asserts Hardcore wins 2 of 3
  of `test_escape` and `test_retreat`.
- ADR-0095 (accepted, not edited) describes `threat_forecast` as "with
  its equipped weapon" and the lone-lord rule without the objective; the
  code docs and `docs/playtesting.md` now say both. Its own reason for the
  rule ("waiting can't win") is what the change restores.
- No gameplay rule decided: the bots only.
- Engine friction: estimate 40 production lines (the AI's own list of a
  unit's actives and arts, `extras` in `ai.rs`), measured 67 (1.7×),
  under 2×.
