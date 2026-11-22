# ADR-0130: The bots field 24 lines of their own recorded wins

- **Status:** Accepted
- **Date:** 2026-11-22
- **Related tickets:** 0531 (this decision), 0520 and 1071 (the arrival
  army), 0864 (found how far it moved), 0535 (the UI's replay tests)
- **Amends:** ADR-0110 (the arrival army: *Decision*, its first
  paragraph, and *Consequences*)

## Context

ADR-0110 builds the army a bot brings to a later chapter by replaying the
earlier chapters' recorded winning playthroughs, `include_str!`'d from
`trpg-ui`'s replay fixtures. A recording is one search's best line: it
knows the battle's luck and gives the EXP to whoever it happened to use.
So any change that breaks one recording moves every later chapter's
bands, though the chapters didn't change. 0864 recorded Chapter 1 again
with no change at all and Chapter 2 went from 45 / 63 / 55 to 14 / 37 /
46 (Casual / Normal / Hardcore); four tuning tickets have paid for it.

Measured for 0531 on seeds 1–400, Chapter 1 recorded with three seeds and
nothing else changed:

| Chapter 1 recording | Chapter 2's army | Chapter 2 |
| --- | --- | --- |
| seed 803 | Lead 5, Hollis 4, Tamsin 6, Aske 3, Maud 2, Rue 3 | 40.5 / 54 / 62 |
| seed 1 | Lead 5, Hollis 3, Tamsin 4, Aske 4, Maud 1, Rue 3 | 41 / 67 / 81 |
| seed 2 | Lead 4, Hollis 3, Tamsin 3, Aske 2, Maud 2, Rue 5 | 46.5 / 69 / 75 |

The army with the most EXP in all (seed 803's) is the hardest: who got
the EXP moves the numbers, not how much.

Three fixes were open (ticket 0531): several recordings, judged over all
of them; a reference army per chapter, the EXP shared out by a rule; or a
recorder that spreads the EXP. The last two change what the army means
(one played line to an average player's), so Nick chose, on 2026-11-21,
between "Several playthroughs", "Even share", "Lead gets more" and "Keep
one recording": **"Several playthroughs"**, about 8 recorded wins per
chapter, each try using one of them. Two sets of 8 lines (seeds 1–8 and
9–16) then gave Chapter 2 45 / 59 / 73 and 45 / 45 / 80 on seeds 1–400:
Normal 14 points apart. Over those 16 lines, a line's own Normal rate
varies by about 11 points (sd), so the mean of 8 still carries about 4;
two 400-try runs differ by about 3.5 from their own luck alone. Nick
chose, on 2026-11-22, between 24, 16 and keeping 8: **"24"** (the
recording takes the same time on this machine's 24 cores).

## Decision

1. **Lines.** The bots keep their own recorded wins,
   `crates/bots/wins/<battle>.ron`, for every battle a later battle's army
   plays (each chapter's guards on the way, then its story battle; not the
   last chapter's). Each file holds `LINES`
   = 24 wins, one per **line**: a line is one player's way through the
   story, its win of a battle recorded from the army its own wins of the
   battles before leave. The army of a later battle is built as ADR-0110
   says, once per line (`trpg_bots::arrival` returns one setup per line,
   line 1's first).
2. **A try fields one line.** `BattleStage` holds every line's army;
   the try with seed `N` fields line `(N − 1) mod 24 + 1`
   (`trpg_bots::line_of`), so 400 tries play each line 16 or 17 times, the
   report judges the chapter over all of them, and `--seed 17 --runs 1`
   (or the `replay` of a kept play) fields try 17's army again. The report
   shows a row per line and each unit's mean level.
3. **Found by id.** `trpg-bots`' build script compiles in every
   `wins/*.ron` as `(battle, text)`: a new battle needs its file, no list.
   The bots read no file under `crates/ui/tests`: the UI's replay fixtures
   stay the replay tests', and re-recording one moves no army.
4. **Recorded by one command.** The search moved from `trpg-ui`'s tests to
   `trpg_bots::record` (unchanged: the best of 300 tries at each Player
   phase). `cargo xtask bot-wins --all` records every line in story order,
   a thread per line (about 15 minutes on 24 cores); `cargo xtask bot-wins <battle>`
   records that battle and every one after it on the way. Line `i`'s
   search is seeded `i` (`--seed` moves them all); a search that finds no
   win is tried again with the seed plus 1,000, then 2,000, and the file
   says which seed won. The files are generated (`.gitattributes`): never
   edited by hand.
5. **Tolerance check.** `trpg_bots`' test
   `the_lines_halves_bring_armies_alike` runs in CI: the odd lines and the
   even ones (different seeds) bring every later battle armies whose
   units' mean levels are within **2 levels** of each other, and whose
   whole army's mean level (every unit's, averaged) is within **0.75 of
   a level**. The first recording's widest gaps were 1.25 levels (the
   poacher, Chapter 4) and 0.43 (Chapter 5's army); with 8 lines they had
   been 2.25 (the lead) and 0.29. The margins leave room for a re-record.
   A set that hangs on one recording again fails it: 0864's two Chapter 1
   recordings left the lead at levels 1 and 5. It replays every line of
   every battle, about 20 s in a debug build (on nextest's slow list).

## Consequences

- **Measured** (Chapter 2, seeds 1–400, Casual / Normal / Hardcore): 24
  lines seeded 1–24 give 47 / 55 / 79, 24 seeded 25–48 give 46 / 58.5 /
  70: 1, 3.75 and 9.25 points apart. Hardcore on seeds 401–800: 72.25 and
  73. One recording each had moved them by up to 15 and 19 points.
- **Stable enough to tune.** A re-record changes the mean of 24 lines,
  not one army: its noise falls by about √24, to about 2 points on Normal
  (the worst). It doesn't vanish: two sets of 24 lines differ by about
  4.5 points (sd) on a 400-try run, most of it the run's own luck.
- **24 times the recording.** A balance change that breaks an earlier
  battle's wins means `cargo xtask bot-wins <battle>` (minutes, not one
  `#[ignore]` test), and every later battle's lines are recorded again
  with it. The playtest still stops with an error naming the battle and
  the line when a win no longer wins.
- **Bigger data, folded.** About 6,400 lines of commands (six files), marked
  generated so `pr-size` and GitHub's diffs leave them out.
- **Two copies of the search for now.** `trpg-ui` may not depend on
  `trpg-bots` (`xtask` checks it), so `crates/ui/tests/it/playthrough.rs`
  keeps its copy for the UI's 11 `record` tests until 0535 records those
  from the tool program too.
- **A playtest lever.** `crates/bots/wins/` is under `crates/bots/`,
  already a lever of `.github/playtest-levers.sh`; the UI's fixtures no
  longer are.

## Alternatives considered

- **Even share** (a reference army, the EXP the recorded wins pay pooled
  and shared evenly, levels by average growths): no drift at all, but an
  average player's army; Nick chose played lines.
- **Lead gets more**: the same with a made-up weight for the lead.
- **Keep one recording**: the bands keep moving with every re-record.
- **Lines read from disk at run time**: every caller of `stage` would load
  them; compiling them in keeps `trpg-bots` pure and the arrival's callers
  unchanged.
