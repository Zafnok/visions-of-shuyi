---
id: "0863"
title: Auto-advance waits for the real end of the clip
type: feature
milestone: M7 Chapter 1 & game flow
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: ["0859"]
nick_input: none
completed: 2026-11-14
---

# 0863 — Auto-advance waits for the real end of the clip

## Context

0859 added *Auto-advance* (`docs/design/options.md`, *Auto-advance*): a
voiced line's last page moves on 0.5 s after its clip ends. The engine
didn't report when a clip ends, so 0859 takes a clip to last 0.08 s a
character (`SPEECH_PER_CHAR_S` in `crates/ui/src/screens/dialogue.rs`,
marked `// engine-friction: engine#2255`). A clip shorter than that waits
too long; a slower one is cut off. engine#2255 (Zafnok/shuyi-engine#301)
has the platform report when a voice clip has ended; the pin move of
2026-11-07 passed it. Read its completion notes for the API it names.

## Nick input

None. Verified by tests.

## Scope

**In:** the dialogue screen reads the real end of its clip; delete
`SPEECH_PER_CHAR_S`, `speech_s` and the countdown.

**Out (do not do):** any change to the pause, the reading time or the
setting; Playdate voices (1048).

## Implementation steps

Expected size: about 20 production lines changed, judged by 0859's
`auto_advance`.

1. ~~Move the pin~~ (done by the pin move of 2026-11-07).
2. In `crates/ui/src/screens/dialogue.rs`: a voiced box counts as ended
   once the engine reports its clip has ended (engine#2255's API; a clip
   skipped as late or never loaded counts as ended); the last page then
   waits `AUTO_PAUSE_S` from that moment.
3. Remove every `engine-friction: engine#2255` mark.

## Acceptance criteria

- [x] On: a voiced line's last page moves on `AUTO_PAUSE_S` after the clip the Harness plays ends, for a short and a long clip (Harness tests, time stepped).
- [x] A clip skipped as late counts as ended (test).
- [x] No `engine#2255` mark is left (`rg "engine#2255" crates`).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: `crates/ui/tests/it/auto_advance.rs`'s voiced test, with
  the Harness clip's real length.

## Completion notes

- `crates/ui/src/screens/dialogue.rs`: a voiced box keeps a `Clip`
  (heard yet, seconds quiet) in place of the 0.08 s-a-character
  countdown. Each frame it reads `Ctx::voice_playing()` (engine#2255):
  while the box's own line sounds, the clip is on; once heard and then
  silent, it has ended. `SPEECH_PER_CHAR_S`, `speech_s` and the
  `engine#2255` marks are gone.
- A clip that never sounds (skipped as late, its file missing or
  failed) counts as ended after `VOICE_WAIT_S`, 0.5 s from when it was
  asked for: the engine's own rule (`Ctx::voice_playing`'s doc: "gives
  up within half a second"; its book's `VoiceWait`). The page then waits
  `AUTO_PAUSE_S` as for any ended clip. The engine exports no constant
  for the half second, so the game names its own.
- Tests: `auto_advance.rs`'s voiced test runs a 1 s clip (moves on at
  1.5 s, before the 3 s reading time) and a 5 s one (at 5.5 s), with
  `Harness::voice_length`; a new test plays a clip that never sounds
  (`voice_length(0.0)`, moves on at 1.0 s); a unit test checks that the
  time a clip waits to load doesn't count toward its pause.
- Pin: already past engine#2255 (moved 2026-11-07), no move here.
- Deviation: the ticket's steps don't mention the half-second wait for
  a clip that never starts; it follows from the engine's API (a screen
  sees its own clip only from the next frame).
- Starting rules: none new. Felt in play (Act 1 playtest, 1064): a voiced
  line now moves on 0.5 s after the actual voice, so short lines go by
  sooner than before.
- Engine friction: estimate 20 production lines (0859's `auto_advance`),
  measured 27 added, 14 removed (1.4×): under 2×.
- PR size: estimate about 150, measured 272 (the moved ticket counts
  as removed and added).
- Gates: `cargo doc` overflowed rustdoc's stack on `trpg-engine-watch`
  (untouched here) on this machine; it passed with `RUST_MIN_STACK`
  raised. CI's doc job is the check.
- Follow-up tickets: none.
