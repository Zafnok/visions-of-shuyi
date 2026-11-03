---
id: "1170"
title: "The Playdate front end on the engine's backend with the game's config start and skins"
type: feature
milestone: "Engine: carve"
model: opus-5.5 / gpt-6.1-sol
effort: high
status: blocked
blocked_by: ["engine#3088"]
nick_input: setup
completed:
---

# 1170 — The Playdate front end on the engine's backend

## Context

The first friction sweep measured the Playdate front end at 1,735
production lines against 100 (`docs/engine/friction.md`, 1153,
ADR-0070). The 2026-11-01 audit confirmed that the platform loop,
storage, audio, stack and exporter have now shipped in the engine:
3010/3029 are at the game's pin; 3028 (PR #214) and the A/B button names
2410 (#215) shipped after it. The pin move of 2026-11-01 reached them; no
new feature implementation is needed for those pieces.

There is one real remaining engine prerequisite: the opt-in report a
shipped game keeps. Current `shuyi_playdate::Driver` has no replacement
for the game's loading-card/first-frame timestamps, slow bundle reads,
minute frame/stack report and bench-phase observations. Do not delete or
silently disable those to make this port fit. That report sat inside
engine 3011, a device research ticket for After MVP; the game/engine
boundary audit of 2026-11-03 split it into
[engine#3088](https://github.com/Zafnok/shuyi-engine/blob/main/tickets/open/3088-the-playdate-backends-opt-in-run-report.md),
tested on the fake SDK and clock, so this ticket no longer waits for a
device run. 1175, 1203, engine 3010, 3029, 3028 and 2410 are done and at
the pin. Crash reports (engine 3076) and voice baking (3078) are absent
from the current game and are not prerequisites for preserving behavior
here.

The same audit found two more things this cutover must carry:
- **The system menu.** The game names its three Playdate system-menu
  entries in code (`trpg_ui::game::system_menu::SystemMenuEntry::ALL`,
  read by `crates/playdate/src/menu.rs` and `pd/entry.rs`). The engine's
  backend instead adds one item per `SystemMenu0`–`SystemMenu7` button the
  keymap's `playdate` table binds, titled by the text key
  `system_menu.<Action>`, and turns a choice into a press of that button.
  `assets/data/keymap.ron`'s `playdate` table binds `DangerZone`,
  `DialogueLog` and `Options` to `[]`, so after the cutover the menu
  would be silently empty.
- **The allocator has drifted.** The game's `pd/allocator.rs` `dealloc`
  lacks the engine's null check, and the engine wraps its `SdkAlloc` in
  `shuyi_alloc_count::CountingAlloc` in debug builds (heap counts); one
  more reason not to keep running the copy.

The original ticket also promised deleting 754 lines of audio tooling
and 1,422 lines of exporter tooling, besides all platform modules/tests.
That cannot fit one PR. The concrete split is: 1203 independently adopts
audio baking first; this ticket makes the real backend/export cutover;
1204 removes retired SDK/storage sources, 1205 then removes retired
runtime audio/report helpers and verifies the physical frontend's
under-200-line target, and 1206 removes the retired exporter. They depend
on this cutover and remove only sources already disconnected here.

## Nick input

**Setup, only for the required device measurements:** connect Nick's
existing Playdate by USB, unlock it and leave it on its home screen;
use the installed SDK's console/device tools. If it cannot be connected,
keep the device acceptance pending. Simulator/link success is not a
startup measurement. No purchase, new account or gameplay sign-off.
Tests verify behavior; subjective startup/sound/save/menu feel goes to
1064 if this optional migration lands before that playtest.

## Scope

**In:**
- Move the pin past engine 3088's merge if an open `engine-pin` PR has
  not (`engine-pin`, with its ledger/adoption review), then verify its
  report against the game's. The pin move of 2026-11-01 passed the others.
- `shuyi_platform::Start`/`shuyi_playdate::main!`, the game's existing
  loading card/data setup, one-bit look, saved settings and stack size.
  Use the engine's SDK, allocator, panic path, driver, storage and audio.
- Wire the game's project/audio/bake adapters and library feature to
  `shuyi export playdate`; preserve selected Playdate music, 4x4 Bayer
  stretch/dither policy, data folder and menu/keymap semantics. A/B moved
  to East/West with 2410 in the pin move of 2026-11-01, with the target
  profile and the keymap's `playdate` table: keep them.
- Replace the public xtask build entry with a thin engine-export call
  where compatibility is needed. Disconnect the old plan/modules from
  compilation and execution here; no dead-code allow, no feature that
  still selects the obsolete runtime, no duplicate allocator/panic path.
- Delete the small obsolete heap/stack/pad/menu/picture/screen/text
  helpers and the old `pd/allocator.rs`, `pd/panic.rs` and
  `pd/game_stack.rs` as part of their cutover. Keep the game's card/start
  only where they still describe its actual loading look/configuration.
- Preserve reports/bench output through engine 3088's `RunReport` (on,
  with the game's `load-report.txt` file; phases on in the bench build)
  and compare real before/after startup and minute reports. The old
  bench's reference-render comparison and every-other-minute music pause
  (1055, 1047, both done) retire: the engine's one-bit speed tests prove
  the renderers equal. Retain Chapter 1 host/one-bit tests.
- The system menu from the keymap: bind `DangerZone`, `DialogueLog` and
  `Options` to `SystemMenu0`, `SystemMenu1` and `SystemMenu2` in
  `assets/data/keymap.ron`'s `playdate` table; rename the text keys
  `system_menu.danger_zone`/`dialogue_log`/`options` to
  `system_menu.DangerZone`/`DialogueLog`/`Options` in
  `assets/lang/en/ui.ron` and `assets-test/lang/test/ui.ron`; delete
  `crates/ui/src/game/system_menu.rs` and `crates/playdate/src/menu.rs`
  (about 75 production lines); make `Harness::system_menu`
  (`crates/ui/src/harness.rs`) press the `SystemMenuN` button, and keep
  the system-menu tests in `crates/ui/tests/it/playdate.rs` on it. Check
  that `cargo shuyi check-keys` accepts the table and that help, tips and
  lessons naming these three actions on the Playdate read as the system
  menu, through the keymap (the `keyboard-input` skill), not a raw
  button name.

**Out (do not do):**
- Bulk deletion of disconnected files: 1204, 1205 and 1206 own the
  bounded cleanup. Record an exact file inventory for each; they must
  not later discover still-running code or remove unsupported behavior.
- Audio converter replacement (1203, done), voice clips (1048),
  sale packaging (1052), AI speed (1051), frame optimization (1057/1059),
  and changing the one-bit skins in `crates/ui`.

**Either order with 1057 and 1059:** if they land first, verify the engine
keeps their measurements/behavior before disconnecting the old frontend;
otherwise update their ticket paths to the engine-backed frontend here.
The 1056 unwind-section fix is already in engine 3010's exporter.

**Either order with 1060:** if it is done, keep its lesson ticks and
placeholders for Danger Zone, Dialogue log and Options passing with the
new `SystemMenuN` bindings; if not, nothing to do here (1060 then reads
the bindings this ticket made).

**Either order with 1182:** that ticket deletes the after-script pad
workaround. If it is done first, keep its ordered engine crank/pad
regression. The pin move of 2026-11-01 already moved A/B and its examples
to East/West; 1182 still removes only its remaining wrapper. Whichever
lands second checks the script and help names against the actual
keymap/profile.

**Either order with 1221:** if 1221 is done, the one-bit pictures
adapter starts from its `.meta` `convert` data and `ConvertSettings::make`
(engine#2245), or uses the engine's bake directly if it makes the same
files; if not, wire the adapter to `playdate_images` as now, and 1221
moves it.

**Either order with 1227:** if 1227 is done, `crates/playdate/src/start.rs`
already calls `PlatformFacts::builtin("playdate")` (engine#3089): keep
that call, or the engine backend's own facts; if not, keep whatever
`trpg_ui::platform::playdate_facts` call the new start still needs, and
1227 replaces it.

## Implementation steps

1. Capture the current device/Simulator package inventory, old save,
   button/chord/menu behavior, audio cues and startup/minute reports.
   Read 1203's bake adapter and engine 3088's `RunReport`.
2. Move the pin past engine 3088's merge (unless an open `engine-pin` PR
   did), then wire the engine-backed library/start
   (`shuyi_playdate::main!(start, stack_bytes = …, report = …)`), context
   and one-bit pictures/pack adapter. Keep SDK ownership in the engine.
   Test after each step; never ship a silent temporary audio backend.
3. Switch actual device and Simulator builds to `shuyi export playdate`.
   Preserve private-art feature policy and the current music selection.
   The old xtask module becomes unreferenced for 1206, not a fallback.
4. Disconnect legacy SDK/storage/audio/report modules and obsolete Cargo
   dependencies/features. Route preserved reports through the engine.
   Move the system menu onto the keymap (Scope) and delete its two files.
   Move only still-relevant game-specific host tests to the new path;
   engine-owned storage conformance and power-cut suites remain run at
   the pinned engine revision, retaining 1196's guarantees.
5. Load the saved pre-port file with the same bundle/data id. Measure
   real startup and minute behavior against step 1/ADR-0070. Record
   pending device evidence honestly; build-only evidence cannot check it.
6. Give 1204–1206 exact retired-source inventories and update build docs,
   friction/boundary ownership and downstream ticket paths.

Expected size: about 250–350 added integration lines, about 75 removed
with the system menu's two files, and small wiring removals, planned under 1,900 changed lines including tests/docs and the
ticket move. Bulk source deletion is explicitly owned by 1204–1206.

## Acceptance criteria

- [ ] Actual device/Simulator packages use the engine backend/exporter;
      old SDK/allocator/panic/driver/audio/storage/report modules and old
      exporter are unreachable from Cargo and the public build command.
- [ ] A save made before the port loads afterward with the same data
      folder and format; engine storage conformance/power-cut tests run
      at the pin, including rename refusal and torn writes from 1196.
- [ ] A/B help, input/chords, crank notch, menu actions, cues/volumes,
      selected music and one-bit image policy agree with the baseline.
- [ ] Required loading/read/minute/stack/bench evidence remains available
      through engine 3088's report. Real device startup to title is no slower than
      baseline; before/after measurements are in the completion notes.
- [ ] The system menu lists Danger Zone, Dialogue Log and Options from
      the keymap's `SystemMenu0`–`2`, and each does what it did;
      `crates/ui/src/game/system_menu.rs` and `crates/playdate/src/menu.rs`
      are gone, and the `crates/ui/tests/it/playdate.rs` menu tests pass.
- [ ] The exact retired-source lists are assigned to 1204–1206; the final
      physical under-200-production-line frontend criterion is in 1205.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Host: existing applicable Chapter 1/start/skin tests, old-save load,
  input/help/menu mappings (system menu through `SystemMenuN` presses)
  and audio schedule comparisons.
- Integration: real device/Simulator export and runtime report, plus
  authoritative engine storage conformance/power-cut suite at the pin.

## Completion notes

*(Filled in by the session that completes the ticket: pin, baseline and
runtime evidence, retired-file inventories, deviations and follow-ups.)*
