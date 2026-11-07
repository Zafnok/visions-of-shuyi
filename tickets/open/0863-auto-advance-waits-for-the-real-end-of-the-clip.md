---
id: "0863"
title: Auto-advance waits for the real end of the clip
type: feature
milestone: M7 Chapter 1 & game flow
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: ["0859"]
nick_input: none
completed:
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

- [ ] On: a voiced line's last page moves on `AUTO_PAUSE_S` after the clip the Harness plays ends, for a short and a long clip (Harness tests, time stepped).
- [ ] A clip skipped as late counts as ended (test).
- [ ] No `engine#2255` mark is left (`rg "engine#2255" crates`).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: `crates/ui/tests/it/auto_advance.rs`'s voiced test, with
  the Harness clip's real length.

## Completion notes

