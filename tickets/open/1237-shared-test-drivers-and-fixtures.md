---
id: "1237"
title: "Shared test drivers and fixtures: the battle screen's in battle::testing, a tests/it support module, a ClassDef fixture, and behavior tests on views"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
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

## Nick input

None. Tests only; nothing a player sees changes.

## Scope

**In:**
- Battle-screen drivers and `p` in `battle::testing`.
- `crates/ui/tests/it/support.rs`.
- A `testing` feature on `trpg-core` with a `ClassDef` fixture.
- The banner/help asserts of `tests/it/battle.rs` and the results
  asserts of `flow.rs` on views.
- Expected size: 0 production lines; about 450-500 test lines removed
  (drivers about 300, `ClassDef` about 200), about 80 added.

**Out (do not do):**
- The frame text helpers (`row`, `shows`, `text`, `help` over cells)
  and the `every_glyph_drawn_is_in_the_font` tests: 1214 replaces them
  with the engine's readers and is blocked by this ticket, so leave
  them where they are.
- `lord_menu` variants whose key steps or return types differ, and
  drivers with another receiver (`Stacked`, `Mode`).
- Look tests and snapshots: they keep reading cells.
- `classes()` and `unit()` fixtures beyond the `ClassDef` literal.

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
   (`"Down f Left f f"`, asserting the screens are `title`,
   `game_flow`, `battle`) and `battle(h) -> &BattleScreen`. Replace
   the copies; one with other steps (`map_zoom.rs`'s skin) stays.
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
5. **Measure** with `cargo xtask code-lines` and `cargo xtask pr-size`.

## Acceptance criteria

- [ ] `grep -rn "fn skip_scenes" crates/ui/tests/it` finds one, in
      `support.rs`.
- [ ] `grep -rnE "^fn (quick|render|step)\(" crates/ui/src/screens/battle/*_tests.rs`
      finds only copies whose body differs, each named in the
      completion notes.
- [ ] `grep -rn "affinities: vec!\[\]," crates/core crates/content`
      finds only the fixture in `crates/core/src/testing.rs`.
- [ ] `tests/it/battle.rs` asserts the banner and the help bar through
      `BattleView`.
- [ ] The same tests pass, none removed (`cargo test --no-fail-fast`).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The existing unit and integration tests, unchanged in what they check.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
