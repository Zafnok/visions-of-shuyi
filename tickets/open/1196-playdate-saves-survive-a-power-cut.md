---
id: "1196"
title: Playdate saves survive a power cut (the engine's power-cut suite)
type: bug
milestone: Post–Chapter 1
model: opus-5.5
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1196 — Playdate saves survive a power cut

## Context

Engine PR #153 (engine ticket 3045) was passed by the pin move of
2026-10-28 (`docs/engine/pins.md`, game PR #465). It defines what "a save
survives a power cut" means as a suite every storage backend runs:
`shuyi_ui::storage::power_cut_suite` over a `PowerCutRig` that fakes
the medium and cuts the power after any operation. The engine's own
desktop storage passes it (`crates/app/src/storage/native.rs` in the
engine: a flushed `<key>.ron.bak` is kept as the last good copy). The
game's PC saves go through that storage, so they get this for free.

The Playdate's saves don't. `FileStorage` in `crates/playdate/src/storage.rs`
is the game's own backend. It runs `conformance_suite` but not the
power-cut suite. Its `write` puts the value in `<key>.ron.tmp` and
renames it onto `<key>.ron`. On a file system that won't rename onto an
existing file, it first unlinks `<key>.ron`, so a power cut (a flat
battery, the console crashing) between the unlink and the rename leaves
no save at all. Nick plays Chapter 1 on his Playdate (1050), so this
is where it matters.

## Nick input

None.

## Scope

**In:**
- A `PowerCutRig` over the Playdate backend's test files
  (`MemoryFiles` in `storage.rs`'s tests), for both behaviors: rename
  onto an existing file allowed, and refused.
- `power_cut_suite` run on `FileStorage` in both.
- `write`, `read` and `delete` changed until both pass, the way the
  engine's native storage does it: the last good copy kept, and a read
  that falls back to it.

**Out (do not do):**
- PC and web saves (the engine's storage at the pin).
- The save format or the slots (`crates/ui/src/screens/save`).

## Implementation steps

1. Read the engine's `crates/ui/src/storage/power_cut.rs` (the
   promises) and `crates/app/src/storage/native.rs` with
   `native/fake_disk.rs` (a backend that passes, and its rig), at the
   pin's checkout (`cargo xtask engine`, or
   `~/.cargo/git/checkouts/shuyi-engine-*/c9fb84c`).
2. In `crates/playdate/src/storage.rs`'s tests, give `MemoryFiles` an
   operation budget. After `cut_after(n)`, the n+1th `write`, `rename`,
   `unlink` or `mkdir` fails, and so does every later one. A write cut
   partway may leave a prefix of the bytes (torn). `restart` keeps what
   the files hold and opens a new `FileStorage` over them. Wrap this as
   a `PowerCutRig`.
3. Run `power_cut_suite` with rename-onto-existing allowed and with it
   refused (`MemoryFiles`'s existing flag). It is expected to fail in
   the refused case.
4. Fix `FileStorage` the way the engine's native backend does: write
   `<key>.ron.bak`, then the tmp file, then move it into place, and on
   `read` fall back to the `.bak` when `<key>.ron` is missing or not
   valid. The Playdate SDK's file API (`crates/playdate/src/pd`)
   decides what "flushed" means there. Check what `pd_file` offers
   before relying on a flush.
5. Update the module docs to say what it promises, as the engine's do.

Expected size: about 60 production lines (the write and read paths)
and about 80 test lines (the rig).

## Acceptance criteria

- [ ] `power_cut_suite` passes on the Playdate `FileStorage` with rename
      onto an existing file allowed and refused.
- [ ] `conformance_suite` still passes in both.
- [ ] The Playdate build and its clippy gates pass (`run-gates`).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the two suite runs above, in `crates/playdate/src/storage.rs`.
- No device check: the suite is the proof. What is left to judge on
  the device goes on 1064's checklist.

## Completion notes

*(Filled in by the session that completes the ticket.)*
