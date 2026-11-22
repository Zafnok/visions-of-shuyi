---
id: "1310"
title: "Tip triggers are names, with a predicate row each"
type: infra
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["1291"]
nick_input: none
completed:
---

# 1310 — Tip triggers are names, with a predicate row each

## Context

Split off 1291 by its *Split* clause (ADR-0101): `pr-size` counts a
moved line twice (removed and added), so 1291's ~900 moved lines with
the registry passed 2,000. 1291 moved the guide, the threat display and
the playback effects out of `BattleScreen`, and the trigger checks
(`triggers_now`, `lesson_triggers_now`, the `detect_*` functions,
`note_level_ups`, `has_skill`, `special_terrain`, `no_unit_left`) into
`crates/ui/src/screens/battle/triggers.rs` as an `impl BattleScreen`
block. This ticket turns them into a registry.

From the 2026-11-17 friction sweep
([report](../../docs/research/friction-2026-11-17.md), *Each tip is a
Rust enum variant*): `TipTrigger` (`crates/content/src/tip.rs`) has 22
variants, and a tip's PR is 9 to 20 files and 130 to 300 lines. Most of
its conditions look at the screen (a forecast open, the zoom available),
so core's `TriggerWhen` fits about 3 of them; a registry of named
predicates fits all of them. Outcome 3 of the `engine-friction` skill, a
game-side structure (ADR-0092).

## Nick input

None. Nothing a player sees changes: verified by the tests below.

## Scope

**In:**
- In `battle/triggers.rs`, a registry: one row per trigger, keyed by the
  name `tips.ron` and `lessons.ron` already write (`FirstBattleStart`,
  …), holding when it is checked and its predicate. The checks today run
  at four moments, and each row says which: the battle's state each frame
  (`triggers_now`, also fired to the lessons at the frame's end), the
  state with the content (`detect_wade_tip`: tips' moment only), after
  the lessons took this frame's in (`detect_zoom_tip`), a command's
  events (`note_level_ups`); and the out-of-battle names
  (`FirstLoadGame`, `FirstProfileScreen`) fired by their screens.
- `TipTrigger` in `crates/content` becomes a name type with no list of
  its own (it must still deserialize from a bare RON identifier, as
  `trigger: FirstForecast` reads). The registry is the list of known
  names; a ui test that loads the tips and the lessons fails on a name
  with no row (the content crate can't see the ui's registry). So a new
  tip is a `tips.ron` entry, its text and one predicate row, with no edit
  to `crates/content`. The names stay the same strings, so the tips a
  profile has seen (`profile.rs`, `save.rs`) still read.
- The name type meets the bounds of the engine's `Tip<T>` and
  `TipTable<T>` (`shuyi_assets::tip`). About 100 test lines name
  `TipTrigger::FirstBattleStart` and the rest (`tip_tests.rs` about 50):
  keep those paths compiling (associated constants of the name type), so
  the test files change only where a test is about the registry.
- Lessons (`battle/lessons.rs`) keep their own `DoneWhen` conditions.

**Either order with 1081** (Vix explains the world map): if 1081 lands
first it adds its trigger the old way and this ticket converts it; if
this lands first, 1081 adds a row.

**Out (do not do):**
- Any change to when a tip or lesson shows, or to its text.
- Moving more of `BattleScreen` (1291 did the guide, threat display and
  playback effects).

## Implementation steps

1. Make `TipTrigger` a name: e.g. `pub struct TipTrigger(&'static str)`
   with a `Deserialize` that reads an identifier and maps it to a known
   name, or an interned `Cow`; it stays `Copy` if the queue code relies
   on it (`TipState`, `LessonState`). Add associated constants for every
   name tests use.
2. Build the registry in `battle/triggers.rs` from the functions there:
   `Row { name, when: When, check }`, `When` one of the four moments
   above, `check` a `fn` over the screen (and `&Ctx` or `&[Event]` where
   the moment has one). Fire by walking the rows of a moment, in the
   order the code fires them today.
3. A ui test: every trigger `tips.ron` and `lessons.ron` name has a row,
   and a name with none fails it (a test tip with an unknown name).
4. A test tip in `tests/it` with a new name and a predicate row shows
   with no edit to `crates/content/src/tip.rs`.
5. Add a row for 1310 in `docs/engine/friction.md` (outcome 3).

## Acceptance criteria

- [ ] Adding a tip trigger needs no edit in `crates/content/src/tip.rs`
      (a test tip with a new name and predicate row, in `tests/it`), and a
      name with no predicate row fails a ui test.
- [ ] `ch01_tips`, `ch01_lessons`, `lessons` and `save_files` in
      `tests/it` pass unchanged; `tip_tests.rs` and `trigger_tests.rs`
      change only where a test is about the registry.
- [ ] A profile's seen tips from before still read (a test).
- [ ] A row for 1310 in `docs/engine/friction.md`, outcome 3.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the registry covers every name the tips and lessons use; an
  unknown name fails. Every existing tip and trigger test passes.
- Integration (Harness): a test tip with a new name fires from its row.
- Snapshot: none new; existing ones unchanged.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
