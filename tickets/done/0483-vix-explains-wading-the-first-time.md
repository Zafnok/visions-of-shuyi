---
id: "0483"
title: "Vix explains wading the first time a unit could step into water"
type: feature
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-09
---

# 0483 — Vix explains wading the first time a unit could step into water

## Context

A friend's playtest of the Act 1 build, passed on by Nick (2026-11-07):

> "There should be some Vix tutorial about wading in water since it's a
> bit unintuitive"

Wading: the `water` terrain (river, shallows; `assets/data/terrain.ron`
~:58–63). Foot units pay 5 Mov to enter, which usually spends a whole
turn; mounted and armored units can't enter; flyers pay 1. It gives Def 0,
Avoid +10. `docs/design/terrain.md` ~:103–106 (Nick, Q2: "2A"): "foot units
can wade at 5, which usually uses a whole turn; mounted and armored units
can't enter."

Where it matters: Chapter 1's Harrowby Ford has a river crossed by
bridges (`assets/maps/ch01.map`); Chapter 2's Kell's Ford is the first
battle built around wading: Hendry holds the ford's north end so his Pack
Hunt hits "whoever comes up out of the water" (`assets/battles/ch02.ron`
~:16–18).

How the guide teaches today: one-time **tips** in `assets/data/tips.ron`
(12 of them, each once per profile, the first time its trigger happens;
triggers in `crates/content/src/tip.rs`, `TipTrigger`) and checklist
**lessons** in `assets/data/lessons.ron`. The only terrain tip is
`terrain` (`FirstTerrain`, "Some tiles cost more movement to cross, and
some can't be crossed at all"), which doesn't say a river can be waded,
what it costs, or who can't. Vix speaks the tips (`docs/design/
tutorial.md`, *The guide*).

## Nick input

None: he asked for it. The tip's words are written by the agent in Vix's
voice (`story-writing` skill; Vix's sheet in `docs/story/characters/`),
and are on the Act 1 playtest's checklist (1064). A tip, not a lesson:
it is one fact with nothing to tick off (*the agent's starting rule*,
written in `tutorial.md`).

## Scope

**In:**
- A trigger `FirstWadeInReach`: a selected unit's move range includes a
  `water` tile it can enter (a foot or flying unit). A doc comment in
  `TipTrigger` and in the header of `tips.ron`.
- Firing it where the move range is built for a selected unit (beside
  where `FirstTerrain` and `FirstEnemyInRange` fire,
  `crates/ui/src/screens/battle/mod.rs` ~:921), in the existing tip queue.
- A tip `wading` in `tips.ron`: at most 3 lines of 60 characters, in
  Vix's voice, saying that foot soldiers can wade a river, that it takes
  most of a turn (Mov 5), that riders and armor can't, and that a unit in
  the water is easy to hit from the bank only if the numbers say so
  (check: Def 0, Avoid +10; say only what is true). Names no key
  (0475's rule, if 0475 is done; if not, still none).
- `tutorial.md`: the tip in its list, and the starting rule above.

**Out (do not do):**
- Changing wading's rules or costs.
- A checklist lesson about water.
- Vix's voice clip for the tip: if 0476 (Vix speaks her tips) is done,
  add the line to its clip list and generate it with the voice tool; if
  not, nothing here (0476 voices every tip that exists then).
- The Playdate's tip box (1058): it shows any tip.

## Implementation steps

1. Read `tip.rs`, `tips.ron`, where `FirstTerrain` fires, and
   `tutorial.md` (*Tips*, *The guide*).
2. Add the trigger and fire it when a selected unit's move range has an
   enterable `water` tile.
3. Write the tip (read Vix's sheet first; American English).
4. Tests; update `tutorial.md`.

## Acceptance criteria

- [x] Selecting a foot unit whose move range reaches a water tile fires
      `FirstWadeInReach` once, and the `wading` tip shows (Harness test
      on a test map with a river).
- [x] Selecting a mounted unit next to water doesn't fire it (test).
- [x] The tip's text fits the tip rules (the existing tip-text checks
      pass) and names no key.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the trigger's condition (foot, flying, mounted, armored).
- Integration: the Harness test above.
- Snapshot: Guides' list of tips, if it has one.

## Completion notes

- `TipTrigger::FirstWadeInReach` (`crates/content/src/tip.rs`, and the
  header of `tips.ron`). It fires from `BattleScreen::detect_wade_tip`,
  right after the state's triggers, when the selected unit's move range
  (`Selection::moves`) has a `water` tile (`doable::wades_in_reach`). It
  is a tip only: no lesson listens. The range already leaves out a tile
  the unit can't enter or can't afford, so a mounted or armored unit never
  fires it, and neither does a foot unit too far from the river to reach
  it this turn. A flying unit fires it (the ticket's "foot or flying").
  `water` is looked up by the terrain's id in the content, so the check
  lives in `ui` (where `FirstTerrain` is), not in `triggers_now`, which
  has no `Ctx`.
- The tip `wading` (face `thinking`, no `how`: nothing to press):
  "Foot units can wade a river, but each tile of water costs 5 Mov, most
  of a turn. Riders and armor can't go in. In the water, a unit gets +10
  AVO and no DEF bonus." True to `terrain.ron` (foot 5, mounted and
  armored none, Def 0, Avoid 10). Fliers (cost 1) aren't mentioned.
- Vix's clip for it (0476 is done): `vix/vix_49c9b883.ogg`, made with
  `cargo xtask voice generate --scene vix`, committed to the private
  repository, pinned. The pin also takes in the private repository's
  commit for 1089 (world map pictures) made before it.
- `tutorial.md`: the tip in its list, and the starting rule below.
- Tests: the condition for foot, flying, mounted and armored
  (`tip_tests.rs`), the screen showing it once for a foot unit and never
  for a mounted one, and a Harness test on the Quick Battle's test map,
  whose archer stands beside its river (`tests/it/battle.rs`). The
  Guides snapshots and two Guides tests that took Terrain for the last
  topic now end on Wading.
- Engine friction: estimate about 20 production lines (like
  `FirstTerrain`'s firing in 0458), measured 19: under 2×. PR size:
  198 changed lines.

**Starting rules (for the Act 1 playtest, 1064):**
- *the agent's starting rule:* one fact with nothing to tick off is a tip,
  not a lesson.
- The tip's wording (Vix's voice; the writer may redo it).
