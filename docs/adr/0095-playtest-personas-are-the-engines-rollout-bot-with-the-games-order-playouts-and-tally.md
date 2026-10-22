# ADR-0095: Playtest personas are the engine's rollout bot with the game's order, playouts and tally; their settings and targets are dev-tooling RON files

- **Status:** Accepted
- **Date:** 2026-10-21
- **Related tickets:** 0506, 0033, 0505, 0508, 0509; engine#2125, engine#2126

## Context

Nick's three player types (`docs/design/playtest-bots.md`) must play a
battle many times and say whether it lands in the band its tier wants.
The engine now has the generic parts (engine#2125, engine#2126, engine
ADR-0027): a flat Monte Carlo rollout bot over `Determinize` games, and a
playtest runner with checks as data. The game has to supply what only it
knows: how a battle is copied for planning, which commands are worth
trying, how a copy is played on, how good an end state is for each type,
and the targets.

Ticket 0506 fixed the shape (skill = thinking budget and mistakes, player
type = what is scored; "don't tune the bots to pass") and proposed the
details: candidates = attacks then the legal order, playouts = the enemy
AI on every side, scoring = won, lost, falls, HP, enemies, objective,
turns, items. Built that way, the personas played **worse than the
mindless baseline**: on Chapter 1, 100 tries each, Casual succeeded 2
times, Normal 3, Hardcore 4 (the baseline wins about 30%), most tries
ending at the 60-turn cap. A traced try showed why:

- **The playout charged for the persona.** After any candidate, the enemy
  AI played the persona's *other* units too, and the enemy AI always
  attacks and walks straight at the nearest enemy. Every move was thus
  scored as "this move, then the whole army charges into the guards";
  only ending the turn skipped the charge, so a careful persona, a few
  enemies from winning, ended its turn for 50 turns in a row.
- **Units far from the fight never got a candidate.** Past the attacks
  and the AI's one pick, the legal order lists the first unit's waits on
  the top-left tiles.
- **Nothing rewarded walking towards enemies that hold their ground**,
  and with own HP costing as much as enemy HP was worth, a careful type
  never started a fight it didn't win outright.

These are faults of the bot, not of the map, and they show on the test
battle `quick` too (Normal stalled in 7 of 24 tries there).

## Decision

1. **`Determinize` for `BattleState`** is in `trpg-core` (the orphan rule):
   a clone with `reseed_luck(seed)` and `drop_pending_reinforcements()`,
   both documented as bot-only (ADR-0033 §2). Every unit on the map is
   visible to both sides, so nothing else is hidden.
2. **One bot, `PersonaBot`** (`trpg-bots`), is the engine's `Rollout`
   with:
   - **the order** (`trpg_bots::order`): every distinct attack (unit,
     weapon, target, art or active) from its best tile, most expected
     damage first; the enemy AI's own pick for the state; heals on wounded
     units, most wounded first; **each ready unit's advance** (its wait on
     the reachable tile nearest the objective, walking); the rest in the
     legal order; `EndPhase` last but always within the candidate cap.
     `Equip` and `Talk` are dropped;
   - **the playout** (`PersonaPlayout`): on the persona's own side the
     phase ends at once (the persona decides its other units itself, one
     at a time); every other side, and a move offered after an attack, is
     the enemy AI. `Horizon::Sides(1)` therefore weighs the candidate
     alone, `Sides(2)` the candidate and the enemy's reply;
   - **the scoring** (`Tally` × `Weights`): won, lost, lords fallen, other
     units fallen, HP lost (standing units, lords apart), enemies fallen,
     objective progress (enemy HP in a rout, the boss's HP, tiles to the
     seize tile), **approach** (tiles the units came closer to the nearest
     enemy, the boss or the seize tile, walking: the engine's flow field,
     as the enemy AI walks), turns and pack items used;
   - **the talk rule**: each pair of units talks once per battle, before
     planning (talking costs nothing).
   Its own seed is the try's seed mixed with a constant, so its draws
   never follow the battle's rolls.
3. **The types are data**: `crates/bots/personas.ron` (mode,
   `RolloutSettings`, `Weights`) and `crates/bots/targets.ron` (success
   conditions, bands per tier, floors, the faster check), read by
   `cargo xtask playtest`. They are dev tooling, beside the bots, never in
   `assets/`. `Targets::checks` turns them into the engine's `Check`s for a
   battle's tier (its `difficulty`).
4. **Own HP costs half what enemy HP is worth** for the careful types, and
   every type gets a small `approach` weight (2–3 points a tile against
   10–15 an enemy HP). The thinking budgets and temperatures are the ones
   first chosen from the persona text and were never changed.
5. **What was looked at, and when.** The fixes above answer the three
   faults (the stalls), found on a 4-try trial and a 100-try run of
   Chapter 1 and a trace of one try, and checked on the test battle
   `quick` (do the personas finish and beat the baseline?), never against
   a band. The Chapter 1 report in ticket 0506 is the first full run of the
   final code; nothing was changed after it.
6. **Settings are fixed** after this ADR: only the comparison with Nick's
   play (0508) changes them, never per battle (ticket 0506).
7. **Measures** are the engine's named measures: `won`, `lord fell`,
   `deaths` (Classic), `retreats` (Casual), `items`, `turns`, and
   `fell <unit>` / `fell <unit> turn` per unit that fell. 0505's
   `play_battle` and its report code are gone; the runner, the band
   verdicts and the table are the engine's.

## Consequences

- A battle's report is the engine's table plus the game's lines; a new
  check is a line of `targets.ron`, a new type a block of `personas.ron`.
- The personas are slow: Hardcore weighs 16 candidates on 16 copies
  through the enemy's reply. 100 tries of all three on Chapter 1 take
  about a quarter of an hour on 24 threads, and much longer on CI's 4
  cores.
- The persona plans one command at a time and sees at most the enemy's
  reply: it can't plan a pincer over two turns or bait an enemy. It plays
  like a careful player of the turn in front of it.
- `approach`, the HP weights and the playout are judgement, not
  measurement; 0508 is where they are checked against Nick's play.

## Alternatives considered

- **The ticket's details as written** — they gave the stalls above.
- **The enemy AI on the persona's own side in the playouts, but only for
  units that can attack** — still a guess at the persona's own later
  choices, and still reckless; ending the phase leaves those choices to
  the persona.
- **A longer horizon instead of `approach`** — two or three times the
  thinking cost for each extra side, and still blind on a map whose
  enemies are more than a turn away.
- **Manhattan distance for `approach`** — across a river or around a wall
  it points at enemies a unit can't walk to.
- **Weights in `assets/`** — they aren't game content; nothing shipped
  reads them.
- **One trained network per type** — that is 0507; search needs no
  training data and its settings read as what each type cares about.
