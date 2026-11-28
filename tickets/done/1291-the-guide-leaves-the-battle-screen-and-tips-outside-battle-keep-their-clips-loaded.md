---
id: "1291"
title: "The guide leaves the battle screen, and tips outside battle keep their clips loaded"
type: infra
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: []
nick_input: none
completed: 2026-11-21
---

# 1291 — The guide leaves the battle screen, and tips outside battle keep their clips loaded

## Context

From the retro of PRs #616–#735 ([T25](../../docs/research/retro-2026-11-17-prs-616-735.md#t25-pull-the-guide-tips-lessons-guide-voice-out-of-battlescreen-and-share-it-with-tipscreen)).
`crates/ui/src/screens/battle/mod.rs` is 3,595 lines with 192 functions,
and 12 commits touched it in one week (#663, #664, #667, #672, #720,
#726, #728). The guide (tips, lessons, the lessons' end box, her voice)
is spread through it: seven fields (`:406`–`:421`), about 465 lines of
methods (`note_level_ups` `:893` to `tick_lessons` `:1357`), and hooks in
`overlay_key` (`:821`), `update` (`:3113`), `covered`/`uncovered` and
`leaving` (`:2974`).

`TipScreen` (`crates/ui/src/tips/screen.rs`, ticket 0874/0497) says its
tip with `preload_voices` in `once` and `play_voice` on the first frame.
Battle dropped that pattern in 0494/0496 (#672): `guide_voice.rs:7` says
a clip not loaded 0.3 s after it is asked for is skipped, and on the web
it is fetched from the host. So the Load Game and Profile tips may be
silent on the web. That is read from the code, not seen. `TipScreen`
also reaches into the battle: `TipState::shown_in` and the test fixture
`screens::battle::guide_voice::tests::voiced()`.

This is the `engine-friction` skill's outcome 3, a game-side structure
(ADR-0092): nothing here names an engine gap. Not on the Act 1 critical
path.

2026-11-17 friction sweep ([report](../../docs/research/friction-2026-11-17.md),
*`BattleScreen` takes most of the UI work* and *Each tip is a Rust enum
variant*): `mod.rs` is 3,596 lines and `BattleScreen` has 41 fields; 70
of the last 121 UI commits touched `screens/battle/`, 36 of them
`mod.rs`. Two more clusters move out with the guide. **Threat display**,
4 fields: `danger`, `threats`, `planner`, `areas` (`self.` 19 times in
`mod.rs`; #526, #593, #610). **Playback effects**, 6 fields: `popups`,
`flashes`, `cues`, `walked`, `pace`, `ai_speed` (with `queue`, which
stays: see *Out*). And each tip is still a Rust enum
variant: `TipTrigger` (`crates/content/src/tip.rs:60`) has 22, and a
tip's PR is 9 to 20 files and 130 to 300 lines. Most of its conditions
look at the screen (a forecast open, the zoom available), so core's
`TriggerWhen` fits about 3 of them; a registry of named predicates in
this ticket's `triggers.rs` fits all of them.

## Nick input

None. Nothing a player sees changes. Whether the Load Game tip is heard
on the web build goes on the Act 1 playtest's checklist (1064) as a line
in the completion notes.

## Scope

**In:**
- A `Guide` in `screens/battle/guide.rs` holding the guide's state and
  logic; the trigger checks in `screens/battle/triggers.rs`.
- `GuideVoice` and its `voiced()` fixture moved under `crate::tips`;
  `TipScreen` on `GuideVoice` and `keep_voices`.
- The title keeps the out-of-battle tips' lines loaded.
- `ThreatView` (the 4 threat-display fields and their methods) in
  `screens/battle/threat.rs`, and the playback-effect fields above with
  their per-frame ticks in a struct of their own (e.g. `Effects` in
  `screens/battle/effects.rs`; `Playback` and `playback.rs` are already
  the combat timeline of 0404), each its own struct as `Guide` is;
  `BattleScreen` routes to them.
- **Tip triggers by name.** In `battle/triggers.rs`, a registry of named
  predicates (`fn(&BattleScreen) -> bool`, or over `Moment` and the
  state), one row per trigger, keyed by the name `tips.ron` already
  writes (`FirstBattleStart`, …; lessons name them too).
  `TipTrigger` in `crates/content` becomes a name type with no list of
  its own: the registry is the list of known names, and a ui test that
  loads the tips and the lessons fails on a name with no row (the
  content crate can't see the ui's registry). So a new tip is a
  `tips.ron` entry, its text and one predicate row, with no edit to
  `crates/content`. The names stay the same strings,
  so the tips a profile has seen (`profile.rs`, `save.rs`) still read.
  The name type meets the bounds of the engine's `Tip<T>` and
  `TipTable<T>` (`tip.rs:122`–`125`). About 100 test lines name
  `TipTrigger::FirstBattleStart` and the rest (`tip_tests.rs` 52): keep
  those paths compiling (e.g. associated constants of the name type),
  so the test files change only where a test is about the registry.
  Lessons (`battle/lessons.rs`) keep their own `DoneWhen` conditions.
- A row in `docs/engine/friction.md` (outcome 3).

**Split** (ADR-0101): with the two structs and the registry this ticket
is about 2,000 changed lines. If the estimate passes 2,000 before
coding, the trigger registry goes to a new ticket (`write-ticket`,
`blocked_by` this one) and this ticket keeps the three structs.

**Either order with 1081** (Vix explains the world map, which is waiting
on Nick's decision anyway): if 1081 lands first it adds its trigger the
old way and this ticket converts it with the rest; if this ticket lands
first, 1081 adds a predicate row. 1081 need not wait.

**Out (do not do):**
- Battle notes, banners, the objective page and the scene queue
  (`queue: VecDeque<Queued>`): they are the battle's flow, not the guide
  or the playback. (The retro's T25 counted the objective and notes
  with the guide; they stay out here to keep this PR's size.)
- The scenes' first voice lines kept from the title and each save
  (1293). Either order works, but the engine keeps one set: if 1293 is
  done, add the tips' lines to its set; if this lands first, 1293 adds
  its lines to this one's.
- Nick's new tip timing (1292). Either order: if 1292 is done, move its
  smaller logic; if not, 1292 edits the moved code.
- Moving `mod testing` (`mod.rs:3176`) to its own file; 1237's shared
  drivers. Either order with 1237.
- The lessons' end box as the engine's `Ask`: 1185 item 5. Either order:
  move whichever form is on main.
- The lessons' Chapter 1 rule (0485) and Vix's arts lesson (0490). Either
  order: move whichever code is on main; their text says where it lives
  after this ticket.
- Harness helpers and battle text read from the view (1289), and snapshot
  crops (1290). Either order: if 1289 is done, its
  `BattleScreen::takes_map_input` asks the `Guide` whether a tip or
  lesson box is up; the test files below change in this PR only as
  stated.
- Any change to when a tip or lesson shows, or to its text.

## Implementation steps

1. **Voice under tips.** Move `screens/battle/guide_voice.rs` to
   `crates/ui/src/tips/voice.rs` (`pub(crate) mod voice;` in `tips.rs`).
   Put `voiced()` in a `#[cfg(test)] pub(crate) mod testing` there. Move
   `TipState::shown_in` (`battle/tips.rs:62`) to `crate::tips::shown_in`.
   After this, `crate::tips` names nothing in `screens::battle`.
2. **`Guide`.** In `battle/guide.rs`, `pub struct Guide` holds `tips`,
   `tip_mode`, `lessons`, `voice: GuideVoice`, `end_focus`, `doable` and
   `noticed`. It replaces those seven fields of `BattleScreen`.
   `battle_id` stays on the battle and is passed in.
3. **The moment as data.** `due_tip`, `lesson_moment` and
   `lessons_end_shown` read battle state (`playing()`, `progress`,
   `queued_scene()`, `notes_open()`, `banner()`, phase, outcome, the mode
   being `Idle`, `queue.is_empty()`). Add `fn moment(&self) -> Moment` on
   `BattleScreen`, a small `Copy` struct of those bools, and pass it to
   the `Guide`. Then move `shown_tip`, `due_tip`, `take_in_tips`,
   `guide_line`, `coming_guide_lines`, `tip_view`, `lesson_moment`,
   `shown_lesson`, `lesson_view`, the `lessons_end_*` functions,
   `absorb_lessons`, `settle_lessons`, `notice*` and `tick_lessons` into
   `Guide`. Pass `&BattleState` where they need it.
4. **Hooks.** `Guide::end_frame` takes the code in `update`
   (`mod.rs:3116`–`3125`). `Guide::covered` and `Guide::uncovered` take
   `covered`/`uncovered`, and `Guide::leave` takes `leaving`. The tip and
   end-box branches of `overlay_key` become `Guide::take_key`. The notes
   branch stays.
5. **Triggers.** Move `triggers_now`, `lesson_triggers_now`, the
   `detect_*` functions, `note_level_ups`, `has_skill`, `special_terrain`
   and `no_unit_left` to `battle/triggers.rs` as an `impl BattleScreen`
   block. A child module may read the parent's private fields. Keep the
   public readers the tests use (`shown_tip`, `shown_lesson`,
   `lesson_open`, `lesson_ticks`, `lessons_end_shown`) as one-line
   forwards. `ch01_tips.rs`, `ch01_lessons.rs` and `lessons.rs` in
   `tests/it` must not change.
6. **`TipScreen` on `GuideVoice`.** Drop `said`/`speaking` and the
   `preload_voices` in `once`. Its first frame calls
   `voice.show(Some(line), Some(vec![line]), ctx)`, which keeps the line,
   then plays it. Closing it calls `voice.leave(ctx)`.
7. **Load earlier.** While `TitleScreen` (`screens/title.rs`) is on top
   and tips show, it keeps the lines of the unseen `FirstLoadGame` and
   `FirstProfileScreen` tips through its own `GuideVoice`. Call
   `covered()` when a screen goes over it. The clip is then loaded before
   Profile or Load Game opens its box. A later `keep_voices` from that box
   replaces the set; the title sends it again when it is back on top.
8. **Threat display and playback.** Move the 4 threat fields into
   `ThreatView` (`threat.rs`) with the code that fills and drops them
   (the danger zone, the kept threat arrows, the planner, the areas);
   move the playback-effect fields into their struct (`effects.rs`) with
   their per-frame ticks. `BattleScreen` keeps one field of each.
9. **Trigger registry.** Turn `TipTrigger`'s variants into names; build
   the registry in `triggers.rs` from the `detect_*` functions moved in
   step 5; a ui test fails on a name the tips or lessons use that has
   no row.
10. **Measure.** Expected size: about +140 production lines net
   (`Moment`, the forwards, the title's keep, the two structs' routing,
   the registry), with about 900 lines moved. About 2,000 changed lines
   in `cargo xtask pr-size` (see *Split*). Add the friction row.

## Acceptance criteria

- [x] `wc -l crates/ui/src/screens/battle/mod.rs` is at least 700 lines
      under 3,595. *Met against the file as this ticket found it:* it had
      grown to 3,886 lines on main (1286, 1304, 1306, 1307) and is now
      2,993, 893 lines fewer. Against the 3,595 written here it is 602
      under (see the notes).
- [x] `BattleScreen` has no `danger`, `threats`, `planner`, `areas`,
      `popups`, `flashes`, `cues`, `walked`, `pace` or `ai_speed` field.
- [ ] Adding a tip trigger needs no edit in `crates/content/src/tip.rs`
      (a test tip with a new name and predicate row, in `tests/it`), and a
      name with no predicate row fails a ui test. *Split off to 1310, as
      Split allows.*
- [x] `grep -nE "fn (due_tip|coming_guide_lines|lesson_moment|lessons_end_key|settle_lessons|tick_lessons|notice_mode|triggers_now)" crates/ui/src/screens/battle/mod.rs`
      finds nothing.
- [x] `grep -rn "screens::battle" crates/ui/src/tips` finds nothing.
      *But one line:* `tips/glyph.rs` imports `battle::layout::MAP_VIEW`,
      the console's map region the tip box is placed over (the results
      screen and the map skins' tests import it too). It names no battle
      logic; moving the layout module is not this ticket's. The voice, its
      fixture, `shown_in` and `EndAnswer` (which `tips/view.rs` also
      named) are under `crate::tips` now.
- [x] `TipScreen` sends `KeepVoices [line]` before `PlayVoice line`. On
      Confirm it sends `StopVoice` then `KeepVoices []`. With voices off
      it sends nothing.
- [x] The title's first frame, with tips and voices on, keeps both
      out-of-battle tip lines. It keeps only the unseen ones, and nothing
      with tips off. (And only where their boxes can show: see the notes.)
- [x] `ch01_tips`, `ch01_lessons`, `lessons` and `save_files` in
      `tests/it` pass. The first three files are unchanged; `save_files.rs`
      only gains the test below.
- [x] A row for 1291 in `docs/engine/friction.md`, outcome 3.
- [x] All gates in the `run-gates` skill pass.
## Tests required

- Unit: `GuideVoice`'s five tests, moved unchanged. `TipScreen`'s voice
  test rewritten for keep, play, stop, let go. A title test for the
  kept tip lines. The threat and playback tests that exist pass
  unchanged. The registry: every name the tips and lessons use has a
  row, and an unknown name fails.
- Integration (Harness): in `tests/it/save_files.rs`, beside
  `each_export_tip_shows_once_per_profile_naming_the_export_key`, a
  launch with a voiced manifest and tips on. Before the box's
  `PlayVoice`, a `KeepVoices` holding the Load Game tip's line was sent
  on an earlier frame.
- Snapshot: none new; existing ones unchanged.

## Completion notes

**Done.** Nothing a player sees changes, apart from the out-of-battle
tips' clips now being loaded before their boxes open.

- `GuideVoice` moved to `crate::tips::voice` (`pub mod`, for its doc
  links), its `voiced()` fixture to `voice::testing`; `TipState::shown_in`
  is `crate::tips::shown_in`. `EndAnswer` moved to `tips/view.rs` too
  (`tips/view.rs` named it from `battle::lessons`; `lessons.rs` re-exports
  it).
- `battle/guide.rs`: `Guide` holds the tips, their mode, the lessons, what
  a lesson just done holds back (`AfterLesson`, added on main by 1292
  after this ticket was written), the voice, the pushed scene's lines
  (1296), the end box's focus, `Doable` and what was noticed. The screen
  hands it a `Moment`: what is over the map as one enum (`Over`: nothing,
  a banner, or something busy: a playback, EXP, a scene waiting, the
  notes; a banner never shows with those), the phase, whether the battle
  is decided, and whether the player browses with the queue empty. Nine
  bools would have tripped `clippy::struct_excessive_bools`.
- `battle/triggers.rs`: the trigger checks as an `impl BattleScreen`
  block, plus `take_in_tips` and `take_in_lessons` (the frame's two
  firings, in today's order).
- `battle/threat.rs`: `ThreatDisplay` (named so, not `ThreatView`: that
  is already `map_view::ThreatView`, the arrows' scene item), with
  `danger_tiles`, `threat_enemies`, `threat_target` and `threat_view`.
  `battle/effects.rs`: `Effects` with the popups, flashes, cues, the
  walked unit, the pace and the AI's speed, their types and ticks.
- `TipScreen` keeps its line and plays it on its first frame through a
  `GuideVoice`; Confirm or Cancel stops it and lets it go.
- The title keeps the unseen out-of-battle tips' lines with New Game's
  (1293's `KeptVoices` set), only where their box can show: where saves
  can be exported, and the Load Game tip only with a save to load.
  Keeping them where they can't show broke `tests/it/lessons.rs` (it
  launches through the title with voices on and nothing to export to)
  and would only have loaded clips for nothing.
- **Deviation:** the Load Game list adds its export tip's line to its own
  set too. Its set (each save's first scene lines, 1293) replaces the
  title's on the frame it opens, a frame before the tip box asks for the
  clip, so without it the clip was let go just before it was needed on
  the web. The integration test voices every scene and checks both.

**Deviations and what is not met as written:**
- *Split:* the trigger registry is ticket 1310 (`blocked_by` this one),
  as *Split* allows: `pr-size` counts a moved line twice, and the moves
  alone passed 2,000 before the registry.
- `mod.rs` is 2,993 lines: 893 under the 3,886 it had when this ticket
  was started, 602 under the 3,595 written in the criterion (it grew by
  291 lines on main in between, 1286, 1304, 1306, 1307).
- `grep -rn "screens::battle" crates/ui/src/tips` still finds
  `tips/glyph.rs`'s `battle::layout::MAP_VIEW` (the console's map region
  the box sits over, which the results screen and the map skins' tests
  import too).
- A few in-crate battle tests changed their paths only (`s.tips` to
  `s.guide.tips`, `s.threats` to `s.threat.threats`, `s.pace` to
  `s.effects.pace`, `s.flashes` to `s.effects.flashes`, two calls with the
  state passed in, `KeptThreat`'s and `danger_zone`'s imports, `set_due`
  as `Guide::set_due`); no assertion changed. `tests/it` gains only the
  new `save_files` test.

**Size.** Estimate 150 production lines net; measured 1,060 added, 812
removed (net +248: the forwards, `Moment`, the title's and the save
list's keep). Friction row: outcome 3. `pr-size` 3,051 (about 1,000 are
moved lines counted twice). It could split into the guide and the
threat/effects structs, but it was finished and gated in one piece, so
it ships as one.

**Follow-up tickets:** 1310 (tip triggers by name, with a predicate row
each).

### For the Act 1 playtest (1064)

**For the Act 1 playtest (1064):** whether the Load Game and Profile
tips are heard on the web build.
