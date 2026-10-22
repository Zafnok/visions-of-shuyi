# ADR-0095: Playtest personas plan their own turn on the engine's turn planner and score what a player sees; their settings and targets are dev-tooling RON files

- **Status:** Accepted
- **Date:** 2026-10-22
- **Related tickets:** 0506, 0033, 0505, 0508, 0509; engine#2125, engine#2126, engine#2128

## Context

Nick's three player types (`docs/design/playtest-bots.md`) must play a
battle many times and say whether it lands in the band its tier wants.
The engine has the generic parts: `Determinize` and `Evaluate`
(engine#2125), the playtest runner with checks as data (engine#2126) and,
since this ticket, a turn planner (engine#2128). The game supplies what
only it knows: how a battle is copied for planning, which commands are
worth trying, how good a state is for each type, and the targets.

Ticket 0506 first described flat Monte Carlo (one command at a time,
copies played on with the enemy AI on every side). Built that way, the
personas played Chapter 1 **worse than the mindless baseline** (about 30%
won): Casual 2/100, Normal 3/100, Hardcore 4/100 successes, most tries at
the 60-turn cap. Traced tries showed three faults:

- **The playout charged for the persona.** After any candidate the enemy
  AI played the persona's other units, and that AI always attacks: every
  move scored as "then the whole army charges". Only ending the turn
  skipped it, so careful personas ended their turns for 50 turns in a row.
- **One command at a time can't coordinate.** The first step of any
  joint move ("weaken it, then finish it"; a sturdy unit stepping in
  first) scores as a unit standing in danger and is refused.
- **Foresight the owner doesn't want.** Playing the enemy's turn out with
  its real AI tells the bot exactly whom the enemy will attack. Nick
  (2026-10-21): bots "know the danger zone since the UI makes it apparent,
  but they shouldn't know exactly who gets attacked and if it will crit
  and what arts are used".

Nick chose an engine turn planner (engine#2128) over hand-written combos
in the game, and the tuning that followed was done with him, run by run,
on Chapter 1 (2026-10-21/22).

## Decision

1. **`Determinize` for `BattleState`** is in `trpg-core` (the orphan rule):
   a clone with `reseed_luck(seed)` and `drop_pending_reinforcements()`.
   **`BattleState::threat_forecast`** gives the forecast of one unit
   striking another where it stands, with its equipped weapon. All three
   are bot-only (ADR-0033 §2); every unit on the map is visible to both
   sides, so nothing else is hidden.
2. **One bot, `PersonaBot`** (`trpg-bots`), is the engine's `TurnPlan`
   (a beam search over the persona's own commands until its turn ends;
   it never plays the enemy's turn) with:
   - **the order** (`trpg_bots::order`), every step of a plan: each
     unit's best attack on each target; the enemy AI's own pick; heals on
     wounded units; each ready unit's **advance** (its reachable tile
     nearest the objective, walking); each ready unit's **hold** (its
     reachable tile out of every enemy's reach nearest the objective); the
     other ways of attacking; the rest; `EndPhase` always within the cap.
     `Equip` and `Talk` are dropped. Every unit gets a say before any
     unit's second way of striking the same target, so a cap of 10 isn't
     filled with one attack in four flavours;
   - **the scoring** (`Tally` × `Weights`), on the state a partial plan
     reached: won, lost, falls (lords apart), HP lost, enemies fallen,
     objective progress, `approach` (tiles closer to the goal, walking),
     and **the danger the units stand in, as a player sees it**: for each
     player unit, the enemies whose threat area reaches it (stationary ones
     only from their own tile) and their forecasts: `exposed` (expected HP
     taken), `at_risk` (falls if every threat hits), `counter` (expected HP
     struck back), lords apart. A lord left alone doesn't count its own
     danger: waiting can't win;
   - **the talk rule**: each pair of units talks once per battle, before
     planning.
   Its own seed is the try's seed mixed with a constant, so its draws
   never follow the battle's rolls.
3. **The types are data**: `crates/bots/personas.ron` (mode,
   `PlanSettings`, `Weights`) and `crates/bots/targets.ron` (success
   conditions, bands per tier, floors, the faster check), read by
   `cargo xtask playtest`. They are dev tooling, never in `assets/`.
   `Targets::checks` turns them into the engine's `Check`s for a battle's
   tier.
4. **The settings** (Nick's decisions in `docs/design/playtest-bots.md`,
   Claude's numbers): Casual is a weaker player (shallow, narrow plan on
   two copies, strays), cares about the win and the lord, and treats its
   units as tools (a fall costs a tenth of a win); Normal plans further
   and wants nobody dead; Hardcore plans the furthest, never strays, and
   guards its HP twice as hard (it won't spend items). Casual's bands are
   the Casual bot's (Nick's first bands shifted down 30 points). The
   faster check compares the medians over the tries won with nobody dying.
5. **Settings are fixed** after this ADR: only the comparison with Nick's
   play (0508) changes them, never per battle.
6. **Measures** are the engine's named measures: `won`, `lord fell`,
   `deaths` (Classic), `retreats` (Casual), `items`, `turns`, and
   `fell <unit>` / `fell <unit> turn`. 0505's `play_battle` and its report
   code are gone; the runner, the verdicts and the table are the engine's.

## Consequences

- On Chapter 1 (100 tries each) all three land in their bands: Casual
  44%, Normal 59%, Hardcore 51%, Hardcore faster than Normal. Normal and
  Hardcore win 83–90% of tries; their misses are deaths and items.
- **These settings were tuned on Chapter 1**, with Nick, because the first
  designs played below a mindless baseline. Every change answers a fault
  seen in a traced try (a stall, a missing kind of move, a rule Nick
  decided), not a band; still, Chapter 1 is the battle they were
  checked on, and 0508 (Nick's own play) is where they are checked
  against a person.
- The personas are slow: about 45 s of one core for a Hardcore try of
  Chapter 1, 9 s for Normal, 2.5 s for Casual. 100 tries of all three
  take about 5 minutes on 24 threads.
- A persona plans only its own turn and judges the enemy's by the danger
  zone and forecasts: it can't bait an enemy over two turns. Casual still
  runs out the turn limit in about a fifth of its tries, once its lord is
  all it has left and nothing is in reach.

## Alternatives considered

- **Flat Monte Carlo with enemy-AI playouts (ticket 0506 as written)** —
  the stalls above, and foresight Nick doesn't want.
- **Hand-written combos in the game** — planning logic for one game where
  a generic search belongs in the engine (Nick chose the engine, ADR-0092).
- **The ticket's order (all attacks, then the legal order)** — a capped
  list held one unit's attacks in every flavour and no unit's move.
- **Manhattan distance for `approach`** — across a river or around a wall
  it points at enemies a unit can't walk to.
- **Casual as Normal's skill with Casual's goals** — Nick kept Casual a
  weaker player and lowered its bands instead.
- **Weights in `assets/`** — they aren't game content; nothing shipped
  reads them.
- **One trained network per type** — that is 0507.
