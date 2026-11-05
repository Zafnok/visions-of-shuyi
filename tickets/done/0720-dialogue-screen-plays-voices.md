---
id: "0720"
title: The dialogue screen plays each line's voice
type: feature
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["0043", "0238"]
nick_input: answer-first
completed: 2026-11-04
---

# 0720 — The dialogue screen plays each line's voice

## Context

0238 can play a clip for a dialogue line
([ADR-0046](../../docs/adr/0046-voice-clips-by-line-id.md)). Nick decides
in 0043 what is voiced and how voice and text go together
(`docs/design/voices-languages-and-script.md`, *Voices*). This ticket
makes the dialogue screen ask for the clips.

## Nick input

**Answer first:** ticket 0043 (questions 2, 4 and 7). Build what the
design doc says; anything a player would notice that it doesn't cover
goes back to Nick through `ask-nick`.

## Scope

**In:** `crates/ui/src/screens/dialogue.rs` asking for, stopping and
preloading voices; scenes shown during battles (ADR-0030) included, since
they use the same screen.

**Out (do not do):** voices outside dialogue scenes (0043 chose no battle
shouts; Vix's lessons and tips are 0476); the auto-advance mode (0859);
Options (0826); changing the typewriter.

## Implementation steps

1. When a text box for a line appears, call `ctx.audio.play_voice(line
   id, lead gender)`. A line split over several pages plays its clip on
   the first page only and keeps playing across the pages.
2. Advancing to another line, skipping the scene, opening a reply choice
   or leaving the screen calls `stop_voice()` first.
3. Which lines ask at all follows 0043: speech lines (the lead's included)
   and narration are voiced; reply text is not. Put the rule in one function
   (`fn is_voiced(step) -> bool`) with the design doc's sentence quoted
   above it.
4. When a scene starts, and after each line, send `PreloadVoices` for the
   lines that can come next (both branches' first lines at a choice).
5. The typewriter and text speed behave as today. The first press fills in
   the box and leaves the voice playing; the next press goes to the next
   line and stops the voice (0043, question 7).
6. Rewinding a battle past a scene, or replaying one, plays voices again
   like the first time.
7. **The dialogue log** (0729, `docs/design/look-and-feel.md`, *Dialogue
   log*): jumping to a line from the log shows its box again, so steps 1–2
   play its voice as usual. **Moving the log's cursor onto a line plays
   that line's voice** (stopping the one playing), unless the Options
   setting *Voice on hover in the log* (0826,
   `ctx.settings().game.log_voice`) is off. Voices off altogether
   (`Settings::voices`) already makes `ctx.play_voice` do nothing. The log's rows know
   their line (`LogRow::line` in `screens/dialogue/log.rs`); a reply row
   follows step 3's rule for reply text.

## Acceptance criteria

- [x] Harness test with `test.dlg` and 0238's test manifest: each voiced line queues one `PlayVoice` when its box appears; advancing queues `StopVoice` before the next.
- [x] Skipping a scene queues no `PlayVoice` for skipped lines and stops the current one.
- [x] With voices off, no voice request is queued.
- [x] Reply text never asks for a voice; speech, the lead's speech and narration each do (one test per kind).
- [x] The first press on a typing line fills the box and queues no `StopVoice`; the next press queues it (test).
- [x] The dialogue log: moving its cursor onto a voiced line queues its `PlayVoice` (and none with *Voice on hover in the log* off); jumping to a line plays it as the box appears.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot / integration: Harness tests on `audio_requests()`; no snapshot changes.

## Completion notes

Done. `crates/ui/src/screens/dialogue.rs` asks for voices after each
frame's reading (`DialogueScreen::voice`): a text box that came up (once
any `@opening` before it has played) stops the voice asked for and says
its line; a choice opening stops it and says nothing; leaving the screen
stops it. `is_voiced(RowKind)` holds the design doc's rule, quoted above
it, and serves both the text box and the log. After each box (and at the
start) `PreloadVoices` lists the next 3 lines, or at a choice the first
line of each reply's reaction (`coming_lines`, ADR-0046's "3 ahead"). The
log's cursor says the line it moves onto unless *Voice on hover in the
log* is off; a jump shows the box again and says it as usual. Tests:
`crates/ui/src/screens/dialogue/tests/voice.rs` (11 tests on the test
scene with 0238's test manifest, a clip for every line). `docs/voice.md`
now describes what the screen does.

Deviations: the tests drive the screen's `update` with a test `Ctx`, as
0238's tests do, not the integration `Harness`: the test manifest helper
(`screen::tests::test_voices`) is crate-private. The debug menu's
**Voice test** stays (it says clips one per press without a scene).

The agent's starting rules (where the design doc is silent):

- Opening the log says nothing; only moving its cursor does. Closing the
  log leaves whatever voice is playing.
- The cursor moving onto a reply, or onto *Back to latest*, leaves the
  voice playing.
- A line whose box waits behind an `@opening` is said when the box comes
  up after the opening, not while the opening plays.

For the Act 1 playtest (1064): how the voice and the typewriter feel
together, and whether the log's hover voice is welcome.
