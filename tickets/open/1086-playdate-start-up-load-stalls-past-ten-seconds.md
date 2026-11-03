---
id: "1086"
title: "Playdate: the start-up load stalls the run loop past 10 seconds and the game crashes on launch"
type: bug
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: []
nick_input: setup
completed:
---

# 1086 — The Playdate's start-up load stalls past 10 seconds

## Context

Found by Nick on 2026-11-03 while measuring ticket 1044 (the Playdate
title, PR #534): a release build with the bought art
(`cargo xtask playdate --release --private-assets --both`, uploaded from
the Simulator) **crashes on launch on the device**:

> run loop stalled for more than 10 seconds, pc=0x9004CEDA lr=0x00000000

No log line was printed, so it stopped before the load report is logged.
The Playdate OS ends a game whose update callback has not returned for
10 seconds.

- `pc` minus the code's load address (0x90000000) is `0x4CEDA`, which
  `arm-none-eabi-addr2line -f -C -e target/playdate/Source/pdex.elf
  0x4CEDA` places in `core::str::pattern::StrSearcher::new`: a `&str`
  search (`find`, `contains`, `split`, `split_once`, `replace` with a
  string pattern) was running when the device gave up.
- **The whole start runs in one update callback:**
  `crates/playdate/src/pd/entry.rs`, `update`, `Step::Load` calls
  `load(card)`, which calls `start_game` (the data, the player's
  storage, the sounds, the title) before returning.
- **It has grown far past what 1036 measured.** 1036's Simulator run
  loaded the data in 11 ms (the device took 4,591 ms in 1033's run, about
  400 times the Simulator). The Simulator's `load-report.txt` for the
  1044 build (`PlaydateSDK/Disk/Data/com.zafnok.visionsofshuyi/`) says:
  game data loaded in 190 ms, first frame at 262 ms; slowest steps
  `(the sounds)` 126 ms, `dialogue/ch08.dlg` 21.5 ms (all eight
  chapters' dialogue is parsed there), the player's settings and saves
  16.9 ms, `(the title)` 14.7 ms. At the device's few-hundred-times
  slowdown, that is minutes, not seconds.
- Nothing in 1044 changes the load (its change is the title's paint and
  the logo's one-bit picture); the loader is unchanged since 1035.

Ticket **1068** (load one chapter's assets at a time) is about memory
and fitting all eight chapters; it would also shrink the load, but it is
a bigger change and does not by itself keep a callback under 10 seconds.

## Nick input

**Setup:** once the fix is built, Nick uploads it to his Playdate (the
Simulator's *Device → Upload Game to Device*), launches it and pastes the
Simulator console's load report lines ("Loading card shown…", "Game
data loaded in…", "Slowest files…"). The same run gives 1044's device
frame time: press A at the title, leave it 3 minutes, paste the
"On title for 60 seconds…" lines too.

## Scope

**In:**
- Find what the device spends the start on: the Simulator's load report
  per step, and a profile on the PC of `start::data` (the content load)
  to find which `&str` search runs so often (the dialogue loader and its
  checks, the language files, the text tokens are the first suspects).
- **Split the start across update callbacks**, so the Playdate OS sees
  the run loop come back between steps: at least the data
  (`start::data`), the player's storage (`start::with_storage`), the
  sounds, and the title, each its own `Step` in `entry.rs`, the loading
  card staying up. Keep the load report's lines as they are (the steps'
  times then add up across callbacks).
- If one step alone is still too slow for the device (aim under 5
  seconds a callback, half the OS's limit), make it cheaper: fix a search
  that runs once per line of every script when once per file would do,
  or split that step further (the dialogue files a few at a time).
- Measure on the device (Nick's setup above) and put the numbers in the
  Completion notes.

**Out (do not do):**
- Loading one chapter at a time (1068).
- Changing what the PC or web build loads, or when.
- 1044's frame time beyond pasting the numbers into its PR (#534) or
  its ticket's notes.

## Implementation steps

1. Read `crates/playdate/src/pd/entry.rs` (`Step`, `update`, `load`,
   `start_game`), `crates/playdate/src/start.rs` (`data`,
   `with_storage`) and `crates/playdate/README.md` (*Load report*).
2. On the PC, time `start::data`'s parts (a test or a bench with
   `std::time::Instant`, or `cargo shuyi perf` spans) and find the
   `StrSearcher` callers (a profiler such as `samply`, or counting calls).
3. Add `Step`s so each part runs in its own update callback; a step's
   result goes in the next step's state. The `Card` stays shown.
4. Fix any hot spot step 2 found.
5. Build `cargo xtask playdate --release --private-assets --both`, run
   it in the Simulator (load report), then Nick's setup.

## Acceptance criteria

- [ ] The release build with the bought art starts on Nick's Playdate
      and shows the title; its load report is in the Completion notes.
- [ ] No update callback during the start takes more than 5 seconds on
      the device (the load report's steps, each its own callback).
- [ ] The Simulator's load report has the same lines as before.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the start's steps run in order, one per call, ending in the
  running game (`entry.rs`'s plain parts, or a step machine moved to a
  tested module beside `report.rs`).
- Integration: the Chapter 1 test against the Playdate's bundle
  (`crates/playdate/src/chapter_1.rs`) still passes, starting through
  the steps.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
