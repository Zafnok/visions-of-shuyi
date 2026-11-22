---
id: "0535"
title: "One table-driven test replays every recorded win, and record-wins records them"
type: infra
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0531"]
nick_input: none
completed:
---

# 0535 — One table-driven test replays every recorded win, and record-wins records them

## Context

Split from 0531 before coding (ADR-0101): 0531 kept the bots' stable
army, and this ticket took the four items its 2026-11-17 sweep added
(B4, [report](../../docs/research/friction-2026-11-17.md), findings
"Each chapter battle needs a hand-chained test module" and "Public
tests pin private content values"). Together they were past 2,500
changed lines.

Each recorded win is its own hand-written module,
`crates/ui/tests/it/ch01_winnable.rs` to `ch05_winnable.rs` (about 900
lines, 130–275 each), plus `skirmishes.rs` (253) and `side_quests.rs`
(340). `chapter_N()` calls `chapter_{N-1}()`, so a new battle edits the
one before it too (#588 edited `ch04_winnable.rs`). There are 11
`*_winning.ron` fixtures and 11 `#[ignore]` `record` tests, run one at
a time; #731 (one stat change) re-recorded 9 of them. The replay tests
also pin tuned values (`ch01_winnable.rs`, `campaign.gold == 1000`), so
a balance change in the private repository edits a public test.

About 6 battles are left on the Act 1 path (0852–0854, 1090, 1083,
1084), each about 200–350 test lines and 8 small edits today. **Either
order** with them: a battle ticket done before this one adds its module
the old way, and this ticket folds it in.

What 0531 changed (read its Completion notes and ADR-0130 first): the
bots no longer build their arrival army from these fixtures, so
re-recording one moves no bot army. The fixtures stay the UI's. The
bots record their own wins, 8 lines a battle, with `cargo xtask
bot-wins` (`crates/tools/src/bot_wins.rs`) and the search moved to
`trpg_bots::record`; `crates/ui/tests/it/playthrough.rs` still keeps its
own copy of that search for the UI's 11 `record` tests, since `trpg-ui`
may not depend on `trpg-bots` (`crates/xtask/src/main.rs` checks it).

## Nick input

None.

## Scope

**In:**
- **One table-driven recorded-win test.** Replace the hand-chained
  `chNN_winnable.rs`, `skirmishes.rs` and `side_quests.rs` replay tests
  with one test that walks every battle with a
  `fixtures/<battle>_winning.ron` in story order (the chapters, then
  the world map's guards and side quests) and replays each by id. What a
  win must leave (joins, items, houses, nobody lost) goes in the
  fixture's header or is read from the data, not a module per battle.
  The army each battle starts with is built by one function (the
  campaign after the earlier recorded wins, as the game applies them),
  shared by the replay test and the recorder.
- A battle on the story path with no fixture is reported by its id, not
  skipped silently.
- **`cargo xtask record-wins [<battle>|--all]`**: runs the `record` step
  for one battle or for all of them in story order (each from the army
  the ones before it leave), so a re-record after a balance change is
  one command. One `record` entry point, not 11 `#[ignore]` tests: it
  uses `trpg_bots::record` from the game's tool program, beside
  `bot-wins` (one command for both is fine), and the UI's copy of the
  search in `playthrough.rs` goes.
- **Relations, not tuned numbers, in public tests.** The replay test
  reads what it expects from the loaded data (the campaign's gold
  equals the battle's `clear_gold`, the fallen equal the enemies at the
  start). Any test that must pin an exact tuned value moves to the
  private repository's `content/tests/` (ADR-0118 §3) with the
  `content-change` skill. Mechanism tests on `assets-test/` keep their
  literals.
- The demo's play-through (0914) and any other caller of
  `chapter_1_fought` and the other per-chapter helpers keep working
  through the shared function.

**Out (do not do):**
- The bots' arrival army (0531).
- Reaching a battle through one Harness call (1289). **Either order**;
  if 1289 is done and its helper fits, the table test uses it.
- Moving public value tests outside the recorded wins (for example
  `crates/content/tests/it/skills.rs`).
- Retuning anything; re-recording a fixture beyond the one check run
  below.

## Implementation steps

1. Read `crates/ui/tests/it/playthrough.rs`, the seven replay modules,
   their callers (`grep -rn "winnable::" crates/ui/tests`), and 0531's
   ADR.
2. Measure with `cargo xtask pr-size` as you go; plan at about 1,800
   lines (mostly deletions).
3. Write the shared army function and the table test; move the
   per-battle expectations to fixture headers or relations.
4. Delete the per-battle modules; add `record-wins` (xtask, or the
   game's tool program, ADR-0129).
5. Run `cargo xtask record-wins --all` once and check that every fixture
   replays (or that its recording is unchanged if the search is
   deterministic).

## Acceptance criteria

- [ ] One test replays every `*_winning.ron` fixture by id;
      `ls crates/ui/tests/it/ch0*_winnable.rs` finds nothing.
- [ ] A test shows a story battle with no fixture is reported by its id.
- [ ] `cargo xtask record-wins --all` re-records every fixture in story
      order, and `cargo xtask record-wins ch03` one battle.
- [ ] `grep -rn "gold, 1000" crates/ui/tests/it` finds nothing: the
      replay test reads tuned values from the data, and any exact pin
      left is under the private `content/tests/`.
- [ ] `docs/playtesting.md` and the fixtures' headers say how to record
      a win.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the table-driven replay test over every fixture, and the
  missing-fixture report.
- Unit: the shared army function (the same content and fixtures give
  the same army).

## Completion notes

*(Filled in by the session that completes the ticket.)*
