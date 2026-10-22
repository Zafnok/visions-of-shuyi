---
id: "0319"
title: "Core rules: an escape objective, a fleeing boss, a boss who can't die, and a scripted death"
type: feature
milestone: M2 Core rules
model: opus-5.5
effort: high
status: todo
blocked_by: ["0069"]
nick_input: answer-first
completed:
---

# 0319 — Escape, a fleeing boss, a boss who can't die, a scripted death

## Context

Act 1's story battles need four rules `core` doesn't have
(`docs/story/outline.md`):

- **Chapter 6, the Ashfields:** reach the Brennish lines (an escape
  objective).
- **Chapter 4, Veyne Hall:** Dace "escapes at low HP".
- **Chapter 8, Saltmere Docks:** Dace "can't be killed; he retreats".
- **Chapter 8:** Hollis dies holding the gangway, in every mode; in
  Classic, if he already fell, the lead tells Dace instead.

0069 asks Nick how each works and records the answers in the design
docs. This ticket builds them in `core`, as data the battle files can
use, so the chapter tickets (0850, 0852, 0854) only write data.

`core` today has the Rout, DefeatUnit, Seize and Survive objectives,
reinforcements, battle scene triggers (turn start, map areas, falls,
talk), and Classic/Casual death and retreat (`death-and-difficulty.md`).

## Nick input

**Answer first:** 0069. Don't build a rule its answers don't settle; if
one is left open, ask about it with `ask-nick` before writing code.

## Scope

**In** (as 0069 decided, `docs/design/turn-structure.md`, *Act 1's
battle rules*, and `death-and-difficulty.md`; the names below are
placeholders):
- An **escape objective** (`Objective::Escape { area }`): a player unit
  that ends its move in the exit area may choose `UnitAction::Escape`; it
  leaves the map and is safe (can't act, be targeted, block or fall). The
  battle is won when no player unit is left on the map and at least one
  escaped; a unit that falls on the way falls as usual. **No turn limit**
  (Nick); the lead may escape early; `Escape` gives no EXP; green units
  needn't escape; escaped units count as standing in the results. The
  "every player unit has fallen" loss doesn't fire while some escaped.
  The objective line shows `Get everyone to <area> (2/6 out)`.
- **A boss who retreats** (Chapter 4): a unit setting "retreats instead
  of falling". At 0 HP its defeated line plays and it leaves the map
  alive; it counts as defeated for the objective (DefeatUnit is won), EXP
  (kill EXP to the striker) and the results. Normal forecast.
- **A unit that can't be killed** (Chapter 8): a unit setting. At 0 HP
  its defeated line plays and it is **down**: it stays on its tile, takes
  no more actions, can't be targeted by anyone (attacks, spells, heals),
  and **blocks its tile** (no unit passes or stops there). Kill EXP to the
  striker; counts as defeated in the results; the battle goes on. Normal
  forecast (his HP shows 0). The map scene marks it down so every map
  skin can draw it grayed out (ADR-0038).
- **A scripted death** (Hollis, Chapter 8): a scene can remove a named
  player unit for good in **every mode** (loadout to the stock, as a
  Classic death). The closing scene branches on whether he is still in
  the army: if he fell in this battle in Classic, or died earlier, the
  "lead tells Dace" version plays; in Casual a retreat in this battle
  doesn't change it (0715's lines by who is in the army may already
  cover the branch: use them if so). While he is in the army he is
  forced into Chapter 8's deployment (*Claude's starting rule*).
- The AI respects them: a boss that retreats still fights by the boss AI;
  enemies never target a down unit; the enemy side never escapes.
- The bots (0504's legal moves) and play records handle the new rules
  (an escaping unit's leaving is a recorded event).
- Content validation: an escape area exists on the map; a retreating or
  unkillable unit is an enemy.
- Test battles in `assets-test/` for each rule.

**Out (do not do):**
- The chapters' real battles (0850, 0852, 0854) and scripts (0746,
  0748, 0750).
- Changing how Rout, Seize, Survive or DefeatUnit work.
- Save the village (houses, `Evacuate`, raiders): ticket 0313.
- Recruiting: "joins if defeated" already exists; Gil uses it as is.
- New art or UI beyond the objective line, the forecast and the
  results.

## Implementation steps

1. Read 0069's answers (`turn-structure.md`, `death-and-difficulty.md`,
   wherever it recorded them) and `core`'s objectives, falls, scene
   triggers and AI.
2. Rules in `core`, through `Command` → `Event` (ADR-0004); unit
   settings and objectives in the battle file format, documented in
   `assets/battles/README.md`.
3. The objective line, forecast and results in `ui` (follow ADR-0038 for
   anything shown on the map: the escape area goes in the map scene and
   every map skin paints it).
4. Test battles and tests; the bots run them (`cargo xtask playtest
   <test battle> --bot all`) without a softlock.

## Acceptance criteria

- [ ] Each rule works as 0069 recorded it (unit and integration tests
      named after the rule).
- [ ] The forecast against an unkillable boss is the normal one, his HP
      reaching 0 (snapshot).
- [ ] The bots finish each test battle; play records replay identically.
- [ ] Chapter 1 plays exactly as before.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: escape win (last one out, last one falls with others out) and the
  lord rule after the lead escaped; retreat at the killing blow winning
  DefeatUnit with kill EXP; a down unit that can't act, be targeted or be
  walked through; the scripted removal in Classic and Casual, and the
  scene branch.
- Property: no sequence of legal commands kills an unkillable unit or
  lets a down unit act; an escaped unit never acts again in that battle.
- Snapshot / integration: the objective line, the forecast, a scripted
  test battle for each rule.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
