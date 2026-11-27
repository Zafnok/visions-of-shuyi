---
id: "1237"
title: "Shared test drivers and fixtures: the battle screen's in battle::testing, a tests/it support module, fixtures for the content definitions, and behavior tests on views"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-27
---

# 1237 — Shared test drivers and fixtures

## Context

From the game/engine boundary audit of 2026-11-03. Game-only: nothing
here is the engine's. The tests re-define the same drivers file by file
although shared modules exist:

- `screens/battle/mod.rs:2886` has `pub(crate) mod testing`, and
  `battle/tests.rs` exports `quick`, `render`, `step`, `frame` and
  `lord_menu` (`:18-27`, `:228`, `:572-596`). Still the `*_tests.rs`
  files beside them define `quick` 8 times, `render` 13, `step` 11,
  `press` 19, `frame` 15 and `p(x, y) -> Pos` about 13 times.
- `tests/it` has no shared module. `skip_scenes` is copied 5 times
  (`battle.rs:659`, `ch01_lessons.rs:40`, `ch01_tips.rs:21`,
  `threat.rs:43`, `playdate.rs:48`) beside the `pub fn` in
  `playthrough.rs:45`; `quick_battle` 5, `battle(h)` 8, `title()` 7.
- `ClassDef` (`crates/core/src/class.rs:100`, 18 fields, no `Default`)
  is written out in full, about 20 lines, in 15 test modules (one base
  literal each, as in `core/src/ai/tests.rs:64-101`; `grep -rn
  "affinities: vec!\[\]," crates/core crates/content` finds them).
- Some behavior tests read cells where a view answers (AGENTS.md rule
  8): `tests/it/battle.rs:14-57` checks the `PLAYER PHASE` banner with
  `shows(&h, "PHASE")` and the mode with `help(&h)` (row 31), and
  `flow.rs` reads the results screen by cell. `BattleScreen::view`
  (`battle/mod.rs:2406`, `banner`, `help`, `shows`) and
  `ResultsScreen::view` (`screens/results.rs:216`) answer, as
  `ch01_lessons.rs:683` already does.

2026-11-17 friction sweep
([report](../../docs/research/friction-2026-11-17.md), bucket B3 with
0258 and 0187): adding one field to a definition edits dozens of
literals, so the fixture covers every definition, not only `ClassDef`.
Re-counted on 2026-11-17 (`grep -rn "<Type> {" crates`, less the
definitions): `ClassDef` 44 literals in 15 files, `SkillDef` 24 in 11,
`SpellDef` 23 in 8, `ArtDef` 13 in 10, `BattleDef` 9 in 4; none derives
`Default` (`class.rs:101`, `skill.rs:388`, `spell.rs:109`, `art.rs:375`,
`campaign.rs:213`). About 37 local test builders (`fn class(`,
`fn item(`, `fn spell(`, `fn skill(`, `fn sword(`) sit in about 30
files, for example `core/src/art/tests.rs:90`, `spell/tests.rs:71` and
`content/src/spell.rs:544`. #725 (one `ItemDef` field) added
`short_name: None` 10 times across 9 files, and the hook tickets
1095-1099 will add more fields. `trpg-content` has 62 `Raw*` serde
types (`item.rs` 10, `battle.rs` 8, `worldmap.rs` 7); some only turn a
`String` into an id, which the core id types could deserialize
themselves (they derive `Deserialize`).

## Nick input

None. Tests only; nothing a player sees changes.

## Scope

**In:**
- Battle-screen drivers and `p` in `battle::testing`.
- `crates/ui/tests/it/support.rs`.
- A `testing` feature on `trpg-core` with a `ClassDef` fixture, and
  (2026-11-17) fixtures for `SkillDef`, `SpellDef`, `ArtDef`,
  `BattleDef` and a weapon `ItemDef`, replacing the local test builders.
- The banner/help asserts of `tests/it/battle.rs` and the results
  asserts of `flow.rs` on views.
- (2026-11-17) Content `Raw*` structs whose conversion only turns
  strings into ids: deserialize the core type directly (step 6).
- Expected size: 0 production lines but the fixtures; about 450-500 test
  lines removed (drivers about 300, `ClassDef` about 200), about 80
  added. The 2026-11-17 additions: about 900 test lines removed and 150
  added (the other definitions and the builders), and step 6 about
  400-600 more. Plan about 2,000 without step 6: if the measured size
  before step 6 passes about 1,500, file step 6 as its own ticket
  (`write-ticket`, blocked by this one) instead of doing it here.

**Out (do not do):**
- The frame text helpers (`row`, `shows`, `text`, `help` over cells)
  and the `every_glyph_drawn_is_in_the_font` tests: 1214 replaces them
  with the engine's readers and is blocked by this ticket, so leave
  them where they are.
- `lord_menu` variants whose key steps or return types differ, and
  drivers with another receiver (`Stacked`, `Mode`).
- Look tests and snapshots: they keep reading cells.
- `classes()` and `unit()` fixtures beyond the definition literals.
- Deriving `Default` on a production type whose zero value isn't a
  sensible game value (the fixtures live in `testing`).
