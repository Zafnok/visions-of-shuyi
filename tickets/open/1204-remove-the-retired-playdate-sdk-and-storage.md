---
id: "1204"
title: Remove the Playdate SDK and storage sources retired by the engine port
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["1170"]
nick_input: none
completed:
---

# 1204 — Remove the retired Playdate SDK and storage

## Context

The 2026-11-01 audit split 1170's oversized deletion into bounded PRs.
1170 makes the actual engine cutover and disconnects the obsolete runtime
from Cargo; this ticket physically removes its remaining SDK entry/file
glue, C shim and storage implementation. The engine now owns those rules.
The source list is recorded by 1170 so this is deletion, not a second port.

## Nick input

None. Host checks and existing export/build gates verify removal; this
does not replace or repeat 1170's device measurements or setup.

## Scope

**In:**
- Retired `crates/playdate/src/pd/entry.rs`, `pd/files.rs`, `pd/mod.rs`,
  `crates/playdate/shim/` and `src/storage.rs`, as inventoried by 1170.
- Remaining references, obsolete build inputs and tests specific to the
  retired implementation. Keep tests of the active game integration.
- Confirm the adopted engine's conformance/power-cut suites remain run,
  preserving 1196's rename-refusal/torn-write guarantees and save format.

**Out (do not do):**
- Backend cutover or behavior changes (1170 must already be done).
- Retired audio/report and remaining `pd/sound.rs`/`pd/bench.rs` sources
  (1205, needed after this deletion), old xtask exporter (1206),
  sale packaging (1052), and changes to save data or folder identity.
- New SDK adapters, test suppressions or copied platform implementations.

## Implementation steps

1. Read 1170's retired-file inventory and prove no active manifest,
   feature, build script or module declaration selects these files.
2. Delete only the retired files and their obsolete tests/references;
   keep the engine backend, the game start/config/card and Chapter 1 tests.
3. Search all spellings of the old entry symbols, shim path and
   `FileStorage` implementation in current source/build instructions.
   Retain historical done-ticket descriptions. Re-run after merging main.
4. Run active export/build gates and old-save integration. Run the pinned
   engine's authoritative storage conformance/power-cut tests through the
   gate established in 1170; do not preserve an unused game storage copy
   just to keep the old tests passing. Update ownership docs/friction row.

Expected size: about 1,600–1,800 changed lines including retired tests,
docs and ticket move; no new production implementation is expected.

## Acceptance criteria

- [ ] Inventoried SDK entry/file/shim/storage sources are gone, with no
      renamed copies, selected old symbols or obsolete Cargo features.
- [ ] Actual device/Simulator build plans still use the engine backend;
      old-save load and the pinned engine storage suites pass.
- [ ] A search across current source/build docs finds no stale use of the
      deleted paths or entry points; 1205 retains its own exact cleanup list.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Existing active integration/export gates and old-save compatibility.
- Authoritative engine conformance/power-cut suite at the game's pin.
  No replacement storage tests mirroring removed code are added.

## Completion notes

*(Filled in by the session that completes the ticket: deleted paths,
active-path verification, suite evidence and remaining 1205 handoff.)*
