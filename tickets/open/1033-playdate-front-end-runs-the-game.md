---
id: "1033"
title: "Playdate: the front end runs the game (frame loop, one-bit frame on screen, D-pad, A and B, saves)"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: ["1031", "1032"]
nick_input: none
completed:
---

# 1033 — The Playdate front end runs the game

## Context

1031 made `crates/playdate` (the C shim, the allocator, the game on its
own 64 KB stack, `cargo xtask playdate`). 1032 made the one-bit renderer.
This ticket runs the real game: `trpg_ui::Game::frame(events, dt)` once
per SDK update, its `GlyphBuffer` drawn by the one-bit renderer to the
Playdate's screen, the buttons fed in as a controller, and the game's
`Storage` on the Playdate's files (ADR-0070 §4: the front end owns the run
loop, buttons, clock, storage, sound and the allocator; no game or screen
logic).

The screens still use today's PC skins, drawn into a 50 × 15 buffer, so
most of them will be cut off: that is expected until their Playdate skins
land (1037 onwards). The point here is the loop, the speed, input and
saves.

## Nick input

None to decide. Nick sideloads one build (see *Measured on Nick's
Playdate*).

## Scope

**In:**
- **The loop:** the SDK update callback reads the buttons, builds this
  frame's `RawInputEvent`s, measures `dt` with the SDK's timer, calls
  `game.frame(&events, dt)` on the game's own stack (1031's
  `run_on_stack`), renders the frame with `trpg_ui::onebit` and copies it
  into the SDK's frame buffer (`getFrame` + `markUpdatedRows`), only when
  the buffer changed. Frame rate: the SDK's `setRefreshRate`, 30 to
  start; record the choice in the crate README.
- **The frame is 50 × 15:** the front end makes the game's `Ctx` with
  `ConsoleSpec::PLAYDATE` (1030) and the Playdate palette (1032).
- **Buttons as a controller:** the Playdate's D-pad, A and B become a pad
  of a new `PadKind::Playdate` fed through `Pads`
  (`crates/ui/src/input/pad.rs`), so they reach screens as `Action`s
  through the player's pad bindings (`keyboard-input` skill, ADR-0034,
  ADR-0053). The Playdate's A reports as the binding position `South`
  and B as `East`, so the default `pad` table in `assets/data/keymap.ron`
  already gives D-pad = cursor, A = Confirm, B = Cancel (check it;
  `docs/design/playdate.md`, *Buttons and the crank*).
  `PadKind::Playdate`'s `button_name` names them "A", "B" and the D-pad
  as the Playdate's manual does, so help text reads right. Only
  `crates/playdate/src/pads.rs` reads the SDK's button state
  (`check-keys`: add it to the allowed files like `app/src/pads.rs`).
- **No keyboard:** the game starts without the first-launch keyboard
  layout picker (`Game::start` shows `LayoutPickerScreen` when no layout
  is saved): add a `Ctx` or `Game` setting "this platform has no
  keyboard" that skips it; the PC is unchanged. The lead's name is typed
  on the letter grid lead select already has for controllers
  (`NameEntry`).
- **Storage:** `trpg_ui::Storage` over the SDK's file calls, in the
  game's Data folder, one file per storage key (keys are already
  restricted by `is_valid_key`). It passes `storage::conformance_suite`
  in the Simulator (a debug build entry point that runs it and logs the
  result) and keeps profiles and saves as on the PC (ADR-0069).
- The shim gains the SDK calls this needs (buttons, frame buffer, refresh
  rate, file listing/unlink for `Storage`).
- **Measure on the device:** render time per frame (min/avg/max over a
  minute of the title and of a battle, if reachable), `Game::frame` time,
  and the stack high-water mark. Logged to the console by a debug build.

**Out (do not do):**
- Chords (A + B, hold B + left/right) and the crank (1034, 1035).
- Any Playdate skin (1037 onwards); screens may look broken.
- Sound (1047), loading card (1036).
- Pausing, locking and system-menu events beyond what the SDK needs to
  not crash (the system menu is 1035).

**Carve (1107, 1110):** either order works. If 1107 moved input to the
engine first, `PadKind::Playdate` goes where `PadKind` went. 1110 keeps
`Game::frame` platform-neutral; if it changed its signature, follow it.

## Implementation steps

1. Read ADR-0070 §4, `crates/app/src/main.rs` (the loop to mirror),
   `crates/app/src/pads.rs`, `crates/app/src/storage/native.rs`,
   `crates/ui/src/game.rs`.
2. `PadKind::Playdate` and its names (tests in `pad.rs`).
3. The "no keyboard" start (a test in `game.rs`: no layout picker).
4. The loop, the frame copy, the buttons.
5. `Storage` on SDK files; conformance run in the Simulator.
6. Build, run in the Simulator: title → New Game → a scene → the battle
   (cut-off look is fine). Ask Nick to sideload.

## Measured on Nick's Playdate

Nick sideloads and plays to the first battle; he pastes the Simulator
console. The PR lists frame render time, `Game::frame` time, frames per
second on the title and the battle, and the stack high-water mark
(must stay well under 64 KB).

## Acceptance criteria

- [ ] In the Simulator, the D-pad, A and B move through the title, New
      Game, the lead's name grid and into Chapter 1's first battle.
- [ ] A profile and a save written in the Simulator are there after a
      restart.
- [ ] The device numbers are in the PR.
- [ ] Tests: `PadKind::Playdate` names; the no-keyboard start; storage
      conformance runs in the Simulator (logged in the PR).
- [ ] The PC game is unchanged (snapshots, harness tests).
- [ ] All gates in the `run-gates` skill pass, and the CI `playdate` job.

## Tests required

- Unit: pad kind names and the A/B positions; no layout picker without a
  keyboard.
- Manual: Simulator walk-through and one device run.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
