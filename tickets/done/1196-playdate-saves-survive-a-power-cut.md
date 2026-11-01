---
id: "1196"
title: Playdate saves survive a power cut (the engine's power-cut suite)
type: bug
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-01
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

Engine PR #197 (ticket 3010), passed by the pin move of 2026-11-01, adds
`shuyi-playdate`, whose storage passes this suite over a fake SDK's files.
The game moves onto it in 1170, which waits on engine#3028. If 1170 lands
first, close this ticket as done by it. Until then the fix below stands,
and the backend's storage is a closer pattern than the native one.

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

- [x] `power_cut_suite` passes on the Playdate `FileStorage` with rename
      onto an existing file allowed and refused.
- [x] `conformance_suite` still passes in both.
- [x] The Playdate build and its clippy gates pass (`run-gates`).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the two suite runs above, in `crates/playdate/src/storage.rs`.
- No device check: the suite is the proof. What is left to judge on
  the device goes on 1064's checklist.

## Completion notes

`FileStorage` now keeps a key's last good copy, as the engine's
`shuyi-playdate` storage does: a write puts the value in `<key>.ron.tmp`,
copies the old `.ron` to `<key>.ron.bak` when it is a good copy, then moves
the `.tmp` into place. A read falls back to a good `.bak`, `list` names a
good lone `.bak`, `delete` removes the `.bak` first, and a start's first
write removes the `.tmp` files a cut left. Before the fix, the rig failed the
suite where rename is refused: a cut after 3 operations read `Ok(None)`, the
lost save this ticket describes.

Flushing: the game's `SdkFiles::write` only closed the file. The SDK has
`file->flush`, so the write now calls it before the close, through a new
`pd_file_flush` in the shim, as the engine's device code does. Only the build
proves this line: the suite runs on fake files.

Deviation: the ticket wrote `.bak` before the tmp file. Here the tmp comes
first, the order of both engine backends. Either passes the suite. It also
goes beyond the listed fix in `list` and the `.tmp` sweep, both needed for the
engine's behavior, and it updates `crates/playdate/README.md`.

Limit: a value that is not itself a good copy (`""`) keeps no backup, so where
rename is refused a cut in that one write can lose the old value. The engine's
backend does the same; saves are never empty.

For the Act 1 playtest (1064): each save now writes two flushed files instead
of one unflushed file, so check that saving on the device has no noticeable
hitch. This was not measured.

No gameplay rule was decided. No follow-up tickets. When 1170 lands, this
code goes with the rest of the backend.
