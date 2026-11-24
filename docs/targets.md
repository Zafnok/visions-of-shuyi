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

## The graphics tiers

Each profile also says its **graphics tier** (engine ticket 3347,
[ADR-0131](adr/0131-each-target-has-a-graphics-tier-and-every-moving-look-names-its-stills.md)):
how much the game moves and draws there beyond its stills. Every look
that moves by itself (a tile that loops through frames, petals drifting
over the map) names its still for each tier below the one it moves from,
and the skins draw that still there.

| Target | Tier | What it shows |
| --- | --- | --- |
| desktop | `Full` | every moving look |
| web | `Full` | every moving look |
| playdate | `Reduced` | stills, and at most a few cheap moving tiles; no particles, no effects over the screen |
| (a GBA) | `Minimal` | stills only. `TODO(gba)`: no target yet (engine ticket 3022 measures one); every moving look names its `Minimal` still now, so a port later is not a big job |

To see a lesser tier on the PC: `cargo shuyi frame-png --graphics reduced`
for a picture, or `SHUYI_GRAPHICS=reduced` before `cargo run -p trpg-app`
for a debug build of the game.

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

Measured 2026-11-18 by the `bench` job of PR #761 (the engine pin move
that brought engine ticket 3219's plain text fills, ticket 1285;
Callgrind on Linux, the heap from the tool program's counter, the web
package without the bought art). The Playdate's package isn't measured:
CI has no Playdate SDK to build it.

| Scenario | On | Frames | Instructions: most (frame) | p99 | Mean | Peak heap |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| battle-idle | desktop, web | 60 | 1,451,217 (6) | 1,451,217 | 1,442,384 | 5.0 MB |
| battle-idle | playdate | 60 | 1,806,693 (34) | 1,806,693 | 1,804,914 | 9.3 MB |
| battle-cursor-sweep | desktop, web | 106 | 1,478,510 (92) | 1,458,806 | 1,313,097 | 5.0 MB |
| battle-cursor-sweep | playdate | 106 | 1,918,808 (92) | 1,904,904 | 1,811,345 | 9.3 MB |
| battle-enemy-turn | desktop, web | 598 | 3,686,421 (63) | 3,112,977 | 1,285,564 | 5.0 MB |
| battle-enemy-turn | playdate | 598 | **4,466,386** (63) | 3,849,307 | 1,946,711 | 9.3 MB |
| battle-attack | desktop, web | 384 | 1,681,760 (83) | 1,482,045 | 1,318,427 | 5.0 MB |
| battle-attack | playdate | 384 | **2,665,720** (15) | 2,525,381 | 1,916,713 | 9.3 MB |
| options-menu | desktop, web | 110 | 531,901 (57) | 531,297 | 505,980 | 4.7 MB |
| options-menu | playdate | 110 | 1,907,849 (3) | 1,903,387 | 886,846 | 9.1 MB |
| dialogue-portrait | desktop, web | 304 | 2,174,466 (157) | 2,089,944 | 1,219,364 | 4.7 MB |
| dialogue-portrait | playdate | 304 | **2,626,898** (157) | 2,609,668 | 1,584,213 | 9.0 MB |

| Package | Size (gated) | Whole package |
| --- | ---: | ---: |
| `visions-of-shuyi-0.3.0-web.zip` | 9.2 MB | 82.0 MB |

The web's package budget counts what a player waits for before the game
starts, not the whole upload (engine ticket 3313, its
`docs/targets/web.md`). The gated size leaves out the folders `shuyi.ron`'s
`web.music` and `web.voice` ship beside the build: the game fetches each
track or clip in the background when it first plays it, and itch.io serves an uploaded HTML5 zip as separate files, so those are
never waited for (ticket 1249 checked it in a browser: the page loads the
scripts and the `.wasm`, the title song is fetched when it starts
and the opening scene's track when New Game reaches it). The whole zip,
with the 75 MB `music/` folder, is the upload's size; the "Whole package"
column keeps it in sight. Measured 2026-11-18 by PR #761's `bench` job,
with the table above (PR #681, ticket 0916, the first package `wasm-opt
-Oz` shrank, had its `.wasm` at 8.2 MB from 10.5 and the package at
8.7 MB).

### The web's start

Ticket 1298 timed CI's release web package at the engine pin of
2026-11-19 (d10d1950), served on one shared 25 Mbit/s link, in the
built-in browser, two runs:

| Moment | Time |
| --- | ---: |
| The pack (1.19 MB) all in | 0.83–0.87 s |
| The `.wasm` (8.65 MB) all in, the game's first step | 3.35–3.37 s |
| The loading card | 3.45 s |
| The title (after the 130 ms load) | 3.63 s |

The page fetches the pack and the `.wasm` together, and the game can't
step until its `.wasm` is in, so the pack has arrived about 2.5 s
before the first step: streaming it, or laying the title's folders
first (`Bake::first`), saves nothing. The load before the title reads
96% of the pack's bytes too (1,353,099 of 1,409,838 with the game's
content). The start waits for the whole pack (`trpg_ui::start`) and the
bake lays no folder first. CI's package has no game content (ADR-0117),
so its load stops at a content error at the moment the title would show.

## Over budget

The numbers in bold are over their budgets. The budgets stay; a ticket
covers each one:

- **The Playdate's one-bit drawing (1248).** Drawing a frame in one bit
  takes about 1 million instructions, half the Playdate's budget, before
  the game does anything; a portrait drawn in one bit takes 2.7 million
  more. The enemy's choice and the attack's first move tip those frames
  over.

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

Ticket 1285 brought it under again. The engine pin move of 2026-11-17
(PR #732) made every text fill parse a message (engine #476): about 80k
instructions more per Playdate frame, and the sweep went from 1,980,877
to 2,048,560, over. Engine ticket 3219 fills plain text without the
parse; past it (PR #761) the sweep is at 1,918,808, the idle battle on
the Playdate at 1,806,693 (1,882,067 before #732).

What is still dearer than before #732 isn't the plain text fill. On
the Playdate, the options menu (1,891,040 before #732, 1,907,849 now)
and the dialogue line (2,579,587, now 2,626,898) still pay #732's
message fills with values; ticket 1299 fills them from the forms. On the
colour targets, the key caps in every hint (#737, ticket 0254) and the
battle's new help bar (#742, ticket 0445) put about 0.3 million on the
idle battle and the cursor sweep, far inside their budget. The
Playdate's peak heap went from 5.7 to 9.3 MB with the chapter title
card's lettering (#746, ticket 0812), under its 10 MB budget; ticket
1300 finds what holds it.

Until they land, CI's `bench` job reports these three as warnings
(`KNOWN_OVER` in `.github/workflows/ci.yml`, read by
`scripts/bench-known-over.sh`). Any other number over its budget, or a
scenario that doesn't play, still fails the job. The job isn't part of
`ci-result`, the required check, yet; ticket 1250 makes it required and
fails on every number again.
