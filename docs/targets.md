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

Measured 2026-11-11 by the `bench` job of PR #624 (the engine pin move
that brought ticket 1261's fix;
Callgrind on Linux, the heap from the tool program's counter, the web
package without the bought art). The Playdate's package isn't measured:
CI has no Playdate SDK to build it.

| Scenario | On | Frames | Instructions: most (frame) | p99 | Mean | Peak heap |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| battle-idle | desktop, web | 60 | 1,055,913 (34) | 1,055,913 | 1,050,422 | 4.8 MB |
| battle-idle | playdate | 60 | 1,872,097 (6) | 1,872,097 | 1,869,943 | 5.6 MB |
| battle-cursor-sweep | desktop, web | 106 | 1,095,036 (92) | 1,063,167 | 1,018,441 | 4.8 MB |
| battle-cursor-sweep | playdate | 106 | 1,958,173 (92) | 1,944,953 | 1,871,779 | 5.6 MB |
| battle-enemy-turn | desktop, web | 598 | 3,598,048 (63) | 3,020,731 | 1,221,469 | 4.8 MB |
| battle-enemy-turn | playdate | 598 | **4,477,234** (63) | 3,864,653 | 1,960,896 | 5.6 MB |
| battle-attack | desktop, web | 384 | 1,561,368 (83) | 1,367,121 | 1,204,249 | 4.8 MB |
| battle-attack | playdate | 384 | **2,678,597** (15) | 2,521,734 | 1,941,084 | 5.6 MB |
| options-menu | desktop, web | 110 | 482,871 (57) | 481,798 | 455,636 | 4.6 MB |
| options-menu | playdate | 110 | 1,886,127 (3) | 1,881,493 | 867,153 | 5.3 MB |
| dialogue-portrait | desktop, web | 304 | 2,089,109 (157) | 1,982,354 | 1,152,168 | 4.6 MB |
| dialogue-portrait | playdate | 304 | **2,581,737** (157) | 2,561,958 | 1,560,167 | 5.3 MB |

| Package | Size |
| --- | ---: |
| `visions-of-shuyi-0.3.0-web.zip` | **81.8 MB** |

## Over budget

The numbers in bold are over their budgets. The budgets stay; a ticket
covers each one:

- **The Playdate's one-bit drawing (1248).** Drawing a frame in one bit
  takes about 1 million instructions, half the Playdate's budget, before
  the game does anything; a portrait drawn in one bit takes 2.7 million
  more. The enemy's choice and the attack's first move tip those frames
  over.
- **The web package (1249).** The 75 MB `music/` folder ships in it.

Ticket 1261 brought the Playdate's cursor sweep under. The engine's `GlyphBuffer::fill_rect`
had rebuilt every item drawn so far for each rectangle it filled, also
the items the rectangle doesn't meet; the unit menu fills one per row.
It now keeps the items it misses as they were (engine #353): the frame
the unit menu opens in (`battle-attack`'s frame 43) went from 2.31 to
1.51 million instructions on the colour targets and from 2.47 to 2.38
million on the Playdate, every frame with the menu open from about 2.01
to 1.22 million (2.15 to 2.06 on the Playdate), and the Playdate's
cursor sweep from 2.14 to 1.96 million, under its budget. Earlier, 1259
kept each enemy's threat area for the battle, so the cursor reaching a
unit no longer searches every enemy's moves: the cursor sweep's costliest
frame went from 2.28 to 1.37 million on the colour targets and from 3.05
to 2.26 million on the Playdate.

Until they land, CI's `bench` job reports these four as warnings
(`KNOWN_OVER` in `.github/workflows/ci.yml`, read by
`scripts/bench-known-over.sh`). Any other number over its budget, or a
scenario that doesn't play, still fails the job. The job isn't part of
`ci-result`, the required check, yet; ticket 1250 makes it required and
fails on every number again.
