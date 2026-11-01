---
id: "0720"
title: The dialogue screen plays each line's voice
type: feature
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["0043", "0238"]
nick_input: answer-first
completed:
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
   setting *Voice on hover in the log* (0826) is off. The log's rows know
   their line (`LogRow::line` in `screens/dialogue/log.rs`); a reply row
   follows step 3's rule for reply text.

## Acceptance criteria

- [ ] Harness test with `test.dlg` and 0238's test manifest: each voiced line queues one `PlayVoice` when its box appears; advancing queues `StopVoice` before the next.
- [ ] Skipping a scene queues no `PlayVoice` for skipped lines and stops the current one.
- [ ] With voices off, no voice request is queued.
- [ ] Reply text never asks for a voice; speech, the lead's speech and narration each do (one test per kind).
- [ ] The first press on a typing line fills the box and queues no `StopVoice`; the next press queues it (test).
- [ ] The dialogue log: moving its cursor onto a voiced line queues its `PlayVoice` (and none with *Voice on hover in the log* off); jumping to a line plays it as the box appears.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot / integration: Harness tests on `audio_requests()`; no snapshot changes.

## Completion notes

