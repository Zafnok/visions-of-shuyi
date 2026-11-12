---
id: "0165"
title: The bots' and tools' tests of the real data run on the private copy
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0162"]
nick_input: none
completed:
---

# 0165 — The bots' and tools' tests of the real data run on the private copy

## Context

Split from 0162 before coding (rule 10): see 0163's Context for the
split and ADR-0118's rules ((a) a test of the content keeps the real
content and starts with `if trpg_content::bundle::skip_without_the_game()
{ return; }`; (b) a test of a mechanism moves to `assets-test/`; a public
file names ids only, never a line, a real map or the plot). Read 0157's
and 0162's completion notes in `tickets/done/`.

This ticket takes the dev tools and the Playdate crate. From 0162's
inventory (2026-11-12; grep again for `load_embedded`, `story_only`,
`with_shipped_story` and real ids in string literals):

- `crates/bots/`: `army.rs`, `arrival/tests.rs`, `balance/tests.rs`,
  `playtest/tests.rs`, `skirmish/tests.rs`, `testkit.rs`,
  `tests/baseline.rs`;
- `crates/xtask/`: `playtest/tests.rs`, `skirmish/tests.rs`,
  `game_tools/balance/tests.rs`, `game_tools/tests.rs` (the battle half;
  0157 re-pointed the `--scene` half), `battler_import.rs`,
  `tileset_import/tests.rs`;
- `crates/playdate/src/chapter_1.rs` (plays Chapter 1 on the Playdate's
  bundle: a test of the content);
- `bench/*.bench.ron`: real battles by id, in the keyed CI job (0160
  keeps them naming ids only): check each names ids only and say so.

Most bot tests that play a real chapter to a win are tests of the
content (a); those that check how a bot or a tool works (b) can play the
test chapter and battles of `assets-test/`.

## Nick input

None.

## Scope

**In:**
- The inventory, in the completion notes, each test marked (a) or (b).
- (a): the guard. (b): pointed at the test files, with test data added
  to `assets-test/`.
- Comments and test names that tell the plot reworded (ADR-0118 §2).

**Out (do not do):**
- The UI (0163, 0164); content and core (0162).
- Moving the content (0160) or the public fallback it builds.
- Changing any real content, or the bots' recorded playthroughs.

## Implementation steps

1. Write the size estimate first (rules 9 and 10). Past about 2,000
   changed lines, split again with `write-ticket` (bots apart from
   xtask), each blocking 0160.
2. The inventory: grep, then read each test.
3. Guard the (a) tests; re-point the (b) tests, adding test data.
4. Run the (b) tests with `story_only` where a test of the test files
   can, and say in the notes what 0160 must run.

## Acceptance criteria

- [ ] The inventory is in the completion notes, each test marked (a) or (b).
- [ ] Every (a) test starts with the guard; every (b) test reads only the
      test files and what 0160 keeps public.
- [ ] `cargo xtask check-leaks` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The re-pointed tests.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
