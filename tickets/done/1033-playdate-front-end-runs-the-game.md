---
id: "1033"
title: "Playdate: the front end runs the game (frame loop, one-bit frame on screen, D-pad, A and B, saves)"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: done
blocked_by: ["1031", "1032"]
nick_input: none
completed: 2026-10-15
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

- [x] In the Simulator, the D-pad, A and B move through the title, New
      Game, the lead's name grid and into Chapter 1's first battle.
- [x] A profile and a save written in the Simulator are there after a
      restart.
- [ ] The device numbers are in the PR. *Waits on Nick's sideload (see
      the completion notes).*
- [x] Tests: `PadKind::Playdate` names; the no-keyboard start; storage
      conformance runs in the Simulator (logged in the PR).
- [x] The PC game is unchanged (snapshots, harness tests).
- [x] All gates in the `run-gates` skill pass, and the CI `playdate` job.

## Tests required

- Unit: pad kind names and the A/B positions; no layout picker without a
  keyboard.
- Manual: Simulator walk-through and one device run.

## Completion notes

**What was done**

- `PadKind::Playdate` (`crates/ui/src/input/pad.rs`): no swap, named
  `A`, `B` and the D-pad arrows as printed on the Playdate (the Xbox
  names, which also stand in for the buttons it lacks until 1034 gives it
  its own `playdate` table). The front end reports A at `South` and B at
  `East`, so the default `pad` table gives D-pad = cursor, A = Confirm,
  B = Cancel (test through the real press-to-action path).
- **No keyboard:** `Ctx::keyboard` (`Keyboard::Present` / `Absent`).
  `Absent` never opens the layout picker (also not in a scene preview);
  `Game` now starts its input on `ctx.device`, so the Playdate's help
  names its buttons from the first frame. PC unchanged (`Present`, the
  keyboard device).
- **The loop** (`crates/playdate/src/pd/entry.rs`): `dt` from the SDK's
  elapsed timer, the buttons through `Pads` (`src/pads.rs`; a tap between
  two updates is held for one frame so it is never lost), `Game::frame`
  on the game stack, and only when the `GlyphBuffer` changed the one-bit
  render plus a copy of the changed rows into `getFrame` with
  `markUpdatedRows` (`src/screen.rs`). `setRefreshRate(30)`, recorded in
  the crate README. The shim gained the button, frame buffer, refresh
  rate and file calls, with `_Static_assert`s pinning the button bits
  and the 52 × 240 frame buffer to `pd_api.h`.
- **Storage** (`src/storage.rs`, `src/pd/files.rs`): `<key>.ron` per key
  in the game's data folder, written to `.ron.tmp` then moved over (old
  file removed first where the file system will not move onto it). The
  SDK calls are behind a `Files` trait so the rules run the conformance
  suite on the host; `trpg-ui` gained a `storage-conformance` feature
  (part of `harness`) so the `no_std` debug build runs the suite on the
  real files at start, in a `storage-check` folder it leaves empty.
- **Measurements:** once a minute, the top screen, frames a second,
  `Game::frame` and render-and-copy min / avg / max ms, and the stack
  high-water mark, logged in **every** build (deviation: the ticket says
  a debug build, but the device must run the release build).
- `check-keys` scans `crates/playdate/src` and allows its `pads.rs`.
- Tests: pad names and positions (`pad/tests.rs`), the no-keyboard start
  (`game.rs`), `pads.rs`, `storage.rs` (conformance on in-memory files,
  also where files cannot be moved onto others), `screen.rs`, the timing
  report, and `ui/tests/it/playdate.rs`: the Playdate's console, no
  keyboard and only A, B and the D-pad from the title through the name
  grid into Chapter 1's first battle, then Suspend from the map menu
  writes the suspend save. New `Harness::first_launch_and`.

**In the Simulator** (debug build, keys posted to its window): the
conformance suite passed on the real files (`storage-check` left empty,
the game carried on); title → New Game → Classic → familiarity → lead
(name grid: B deleted a letter, A added `A`, Done) → the opening scenes
→ Chapter 1's battle (`Idle`) → map menu → Suspend wrote
`p1-suspend.ron` (19 KB) and `p1-last_save.ron`. After a restart the
profile was there and A on the title's Continue put the battle back
(the suspend save then used up, as on the PC). The release build starts
and plays to the familiarity question too. Screens are cut off as the
ticket expects (the title is all backdrop picture, a grey box until
1040; the map menu opens below row 15).

**Deviations**

- The debug tools are off on the Playdate: they are made of the test
  files, which the `no_std` build does not have (ADR-0057).
- The device numbers are not in yet: they need Nick's sideload of the
  release build (below).

**For Nick:** sideload `target/playdate/VisionsOfShuyi.pdx` (built with
`cargo xtask playdate --release`), play to the first battle, stay a
minute on the title and a minute in the battle, and paste the
Simulator's console: the lines starting "On title for 60 seconds" /
"On battle for 60 seconds" and the ones after them are the numbers.

No gameplay rules were decided here. No follow-up tickets: what the
Playdate still lacks (chords, crank, skins, sound) already has tickets
(1034–1047).
