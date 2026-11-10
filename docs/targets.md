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

Measured 2026-11-10 by the `bench` job of PR #610 (ticket 1259;
Callgrind on Linux, the heap from the tool program's counter, the web
package without the bought art). The Playdate's package isn't measured:
CI has no Playdate SDK to build it.

| Scenario | On | Frames | Instructions: most (frame) | p99 | Mean | Peak heap |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| battle-idle | desktop, web | 60 | 1,133,122 (34) | 1,133,122 | 1,128,287 | 4.7 MB |
| battle-idle | playdate | 60 | 1,946,740 (34) | 1,946,740 | 1,942,467 | 5.5 MB |
| battle-cursor-sweep | desktop, web | 106 | 1,371,007 (92) | 1,330,645 | 1,081,078 | 4.7 MB |
| battle-cursor-sweep | playdate | 106 | **2,256,720** (92) | 2,116,791 | 1,915,796 | 5.5 MB |
| battle-enemy-turn | desktop, web | 598 | 3,666,123 (63) | 3,100,339 | 1,282,417 | 4.7 MB |
| battle-enemy-turn | playdate | 598 | **4,513,644** (63) | 3,940,159 | 1,986,993 | 5.5 MB |
| battle-attack | desktop, web | 384 | 2,305,299 (43) | 2,015,390 | 1,342,111 | 4.7 MB |
| battle-attack | playdate | 384 | **2,703,238** (15) | 2,560,398 | 1,983,333 | 5.5 MB |
| options-menu | desktop, web | 110 | 523,758 (3) | 514,416 | 453,909 | 4.4 MB |
| options-menu | playdate | 110 | 1,890,142 (3) | 1,886,190 | 866,602 | 5.3 MB |
| dialogue-portrait | desktop, web | 304 | 2,093,842 (157) | 1,989,268 | 1,154,638 | 4.4 MB |
| dialogue-portrait | playdate | 304 | **2,578,598** (157) | 2,559,619 | 1,557,592 | 5.3 MB |

| Package | Size |
| --- | ---: |
| `visions-of-shuyi-0.3.0-web.zip` | **81.8 MB** |

## Over budget

The numbers in bold are over their budgets. The budgets stay; a ticket
covers each one:

- **The unit menu's frame (1261).** `battle-attack`'s costliest frame
  on the colour targets is frame 43, where the unit menu opens (2.31
  million instructions; every frame with it open, about 2.01 million).
  Most of it is the engine's `GlyphBuffer::fill_rect`, which rebuilds
  every item drawn so far for each rectangle it fills, also the items
  the rectangle doesn't meet; the menu fills one per row. On the colour
  targets it is under their budget; on the Playdate the menu's frames
  are over it with the one-bit drawing below. 1259 kept each enemy's
  threat area for the battle, so the cursor reaching a unit no longer
  searches every enemy's moves (14 searches a unit before): the cursor
  sweep's costliest frame went from 2.28 to 1.37 million on the colour
  targets and from 3.05 to 2.26 million on the Playdate, and
  `battle-attack`'s first move from 3.05 to 2.12 million there.
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
