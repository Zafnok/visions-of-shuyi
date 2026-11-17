---
id: "1093"
title: "The timing ring and bar in the combat frame, and the Timed presses option"
type: feature
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["1092", "1078"]
nick_input: answer-first
completed:
---

# 1093 — The timing ring and bar in the combat frame

## Context

Nick's search for the game's storefront hook (2026-11-16) chose
legendary beasts fought as a raid, on top of three battle systems that
make the enemy's turn the exciting part (0085–0088, 0772, 1091–1099,
1275–1281). 1092 put timed presses in the rules: a strike due a press is
graded miss, good or perfect, the grades ride in the command
(`Command::Pressed`, missing ones Good), and `BattleState::rehearse`
says what a command would do with the presses so far. Here the player
presses **in the combat frame** (Nick: "the parry should be inside the
combat frame rather than overworld frame. And there should be some
timing indicator"): a ring closes on the target and a bar shows the good
and perfect zones, as in the rough animation Nick saw. Plus the **Timed
presses** option (On, Assisted, Off: Off counts every press as Good).
1094 (Steal the Turn) builds on the perfect parry shown here.

**The look, the key and the windows come from 0086's answers** in
`docs/design/timing.md` (the ring, the bar or both; which Action is the
press; the options and their default; whether bots and Casual play
Good). Where they differ from the sketch here, the decision wins.

**The hard part:** today a command is applied first and its fight
played after. `BattleScreen::apply` (`crates/ui/src/screens/battle/mod.rs`)
applies it, then builds a `Playback` (`playback.rs`: `Beat::Intro`,
`Flash`, `Result`, `Gap`, `Fall`, `Scene`, `Outro`; `Timings`; a
`HurryClock`: held Confirm ×4, Cancel skips). In an AI phase `apply_ai`
applies the AI's command, then `AiAction` (`ai_phase.rs`) shows the pan
and walk from the units as they were, then the fight. **Presses must be
known before the command is applied**, so a fight with a press due plays
from rehearsals and is applied after its last press.

The combat frame is the stage (`stage.rs`, `stage/view.rs` `StageView`,
`stage/motion.rs`; painted by `stage/glyph.rs` and `playdate/stage.rs`;
ADR-0054, ADR-0056); with Combat animations Off
(`ShuyiSettings::combat_animations`) the fight box shows instead. The
Options screen is `screens/options.rs` (`Row`, `OptionRow::new`) and
`options/playdate.rs`. The Playdate's fights run below 30 frames a
second until 1078 (`blocked_by`), and a window can't be fair before.

## Nick input

**Answer first:** ticket 0086 (through 1092). Nothing after merge:
verified by tests and `cargo shuyi frame-png` pictures in the PR; how
the windows feel goes in Completion notes for a later playtest.

## Scope

**In:**
- A pressing mode: the fight plays from rehearsals, each strike due a
  press shows the ring and bar and is graded, and the command is applied
  with its presses; for the player's attacks, the AI's attacks on the
  player's units and Overwatch shots, in every phase.
- The ring, bar and grade in the stage view, painted by the PC and
  Playdate stage skins.
- The press Action, if 0086 adds one, with its defaults.
- The Timed presses option and its Options rows (PC and Playdate);
  Assisted and the Playdate widen the windows.

**Out (do not do):**
- Steal the Turn and the grey world (1094).
- Beast parts in the frame (1097).
- Lag calibration, unless 0086 decides it (then a ticket of its own).

## Implementation steps

Expected size (ADR-0092): about **700 production lines** in `ui`
(pressing mode ~250, press beats ~100, stage view and PC skin ~150,
Playdate skin ~80, option ~60, AI phase ~60) and the ring picture.
Comparable: 0413 (the combat stage). Measure with `cargo xtask
code-lines`; at 3× run `engine-friction`.

1. Read `docs/design/timing.md`, `options.md`, `controls.md`, the
   `keyboard-input` and `shuyi-input` skills, ADR-0038, ADR-0054,
   ADR-0056, and 1092's ADR.
2. **Press beats.** In `playback.rs`, a strike whose `Strike::press`
   is `Some` gets a `Beat::Press { bout, strike }` before its `Result`,
   `Timings::press` long, during which the ring closes; the window is
   centred on its end (the blow lands). The press beats run in real
   time: held Confirm and the speed settings don't hurry them (if
   Confirm is the press key, holding it no longer hurries a fight with
   presses). Cancel skips the fight and counts every press left as
   Good.