- The serde mirror enums of `content/src/enums.rs`: 1230 deletes them.
  Either order: step 6 skips any `Raw*` type 1230 already removed.

**Either order with 1232:** if 1232 is done, the `tests/it` files it
deleted need no change here; if not, change them here as well, and
1232 deletes them with the shared drivers they now use.

**Either order with 1231:** if 1231 is done, skip the content test
files it deleted; if not, `crates/content/src/skill/tests.rs` moves to
the fixture here.

## Implementation steps

1. **Battle drivers.** Move `battle/tests.rs`'s `quick`, `render`,
   `step` and `frame` into `battle::testing` as `pub(crate)`, with a
   `pub(crate) fn p(x: i32, y: i32) -> Pos`. Delete each per-file copy
   whose body is the same (`ai_phase`, `art`, `item`, `magic`, `skill`,
   `tip`, `turn`, `scene`, `heal_aim`, `threat` `_tests.rs`). A copy
   that differs (another fill cell, held keys) stays unless its tests
   pass unchanged on the shared one.
2. **tests/it support.** Add `support.rs` (`mod support;` in
   `main.rs`) with `skip_scenes` (moved from `playthrough.rs`),
   `title()` (`Harness::with_layout(RIGHT_HANDED)`), `quick_battle()`
   (`"Down f Left f f f"`, asserting the screens are `title`,
   `game_flow`, `battle`) and `battle(h) -> &BattleScreen`. Replace
   the copies; one with other steps (`map_zoom.rs`'s skin) stays.
   1289 (blocked by this ticket) then replaces `quick_battle()` with
   `Harness::quick_battle()`.
3. **ClassDef fixture.** In `crates/core`: feature `testing = []`;
   `#[cfg(any(test, feature = "testing"))] pub mod testing` with
   `pub fn class(id: &str) -> ClassDef` (tier 1, move 4, default stats
   and growths, no weapons, 3 weapon slots, empty lists, flags false).
   `trpg-content` and `trpg-core`'s own `tests/it` take it as a
   dev-dependency feature. Rewrite each test literal as
   `ClassDef { <its own fields>, ..testing::class("fighter") }`; the
   test `class(id)` in `core/src/unit.rs:438` goes.
4. **Views, not cells.** In `tests/it/battle.rs`, the banner and help
   asserts read `support::battle(&h).view(h.game().ctx())` (`banner`,
   `help`); in `flow.rs`, the results screen's asserts read its view.
   Keep cell reads that check the whole frame end to end, and say so in
   a comment.
5. **The other definitions** (2026-11-17). Beside `testing::class`, add
   `skill(id)`, `spell(id)`, `art(id)`, `battle(id)` and `sword(id)`
   (a weapon `ItemDef`), each the smallest valid value, and rewrite
   each test literal as `<Type> { <its own fields>, ..testing::x("id") }`.
   Replace each local builder whose body is one of these with a call;
   one that sets more stays as a thin wrapper over the fixture. A
   builder in `trpg-content`'s or `trpg-ui`'s tests uses the same
   dev-dependency feature.
6. **Raw structs that only convert ids** (2026-11-17). List the `Raw*`
   types of `crates/content/src` and, for each whose conversion only
   wraps strings into core ids (no lookup, no check, no default worked
   out from other fields), deserialize the core type directly and
   delete the `Raw*` type and its conversion. Keep any `Raw*` type
   whose conversion checks or looks something up, and list the kept
   ones with their reason in the completion notes. Content errors must
   keep their file and line (`cargo test -p trpg-content` covers them).
   See the size note in *Scope* before starting this step.
7. **Measure** with `cargo xtask code-lines` and `cargo xtask pr-size`.

## Acceptance criteria

- [x] `grep -rn "fn skip_scenes" crates/ui/tests/it` finds one, in
      `support.rs`.
- [x] `grep -rnE "^fn (quick|render|step)\(" crates/ui/src/screens/battle/*_tests.rs`
      finds only copies whose body differs, each named in the
      completion notes.
- [x] `grep -rn "affinities: vec!\[\]," crates/core crates/content`
      finds only the fixture in `crates/core/src/testing.rs`.
- [ ] `SkillDef`, `SpellDef`, `ArtDef` and `BattleDef` test literals
      each name only their own fields and end with `..testing::`; the
      number of local `fn class(` / `fn skill(` / `fn spell(` /
      `fn sword(` test builders is down from about 37, each one left
      named in the completion notes.
- [x] Step 6 is done, or filed as its own ticket blocked by this one.
- [x] `tests/it/battle.rs` asserts the banner and the help bar through
      `BattleView`.
- [x] The same tests pass, none removed (`cargo test --no-fail-fast`).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The existing unit and integration tests, unchanged in what they check.

## Completion notes

**Done.**
- *Battle drivers:* `battle::testing` has `p`, `quick`, `step` and `frame`,
  and re-exports `render`. `render` itself stays in `battle/tests.rs`
  (now `pub(crate)`): `title/tests.rs`'s
  `the_logic_modules_name_no_cells_or_colours` forbids the word `Cell` in
  `battle/mod.rs`, and the render fills its buffer with stale cells.
  Deleted the copies in `ai_phase`, `art`, `item`, `magic`, `scene`,
  `skill`, `tip`, `turn`, `heal_aim` and `threat` `_tests.rs` (and the
  inner `p` of `attack.rs`, `mode.rs` and `path.rs`). The blank-filled
  `render`s of `ai_phase`, `tip` and `turn` went too: their tests pass on
  the stale-filled one. `press` of `ai_phase`, `lesson` and `tip` was the
  same as `step` and its calls are now `step` (method calls `.press` on
  `Stacked` untouched).
- *Copies kept, their bodies differ:* `lesson_tests.rs` `quick` (puts the
  screen in `QUICK_BATTLE`), `threat_tests.rs` `quick(c)` (another
  receiver) and `press` (an `FRAME_DT` frame per action), `tip_tests.rs`
  `frame` (no `held`), `turn_tests.rs` `frame`/`press` and
  `notes`/`trigger`/`scene_voice` `frame` (return the transition as
  text), `progress`/`zoom` `press` (frame timing), `rewind_tests.rs`
  `press` (`Stacked`), `help_bar_tests.rs` `frame(h)` (a harness).
- *tests/it support:* `support.rs` has `title`, `quick_battle`,
  `skip_scenes` (moved from `playthrough.rs`), `battle(h) ->
  &BattleScreen` and `state(h) -> BattleState` (the four copies that
  cloned the state: `flow`, `playthrough`, `preparations`, `save`, named
  `battle` there by `use … state as battle`). `threat.rs` and
  `sound_cues.rs` keep `quick_battle` as thin wrappers (a cursor assert,
  `clear_audio`). Kept: `battle.rs` `quick_battle` (closes the banner
  too), `ambience`, `debug_view` and `map_zoom`'s (other setup),
  `language.rs`'s `title`/`quick_battle` (the test language),
  `focus.rs` and `title_cinematic.rs`'s `title` (a wait, the test
  title), `hot_reload.rs`/`sessions.rs` `title` (not a harness).
  `playdate.rs`'s `skip_scenes` presses pad buttons; it is renamed
  `pad_through_scenes`.
