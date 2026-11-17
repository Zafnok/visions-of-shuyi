---
id: "1097"
title: "Aiming at a part: the part bar in the forecast, the glowing part and the combat frame"
type: feature
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["1096"]
nick_input: answer-first
completed:
---

# 1097 — Aiming at a part: the part bar, the glowing part and the combat frame

## Context

Part of the legendary-beasts chain from Nick's storefront-hook search
(2026-11-16; 0085–0088, 0772, 1091–1099, 1275–1281). 1096 makes a beast a
body of parts, each part a unit on its own tile with its own HP and
attack. This ticket is what the player **sees** of that, so a beast
fight reads as a raid and not as a crowd of enemies:

- **You target a part like any enemy.** The forecast adds a **part bar**,
  as Nick approved it in 0087: "Claw 14/20 → 5/20 · If broken: Claw
  Sweep gone · Drops Rime Shard" ("sounds good").
- **The part about to attack glows** before the Beast Phase: only the
  beast's next move, never a preview of the whole enemy phase (Nick
  rejected that).
- **The combat frame** (the combat stage) shows the beast's picture and
  the part being struck or striking.

**The rules come from 0087's answers** in `docs/design/beasts.md`
(question 7: whether the beast shows only its next attacking part
glowing; question 8: the parts). Where they differ from the sketch here,
the decision wins.

**What the game does today:**

- **Aiming** (`crates/ui/src/screens/battle/attack.rs`): `Targeting`
  (~:265) holds the targets in `(y, x)` order and the one under the
  cursor; every check is `BattleState::preview_attack`
  (`crates/core/src/battle/arts.rs` ~:655), which returns an
  `AttackPreview` (~:126). Parts are units (1096), so they are already
  targets.
- **The forecast** (`screens/battle/forecast.rs`, `view` ~:70) builds a
  `ForecastView` (`screens/battle/view/forecast.rs` ~:11, each side a
  `ForecastSideView` ~:54 with its `ForecastHp`: now, after every strike
  hits, max). The PC skin paints it (`screens/battle/glyph/forecast.rs`
  `paint` ~:97, `paint_bar` ~:209), the Playdate's too
  (`screens/battle/playdate/forecast.rs` `paint` ~:111). ADR-0054.
