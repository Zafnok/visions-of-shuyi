# ADR-0111: The Playdate's start runs a step per update callback

- **Status:** Accepted
- **Date:** 2026-11-03
- **Related tickets:** 1086

## Context

- The Playdate OS ends a game whose update callback has not returned
  for 10 seconds ("run loop stalled for more than 10 seconds"). Before
  ticket 1086 the front end ran the whole start in one callback: the
  data, the player's storage, the sounds and the title
  (`crates/playdate/src/pd/entry.rs`, ADR-0070).
- By ticket 1044 the data held all of Act 1's dialogue, and the release
  build crashed on launch. The device stopped in the dialogue's
  literal-name check, which ran a substring search per name per line.
  The engine made that check cheap (`Zafnok/shuyi-engine#250`), but the
  start still took 15 seconds on the device.
- The device runs the start 600 to 1,300 times slower than the Simulator
  (the Simulator runs native PC code). A step that takes a few
  milliseconds in the Simulator can take seconds on the device, so the
  Simulator alone never shows a stall coming.
- The PC and web builds load everything at once, and the content's
  tests and `cargo xtask check-script` depend on that one call
  (`trpg_content::load_embedded`, `load_with_scripts`).

## Decision

1. **One step per callback.** `crates/playdate/src/boot.rs` is the
   start as a step machine (`Boot`). Each update callback runs one
   step and returns, with the loading card on the screen. The steps,
   in order:
   1. `data`: what the dialogue is checked against
      (`trpg_content::StagedLoad::start`).
   2. `dialogue`: the dialogue's files read and parsed.
   3. `scenes`: the scenes checked one at a time (the engine's
      `ScriptsCheck`).
   4. `context`: the rest of the data, the content put together, and
      the Playdate's context over it.
   5. `storage`: the player's settings and saves.
   6. `sounds`: the sounds, one file at a time (`Audio::new`,
      `Audio::load_next`).
   7. `title`: the title.

   The plain step machine is tested on fakes (`boot.rs`, and
   `chapter_1.rs` starts through it). `entry.rs` runs it on the SDK.
2. **A time budget for the steps made of many small parts.** The
   scenes' checks and the sounds go on until the callback has run one
   second (`STEP_SECONDS`), then continue in the next callback. Every
   other step is one piece of work. Each of those loops runs at most as
   many times as there are parts left, counted before it starts
   (`scenes_to_check`, `to_load`), never "until a call says it is done":
   a mutant that never says so would hang the mutation tests.
3. **`trpg_content::StagedLoad`** is `load_embedded` in parts: `start`,
   `read_dialogue`, `check_scene` once per scene, `finish` (which checks
   any scene left). The content and the
   errors are the same. `load` itself runs the same stages back to back,
   so the PC and web builds load as before.
4. **The load report times each callback.** "Start, ms per update
   callback: …" lists each callback's step and time. The other lines
   are unchanged, and their times leave out the time between callbacks.
   The target is under 5 seconds per callback on the device, half the
   OS's limit. A step that passes it is split again or made cheaper,
   measured on the device.

## Consequences

- The start no longer has a single long callback. Measured on Nick's
  Playdate (ticket 1086), with the files read and the first checks in
  one callback: data 2.1 s, the dialogue 2.0 s and then six callbacks
  of about 1 s, context 4.1 s, storage 0.1 s, sounds 0.2 s, title
  0.9 s. The first frame showed at 15.9 s.
- The start takes a few frames longer in all (a frame between steps,
  33 ms at 30 frames a second), which is negligible here.
- A new loader added to the content goes in one of `StagedLoad`'s
  stages, so it adds to that stage's time on the device. `context` is
  the largest single piece (4.1 s), and the next one to split.
- The start's state lives in `Boot` between callbacks, so a step can't
  borrow from an earlier one; each step owns what it hands on.

## Alternatives considered

- **Only make the load cheaper.** The engine's check got cheaper (28 ms
  to 15 ms on the PC), but the same load was still 15 s on the device,
  and every new chapter adds to it.
- **Skip the content's checks on the Playdate** (the gates check the
  same bundle). This is the biggest saving, but it is a new loading mode
  for the whole content crate, and it would leave the device running
  data no one has checked in that build. Better left for a baked pack
  (ticket 1068's territory).
- **Load in the background** (a thread or coroutine). The Playdate has
  no threads for the game, and the SDK runs the game only from its
  callback.
