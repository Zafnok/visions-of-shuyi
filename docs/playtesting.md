# Automated playtesting

Bots play a battle many times and a report says how it went: how often they
won, who fell and when, how many items they used, how many turns it took,
and whether each player type succeeded about as often as it should. What
the numbers should be (the bands per player type) is Nick's decision in
[`design/playtest-bots.md`](design/playtest-bots.md); how the bots are built
is [ADR-0033](adr/0033-playtest-bots.md).

## What a bot is

A bot is a player made of code. It plays the Player phase with the same
commands a person gives (move, attack, use an item, end the turn); the enemy
plays its own phases exactly as in the game. A bot:

- **never rewinds** and never restarts: one try is one play from the first
  turn;
- **can't see the dice**: it can't know whether an attack will hit before
  making it;
- **can't see reinforcements** before they arrive, like someone playing
  the map for the first time;
- knows nothing about how the game looks, so a change of graphics can't
  change a result.

A **try** is one play of a battle with fresh luck. Each try has a **seed**,
a number that fixes its luck: the same battle, bot and seed always play out
the same way, so any try in a report can be played again.

Bots:

| Bot | Plays in | How it plays | A try is a success when… |
| --- | -------- | ------------ | ------------------------ |
| `casual` | Casual mode | Cares about winning and the lord. A unit falling doesn't upset it, but its units are tools it needs to win. Plans little and makes mistakes: a weaker player than Normal. | it wins |
| `normal` | Classic | Wants nobody dead, spends items when that helps, doesn't hurry. Plans further, makes few mistakes. | it wins, nobody dies, and it used at most 3 items |
| `hardcore` | Classic | Nobody dead, few turns, few items. Plans the furthest and never strays: it stands in for veterans. | it wins, nobody dies, and it used at most 1 item |
| `baseline` | `--mode` | The enemy AI playing the player's side: charges, attacks the best target, never uses items. A yardstick, with no target. | — |

The army a bot fields is the characters of the battle's player slots, as
they are in the character data (the same army the debug Quick Battle uses).

### How the personas think

Casual, Normal and Hardcore are the same bot with different settings
(`crates/bots/personas.ron`): the engine's turn planner. Before each
command it:

1. **talks** first, if two units can talk that haven't yet (talking costs
   nothing);
2. **plans its turn**: short sequences of its own commands, each step from
   a list of **candidates**: each unit's best attack on each target; what
   the enemy AI would do in its place; healing a wounded unit; moving each
   unit as far towards the enemy (the boss, the tile to seize) as it can
   walk; moving each unit to the nearest tile out of every enemy's reach;
   the other ways of attacking; and ending the turn last;
3. plays each partial plan on a few **copies** of the battle with fresh
   luck and no hidden reinforcements, and **scores** where each copy got
   to, by what the persona cares about. It never plays the enemy's turn
   out: like a player, it sees the danger zone and the forecasts (which
   enemies can reach which unit, and how hard they would hit and be hit
   back), not whom the enemy will choose (Nick, 2026-10-21);
4. keeps the best few partial plans at each step, extends them, and plays
   the first command of the best plan (or, with a temperature, sometimes
   a little worse one); then it plans again for its next command.

**Skill** is the thinking budget and the mistakes:

| Setting | Casual | Normal | Hardcore | What |
| ------- | -----: | -----: | -------: | ---- |
| `width` | 2 | 4 | 8 | Partial plans kept at each step. |
| `candidates` | 10 | 10 | 12 | Commands tried from each partial plan. |
| `depth` | 2 | 3 | 4 | The most of its own commands a plan holds. |
| `samples` | 2 | 4 | 6 | Copies each partial plan is played on. |
| `temperature` | 100 | 25 | 0 | How far a pick may stray, in points: a command this many points below the best is half as likely. 0 always picks the best. |

**Player type** is what it scores, in points per thing counted (a cost is
negative):

