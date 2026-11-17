---
id: "1095"
title: A beast army with its own Beast Phase
type: feature
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0087", "1064"]
nick_input: answer-first
completed:
---

# 1095 — A beast army with its own Beast Phase

## Context

Nick's storefront-hook search (2026-11-16, five research rounds) ended on
legendary beasts hidden at the bottom of dungeon delves and fought as a
raid. The chain is 0085–0088, 0772, 1091–1099 and 1275–1281. This ticket
is the first brick of the beast fights: **a beast is a third army,
hostile to everyone, with its own Beast Phase**. Several tiles and
breakable parts come next (1096), then aiming at a part (1097) and what a
broken part leaves (1098). The first real beast is 1280's.

**The rules come from 0087's answers** in `docs/design/beasts.md`
(question 7: where the Beast Phase falls, whether the rival enemy army
fights the beast; question 8: the turn limit before it retreats deeper).
Where they differ from the sketch here, the decision wins. The story's
framing is decided by 0087 and recorded in the private repository's
`story/design/` (ticket 0772); nothing of it goes in this repository.

**What the game does today:**

- **Factions** (`crates/core/src/unit.rs` ~:25): `Player`, `Enemy`,
  `Ally`, `Neutral`. `Faction::is_hostile_to` (~:39, ticket 0302) makes
  Enemy hostile to Player and Ally; Neutral is hostile to nobody.
  `is_allied_to` (~:49) feeds heal targets.
- **Phases** (`crates/core/src/battle.rs`, module doc ~:12 and `Phase`
  ~:533): `Player → Enemy → Other`, `Phase::ALL`, `Phase::of(faction)`,
  `Phase::next`. `end_phase` (~:3682) ends the turn when the phase *is*
  `Other` (`if self.phase == Phase::Other`); `start_phase` (~:3704)
  readies the phase's units, burns out tiles, expires effects and brings
  reinforcements; a phase with no units is skipped. Design:
  `docs/design/turn-structure.md`, *Sides and phases* and *Exact
  sequence*.
- **Outcome** (`judge` ~:3753): `Rout` is won when no `Enemy` is left;
  `DefeatUnit` when that unit is defeated. `flee` (~:3792) moves every
  `Enemy` left into `fled` with `Event::Fled`. Unit EXP (`award`
  ~:3814) matches on the faction (`Enemy | Neutral` get nothing).
- **The AI** (`crates/core/src/ai.rs`, module doc ~:1–30): `next_command`
  "works in any phase"; its targets are the units `is_hostile_to` the
  actor, so a faction hostile to everyone attacks everyone with no new
  AI code. `may_use_arts` (`crates/core/src/art.rs` ~:434) matches on the
  faction. Movement, the danger zone and threat arrows
  (`crates/core/src/movement.rs`, `danger_zone`, `threats_at`) work from
  `is_hostile_to` too.
- **Content** (`crates/content/src/battle.rs`): `RawEnemy.faction`
  (~:194) is `Enemy` or `Ally`; `BattleLoader::faction` (~:974) refuses
  any other.
- **The screen** (`crates/ui`): `ai_phase()` (`screens/battle/mod.rs`
  ~:1382) plays every phase but the player's with
  `screens/battle/ai_phase.rs`; the banner (`banner.rs` ~:60,
  `battle.banner.*_phase`), the phase colour (`glyph.rs` ~:515), the
  faction's colour (`units.rs` `faction_color` ~:11), its name
  (`panel.rs` `faction_name` ~:15), the sprite outline
  (`map_view/sprite/units.rs` `outline_color` ~:34), the Playdate's
  inks (`map_view/playdate/units.rs` ~:145), the scene's text
  (`map_view/scene.rs` `unit_text` ~:251), the rewind's "phase ended"
  line (`rewind.rs` ~:442) and an effect's "until … phase"
  (`skills.rs` ~:264) all match on `Faction` or `Phase`.
- **The bots** (`crates/bots/src/persona.rs` ~:164, ~:281, ~:434) count
  "enemies" as `u.faction == Faction::Enemy`; `playtest.rs` ~:119 gives
  each AI phase a `BaselineBot`.

## Nick input

**Answer first:** ticket 0087 (the Beast Phase's place, whether the
rival column and the beast fight, the retreat's turn limit). Nothing else
is asked here. Verified by tests, `cargo shuyi frame-png` pictures in the
PR and the bots; what is left to judge by playing goes in the Completion
notes for the playtest of the first beast fight (1280).

## Scope

**In:**

- **Core** (ADR-0004: state changes only through a `Command` and its
  `Event`s):
  - `Faction::Beast`, hostile to every other faction and not to itself
    (Neutral too, unless 0087 says otherwise), allied to nobody else.
  - `Phase::Beast`, in `Phase::ALL` where 0087 puts it (the sketch:
    after Enemy, before Other). `Phase::of(Faction::Beast)`, `next`,
    and `end_phase` ending the turn after the **last** of `Phase::ALL`
    instead of naming `Other`.
  - Every match on `Faction` or `Phase` handles the new arm: EXP
    (`award`: a beast gets none), `may_use_arts` (as 0087 says; a
    starting rule: yes, a beast is a boss), `judge` (*starting rule*:
    `Rout` counts only `Enemy`; a beast fight is a `DefeatUnit`), and
    `flee`.
  - **The retreat** (0087 question 8): a beast unit may carry the turn
    it retreats on (e.g. `Unit::retreats_on: Option<Turn>`, left out of
    a save when `None`, ADR-0039). At the start of its own phase on that
    turn, after burn-outs and before it acts, it leaves the map into
    `fled` (`Event::Fled`, the existing event). The battle then ends as
    0087 says. *Starting rule:* if a `DefeatUnit` target retreated, the
    battle ends at once (`Event::BattleEnded`) as a `Victory` whose
    `fled()` names the beast, so a caller (the hunt, 1279; the lair,
    1280) can tell "it got away" from "it was beaten" without a new
    `Outcome`.
- **Content:** `faction: Beast` on a battle file's unit entry (the
  checks in `BattleLoader::faction`: a beast is never a green unit and
  carries no items), and `retreats_on`. A test battle,
  `assets-test/battles/test_beast.ron` (on `maps/test_small.map`): the
  test army, a few `test_brigand`s and one single-tile beast built from
  an existing generic template (`test_frost_elemental`), so the three
  sides fight. Its row in `assets-test/README.md`.
