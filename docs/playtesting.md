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
| `casual` | Casual mode | Cares about winning and the lord only; other units falling costs it nothing. Thinks little and makes mistakes. | it wins |
| `normal` | Classic | Wants nobody dead, spends items when that helps, doesn't hurry. Thinks more, makes few mistakes. | it wins, nobody dies, and it used at most 3 items |
| `hardcore` | Classic | Nobody dead, few turns, few items. Thinks the most and never strays: it stands in for veterans. | it wins, nobody dies, and it used at most 1 item |
| `baseline` | `--mode` | The enemy AI playing the player's side: charges, attacks the best target, never uses items. A yardstick, with no target. | — |

The army a bot fields is the characters of the battle's player slots, as
they are in the character data (the same army the debug Quick Battle uses).

### How the personas think

Casual, Normal and Hardcore are the same bot with different settings
(`crates/bots/personas.ron`). Before each command it:

1. **talks** first, if a unit can talk to someone it hasn't talked to yet
   (talking costs nothing);
2. lists a few **candidate** commands: every attack, the one doing the most
   damage first; what the enemy AI would do in its place; healing a
   wounded unit; moving each unit as far towards the enemy (the boss, the
   tile to seize) as it can walk; and ending the turn last;
3. plays each candidate out a few times on **copies** of the battle with
   fresh luck and no hidden reinforcements: after the candidate, its own
   turn ends (it decides its other units itself, one by one) and, if it
   looks that far, the enemy replies as the enemy AI does;
4. **scores** where each copy ended, by what the persona cares about, and
   picks the candidate whose copies ended best (or, with a temperature,
   sometimes one a little worse).

**Skill** is the thinking budget and the mistakes:

| Setting | Casual | Normal | Hardcore | What |
| ------- | -----: | -----: | -------: | ---- |
| `candidates` | 8 | 12 | 16 | Commands weighed per decision. |
| `samples` | 4 | 8 | 16 | Copies each is played out on. |
| `horizon` | `Sides(1)` | `Sides(2)` | `Sides(2)` | How far a copy is played: the candidate alone, or through the enemy's reply. |
| `commands` | 60 | 120 | 120 | The most commands a copy plays. |
| `temperature` | 100 | 25 | 0 | How far a pick may stray, in points: a candidate this many points below the best is half as likely. 0 always picks the best. |

**Player type** is what it scores, in points per thing a copy changed:

| Weight | Casual | Normal | Hardcore | Per |
| ------ | -----: | -----: | -------: | --- |
| `won` | 10000 | 10000 | 10000 | the battle won |
| `lost` | −10000 | −10000 | −10000 | the battle lost |
| `lord_fell` | −10000 | −10000 | −10000 | lord fallen |
| `unit_fell` | 0 | −15000 | −15000 | other unit fallen (more than a win is worth) |
| `hp_lost` | 0 | −5 | −5 | HP a standing unit lost (a heal counts back) |
| `lord_hp_lost` | −20 | −5 | −5 | HP the lord lost |
| `enemy_fell` | 100 | 100 | 150 | enemy fallen |
| `progress` | 10 | 10 | 15 | enemy HP taken in a rout (the boss's in a boss fight, a tile closer in a seize) |
| `approach` | 2 | 2 | 3 | tile the units came closer to the nearest enemy (the boss, the tile to seize), walking |
| `turns` | 0 | 0 | −200 | turn that began |
| `items` | 0 | −50 | −300 | item used |

Why the scoring and the copies work this way, and how they were fixed
after the first runs stalled, is [ADR-0095](adr/0095-playtest-personas-are-the-engines-rollout-bot-with-the-games-order-playouts-and-tally.md).
The settings were chosen from the player types' descriptions and are
**fixed**: they are never changed to make a battle pass. Only the
comparison with Nick's own play (ticket 0508) may change them, and never
for one battle.

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
think a lot: 100 tries of all three on Chapter 1 take about 15 minutes of
one core per try in all (Hardcore about 90 s a try, Normal 45 s, Casual
6 s), so about 10 minutes on a 24-thread machine.

The command exits with 1 when the battle is **blocked**: Casual succeeds in
under 20% of tries, the battle is basically unwinnable. A band missed is
only reported. CI plays every battle in `assets/battles` with `--bot all`
when the battles, the rules or the bots change
(`.github/workflows/playtest.yml`), and fails only on a blocked battle;
the tables are in the job's summary.

## Reading the report

```
ch01 · Normal tier · casual, normal, hardcore · turn cap 60
100 tries (seeds 1–100)

casual   · 64/100 succeed                band 60–85%   ✓
normal   · 47/100 succeed                band 50–75%   ✗ too hard
hardcore · 52/100 succeed                band 50–75%   ✓
hardcore · turns median 10 (normal 14)   lower ✓

try 1 · casual · Victory · turns 12 · deaths 0 · fell Maud 1 · fell Maud turn 6 · items 0 · lord fell 0 · retreats 1 · turns 12 · won 1
…

casual   · Casual  · won 64/100 · retreats per try  0: 30  1: 21  2: 25  3+: 24 · most: Maud 41×
                     items per try  median 0  max 0 · turns median 12 · 6.44 s per try · was 58/100 succeed
normal   · Classic · won 61/100 · deaths per try  0: 55  1: 30  2: 12  3+: 3 · most: Maud 22×
                     items per try  median 2  max 3 · turns median 14 · 44.48 s per try
hardcore · Classic · won 60/100 · deaths per try  0: 70  1: 25  2: 4  3+: 1 · most: Rue 9×
                     items per try  median 1  max 2 · turns median 10 · 90.00 s per try
previous  2026-10-21 14:05 UTC · commit d6bd82b
```

| Part | Meaning |
| ---- | ------- |
| first line | Battle, its tier (its `difficulty`, which picks the bands), the bots, the turn cap. |
| `N/100 succeed … band …` | How many tries met the bot's success rule, and the band its tier wants: **✓** in the band, **✗ too hard** under it, **⚠ too easy** over it. `blocked` after it: under the 20% floor. |
| `turns median … lower` | Hardcore's median turns against Normal's (over the tries that meet Hardcore's success rule): **✓** when Hardcore is faster. A **✗** here says the bots are wrong, not the map. **–** when one has no such try. |
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

## For developers

- **The engine's part** (`shuyi_core::bots`): the bot trait and the play
  loop (ticket 1116); the rollout bot, `Determinize`, `Evaluate`, `Order`,
  `Playout`, `Rollout` and `RolloutSettings` (engine#2125); the playtest
  runner, `Entrant`, `Measure`, `Tries`, `play_tries`, `judge`, the checks
  (`Check::Rate`, `Check::Median`) and the report's table (engine#2126).
- **The game's part**:
  - `trpg-core`: `Determinize` for `BattleState` (fresh luck, pending
    reinforcements dropped: `reseed_luck`, `drop_pending_reinforcements`,
    both for the bots only).
  - `crates/bots` (`trpg-bots`): `BaselineBot`; the personas
    (`persona.rs`: the candidates' `order`, the scoring `Tally` and
    `Weights`, `PersonaBot` with the talk rule); the battle on the runner
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
