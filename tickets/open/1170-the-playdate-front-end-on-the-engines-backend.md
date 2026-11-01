---
id: "1170"
title: "The Playdate front end on the engine's backend with the game's config start and skins"
type: feature
milestone: "Engine: carve"
model: opus-5.5 / gpt-6.1-sol
effort: high
status: blocked
blocked_by: ["1175", "1203", "engine#3010", "engine#3029", "engine#3028", "engine#2410", "engine#3011"]
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
2410 (#215) shipped after it. A pin move reaches them; no new feature
implementation is needed for those pieces.

There is one real remaining engine prerequisite: 3011 owns the opt-in
report a shipped game keeps. Current `shuyi_playdate::Driver` has no
replacement for the game's loading-card/first-frame timestamps, slow
bundle reads, minute frame/stack report and bench-phase observations.
Do not delete or silently disable those to make this port fit. Wait for
3011 and verify its hooks before cutover. Crash reports (engine 3076)
and voice baking (3078) are absent from the current game and are not
prerequisites for preserving behavior here.

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
- Verify 3011's report hooks, then move the pin past all named engine
  tickets with `engine-pin` and its ledger/adoption review.
- `shuyi_platform::Start`/`shuyi_playdate::main!`, the game's existing
  loading card/data setup, one-bit look, saved settings and stack size.
  Use the engine's SDK, allocator, panic path, driver, storage and audio.
- Wire the game's project/audio/bake adapters and library feature to
  `shuyi export playdate`; preserve selected Playdate music, 4x4 Bayer
  stretch/dither policy, data folder and menu/keymap semantics. A/B move
  to East/West consistently with 2410, including target profile/help.
- Replace the public xtask build entry with a thin engine-export call
  where compatibility is needed. Disconnect the old plan/modules from
  compilation and execution here; no dead-code allow, no feature that
  still selects the obsolete runtime, no duplicate allocator/panic path.
- Delete the small obsolete heap/stack/pad/menu/picture/screen/text
  helpers and the old `pd/allocator.rs`, `pd/panic.rs` and
  `pd/game_stack.rs` as part of their cutover. Keep the game's card/start
  only where they still describe its actual loading look/configuration.
- Preserve reports/bench output through 3011 and compare real before/after
  startup and minute reports. Retain Chapter 1 host/one-bit tests.

**Out (do not do):**
- Bulk deletion of disconnected files: 1204, 1205 and 1206 own the
  bounded cleanup. Record an exact file inventory for each; they must
  not later discover still-running code or remove unsupported behavior.
- Audio converter replacement (1203, needed first), voice clips (1048),
  sale packaging (1052), AI speed (1051), frame optimization (1057/1059),
  and changing the one-bit skins in `crates/ui`.

**Either order with 1057 and 1059:** if they land first, verify the engine
keeps their measurements/behavior before disconnecting the old frontend;
otherwise update their ticket paths to the engine-backed frontend here.
The 1056 unwind-section fix is already in engine 3010's exporter.

**Either order with 1182:** that ticket deletes the after-script pad
workaround. If it is done first, keep its ordered engine crank/pad
regression and update its button script to East/West during this port.
If this port is first, update 1182's examples/test instructions to the
adopted A/B positions here; 1182 still removes only its remaining wrapper.
Whichever lands second checks the script and help names against the
actual keymap/profile, not the old South/East positions.

## Implementation steps

1. Capture the current device/Simulator package inventory, old save,
   button/chord/menu behavior, audio cues and startup/minute reports.
   Read 1203's bake adapter and the now-shipped engine report hooks.
2. Move the pin and wire the engine-backed library/start, context and
   one-bit pictures/pack adapter. Keep SDK ownership in the engine.
   Test after each step; never ship a silent temporary audio backend.
3. Switch actual device and Simulator builds to `shuyi export playdate`.
   Preserve private-art feature policy and the current music selection.
   The old xtask module becomes unreferenced for 1206, not a fallback.
4. Disconnect legacy SDK/storage/audio/report modules and obsolete Cargo
   dependencies/features. Route preserved reports through the engine.
   Move only still-relevant game-specific host tests to the new path;
   engine-owned storage conformance and power-cut suites remain run at
   the pinned engine revision, retaining 1196's guarantees.
5. Load the saved pre-port file with the same bundle/data id. Measure
   real startup and minute behavior against step 1/ADR-0070. Record
   pending device evidence honestly; build-only evidence cannot check it.
6. Give 1204–1206 exact retired-source inventories and update build docs,
   friction/boundary ownership and downstream ticket paths.

Expected size: about 250–350 added integration lines and small wiring
removals, planned under 1,900 changed lines including tests/docs and the
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
      through engine 3011. Real device startup to title is no slower than
      baseline; before/after measurements are in the completion notes.
- [ ] The exact retired-source lists are assigned to 1204–1206; the final
      physical under-200-production-line frontend criterion is in 1205.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Host: existing applicable Chapter 1/start/skin tests, old-save load,
  input/help/menu mappings and audio schedule comparisons.
- Integration: real device/Simulator export and runtime report, plus
  authoritative engine storage conformance/power-cut suite at the pin.

## Completion notes

*(Filled in by the session that completes the ticket: pin, baseline and
runtime evidence, retired-file inventories, deviations and follow-ups.)*