| Weight | Casual | Normal | Hardcore | Per |
| ------ | -----: | -----: | -------: | --- |
| `won` | 10000 | 10000 | 10000 | the battle won |
| `lost` | −10000 | −10000 | −10000 | the battle lost |
| `lord_fell` | −10000 | −10000 | −10000 | lord fallen |
| `unit_fell` | −1000 | −15000 | −15000 | other unit fallen (for Normal and Hardcore more than a win is worth) |
| `hp_lost` | 0 | −5 | −10 | HP a standing unit lost (a heal counts back) |
| `lord_hp_lost` | −20 | −5 | −10 | HP the lord lost |
| `enemy_fell` | 100 | 100 | 150 | enemy fallen, or gone down (a boss who can't be killed) |
| `progress` | 10 | 10 | 15 | enemy HP taken in a rout (the boss's in a boss fight, a tile closer in a seize, a unit escaped in an escape) |
| `approach` | 2 | 2 | 3 | tile the units came closer to the nearest enemy (the boss, the tile to seize, the exit), walking; a unit that escaped reached the exit |
| `exposed` | 0 | −5 | −10 | HP the enemies that can reach a unit are expected to take from it next phase, by the forecast |
| `lord_exposed` | −20 | −5 | −10 | the same for the lord (not counted once the lord is alone: waiting can't win) |
| `at_risk` | −300 | −3000 | −3000 | unit that falls if every enemy that can reach it strikes and hits |
| `lord_at_risk` | −5000 | −5000 | −5000 | the same for the lord (not counted once it is alone) |
| `counter` | 10 | 10 | 15 | HP the units are expected to strike back at the enemies that can reach them |
| `turns` | 0 | 0 | −200 | turn that began (a plan ends with its turn, so this counts only in a longer search) |
| `items` | 0 | −50 | −300 | item used |
| `houses` | 500 | 500 | 500 | house saved, less house burned, in a battle with a village goal (0313; one value for all, added with the rule) |

Why the bots work this way, and what was changed after the first runs
of Chapter 1 and why, is
[ADR-0095](adr/0095-playtest-personas-plan-their-turn-and-score-what-a-player-sees.md).
From now on the settings are **fixed**: they are never changed to make a
battle pass. Only the comparison with Nick's own play (ticket 0508) may
change them, and never for one battle.

## Running it

```bash
cargo xtask playtest ch01 --bot all
```

`ch01` is a battle id: the name of a file in `assets/battles/` (or the
test files' `assets-test/battles/`) without `.ron`. Options:

| Option | Default | What |
| ------ | ------- | ---- |
| `--bot <bot>` | baseline | Who plays: `baseline`, `casual`, `normal`, `hardcore`, or `all` (the three personas). |
| `--runs <n>` | 100 | How many tries per bot. |
| `--seed <n>` | 1 | Seed of the first try. The tries play seeds `n`, `n + 1`, … |
| `--mode classic\|casual` | classic | The baseline's game mode. A persona plays its own. |
| `--turn-cap <n>` | 60 | A try still going after this many turns is stopped (unfinished, a failure). |
| `--json <path>` | | Also write the whole report as JSON. |
| `--history <dir>` | `target/playtest-history/` | Where every run is kept. |
| `--threads <n>` | every core | Threads to play on. Changes the speed only, never the numbers. |

`cargo xtask playtest --help` prints the same list. Run it optimised:
`cargo run --release -q -p xtask -- playtest ch01 --bot all`. The personas
think a lot: on Chapter 1 a try takes one core about 45 s for Hardcore,
9 s for Normal and 2.5 s for Casual, so 100 tries of all three take about
5 minutes on a 24-thread machine (and about 25 on CI's 4 cores).

The command exits with 1 when the battle is **blocked**: Casual succeeds in
under 20% of tries, the battle is basically unwinnable. A band missed is
only reported. CI plays every battle in `assets/battles` with `--bot all`
(`.github/workflows/playtest.yml`) only when a pull request or a push to
`main` changes a **balance lever**: a battle, a map, a number table in
`assets/data` (classes, items, spells, skills, arts, characters, AI,
terrain, supports, the new game), the rules (`crates/core`), how content
becomes a battle (`crates/content`), the bots, the engine pin, or the
playtest itself. The list is `.github/playtest-levers.sh` (ticket 0514);
"Run workflow" on the Actions page always plays. It fails only on a
blocked battle; the tables are in the job's summary.

## Reading the report

Chapter 1 on 2026-10-22 (ticket 0506):

```
ch01 · Normal tier · casual, normal, hardcore · turn cap 60
100 tries (seeds 1–100)

casual   · 44/100 succeed                band 30–55%   ✓
normal   · 59/100 succeed                band 50–75%   ✓
hardcore · 51/100 succeed                band 50–75%   ✓
hardcore · turns median 8 (normal 10)    lower ✓

try 3 · casual · Victory · turns 17 · deaths 0 · items 3 · lord fell 0 · retreats 0 · turns 17 · won 1
…

casual   · Casual  · won 44/100 · retreats per try  0: 1  1: 7  2: 11  3+: 81 · most: Tamsin Rook 93×
                     items per try  median 3  max 3 · turns median 23 · 2.52 s per try
normal   · Classic · won 90/100 · deaths per try  0: 59  1: 32  2: 7  3+: 2 · most: Hollis Marr 24×
                     items per try  median 3  max 3 · turns median 10 · 9.30 s per try
hardcore · Classic · won 83/100 · deaths per try  0: 79  1: 17  2: 4  3+: 0 · most: Hollis Marr 10×
                     items per try  median 1  max 3 · turns median 8 · 45.46 s per try
```

| Part | Meaning |
| ---- | ------- |
| first line | Battle, its tier (its `difficulty`, which picks the bands), the bots, the turn cap. |
| `N/100 succeed … band …` | How many tries met the bot's success rule, and the band its tier wants: **✓** in the band, **✗ too hard** under it, **⚠ too easy** over it. `blocked` after it: under the 20% floor. |
| `turns median … lower` | Hardcore's median turns against Normal's, both over the tries won with nobody dying: **✓** when Hardcore is faster. A **✗** here says the bots are wrong, not the map. **–** when one has no such try. |
| `try N · …` | One line per try: `N` is its seed, how it ended (`Victory`, `Defeat`, or `unfinished` at the turn cap), the turn it stopped on, then its measures: `deaths` (Classic) or `retreats` (Casual), each unit that fell (`fell Maud 1`) and on which turn (`fell Maud turn 6`), `items` used, whether the `lord fell`, `turns` played (the cap if it stopped there), `won`. |
| bot lines | Per bot: its mode, the tries won, how many tries had 0, 1, 2, 3 or more units fall and the unit that fell most often, items used per try, the median turns, how long a try took (the only numbers that differ between two runs with the same arguments), and the previous run's success count as `was …`. |
| `previous` | The run before this one, when there was one; its numbers are the `was …`. |

To watch one try again: `cargo xtask playtest ch01 --bot normal --seed 17
--runs 1`.

## History

Every run is added to a file in the history folder, one file per battle and
bot (`ch01-all.jsonl`, `quick-baseline-classic.jsonl`; one run per line,
with the date and the git commit). The next run of the same battle and bot
prints the last one's success counts as `was …`, so a change to a battle
shows as "47/100 succeed … was 58/100 succeed". The folder is under
`target/`, so it isn't committed and `cargo clean` empties it; pass
`--history <dir>` to keep runs somewhere else.

## Autobalancing

```bash
cargo run --release -q -p xtask -- shuyi autobalance --stage ch01
```

(`cargo shuyi autobalance --stage ch01`, built optimised.) The three
personas play the battle and, while one of them lands outside its band
for the battle's tier, the search changes the battle and plays it again,
until every band is met or its budget runs out. Only the battle changes,
never the bots (`docs/design/playtest-bots.md`, *Autobalancing*). The
search and the command are Shuyi Engine's (`autobalance`, `Knob`,
engine#2127; `shuyi autobalance`, engine#2541); the game gives the plan
(`crates/xtask/src/game_tools/balance.rs`) and the knobs
(`crates/bots/src/balance.rs`).

**What it may change** depends on the battle file's `kind`
(`assets/battles/README.md`). Every kind has the *enemy numbers* for now:
the unnamed enemies only (a template with no `id` or `name`, not a boss,
not a green unit), each knob in the order the search tries it:

| Knob | Steps | What a step does |
| ---- | ----- | ---------------- |
| `unnamed enemies' level` | −10 to +10, first in steps of 4 | Every unnamed enemy that many levels up or down (kept within 1 and the level cap). |
| `<enemy>: level` | −5 to +5 | That enemy alone. |
| `<enemy>: count` | −1, +1 | −1 removes it; +1 adds a copy of it (same group, same AI) on the nearest free tile within 2 of it, after it in the list. |
| `<enemy>: class` | the battle's templates | Another template the battle already uses, in the battle's order, at the enemy's level, with that template's gear. |
| `<enemy>: weapon` | −1, +1 | Its first weapon a tier down or up in the same kind (iron → steel). |

`<enemy>` reads like `Brigand at 16,11`: its name and its tile as
authored. Named units, the player's slots, reinforcements, the map, the
objective and the story moments are never changed (a test checks every
step of every knob). A step that can't apply (no free tile, no higher
tier, a class that can't stand there) is skipped. A random skirmish
gains terrain and layout knobs with its generator (0510).

**The output** first names the battle, its tier and kind, then why the
search stopped and how many evaluations it made, then each change it
kept, in the knobs' words:

```
test_too_hard · Normal tier · FixedSkirmish

stuck: no knob brings it nearer its bands · 28 evaluations
unnamed enemies' level -8: every unnamed enemy -8 levels

before: 20 tries (seeds 1–20)

casual   · 0/20 succeed                 band 30–55%   ✗ too hard · blocked: under the 20% floor
normal   · 1/20 succeed                 band 50–75%   ✗ too hard
hardcore · 2/20 succeed                 band 50–75%   ✗ too hard
…
after: 20 tries (seeds 1–20)

casual   · 8/20 succeed                 band 30–55%   ✓
normal   · 10/20 succeed                band 50–75%   ✓
hardcore · 13/20 succeed                band 50–75%   ✓
```

(`--tries 20 --quick 10 --budget 40 --turns 20` on the test battle.)
Each change reads `<knob> <step>: <what it does>`. Then come the playtest
reports (as above) of the battle as authored (`before`) and as changed
(`after`), each at the full `--tries`: `after` decides whether the
battle is in its bands, whatever the stop line says (the search judges
its own runs at `--quick` tries, which can miss a band the full run
meets). `already in bands` means the battle as authored meets them and
nothing changed. The
search's own runs use `--quick` tries (a fifth); `--budget` caps the
evaluations (100), `--time` the seconds, `--threads` the threads (never
the result: the same seed finds the same changes). It exits with 1 while
a rate is still outside its band. `cargo shuyi autobalance --help` lists
every option.

**`--apply`** writes the changes to the battle file
(`assets/battles/<id>.ron`, or the test files' `assets-test/battles/`).
Only the enemy list is rewritten: everything else, comments too, stays as
written; a changed enemy keeps its own line with only the changed fields
rewritten (`level: Some(4)` added or replaced), a copy goes on its own
line after its original, and a removed enemy's line goes with it. The
comment lines above a removed enemy go too, unless it was the list's
first. Review the diff and the `after` report before committing it.

Autobalance is a dev tool Claude runs; CI never runs it, only the
playtest.

## For developers

- **The engine's part** (`shuyi_core::bots`): the bot trait and the play
  loop (ticket 1116); `Determinize`, `Evaluate` and `Order` (engine#2125);
  the turn planner, `TurnPlan` and `PlanSettings` (engine#2128); the
  playtest runner, `Entrant`, `Measure`, `Tries`, `play_tries`, `judge`,
  the checks (`Check::Rate`, `Check::Median`) and the report's table
  (engine#2126).
- **The game's part**:
  - `trpg-core`: `Determinize` for `BattleState` (fresh luck, pending
    reinforcements dropped: `reseed_luck`, `drop_pending_reinforcements`)
    and `BattleState::threat_forecast` (the forecast a player reads off a
    threatening enemy), all for the bots only.
  - `crates/bots` (`trpg-bots`): `BaselineBot`; the personas
    (`persona.rs`: the candidates' `order`, the scoring `Tally` and
    `Weights` with the danger the units stand in, `PersonaBot` with the
    talk rule); the battle on the runner
    (`playtest.rs`: `BattleMeasure`, `BattleEntrant`, `Targets`). Pure, like
    `core`: no files, no clock. It is dev tooling: `trpg-ui` and
    `trpg-app` never depend on it, and it never depends on them (a test in
    `xtask` checks the manifests). `personas.ron` and `targets.ron` are
    beside it, dev tooling too, not game content.
  - `crates/xtask/src/playtest.rs`: the command. It loads the battle and
    the two files, plays the tries on threads with the engine's
    `play_tries`, judges them once with `judge`, and writes the report,
    the JSON and the history.
- A new bot implements the engine's `Bot<BattleState>` and gets a
  `BattleEntrant` (and a name in `Bot` in `playtest.rs`). It gets the try's
  seed for its own randomness; the same seed must give the same play.
