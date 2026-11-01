---
id: "1205"
title: Remove retired Playdate audio and report helpers and finish the small frontend
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["1170", "1204"]
nick_input: none
completed:
---

# 1205 — Finish the small Playdate frontend

## Context

1170's engine cutover preserves runtime audio and reporting through the
engine and disconnects the old modules. 1204 removes retired SDK/storage;
this bounded final frontend cleanup removes the remaining runtime audio,
report and bench helpers. The 2026-11-01 audit moved the original 1170
criterion, a physical frontend under 200 production lines, here so it
cannot disappear behind an incomplete migration or dead-file count.

## Nick input

None. This deletes already disconnected sources. Real device report and
startup evidence belongs to 1170 and must exist before this ticket starts;
no repeated setup or new gameplay sign-off is requested.

## Scope

**In:**
- Retired `crates/playdate/src/audio.rs`, `report.rs`, `pd/sound.rs` and
  `pd/bench.rs`, and any remaining helper paths explicitly assigned here
  by 1170. Remove obsolete tests of those implementations.
- The physical frontend's production-line measurement and ownership
  check: game config/context/start/card/skin choices remain; SDK, driver,
  storage, shared audio rules and generic reporting live in the engine.
- Tests of the active game's cue/music selection, loading card/config,
  Chapter 1 skins and old-save compatibility remain on the new path.

**Out (do not do):**
- New audio/report behavior, weakening the original report or suppressing
  tests. 1170 already waits for engine 3011's equivalent reporting.
- Old exporter deletion (1206): either order works after 1170; it removes
  xtask-only files while this ticket removes frontend-only files.
- New sound content/voices (1048), performance optimization (1057/1059),
  or sale packaging (1052).

## Implementation steps

1. Check 1170's before/after audio/report evidence and exact retired list.
   Verify every path is disconnected from all supported Cargo features,
   not merely unused in one build profile.
2. Remove those sources and old unit tests. Keep game integration checks
   of selection, cue/gain behavior and reports on the adopted engine;
   generic player correctness remains in the engine's existing tests.
3. Search source/manifests/build docs for legacy `Audio`, `Speaker`,
   minute/read report and benchmark implementations in every spelling.
   Do not relocate them into the game start or another module. Re-run the
   search after merging main.
4. Measure `crates/playdate` production lines using `cargo xtask code-lines`
   and record the complete surviving-file inventory. Finish the friction
   and boundary entries with links to the actual engine owners.

Expected size: about 1,800–2,000 changed lines including retired tests,
docs and ticket move. The larger SDK/storage deletion is already 1204;
no new production implementation is expected here.

## Acceptance criteria

- [ ] All assigned retired audio/report/bench sources and obsolete
      modules/features are physically gone; no moved or renamed copies
      remain in the game.
- [ ] `crates/playdate` is under 200 production lines, counted by the
      existing code-lines tool, and the completion notes inventory every
      surviving source and explain its game-owned purpose.
- [ ] Active game audio selection/gains, loading/read/minute/stack/bench
      report availability, old saves and Chapter 1 host/skin tests still
      pass. Report deletion cannot be excused by Simulator build success.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Existing active game integration and engine-backed audio/report checks;
  no duplicate game unit tests of deleted engine algorithms.
- Production-line and source-ownership verification above.

## Completion notes

*(Filled in by the session that completes the ticket: deleted/surviving
file inventory, production-line count and preserved integration evidence.)*
