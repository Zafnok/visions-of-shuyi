---
id: "0762"
title: "Voiced lines without long, unnatural pauses at ellipses and short sentences"
type: tuning
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: high
status: in-progress
blocked_by: []
nick_input: none
completed:
---

# 0762 — Voiced lines without long, unnatural pauses

## Context

A friend's playtest of the Act 1 build, passed on by Nick (2026-11-07):

> "Some lines (not all, so we probably need an audit) of spoken / AI
> generated dialogue with many ellipses or periods will take too long to
> speak or have many pauses that sound unnatural."

The voice tool: `cargo xtask voice generate` (`docs/voice.md` ~:148–200).
A line's text goes through `trpg_content::voice::spoken_text`
(`crates/content/src/voice.rs` ~:103: names and pronouns) and `respell`
with `say.ron` (`crates/xtask/src/voice/files.rs` ~:98–125, whole words
only), then to the reader (`crates/xtask/src/voice.rs`, `request` ~:545;
`voice/plan.rs`, `spoken` ~:138, `respell` call ~:169). The readers are in
`tools/voice/lab.py`: Qwen3-TTS Base for cloned voices, Qwen3 VoiceDesign
for directed lines (ADR-0109), Chatterbox multilingual. **Nothing
normalizes punctuation:** `...` and `--` reach the reader as written.

The checks today: `synth::says` (`voice/synth.rs` ~:292) compares the
faster-whisper transcript at 85 %; 0761 added a same-sound word check for
short lines. `trim_filters` (~:374) trims only leading and trailing
silence (`KEEP_SILENCE_S`, ~:252). **Nothing measures a clip's length
against its text or the pauses inside it.**

The size (voiced lines are `speaker:` and `>` lines in
`assets/dialogue/*.dlg`, 2026-11-07): 1,059 voiced lines; 227 with `...`
(Chapter 1: 10), 12 with two or more; 188 with `--`; 395 with four or more
sentence ends; none with the single `…` character. Only Chapter 1 and Vix
are voiced; 0757 voices Chapters 2–8 and waits for this ticket, so the fix
is in place before those clips are made.

The clips are in the private repository's `voice/` (ADR-0046;
`cargo shuyi private-assets` copies them to `voice/`).

## Nick input

None. The fix is measured: no clip over the pause and pace limits below.
The limits are *the agent's starting values*, judged in the Act 1
playtest (1064's checklist).

## Scope

**In:**
1. **An audit command** (`cargo xtask voice audit`, or a flag on an
   existing voice command): for every clip of the language, its length
   (`shuyi_assets::audio::ogg_length_ms`), its seconds per spoken
   character, and its pauses inside the clip (ffmpeg `silencedetect`,
   which the tool already runs ffmpeg for: `FfmpegFinisher`, `synth.rs`
   ~:411). It flags a clip with a pause longer than a limit (start at
   0.6 s) or a pace slower than a limit (start at the 90th percentile of
   Chapter 1's clips plus a margin), and writes the list as a report.
2. **Text sent to the reader** gets a normalization step beside
   `respell`: `...` and `…` become what reads naturally for the reader
   (try `,` mid-line and `.` at the end, or one `…`; pick by ear on a
   handful of lines and by the audit's numbers), `--` becomes `,` or
   `—` the same way. The line's text on screen doesn't change. A change
   to the text sent changes `sent`, so those clips are made again by
   the next `generate` (that is how the tool already notices a changed
   line).
3. **Regenerate Chapter 1's and Vix's flagged clips** with the new step,
   on Nick's machine's tool setup (`docs/voice.md`), and audit again
   until none is flagged, or the remaining ones are listed in the
   Completion notes with why (e.g. a dramatic pause the script asks for
   with a direction).
4. The audit as a check the voice tool runs after `generate` (a warning
   per flagged clip), so 0757's clips get it.
5. `docs/voice.md`: the normalization and the audit.

**Out (do not do):**
- Rewriting the script's lines to drop ellipses (story text is the
  script tickets'; if the audit shows a line that can't be read well,
  list it for the next script pass).
- Japanese and Chinese clips (0755, 0756): the audit takes a language,
  but run it only on English here.
- Moving the tool onto the engine's cast and plan (1242). Either order:
  if 1242 is done, put the normalization where the plan builds the text
  sent; if not, beside `respell`.
- Ducking the music under voices (0865).

## Implementation steps

1. Read `docs/voice.md`, `voice.rs`, `voice/plan.rs`, `voice/files.rs`,
   `voice/synth.rs`, `tools/voice/lab.py`; check out the private
   repository's `voice/` (`cargo shuyi private-assets`).
2. Write the audit; run it on Chapter 1 and Vix; keep the report.
3. Try the normalization on the worst ten flagged lines (generate into
   the scratchpad), compare the numbers, pick.
4. Put it in the tool with tests; regenerate the flagged clips; audit
   again; commit the clips in the private repository and
   `assets-private.rev` here (`cargo shuyi private-assets --pin`).
5. The check after `generate`; docs.

## Acceptance criteria

- [ ] `cargo xtask voice audit en` (or as named) lists each clip's
      length, pace and longest inner pause, and flags those over the
      limits (unit tests on the parsing of `silencedetect`'s output and
      the flagging).
- [ ] The text sent for `Wait... no. Not yet.` (or a real Chapter 1
      line with ellipses) is the normalized form (unit test); the line
      on screen is unchanged (test).
- [ ] After regenerating, the audit flags no Chapter 1 or Vix clip, or
      the Completion notes list each one left with its reason; the
      before/after report is in the PR.
- [ ] `assets-private.rev` points at the commit with the new clips.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the normalization; the audit's flagging; reading
  `silencedetect` output.
- Integration: none beyond the existing voice-tool tests (the readers
  run only on Nick's machine).

## Completion notes

*(Filled in by the session that completes the ticket.)*
