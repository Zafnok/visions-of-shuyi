---
id: "1298"
title: "The web build starts while its pack streams in: the title's files laid first"
type: infra
milestone: M8 Release
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1298 — The web build starts while its pack streams in

## Context

The engine pin move of 2026-11-18 passed engine PR #481 (engine ticket
2265, the engine's ADR-0128 §3): pack version 19 lays the folders a
`Bake` names first (`Bake::first(dir)`) at the front of the pack, in that
order, and records each one's byte range (`Index::spans()`).
`Source::arrived(at, len)` and `Bundle::arrived(dir)` say whether a
folder's bytes are in yet. Engine ticket 3318 (open) builds the web's
side: the page fetches the `.wasm` first and streams the pack into the
game in the order it was baked, so the loading card shows at about 1.2 s
and the first frame waits only for the files it reads; a step that
reads a file not yet there answers `Progress::Wait` (engine #501, 3316).

The game's release build of the desktops and the web reads `assets/`
(with the content laid over it) from the pack beside the program
(ADR-0120; `crates/app/src/main.rs`'s `startup`, `packed` feature), baked
by `crates/tools/src/game_tools.rs`'s `pack_bake`, which names no folder
first. 1284 measured that pack at 776,966 bytes; the bought art is inside
the `.wasm`, not the pack.

Once the pin passes engine 3318, the web page streams the pack. The game
must then read it as it arrives: either wait for the whole pack before
its first file read (as today, with the stream's gain lost) or lay the
title's files first and start on them.

## Nick input

None.

## Scope

**In:**
- ~~Move the pin past engine 3318 (first step), or wait for the open
  `engine-pin` PR that does.~~ Done by the pin move of 2026-11-19
  (engine #509), which also made the start wait for the whole pack
  before its load reads a file (`trpg_ui::start::when_files_arrived`,
  `trpg_content::bundle::arrived`): the behavior of a whole pack, kept.
  This ticket narrows that wait to the folders the title reads.
- Measure first, on the release web build (`cargo shuyi web --release`
  is CI's: use the `export` job's package): time to the loading card and
  to the title, with the pack whole and streamed, at the engine's
  throttled 25 Mbit/s.
- `pack_bake` names the folders the title and the first load read
  first (`Bake::first`), in the order the start reads them; the start
  reads through the engine's `Progress::Wait` while a folder hasn't
  arrived (`Bundle::arrived`), and later folders are asked for only when
  first needed.
- If the gain is under about 0.3 s (the pack is small), keep the game
  waiting for the whole pack before its first read, say so with the
  numbers, and still make the start correct on a streamed pack.

**Out (do not do):**
- Moving content or art into or out of the pack.
- The Playdate's bundle, which isn't a pack.

Estimate: about 60 production lines (the bake's folder list and the
start's wait), 80 of tests.

## Implementation steps

1. ~~Move the pin past engine 3318 (`engine-pin` skill) if no open pin PR
   has.~~ Done by the pin move of 2026-11-19.
2. Measure the release web build as it is (whole pack).
3. List the folders the start and the title read (`crates/content`'s
   loaders at start, `crates/ui`'s title); name them with `Bake::first`
   in `pack_bake`, the order the start reads them.
4. In the packed start, wait on `Bundle::arrived` for those folders with
   `Progress::Wait` (in place of `when_files_arrived`'s wait on the whole
   bundle, `arrived("")`); check a later folder (a chapter's) before its first
   read.
5. Measure again; record both in the completion notes and
   `docs/targets.md`.

## Acceptance criteria

- [ ] On a pack read through a source that answers `NotYet`, the game's
      start waits and then starts (a test over the engine's test source).
- [ ] `cargo shuyi bake`'s `laid first:` line names the title's folders.
- [ ] The web build's time to the loading card and to the title, before
      and after, are in the completion notes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `pack_bake`'s first folders; the start over a source that has
  sent only the first span.
- Integration: the packed build's existing pack test.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
