# The game's targets and their budgets

The game ships on the targets `shuyi.ron`'s `targets` lists: **desktop**
(Windows, Linux, macOS), **web** and **playdate**. Each target's profile
holds its budgets: the most instructions one frame may take, the most heap
in use at once, and the biggest package. The desktop's and the web's
profiles are the engine's (`docs/targets/` in Shuyi Engine); the
Playdate's is the game's, `targets/playdate.ron`, with the engine's
numbers. Every budget is provisional until its device is measured (engine
tickets 3011 and 3013), and only a device measurement changes one: a
scenario over its budget gets a ticket for what is slow, never a higher
budget (ticket 1244).

| Target | Frame instructions | Peak heap | Package |
| --- | ---: | ---: | ---: |
| desktop | 20,000,000 | 512 MB | (not gated) |
| web | 20,000,000 | 256 MB | 25 MB |
| playdate | 2,000,000 | 10 MB | 40 MB |

## The scenarios

`cargo shuyi bench` (engine ADR-0081) plays the scenarios in `bench/`
headless, counts each frame's instructions under Valgrind's Callgrind on
Linux, reads the heap's peak from the tool program's heap counter, and
compares each number with the budgets of each target. It plays each
scenario once for the colour targets (desktop and web share it) and once
on the Playdate, on its 50 × 15 console with its buttons and each frame
drawn in one bit as the Playdate's build draws it (`session_on` in
`crates/xtask/src/game_tools.rs`).

| Scenario | What it plays |
| --- | --- |
| `battle-idle` | the biggest battle (`side_company_ledger`, 22 units), open with no input |
| `battle-cursor-sweep` | the cursor swept round that battle's map |
| `battle-enemy-turn` | that battle's first enemy turn, five of fifteen enemies moving (its first 9.9 s) |
| `battle-attack` | a unit sent beside an enemy, the forecast, the attack and its animation |
| `options-menu` | the fullest menu, Options (18 rows), opened from the title and walked |
| `dialogue-portrait` | Chapter 1's opening scene to a line beside the lead's portrait, typing out |

The battles open with the tools' `--battle <id>` (the battle at its start,
fought by its own characters) and the scene with `--scene <id>`. The steps
press the game's actions by name, so they hold for any key layout and on
the Playdate's buttons. `the_bench_scenarios_play_what_they_say` (xtask)
plays each one and checks it ends where it says.

The build measured has no bought art (`private-assets`), as no gate reads
it: the companions' portraits and the battle sprites a build with the art
shows aren't in these numbers.

## Running it

```bash
cargo shuyi bench --no-instructions   # on Windows: every scenario plays, the heap and the packages
cargo shuyi bench --scenario battle-attack --no-instructions
```

Instructions need Valgrind, on Linux (or WSL with Rust and Valgrind
installed). CI's `bench` job runs every scenario on every pull request,
with the web package its `export` job made, posts the table as the job's
summary and fails on a number over its budget, naming the scenario, the
frame and the target.

## The numbers

Measured 2026-11-06 by the `bench` job of PR #558 (Callgrind on Linux,
the heap from the tool program's counter, the web package without the
bought art). The Playdate's package isn't measured: CI has no Playdate
SDK to build it. `battle-attack`'s rows are from PR #568's run
(2026-11-07, ticket 1247); the others are still PR #558's.

| Scenario | On | Frames | Instructions: most (frame) | p99 | Mean | Peak heap |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| battle-idle | desktop, web | 60 | 1,121,961 (34) | 1,121,961 | 1,116,948 | 4.5 MB |
| battle-idle | playdate | 60 | 1,890,788 (1) | 1,890,788 | 1,879,100 | 5.4 MB |
| battle-cursor-sweep | desktop, web | 106 | 2,236,860 (91) | 1,999,255 | 1,097,865 | 4.5 MB |
| battle-cursor-sweep | playdate | 106 | **3,111,370** (91) | 2,843,669 | 1,905,657 | 5.4 MB |
| battle-enemy-turn | desktop, web | 598 | 3,698,190 (63) | 3,125,540 | 1,272,784 | 4.5 MB |
| battle-enemy-turn | playdate | 598 | **4,532,390** (63) | 3,949,376 | 1,967,707 | 5.4 MB |
| battle-attack | desktop, web | 384 | 2,695,971 (11) | 2,257,062 | 1,340,394 | 4.5 MB |
| battle-attack | playdate | 384 | **3,406,850** (13) | 3,226,561 | 1,979,822 | 5.4 MB |
| options-menu | desktop, web | 110 | 532,148 (3) | 506,622 | 439,420 | 4.3 MB |
| options-menu | playdate | 110 | 1,883,148 (3) | 1,879,405 | 838,930 | 5.2 MB |
| dialogue-portrait | desktop, web | 304 | 2,084,920 (157) | 1,967,878 | 1,145,114 | 4.3 MB |
| dialogue-portrait | playdate | 304 | **4,742,652** (157) | 4,542,516 | 2,436,731 | 5.2 MB |

| Package | Size |
| --- | ---: |
| `visions-of-shuyi-0.3.0-web.zip` | **76.3 MB** |

## Over budget

The numbers in bold are over their budgets. The budgets stay; a ticket
covers each one:

- **The threat arrows (1251).** While a unit is selected, each cursor
  move searches every enemy's moves again (14 searches in
  `battle-attack`'s battle): `battle-attack`'s frames 11 and 13 and,
  very likely, `battle-cursor-sweep`'s frame 91. 1247 already took the
  unit menu's frame (43) from 17 million instructions to under the
  p99: one path search, not 298.
- **The Playdate's one-bit drawing (1248).** Drawing a frame in one bit
  takes about 1 million instructions, half the Playdate's budget, before
  the game does anything; a portrait drawn in one bit takes 2.7 million
  more. The cursor sweep's reach and the enemy's choice tip those frames
  over.
- **The web package (1249).** The 75 MB `music/` folder ships in it.

Until they land, CI's `bench` job reports these five as warnings
(`KNOWN_OVER` in `.github/workflows/ci.yml`, read by
`scripts/bench-known-over.sh`). Any other number over its budget, or a
scenario that doesn't play, still fails the job. The job isn't part of
`ci-result`, the required check, yet; ticket 1250 makes it required and
fails on every number again.
