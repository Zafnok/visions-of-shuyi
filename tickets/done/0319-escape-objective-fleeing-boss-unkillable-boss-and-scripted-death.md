---
id: "0319"
title: "Core rules: an escape objective, a fleeing boss, a boss who can't die, and a scripted death"
type: feature
milestone: M2 Core rules
model: opus-5.5
effort: high
status: done
blocked_by: ["0069"]
nick_input: answer-first
completed: 2026-10-22
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

- [x] Each rule works as 0069 recorded it (unit and integration tests
      named after the rule).
- [x] The forecast against an unkillable boss is the normal one, his HP
      reaching 0 (snapshot).
- [x] The bots finish each test battle; play records replay identically.
- [x] Chapter 1 plays exactly as before.
- [x] All gates in the `run-gates` skill pass.

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

**Estimate (ADR-0092), before coding:** about 700 production lines with
doc comments: four rules each the size of Seize or chest opening (an
action, an event, a judge arm, a menu entry, a line of text, about 60
lines each) plus the down unit's tile blocking, the AI's escape for the
bots, the battle file fields and checks, the `@dies` directive and the
flow applying it, and the map scene's exit range and grayed unit in three
skins.

**Measured:** 415 production lines (`cargo xtask code-lines`): 0.6× the
estimate, nothing to record.

**Done:**

- **Escape** (`Objective::Escape { area: TileRect }`, battle file
  `Escape(area: (x, y, w, h))`): a player unit that ends its move in the
  exit area may choose `UnitAction::Escape`; it leaves the map
  (`Event::UnitEscaped`) into `BattleState::escaped`: it can't act, be
  targeted, block or fall. Won when no player unit is left on the map and
  at least one escaped; the "every player unit fell" loss doesn't fire
  while some escaped; an escaped lead can't lose the lord rule. No turn
  limit, no EXP for escaping; enemies and green units never escape.
  Escaped units count as standing in `Campaign::apply_result` (and so on
  the results screen, which lists the deployed units still in the army).
- **A boss who retreats** (`defeat: Retreats` on an enemy, `Defeat` in
  `core`): falls exactly as before inside the battle (`Event::UnitFell`,
  into `fallen`), its fall triggers playing first (the defeated line), so
  it counts as defeated for DefeatUnit, kill EXP and the results; the
  story keeps it alive.
- **A boss who can't be killed** (`defeat: GoesDown`): at 0 HP its fall
  triggers play and it goes down (`Event::UnitDowned`) into
  `BattleState::down`, on its tile, which joins the map's `blocked` set:
  no unit passes or stops there (movement, the danger zone, the AI, Shove,
  move-after, tile casts and reinforcement arrivals all respect it). It
  never acts, nobody can target it (`CommandError::UnitDown`), the
  striker gets a kill's EXP, and it counts as defeated. The forecast is
  the normal one (snapshot test). The map scene marks it
  (`UnitView::fight == Fight::Down`), and every skin grays it out.
- **A scripted death:** a dialogue directive, `@dies <character>`
  (content checks: a named character, never the lead, not in a reply's
  reaction). When the flow plays a scene it carries out the deaths of the
  scene's version for those there (`trpg_content::deaths`,
  `Campaign::dies_in_scene`): in every mode the character leaves the
  roster, its loadout goes to the stock and its supports end. The branch
  is 0715's `@if`: the victory scenes play for the roster after
  `apply_result`, so a Classic death in the battle (or an earlier one)
  plays the `@else` (the lead tells Dace), and a Casual retreat doesn't.
- **UI:** the objective line, the `Escape` action-menu entry, the rewind
  list's line, the exit area as a map range (`range::EXIT`, a new palette
  colour `exit_area` in every palette), the down unit drawn on its tile
  during and after playback.
- **AI and bots:** enemies can't escape (the battle refuses them) and
  can't target a down unit (it isn't among the units). When the AI plays
  the player's side (the baseline bot) on an escape map, each unit runs
  for the exit. `legal_commands` lists `Escape`.
- **Test battles** (`assets-test/`): `test_escape`, `test_retreat`,
  `test_hold` (+ chapter `test_hold` whose closing scene uses `@dies`).
  The baseline bot finishes every try of each (`cargo xtask playtest
  <id> --runs 20`), and `the_act_1_rule_battles_finish_and_replay_identically`
  checks each rule comes up and every try replays to the same events and
  end state.
- The debug palette sampler's canary (`demo_panels_keep_margin_for_a_larger_palette`)
  now asks for one spare row instead of two: the new `exit_area` colour
  makes the palette 56 colours, a twelfth row of swatches, the same as the
  60-colour headroom the test still checks; the demo panels lose a blank
  row (snapshot accepted).
- Save format unchanged: the new fields are left out of a save at their
  defaults (as the campaign's tutorial level), so the golden saves still
  read and write back the same.

**Deviations:**

- The objective line says `Get everyone to the exit (2/6 out)`: maps
  don't carry their own text yet, so the exit isn't named. Follow-up
  **0324** (in 0852's and 1064's `blocked_by`).
- `cargo xtask playtest <battle> --bot all` doesn't exist: the baseline
  bot is the only one xtask has; it played each test battle.
- The retreat has no event of its own: in the battle it is a fall
  (`UnitFell`); only the story differs.
- The forced deployment of Hollis needs no code: a battle's player slots
  fix who deploys and Preparations has no deploy choice, so a character
  with a slot who is in the army is always deployed.

**Claude's starting rules** (Nick may veto):

- A downed unit's passives and auras stop counting: it is out of the
  fight.
- A downed unit counts as defeated for the objective too (DefeatUnit, a
  rout), not only on the results screen.
- A unit pushed (Shove) into a downed unit's tile is blocked: it stays
  and takes the collision damage; the downed unit takes none. A tile cast
  can't change a downed unit's tile; a reinforcement whose tile it holds
  waits.
- A downed unit is never recruited ("joins if defeated" can't be set on
  one: content refuses it).
- Escaped units get their share of the battle's EXP pool (green units'
  EXP), as standing units do.
- `Escape` is never where the action menu opens (leaving isn't a
  default); Seize still is.
- A scene's death happens as the scene starts (skipping the scene can't
  skip it); the lead never dies in a scene.
- The look: the exit area is a muted gold tint (`exit_area`; grey dither
  on the Playdate); a down unit's label is the dim text colour (glyph
  skin), its outline grey and its picture dimmed (sprite skin), the acted
  look on the Playdate.
- When the AI plays the player's side on an escape map (bots only), it
  only runs for the exit and never attacks.

**For the Act 1 playtest (1064):** the exit tint and the down unit's
grayed look are starting looks; judge them in Chapters 6 and 8.