- **The map scene** (`crates/ui/src/map_view/scene.rs`): `UnitView`
  (~:86) already has `highlight` (a battle note's blinking unit, 0411)
  and `flash`; the scene has `clock_ms` for marks that move. Painted by
  the glyph skin (`map_view/glyph/units.rs` `draw_unit` ~:57), the
  picture skin (`map_view/sprite/units.rs`) and the Playdate's
  (`map_view/playdate/units.rs`, `map_view/playdate/marks.rs`).
- **The combat stage** (`screens/battle/stage.rs`, ticket 0413,
  ADR-0056): `stage::picture` (~:54) picks a fighter's battler by its
  character, then its class (`crates/content/src/battler.rs`,
  `assets/battlers/<id>.ron`, at most `MAX_SCALED_PX` 240×144); a
  `FighterView` (`stage/view.rs` ~:39) has the name, HP and numbers.
  Painted by `stage/glyph.rs` and the Playdate's
  `screens/battle/playdate/stage.rs`.
- **The AI** (`crates/core/src/ai.rs`) picks the best-scoring part each
  Beast Phase (1096); nothing commits it to a part in advance.

## Nick input

**Answer first:** ticket 0087 (with its mockups of the part bar and the
glow). Nothing else is asked here. Verified by tests, `cargo shuyi
frame-png` pictures in the PR and the bots; what is left to judge by
playing goes in the Completion notes for the playtest of the first beast
fight (1280).

## Scope

**In:**

- **Core: the readied part** (ADR-0004). The glow must tell the truth, so
  the beast commits to it: each body has a readied part
  (`Body::readied: Option<UnitId>`), set at battle start and at the end
  of the body's action (`Event::PartReadied { body, part }`). In the
  Beast Phase the AI acts with the readied part: its best attack from the
  body's reach if it has one, else it moves the body (with that part as
  the `Act`'s unit) towards its targets and waits. *Claude's starting
  rules*, in `ai.rs`'s module doc with an example: the next part readied
  is the one whose attack scores best on the board as the body's action
  ends (ties: lowest id); if the readied part breaks before it acts, the
  next is readied at once (`Event::PartReadied` again) so the glow is
  never stale. 0087's answer replaces either rule.
- **Core: what the part bar says**, one query the UI reads (e.g.
  `BattleState::part_info(part) -> Option<PartInfo>`): the beast's and
  the part's names, the part's HP and max, the names of the attacks the
  part takes with it (its weapons and arts), whether it is readied, and
  what it drops if 1098 is done (else 1098 adds the drop).
- **The forecast** (ADR-0054): `ForecastView` gets `part:
  Option<PartBarView>` (the part's name, HP now → after every strike
  hits / max, the attacks lost if broken, the drop), built in
  `forecast::view` from `part_info` and the preview's numbers; a part
  the attack would break says so (`breaks`). The target's name is the
  beast's and the part's ("Frost Wyrm · Claw"). Painted by the PC skin
  (a row under the target's side) and the Playdate's (one line, as its
  `paint_side_line` lays out; wrapped by the engine's rule if long,
  1077). Words in `assets/lang/en/ui.ron` (`battle.part.*`).
- **The glow** (ADR-0038: the map scene, painted by every skin):
  `UnitView::readied: bool` (its text: `readied`), shown in every phase
  until the part acts. The glyph skin, the picture skin and the
  Playdate's each paint it as a pulse driven by `clock_ms`, distinct
  from a battle note's blink and from a hit's flash (on the Playdate, a
  one-bit pattern or outline that reads apart from the cursor).
- **The combat frame:** `stage::picture` tries the body's beast id first
  (`assets/battlers/<beast>.ron`), and `FighterView` gets the part's name
  (e.g. `part: Option<String>`) under the beast's; the PC and Playdate
  stage skins show it. A test battler for the test beast is a public
  placeholder (`assets/battlers/`, as `default.ron`); the bought "epic
  monsters" battlers are picked by 1280.
- **The side panel and unit info** show a part as "beast · part" with
  the part's HP (they read the unit; a test that they do).

**Expected size** (ADR-0092, ADR-0101): about 600 production lines (core
~180: the readied part and `part_info`; the forecast view and two skins
~200; the scene field and three skins ~120; the stage ~100) and 600 of
tests, about 1,500 changed lines with the ticket. Comparable: 0489 (the
note on an effective strike) for the forecast, 0411 (battle notes' blink)
for the map mark.

**Out (do not do):**
- What a broken part leaves on the map and carving (1098); the drop line
  shows only if 1098 is done.
- A forecast of the whole enemy or Beast Phase (Nick rejected it).
- The timing ring and Steal the Turn in the combat frame (1093, 1094).
- One beast picture across several map tiles (1096's *Out*).
- Picking the bought battlers and the frost wyrm itself (1280).

## Implementation steps

1. Read 0087's answers and mockups, 1096's code, `ai.rs`'s ordering and
   attack choice, `forecast.rs`, `attack.rs`, `scene.rs`, `stage.rs` and
   the skins named above.
2. Core: `Body::readied`, the event, the AI's commitment and the
   starting rules; `part_info`. Tests that the glowing part is always
   the one that acts.
3. The forecast view and both skins.
4. The scene field and the three map skins.
5. The stage's picture and part name, both skins.
6. Snapshots, `frame-png` pictures, and `cargo xtask playtest
   test_beast` (the readied rule may change the bots' results; say how
   in the PR).

## Acceptance criteria

- [ ] In `test_beast`, the part readied before a Beast Phase is the part
      that attacks in it, over many seeds (a property test), and a part
      broken while readied hands the glow on at once (test).
- [ ] Aiming at a part, the forecast shows its bar: HP now → after / max,
      the attacks it takes with it and "breaks" when every strike hitting
      would break it (view tests, `screens/battle/forecast/tests.rs`).
- [ ] The readied part's `UnitView` says `readied` in every phase until
      it acts (scene tests).
- [ ] `cargo shuyi frame-png` pictures: the forecast on a part (PC and
      Playdate), the glow on the map in the glyph, picture and Playdate
      looks, and the combat stage with a part struck.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the readied part's choice and hand-on, the AI acting with it,
  `part_info`, the forecast view's part bar, the stage's part name.
- Property: over random `test_beast` play, every Beast Phase action is
  made by the part readied before it.
- Snapshot / integration: the forecast on a part in both skins, the map's
  glow in three skins, the stage, the scene's text.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
