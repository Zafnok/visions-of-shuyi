---
id: "1086"
title: "Playdate: the start-up load stalls the run loop past 10 seconds and the game crashes on launch"
type: bug
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: []
nick_input: setup
completed: 2026-11-03
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

- [x] The release build with the bought art starts on Nick's Playdate
      and shows the title; its load report is in the Completion notes.
- [x] No update callback during the start takes more than 5 seconds on
      the device (the load report's steps, each its own callback).
- [x] The Simulator's load report has the same lines as before, plus
      one: "Start, ms per update callback" (see the notes).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the start's steps run in order, one per call, ending in the
  running game (`entry.rs`'s plain parts, or a step machine moved to a
  tested module beside `report.rs`).
- Integration: the Chapter 1 test against the Playdate's bundle
  (`crates/playdate/src/chapter_1.rs`) still passes, starting through
  the steps.

## Completion notes

**What the device was doing.** The first crash's address
(`StrSearcher::new`) and the second's (`literal_form`, after the first
fix) were both in the engine's literal-name check. That check ran a
substring search for each of the 130 names on each of the 5,000 lines,
and found each name's literal form again on each line. The longest
name's width was also worked out over the whole table two or three
times per line. All three are fixed in the engine
(`Zafnok/shuyi-engine#250`, which this PR's pin move brings in): the
literal forms are found once per scene, indexed by first byte, and the
longest name once per scene. The PC's content load went from 28 ms to
15 ms.

**What was done.**
- The start is a step machine, `crates/playdate/src/boot.rs` (`Boot`),
  that `entry.rs` runs one step per update callback, the loading card
  on the screen. The steps are data, dialogue, scenes, context,
  storage, sounds, title. ADR-0111 records it.
- The data is four of those steps. `trpg_content::StagedLoad` is
  `load_embedded` in parts, and the PC and web still load in one call
  (`load` runs the same stages back to back).
- The dialogue's scenes are checked one at a time (the engine's new
  `ScriptsCheck`; `trpg_content::dialogue::ScriptsLoad`), and the
  sounds load one file at a time (`Audio::new`, `Audio::load_next`).
  Both stop after one second of a callback and go on in the next. Each
  loop is bounded by the parts left, counted before it starts, so a
  mutant can't make it endless (the engine's mutation run timed out on
  a `while` loop).
- The load report's times leave out the time between callbacks, so the
  lines read as before.

**Deviations.**
- The four steps the ticket named weren't enough. After the first fix
  the data step alone stalled past 10 s on the device: the device runs
  this load about 600 to 1,300 times slower than the Simulator, not
  400. So the data was split into three steps and the dialogue into a
  scene at a time.
- The load report has one new line, "Start, ms per update callback:
  ...". It is the only way to check the 5-second criterion on the
  device. The other lines are unchanged.

**The device's load report** (Nick's Playdate, the release build with
the bought art, the last run). That build read the dialogue's files
and began the checks in one callback, the "dialogue 2,014". After this
run the reading got a callback of its own ("dialogue", then "scenes"
callbacks), which only splits that callback further. The Simulator
showed the same steps in the same order with the new names.

```
Loading card shown 322 ms after start
Game data loaded in 15,351 ms
Game stack used: 31,912 of 65,536 bytes
Bundle: 400 reads of 237 files, 0 not in the bundle
Slowest files, ms to the next read: dialogue/ch08.dlg 6700, (the title) 874, tilesets/tiny_tales.ron 771, data/camp.ron 619, battles/act1_random_ash.ron 598, fonts/atlas.png 484, battles/ch01.ron 464, data/classes.ron 303, audio/audio.ron 256, lang/en/ui.ron 248
Start, ms per update callback: data 2,097, dialogue 2,014, dialogue 1,029, dialogue 1,022, dialogue 1,008, dialogue 1,016, dialogue 1,021, dialogue 892, context 4,069, storage 98, sounds 190, title 895
Sounds: 21 files loaded
First frame of the game shown 15,891 ms after start
```

The run before it, with the dialogue still in one callback, gave
1044's title numbers. Nothing in this ticket's last change touches the
title.

```
On title for 60 seconds: 19.3 frames a second
Game::frame: min 4.1 / avg 6.4 / max 61.1 ms, n = 1,158
Render and copy: min 7.5 / avg 44.4 / max 84.3 ms, n = 1,086
Game stack used: 31,928 of 65,536 bytes
Music: none
```

The Simulator, same build: data 3, dialogue 7, context 5, storage 1,
sounds 3, title 7 ms. Game data loaded in 26 ms (190 ms in the 1044
build), first frame at 219 ms.

**Notes.**
- The longest callback left is "context", 4.1 s on the device: the rest
  of the data after the dialogue, and the content put together. It is
  the next to split if the data grows (ADR-0111). Loading one chapter
  at a time (1068) would shrink every step.
- The start takes about 16 seconds on the device behind the loading
  card. That's a feel for the Act 1 playtest (1064), not a rule.
- Friction: estimate 250, measured 585 (2.3×), accepted. See
  `docs/engine/friction.md`. The estimate was written after the first
  build, not before coding as the rule asks.

No gameplay rules were decided.