- **UI:** the Beast Phase is an AI phase (`ai_phase()` already covers
  it): its banner (`battle.banner.beast_phase`), its colour, the beast
  faction's colour (a new `UiColor::Beast`, with its name in
  `assets/data/palette.ron` and every palette in `assets/data/palettes/`),
  its name (`battle.faction.beast`), the sprite outline, the Playdate's
  one-bit look (a look of its own, not the enemy's inverted ink: it must
  read apart from the enemy on one bit), the scene's text (`beast`), the
  rewind and "until" lines. UI text in `assets/lang/en/ui.ron` (and the
  test pack `assets-test/lang/test/`); other languages through
  `cargo shuyi lang-status` as usual.
- **The bots:** "enemies" in `persona.rs` become the units hostile to the
  player (`is_hostile_to`), and the Beast Phase gets a `BaselineBot` in
  `playtest.rs`. If 1180 is done, the same through the engine's playtest
  command's targets instead; if not, 1180 keeps the hostility rule.
- **Docs:** `turn-structure.md`'s table and *Exact sequence* get the
  Beast Phase if 0087 didn't add them; `battle.rs`'s and `ai.rs`'s module
  docs.

**Expected size** (ADR-0092, ADR-0101): about 350 production lines
(core ~150, content ~40, UI and skins ~120, bots ~40) and 500 of tests,
about 1,200 changed lines with docs and the ticket. Comparable: 0513
(green units' own items), a faction-wide rule through the same files.

**Out (do not do):**
- Beasts covering several tiles, parts and breaking them (1096).
- The glowing next part, the part bar, the beast's combat picture (1097).
- What a broken part leaves, materials (1098, 1099).
- Steal the Turn in the Beast Phase: **either order with 1094**. If 1094
  is done, the stolen move works when a beast's blow is parried (a test
  here); if not, 1094 handles `Phase::Beast` (a line in 1094 says so).
- The hunt, the delve and the lair (1277–1281); a beast in a story
  chapter.
- A Beast Phase tip or lesson (guide text waits for the writer, 0766).

## Implementation steps

1. Read 0087's answers in `docs/design/beasts.md`, then the files in
   *Context*. `grep -rn "Faction::\|Phase::" crates --include=*.rs`
   lists every match to extend; the compiler finds the exhaustive ones,
   the grep finds the `==` comparisons (`persona.rs`, `flee`).
2. Core: the faction, its hostility table (extend the 4×4 table test in
   `unit.rs` ~:503 to 5×5), the phase and the turn's end, the arms, the
   retreat. Write the starting rules in `battle.rs`'s module doc as
   *Claude's starting rule* with an example.
3. Content: the faction and `retreats_on` in `RawEnemy`, its checks,
   `test_beast.ron`.
4. UI: the colour, texts, banner, skins; snapshots and pictures.
5. Bots: hostility, the Beast Phase's player; run `cargo xtask playtest
   test_beast` and the Act 1 bots to check nothing else moved.
6. Docs.

## Acceptance criteria

- [ ] In `test_beast`, a turn runs Player → Enemy → Beast → Other (or
      0087's order): `Event::PhaseStarted { phase: Beast }` comes in its
      place, and a turn with no beast left skips it (core tests).
- [ ] The beast attacks a player unit and an enemy; an enemy attacks the
      beast (core tests on `next_command`).
- [ ] The danger zone and threat arrows count the beast (tests through
      `danger_zone` and `threats_at`).
- [ ] A beast with `retreats_on: Some(5)` leaves at the start of its phase
      on turn 5 (`Event::Fled`), and the battle ends as 0087 decided
      (tests); a save made before and loaded after keeps it.
- [ ] Older saves load unchanged (a save fixture test).
- [ ] `cargo shuyi frame-png` pictures of the Beast Phase banner and of
      the beast on the map, in the glyph, picture and Playdate looks.
- [ ] `cargo xtask playtest test_beast` ends with each bot's result in
      the PR; the Act 1 bots' numbers don't move.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: hostility and alliance for every pair, phase order and skips,
  the turn's end, EXP, `judge`, the retreat, content checks.
- Property: in random battles with a beast (the random-play tests of
  `legal.rs`), a turn never passes without a phase starting, and no
  command lets a unit attack a unit of its own faction.
- Snapshot / integration: the banner, the scene's text with a beast, the
  three skins; the save round trip; the bots on `test_beast`.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
