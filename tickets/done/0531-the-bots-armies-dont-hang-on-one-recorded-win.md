---
id: "0531"
title: "The bots' armies don't hang on one recorded win"
type: feature
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: []
nick_input: none
completed: 2026-11-22
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
- (2026-11-17 sweep) **Bots find fixtures by id**, or read this ticket's
  new army data, instead of the hand-kept `PLAYTHROUGHS` array
  (`arrival.rs:23`). Adding a battle is its files plus one recorded
  fixture: no new module, no list to extend.
- (Split to **0535** before coding, 2026-11-21, ADR-0101: with them this
  ticket was past 2,500 changed lines.) The sweep's other items: one
  table-driven recorded-win test in place of the per-battle replay
  modules, `cargo xtask record-wins`, and relations in place of tuned
  numbers in the public replay tests.

**Out (do not do):**
- Retuning any chapter (0876 and later tuning tickets; 0876 waits for
  this one).
- The bands, the success rules, the personas' play.
- The report's 95% intervals (0532). Either order: if 0532 is done, read
  "within the noise" from its intervals; if not, use seeds 1–400.
- The table-driven replay test, `record-wins` and the replay tests'
  tuned pins (0535, blocked by this ticket).

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
5. (2026-11-17 sweep) Switch the bots to their own data, found by id,
   and delete `PLAYTHROUGHS`. The rest of the sweep's step is 0535's.

## Acceptance criteria

- [ ] Chapter 2's three success rates move by no more than about 5
      points between armies built from Chapter 1 recordings with
      different seeds, on seeds 1–400 (numbers in the PR). **Partly:**
      Casual 1 and Normal 3.75 points; Hardcore 9.25 on seeds 1–400,
      0.75 on 401–800 and 4.25 over all 800 (Completion notes).
- [x] No `include_str!` (or any other read) in `crates/bots` names a
      path under `crates/ui/tests`.
- [x] The tolerance check in Scope runs in CI and passes.
- [x] An ADR records the choice; `docs/playtesting.md` says how the army
      is built.
- [x] `grep -n PLAYTHROUGHS crates/bots/src` finds nothing.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the army builder (the same content gives the same army; two
  recordings give one army, or the reports say which army each try used).
- Unit or integration: the tolerance check.
- The bots' playtest runs (reported in the PR).

## Completion notes

**Nick chose the fix, twice** (the ticket's *Nick input*: an army built
any other way than from played lines changes its meaning). 2026-11-21,
after the first measure: **"Several playthroughs"** over "Even share",
"Lead gets more" and "Keep one recording". 2026-11-22, after 8 lines
still left Chapter 2's Normal 14 points apart between two sets: **"24"**
over 16 and keeping 8. ADR-0130 records both; it amends ADR-0110.

**What the bots do now** (`docs/playtesting.md`, *The army*):

- The bots keep their own recorded wins, `crates/bots/wins/<battle>.ron`,
  24 **lines** per battle a later army plays (ch01, ch02, ch03,
  aldwater, ch04, salt_road). A line is one player's way through the
  story: its win of a battle is recorded from the army its own wins
  before leave. `trpg-bots`' build script compiles them in, found by
  battle id (no `PLAYTHROUGHS` list; nothing read from
  `crates/ui/tests`). Marked generated in `.gitattributes`.
- `trpg_bots::arrival` returns one army per line; the playtest's stage
  holds them all and the try with seed N fields line (N − 1) mod 24 + 1,
  so `--seed 17 --runs 1` and the `replay` of a kept play field the same
  army again. The report shows a row per line and each unit's mean
  level.
- The recorder's search moved from `trpg-ui`'s tests to
  `trpg_bots::record`, unchanged. `cargo xtask bot-wins --all` records
  every line in story order, a thread per line (about 15–30 minutes on 24
  cores: Chapter 4's search sometimes runs past turn 60, and is then tried
  again with its seed + 1,000); `cargo xtask bot-wins <battle>` records it
  and every battle after it.
- Tolerance check in CI: `arrival::tests::the_lines_halves_bring_armies_alike`,
  the odd lines against the even ones, every later battle: each unit's
  mean level within 2 levels, the army's within 0.75 (measured widest:
  1.25 and 0.43).
- `.github/playtest-levers.sh` and `playtest-inputs.sh` no longer count
  the UI's fixtures; the bots' wins are under `crates/bots/`, a lever.

**Measured, Chapter 2, seeds 1–400** (Casual / Normal / Hardcore):

| Army | Chapter 2 |
| --- | --- |
| `main`: one recording, Chapter 1 seed 803 | 40.5 / 54 / 62 |
| one recording, seed 1 | 41 / 67 / 81 |
| one recording, seed 2 | 46.5 / 69 / 75 |
| 8 lines, seeds 1–8 | 45 / 59 / 73 |
| 8 lines, seeds 9–16 | 45 / 45 / 80 |
| **24 lines, seeds 1–24 (shipped)** | **47 / 55 / 79** |
| 24 lines, seeds 25–48 | 46 / 58.5 / 70 |

The two 24-line sets are 1, 3.75 and 9.25 points apart. Hardcore again
on seeds 401–800: 72.25 and 73 (0.75 apart); over all 800 seeds 75.6
and 71.4. So the first criterion is met for Casual and Normal on seeds
1–400 and for Hardcore only over 800 seeds: two 400-try runs of the
same army already differ by about 3.5 points (sd) from their own luck,
so "no more than about 5" can't hold every time for any army. From the
16 lines of the 8-line sets, a line's own Normal rate varies by about
11 points; the mean of 24 by about 2.

**Every later story battle, after** (24 lines, seeds 1–200), against
**before** (`main` after 0864, seeds 1–100, 0876's table):

| Battle | Before | After | Bands |
| --- | --- | --- | --- |
| ch02 | 40.5 / 54 / 62 (1–400) | 47 / 55 / 79 ⚠ (1–400) | 30–55 / 50–75 / 50–75 |
| ch03 | 68 ⚠ / 70 / 42 ✗ | 67 ⚠ / 78.5 ⚠ / 57 | 30–55 / 50–75 / 50–75 |
| aldwater | 100 / 50 ✗ / 80 | 100 / 85.5 / 82 | 70–100 / 70–95 / 70–95 |
| ch04 | 50 / 32 ✗ / 31 ✗ | 64 ⚠ / 47.5 ✗ / 52 | 30–55 / 50–75 / 50–75 |
| salt_road | 100 / 94 / 96 ⚠ | 100 / 86.5 / 94 | 70–100 / 70–95 / 70–95 |
| ch05 | 51 / 15 ✗ / 11 ✗ | 59 / 32 ✗ / 22.5 ✗ | 40–70 / 50–75 / 50–75 |

None is blocked. Bringing them back into their bands is 0876's (it
waits for this ticket, 0527 and 0528); it now tunes against an army
that holds.

**Split before coding** (ADR-0101): the 2026-11-17 sweep's table-driven
replay test, `cargo xtask record-wins` and the replay tests' tuned pins
went to **0535** (blocked by this one). `trpg-ui` may not depend on
`trpg-bots`, so `crates/ui/tests/it/playthrough.rs` keeps its copy of
the search until 0535.

- **The agent's starting rules:** none for the game. For the bots: 24
  lines and the seed-to-line rule above; a failed search retried twice.
- Engine friction: estimate 550 production lines, measured 749 (1.4×,
  under 2×); signals: workaround no · third copy no · budget no.
- PR size: estimate about 2,000 after the split, measured 2,101 before
  these notes (the 6,400 lines of recorded wins are generated).
