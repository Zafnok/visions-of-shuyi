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

Measured 2026-11-05 (the heap on Windows; the instructions and the web
package in CI). The Playdate's package isn't measured in CI, which has no
Playdate SDK to build it.

| Scenario | On | Frames | Instructions: most | Mean | Peak heap |
| --- | --- | ---: | ---: | ---: | ---: |
| battle-idle | desktop, web | 60 | | | 4.5 MB |
| battle-idle | playdate | 60 | | | 5.4 MB |
| battle-cursor-sweep | desktop, web | 106 | | | 4.5 MB |
| battle-cursor-sweep | playdate | 106 | | | 5.4 MB |
| battle-enemy-turn | desktop, web | 598 | | | 4.5 MB |
| battle-enemy-turn | playdate | 598 | | | 5.4 MB |
| battle-attack | desktop, web | 384 | | | 4.5 MB |
| battle-attack | playdate | 384 | | | 5.4 MB |
| options-menu | desktop, web | 110 | | | 4.3 MB |
| options-menu | playdate | 110 | | | 5.2 MB |
| dialogue-portrait | desktop, web | 304 | | | 4.3 MB |
| dialogue-portrait | playdate | 304 | | | 5.2 MB |
