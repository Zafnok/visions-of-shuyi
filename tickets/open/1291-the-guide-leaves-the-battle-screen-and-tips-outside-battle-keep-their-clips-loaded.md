---
id: "1291"
title: "The guide leaves the battle screen, and tips outside battle keep their clips loaded"
type: infra
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: []
nick_input: none
completed:
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
- A row in `docs/engine/friction.md` (outcome 3).

**Out (do not do):**
- Battle notes, banners, the objective page and the scene queue
  (`queue: VecDeque<Queued>`): they are the battle's flow, not the guide.
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
8. **Measure.** Expected size: about +60 production lines net (`Moment`,
   the forwards, the title's keep), with about 500 lines moved. About
   1,500 changed lines in `cargo xtask pr-size`. Add the friction row.

## Acceptance criteria

- [ ] `wc -l crates/ui/src/screens/battle/mod.rs` is at least 400 lines
      under 3,595.
- [ ] `grep -nE "fn (due_tip|coming_guide_lines|lesson_moment|lessons_end_key|settle_lessons|tick_lessons|notice_mode|triggers_now)\b" crates/ui/src/screens/battle/mod.rs`
      finds nothing.
- [ ] `grep -rn "screens::battle" crates/ui/src/tips` finds nothing.
- [ ] `TipScreen` sends `KeepVoices [line]` before `PlayVoice line`. On
      Confirm it sends `StopVoice` then `KeepVoices []`. With voices off
      it sends nothing.
- [ ] The title's first frame, with tips and voices on, keeps both
      out-of-battle tip lines. It keeps only the unseen ones, and nothing
      with tips off.
- [ ] `ch01_tips`, `ch01_lessons`, `lessons` and `save_files` in
      `tests/it` pass. The first three files are unchanged; `save_files.rs`
      only gains the test below.
- [ ] A row for 1291 in `docs/engine/friction.md`, outcome 3.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `GuideVoice`'s five tests, moved unchanged. `TipScreen`'s voice
  test rewritten for keep, play, stop, let go. A title test for the
  kept tip lines.
- Integration (Harness): in `tests/it/save_files.rs`, beside
  `each_export_tip_shows_once_per_profile_naming_the_export_key`, a
  launch with a voiced manifest and tips on. Before the box's
  `PlayVoice`, a `KeepVoices` holding the Load Game tip's line was sent
  on an earlier frame.
- Snapshot: none new; existing ones unchanged.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
