---
id: "0531"
title: "The bots' armies don't hang on one recorded win"
type: feature
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0531 — The bots' armies don't hang on one recorded win

## Context

The bots start every story chapter after the first with the army the
recorded winning playthroughs of the chapters before it leave
(`docs/playtesting.md`, *The army*; `crates/bots/src/arrival.rs`). A
recording is one search's best line (`crates/ui/tests/it/playthrough.rs`,
`record`): it knows the battle's luck and feeds the EXP to whoever it
happened to use. Any change that alters one roll of an earlier chapter
breaks the recording; the new one spreads the EXP differently, and every
later chapter's bands move with it, though the chapters didn't change.

0864 showed how far: Chapter 1 recorded again **with no change at all**
moved Chapter 2 from 45 / 63 / 55 (Casual / Normal / Hardcore, all in
band) to 14 / 37 / 46 (blocked). The old recording had fed Tamsin to
level 8 with the lead at 1; the new one ended Tamsin 5 and the lead 5.
After 0864's change, Chapters 4 and 5 dropped out of band the same way
(0876). Every tuning ticket since 0524 has paid for this.

2026-11-17 friction sweep (B4,
[report](../../docs/research/friction-2026-11-17.md), findings "Each
chapter battle needs a hand-chained test module" and "Public tests pin
private content values"): each recorded win is its own hand-written module,
`crates/ui/tests/it/ch01_winnable.rs` to `ch05_winnable.rs` (909 lines,
128–271 each) plus `skirmishes.rs` (253) and `side_quests.rs` (340), and
`chapter_N()` calls `chapter_{N-1}()`, so a new battle edits the one before
it too (#588 edited `ch04_winnable.rs`). `PLAYTHROUGHS` in
`crates/bots/src/arrival.rs:23` is a second hand-kept list of the same
chain. There are 11 `*_winning.ron` fixtures and 11 `#[ignore]` `record`
tests, run one at a time; #731 (one stat change) re-recorded 9 of them.
The public replay tests also pin tuned values
(`ch01_winnable.rs:103`, `campaign.gold == 1000`), so a balance change
in the private repository edits a public test (about 1 in 4 pin PRs since
#676; only 4 days of evidence). About 6 battles are left on the Act 1
path (0852–0854, 1090, 1083, 1084), each about 200–350 test lines and 8
small edits today. Sibling ticket in this bucket: 1290.

## Nick input

None while the army stays what `docs/playtesting.md` says it is (the
army a player brings). If the fix changes that meaning (for example an
"average player's" army rather than one played line), ask with
`ask-nick` first, since `docs/design/playtest-bots.md` is Nick's.

## Scope

**In:**
- Make the arrival army stable against a new recording: the agent's
  choice, recorded as an ADR. Candidates: build the army from several
  recordings (different seeds) and judge each chapter over all of them;
  or keep the recording for the replay tests only and give the bots a
  reference army per chapter (as the side quests' `bots_army` does),
  levelled by the EXP the chapters before it pay; or a recorder that
  spreads the EXP the way the personas do.
- The bots crate stops reading the UI's test fixtures:
  `crates/bots/src/arrival.rs` (lines 26–50) `include_str!`s the seven
  `crates/ui/tests/it/fixtures/*_winning.ron`. Whatever the army is built
  from becomes the bots' own data (for example under `crates/bots/`).
  The UI's replay tests keep their fixtures, and re-recording one of
  them no longer moves an army by itself.
- Show it on 0864's case: Chapter 1 recorded with two different seeds
  gives Chapter 2 reports within the noise of each other. Measure on
  seeds 1–400: at 100 tries a rate carries about ±10 points of noise,
  at 400 about ±5 (retrospective T2,
  `docs/research/retro-2026-11-17-prs-616-735.md`).
- A tolerance check that keeps it so, cheap enough for CI (the form is
  the agent's, after the fix): if the army no longer reads any
  recording, a test that it is built from the bots' data alone; if it
  still reads recordings, a test that two recordings of a chapter with
  different seeds give every later chapter's army within a set margin
  (levels, or the bots' rates on a short run), with the margin stated
  in the ADR.
- `docs/playtesting.md`, *The army*.
- (2026-11-17 sweep) **One table-driven recorded-win test.** Replace the
  hand-chained `chNN_winnable.rs`, `skirmishes.rs` and `side_quests.rs`
  replay tests with one test that walks every battle with a
  `fixtures/<battle>_winning.ron` in story order (the chapters, then the
  world map's guards and side quests) and replays each by id. What a win
  must leave (joins, items, houses, nobody lost) goes in the fixture's
  header or is read from the data, not a module per battle. The army a
  battle starts with is built by one shared function, used by this test
  and by the bots.
- (2026-11-17 sweep) **Bots find fixtures by id**, or read this ticket's
  new army data, instead of the hand-kept `PLAYTHROUGHS` array
  (`arrival.rs:23`). Adding a battle is its files plus one recorded
  fixture: no new module, no list to extend.
- (2026-11-17 sweep) **`cargo xtask record-wins [<battle>|--all]`**: runs
  the `record` step for one battle or for all of them in story order, so a
  re-record after a balance change is one command.
- (2026-11-17 sweep) **Relations, not tuned numbers, in public tests.** The
  replay test reads what it expects from the loaded data (for example the
  campaign's gold equals the battle's `clear_gold`, the fallen equal the
  enemies at the start). Any test that must pin an exact tuned value moves
  to the private repository's `content/tests/` (ADR-0118 §3) with the
  `content-change` skill. Mechanism tests on `assets-test/` keep their
  literals.
- (2026-11-17 sweep) Size: this ticket is now likely past 2,000 changed
  lines (deletions count). Measure with `cargo xtask pr-size` before
  coding; if it won't fit, split the table-driven test and
  `record-wins` into a ticket of their own with `write-ticket` first.

**Out (do not do):**
- Retuning any chapter (0876 and later tuning tickets; 0876 waits for
  this one).
- The bands, the success rules, the personas' play.
- The report's 95% intervals (0532). Either order: if 0532 is done, read
  "within the noise" from its intervals; if not, use seeds 1–400.
- Reaching a battle through one Harness call (1289). **Either order**;
  if 1289 is done and its helper fits, the table test uses it.
- Moving public value tests outside the recorded wins (for example
  `crates/content/tests/it/skills.rs`); only the replay tests' pins are
  this ticket's.

## Implementation steps

1. Read `crates/bots/src/arrival.rs`, `crates/tools/src/playtest.rs`
   (`stage`), `crates/ui/tests/it/playthrough.rs` and 0864's Completion
   notes.
2. Measure the spread first: record Chapter 1 with three seeds (the
   `seed` argument of `playthrough::record`, 803 today) and play Chapter
   2 with each army.
3. Pick the fix, write the ADR, build it, and measure the spread again.
4. Re-run every story battle's playtest; report the numbers before and
   after in the PR.
5. (2026-11-17 sweep) Write the table-driven replay test and the shared
   army builder, switch the bots to it, delete the per-battle modules and
   `PLAYTHROUGHS`, add `record-wins`, and turn tuned literals into
   relations (or move them to `content/tests/`). Run
   `cargo xtask record-wins --all` once and check that every fixture
   replays.

## Acceptance criteria

- [ ] Chapter 2's three success rates move by no more than about 5
      points between armies built from Chapter 1 recordings with
      different seeds, on seeds 1–400 (numbers in the PR).
- [ ] No `include_str!` (or any other read) in `crates/bots` names a
      path under `crates/ui/tests`.
- [ ] The tolerance check in Scope runs in CI and passes.
- [ ] An ADR records the choice; `docs/playtesting.md` says how the army
      is built.
- [ ] One test replays every `*_winning.ron` fixture by id;
      `ls crates/ui/tests/it/ch0*_winnable.rs` finds nothing, and
      `grep -n PLAYTHROUGHS crates/bots/src` finds nothing.
- [ ] `cargo xtask record-wins --all` re-records every fixture in story
      order (and a single battle by id).
- [ ] `grep -n "gold, 1000" crates/ui/tests/it` finds nothing: the
      replay test reads tuned values from the data, and any exact pin
      left is under the private `content/tests/`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the army builder (the same content gives the same army; two
  recordings give one army, or the reports say which army each try used).
- Unit or integration: the tolerance check.
- Integration: the table-driven replay test over every fixture, and a
  test that a battle with no fixture is reported by id, not skipped
  silently.
- The bots' playtest runs (reported in the PR).

## Completion notes

*(Filled in by the session that completes the ticket.)*
