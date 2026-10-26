---
id: "0313"
title: "Save the village: houses, Evacuate, raiders who burn them (core rules)"
type: feature
milestone: M2 Core rules
model: opus-5.5
effort: medium
status: done
blocked_by: ["0308", "0037", "0322"]
nick_input: none
completed: 2026-10-26
---

# 0313 — Save the village: houses, Evacuate, raiders who burn them

## Context

This ticket was "Villages and the Visit action": FE GBA's village a player
unit visits for a one-time gift. Ticket 0069 (2026-10-22) replaced that
rule. Nick: villages in a battle only when the story is about them, "a real
race not an optional one... like save the village with a minimum number
houses allowed to pass, and past that get extra rewards at the end", and no
plain gift villages at all. The rule is in `docs/design/turn-structure.md`
(*Act 1's battle rules*, *Save the village*). Chapter 2, Kell's Ford (0848),
is its first and, in Act 1, only user.

In short: a map marks **houses** and a **minimum** to save. A player unit on
an unsaved house's door uses **`Evacuate`** (its action) and the house is
saved for good. An enemy marked as a **raider** that ends its turn on or next
to an unsaved house's door burns it (lost for good). A house still standing
when the battle is won counts as saved. The battle is lost the moment the
minimum can no longer be met; each house standing past the minimum adds a
reward at the end.

The house tile is decided (0037, 2026-10-22, `docs/design/terrain.md`,
*Building tiles*): a one-tile `house` terrain (cost 1, +10 Avoid, no
healing), drawn as a lean-to shack; "the house's door" in the rule is
that tile. Ticket **0322** adds it to `terrain.ron` and draws it on every
map skin. The UI half (the action menu entry, the objective line) is
ticket 0409.

## Nick input

None: the rule (0069) and the tile (0037) are decided. Verified by tests
and the bots; what is left to judge by playing goes on the Act 1
playtest's checklist (1064).

## Scope

**In:**
- Houses on the map: every `house` terrain tile (0322) is a house. Keep
  its saved / burned state in the battle (a burned house may turn into
  `burnt` terrain or keep a flag: your choice, recorded in the module
  docs). Map validation: a battle with a village goal has at least as
  many `house` tiles as its minimum.
- The battle's village goal: a minimum to save and the extra reward per
  house past it, in the battle file (`assets/battles/README.md`), validated
  (minimum ≤ number of houses, rewards' item ids known, `Gold(0)` refused).
- A unit setting that marks an enemy as a **raider**.
- `UnitAction::Evacuate` for a **player** unit on an unsaved house's door
  (reuse `CommandError::PlayerOnly`; add errors for no house and a house
  already saved or lost). `Event::HouseSaved`, `Event::HouseBurned`.
- Burning: at the end of a raider's action, every unsaved, unburned house
  whose door is the raider's tile or one of the four beside it burns.
- Loss when too many houses are lost for the minimum; on a win, standing
  houses count as saved and each one past the minimum gives its reward
  (gold to the purse, items where chest items go).
- The saved / lost counts in `BattleState` (saved with the battle, like
  opened chests), so the objective line (0409) can show
  `Save at least 3 of 5 houses (1 saved, 1 lost)`.
- The AI: a raider walks toward the nearest unsaved house and prefers
  burning it to attacking (*Claude's starting rule*, tuned by the bots).
- The bots (0504's legal moves) and play records handle `Evacuate` and the
  burning events.
- A test battle in `assets-test/` with houses and raiders.

**Out (do not do):**
- Plain "Visit for a gift" villages: Nick dropped them (0069).
- The UI (0409), the Kell's Ford battle itself (0848), playing a house's
  line (0705's scene triggers do that).

## Implementation steps

1. Read `turn-structure.md` (*Save the village*) and `terrain.md`
   (*Building tiles*, the `house` tile).
2. Map: a lookup of the `house` tiles and their state, and its checks.
3. Battle: `Evacuate`, burning, the loss check (after every action, with the
   other loss conditions), the rewards on a win, through `Command` →
   `Event` (ADR-0004). Document the rule in the `battle.rs` module docs.
4. AI for raiders; extend `legal_commands` in `crates/core/src/battle/tests.rs`
   with `Evacuate` and add houses to `prop_map()`.
5. The test battle; the bots run it (`cargo xtask playtest <test battle>
   --bot all`) without a softlock.

## Acceptance criteria

- [x] `Evacuate` saves a house; each error case leaves the state unchanged
      (`refused_act`).
- [x] A raider ending its action on or next to an unsaved door burns it; a
      saved house never burns.
- [x] The battle is lost the moment the minimum can't be met, and won with
      the extra rewards counted from the standing houses.
- [x] Saved and lost houses survive a RON save/load; play records replay
      identically.
- [x] Map and battle files with houses parse, print back and are validated.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: Evacuate, burning (on the door, beside it, diagonal does not), the
  loss, the rewards; parsing, printing, validation.
- Property: the random-command test in `battle/tests.rs` includes
  `Evacuate`; a saved house never becomes burned.

## Completion notes

Done (2026-10-26).

- **Core** (`crates/core/src/battle/village.rs`, rules in the `battle.rs`
  module docs): `VillageGoal` (houses, minimum, rewards) in `BattleSetup`
  and `BattleDef`; `Village` with each house's `HouseState` in
  `BattleState` (saved with the battle, left out of saves without one);
  `Unit::raider`; `UnitAction::Evacuate` with `CommandError::NoHouse`,
  `HouseAlreadySaved`, `HouseLost` (and `PlayerOnly`);
  `Event::HouseSaved` (`unit: None` for the houses standing at a win),
  `HouseBurned` and `HouseReward`. Burning runs at the end of a raider's
  action, before the outcome check; the loss is checked with the other
  loss conditions; rewards come at a win after the EXP pool's shares,
  taken as a chest's loot (the chest's code was factored into
  `gain_loot`). `BattleState::can_evacuate` and `village()` are there
  for the UI (0409).
- **A burned house turns into `burnt` terrain** (cost 1, no Defense or
  Avoid, drawn as ash) and is marked `Burned` in the village, with an
  `Event::TerrainChanged` after its `HouseBurned`. Nick, after the PR
  opened: "burned house should give no bonuses when standing on". The
  first version kept the `house` tile and its +10 Avoid.
- **Content**: `village: (minimum, rewards)` in battle files (houses =
  every `house` tile, found from the map), `raider: true` on enemies,
  with the checks of the ticket plus: minimum 0, rewards not one per
  house past the minimum, a raider that isn't an enemy or in a battle
  without a village. `assets/battles/README.md` and `assets/maps/README.md`
  document both.
- **AI**: raiders and the player side's evacuation (below); the bots'
  legal moves list `Evacuate`; the persona bots count houses saved less
  burned (`houses`, 500 for every persona, added with the rule rather
  than tuned) and walk towards standing houses as goals.
- **Test battle**: `assets-test/battles/test_village.ron` on
  `assets-test/maps/test_village.map` (five houses, save three, two
  raiders and a brigand). `cargo xtask playtest test_village --bot all`
  ends every try (no softlock): Casual won 81/100, Normal 94/100,
  Hardcore 90/100, median 4 turns. It is a test file, not balanced to the
  bands. It is in the bots' replay test (`crates/bots/tests/baseline.rs`),
  which checks a house burns in some try and every try replays to the
  same events.
- **Tests**: unit tests of Evacuate and its refusals (state unchanged),
  burning on the door, beside it, not diagonally, not by a non-raider,
  not a saved house, after an attack and not by a raider that fell; the
  loss; the rewards (and none past a burned house, none on a defeat);
  RON save/load. The random-command property test has a village and
  raiders (houses in `PROP_HOUSES` beside `prop_map()`, since houses are
  the goal's tiles in core) and checks that a house changes only with its
  event and never after it is saved or burned. AI tests for raiders and
  evacuating; content tests for parsing and every error.
- The rewind list names the action (`battle.rewind.evacuated`, "{who}
  evacuated a house"): the one UI line the new action needed to compile.
  The rest of the UI is 0409.

Engine friction: estimate 380 production lines (judged against the chest
and escape rules, 0308/0319), measured 404 (1.06×), under 2×.

### Claude's starting rules (for the Act 1 playtest, 1064)

- **Raider AI:** while a house stands, a raider (whatever its AI setting)
  goes where it burns one if it can reach such a tile, and attacks from
  there if it can, else waits there (the tile burning the most houses);
  it never skips a burn to attack. Otherwise it attacks if it can, else
  walks towards the nearest standing house.
- **Burning happens only at the end of the raider's own action:** a raider
  pushed (Shove) or arriving as a reinforcement next to a house burns
  nothing until it acts. One offered a move after its attack burns where
  that move ends.
- **Rewards in order:** the battle file lists one reward per house past the
  minimum; with N houses standing past it, the first N rewards are given.
- **The player side's AI (the bots) evacuates** any house it can reach
  before doing anything else (after escaping on an escape map).
- (Already in `turn-structure.md`: "next to" is the four tiles beside the
  door, never diagonals.)