3. **The pressing mode** (`screens/battle/press.rs`, `Mode::Pressing`
   in `mode.rs`): holds the command, the presses so far and a
   `Playback` built from `state.rehearse(Pressed { presses so far })`.
   Each time a window closes the grade is added and the rehearsal is
   built again; by 1092's promise everything before the clock stays the
   same, so the new playback continues at the same time. When the last
   press is graded, `apply` the wrapped command and keep playing its
   playback from that time (scenes already taken stay taken). A test
   asserts the applied events equal the last rehearsal's.
   A rehearsal copies the battle once per press, never per frame:
   measure one on the Playdate's bench (`bench/`); if it misses the
   frame budget, start it during the run-up beat before it is needed.
4. **When it presses.** Only with Timed presses On or Assisted, Combat
   animations On, and a strike due a press in the rehearsal; otherwise
   the screen sends the command as today (every press Good).
5. **The AI phase.** `apply_ai`: if the AI's command has a press due,
   don't apply it yet. Build the `AiAction` from the rehearsal's
   `UnitMoved` path, with `Mode::Pressing` as what follows; until it is
   applied the map shows the walker at the end of its path, not back at
   its start (`AiAction::units`). Update `ai_phase.rs`'s module docs.
6. **Grading.** The press is timed from the frame its button went down,
   on the screen's clock (no wall clock). On the Playdate, A is a chord
   button and its Confirm can arrive after the chord window
   (`keyboard-input`, *The Playdate*): use the frame `pad_pressed()`
   saw. The windows (good, perfect, per press kind; Assisted's; the
   Playdate's wider ones from the platform's facts, ticket 1200) are
   *tunable* constants beside `Timings`, never read in a skin.
7. **The view.** `StageView::press: Option<PressView>` (the press's
   word, such as `PARRY`; the ring's closing from 1 to 0; the bar's
   good and perfect zones and its marker; the grade once pressed),
   built in `stage::view_for`. The help bar names the player's press
   key (`widgets::help`), never a fixed key.
8. **The skins** (ADR-0038). `stage/glyph.rs`: the ring as frames of a
   sprite from an asset file (`assets/images/press_ring.png`, our own
   picture made with the `ascii-art` skill, a row in
   `assets/images/README.md`), around the target's picture; the bar as
   cells under the plates, as `paint_hp_bar` draws. `playdate/stage.rs`:
   the same in one bit. The grade pops up like a strike's result; a
   perfect one flashes the stage (`StageView::veil`).
9. **The press Action** (if 0086 adds one): `keyboard-input` *Adding an
   action* (the `Action`, `keymap.ron` in every layout, `pad` and
   `playdate`, `KEY_ROWS`, the keymap test).
10. **The option.** `ShuyiSettings::timed_presses: PressTiming { On,
    Assisted, Off }`, `#[serde(default)]` at 0086's default (old
    settings load with it), a `Row::TimedPresses` on the Options
    screen and its Playdate list, words in `assets/lang/`. If 1168 is
    done, as the engine's enum row; if not, as `OptionRow::new` like
    `CombatAnimations`, and 1168 takes it along.

## Acceptance criteria

- [ ] Harness tests: `a_press_in_the_perfect_zone_records_perfect`,
      `no_press_records_a_miss`, `with_presses_off_no_ring_shows_and_the_command_has_no_presses`,
      `skipping_a_fight_counts_the_presses_left_as_good`,
      `holding_confirm_does_not_hurry_a_press`.
- [ ] `an_enemy_attack_is_applied_after_its_parry`: in an AI phase the
      battle is unchanged until the last press, and the walker stays at
      its destination meanwhile.
- [ ] `the_applied_fight_matches_its_last_rehearsal`.
- [ ] A rebound press key works; the help bar names it (both layouts, a
      pad and the Playdate's buttons).
- [ ] Old settings load with Timed presses at its default.
- [ ] Snapshot tests of the stage view with the ring at three moments,
      and of the Options screen with the new row.
- [ ] `cargo shuyi frame-png` frames over time (`--every 0.05 --count
      8`) of a parry in the PC look and in the Playdate look, in the PR.
- [ ] The Playdate bench: a rehearsal fits the fight's frame budget, or
      the PR says what it does instead.
- [ ] All gates in the `run-gates` skill pass, `cargo shuyi check-keys`
      included.

## Tests required

- Unit: grading at window edges; press beats laid out only for strikes
  due a press; Assisted and Playdate windows wider.
- Integration (Harness): the cases above, on the PC and the Playdate
  skin set.
- Snapshot: the stage with the ring; the Options rows.
- Bots: unchanged (they send no presses).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
