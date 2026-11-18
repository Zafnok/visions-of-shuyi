---
id: "1237"
title: "Shared test drivers and fixtures: the battle screen's in battle::testing, a tests/it support module, fixtures for the content definitions, and behavior tests on views"
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

- [ ] `grep -rn "fn skip_scenes" crates/ui/tests/it` finds one, in
      `support.rs`.
- [ ] `grep -rnE "^fn (quick|render|step)\(" crates/ui/src/screens/battle/*_tests.rs`
      finds only copies whose body differs, each named in the
      completion notes.
- [ ] `grep -rn "affinities: vec!\[\]," crates/core crates/content`
      finds only the fixture in `crates/core/src/testing.rs`.
- [ ] `SkillDef`, `SpellDef`, `ArtDef` and `BattleDef` test literals
      each name only their own fields and end with `..testing::`; the
      number of local `fn class(` / `fn skill(` / `fn spell(` /
      `fn sword(` test builders is down from about 37, each one left
      named in the completion notes.
- [ ] Step 6 is done, or filed as its own ticket blocked by this one.
- [ ] `tests/it/battle.rs` asserts the banner and the help bar through
      `BattleView`.
- [ ] The same tests pass, none removed (`cargo test --no-fail-fast`).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The existing unit and integration tests, unchanged in what they check.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