- *Fixtures:* `trpg-core` feature `testing` and
  `#[cfg(any(test, feature = "testing"))] pub mod testing` with
  `class`, `skill`, `spell`, `art`, `battle`, `sword_def` (a weapon's
  `WeaponDef`, the type most literals build) and `sword` (it as an
  `ItemDef`). `trpg-content` and `trpg-core`'s own tests take the
  feature as a dev-dependency. 56 test literals (47 in `trpg-core`, 9 in
  `trpg-content`) now name only their own fields and end with
  `..testing::x(id)`; the rewrite dropped exactly the fields equal to the
  fixture's value. Literals that already ended with a base of their own
  (`..base`, `..def`) were left as they are.
- *Views:* `tests/it/battle.rs` reads the banner (`banner(h)`: title,
  subtitle and objective lines) and the help line from `BattleView`; its
  `wait_for` became `wait_for_banner`. `flow.rs` reads the results'
  gold, rewinds, bonus and EXP bars from `ResultsView`'s tally; the
  frame snapshot stays, commented as the end-to-end check.

**Deviations.**
- *Step 6 filed as 1392* (blocked by this ticket): the branch measured
  1,747 changed lines before it, past the 1,500 *Scope* sets.
- *Builders not fewer:* every local builder (`fn class(`, `fn skill(`,
  `fn spell(`, `fn weapon(`, …) sets more than its fixture, so each
  stays as a thin wrapper over it (step 5's rule): 29 before and after.
  Left: content `skill/ranks/tests.rs` `skill`, `skill/tests.rs`
  `class`, `spell.rs` `heal` and `class` (the latter reads loaded
  classes); core `ai/tests.rs` `strike`, `art/tests.rs` `art`, `weapon`,
  `class`, `battle/tests.rs` `class`, `test_rending_arrow`,
  `test_skill`, `sword`, `seal`, `campaign/tests.rs` `def`,
  `item/tests.rs` `class`, `mage_class`,
  `progression/class_change/tests.rs` `def`, `progression/tests.rs`
  `swordsman`, `custom`, `shop/tests.rs` `weapon`, `armour`,
  `skill/tests.rs` `passive`, `active`, `class`, `spell/tests.rs`
  `attack`, `heal`, `unit.rs` `class`, `tests/it/replay.rs` `blade`;
  ui `battle/shop/tests.rs` `item` (reads loaded items). The ticket's
  "about 37" counted the builders of other types too.
- `unit.rs`'s test `class(id)` stays as a thin wrapper rather than going:
  its tests read the Brigand stats and weapons it sets.
- 1231 and 1232 aren't done: their files were changed here as well.

**Size.** Engine friction: estimate 110 production lines (six struct
literals of the definitions, one per fixture), measured 122 (1.1×),
under 2×. PR size: estimate 2,000 (the ticket's), measured 1,958 (with this ticket and 1392).

**Follow-ups:** 1392 (step 6). Nothing for Nick: tests only.
