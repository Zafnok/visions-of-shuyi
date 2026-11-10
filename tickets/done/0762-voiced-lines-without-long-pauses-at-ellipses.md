---
id: "0762"
title: "Voiced lines without long, unnatural pauses at ellipses and short sentences"
type: tuning
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: []
nick_input: none
completed: 2026-11-10
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

- [x] `cargo xtask voice audit en` (or as named) lists each clip's
      length, pace and longest inner pause, and flags those over the
      limits (unit tests on the parsing of `silencedetect`'s output and
      the flagging).
- [x] The text sent for `Wait... no. Not yet.` (or a real Chapter 1
      line with ellipses) is the normalized form (unit test); the line
      on screen is unchanged (test).
- [x] After regenerating, the audit flags no Chapter 1 or Vix clip, or
      the Completion notes list each one left with its reason; the
      before/after report is in the PR.
- [x] `assets-private.rev` points at the commit with the new clips.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the normalization; the audit's flagging; reading
  `silencedetect` output.
- Integration: none beyond the existing voice-tool tests (the readers
  run only on Nick's machine).

## Completion notes

**Done.** `cargo xtask voice audit` (`crates/xtask/src/voice/audit.rs`)
lists every clip's length, letters, pace (s/letter) and longest pause
inside it (ffmpeg `silencedetect`, every sample under −40 dB for 0.15 s
or more, stretches 0.05 s apart joined, the ends left out) and flags those
over the limits. `generate` audits every take. The text sent to a reader
goes through `for_reader` (`voice/files.rs`) after `respell`; the line on
screen and the manifest's `spoken` keep the script's marks. 41 Chapter 1
and guide clips were made again (private repository commit `87bb99b`,
pinned).

**The trial** (step 3): the 14 worst lines, seed 7, four ways of sending
the marks.

| Sent as | Total | Mean pace | Pause > 0.6 s | Pace > 0.17 |
| ------- | ----- | --------- | ------------- | ----------- |
| written | 159.4 s | 0.196 | 10 | 8 |
| `...` and `--` → `,` or `.` | 135.5 s | 0.164 | 10 | 7 |
| one `…` | 150.6 s | 0.183 | 11 | 8 |
| `...` → `,` or `.`, `--` → `—` | 136.4 s | 0.165 | 10 | 6 |

Picked the last: punctuation saves about 15 %, and at a break (`--`)
the em dash read the cut-off as one; a full stop there made "that's all
they. Your Grace." its own sentence and was slower (12.3 s against
10.3 s). A run after a sentence's end goes (the full stop already
pauses); the table in `docs/voice.md` has every case.

**Deviations from the ticket:**

- **Pause limit 0.8 s, not 0.6 s** (the agent's starting value). At 0.6 s
  the audit flagged 13 of the 75 Chapter 1 and guide lines with no
  direction and no `...` or `--`: their 90th percentile was 0.64 s, so
  0.6 s flagged ordinary sentence breaks. 0.8 s is that percentile plus a
  margin, the way the ticket sets the pace limit.
- **Pace limit 0.17 s/letter**, per the ticket: Chapter 1's and the guide's
  90th percentile before (0.157) plus a margin; judged only for lines of
  20 letters or more (a word or two is mostly breath).
- **A take over the limits gets the next seed** (the ticket asked only for
  a warning). The punctuation alone left most directed lines over: their
  median longest pause was 1.04 s (plain lines 0.41 s), and the same line
  varies a lot between takes. `generate` now treats a take over the limits
  like one with the wrong words, keeps the line's best take, and warns
  only if all four seeds are over. 0757's clips get it too.
- Shortening the silences inside a take with ffmpeg's `silenceremove` was
  tried and dropped: on the retainer's death line it cut 6.7 s from 19.6 s,
  so it was eating quiet speech, not only pauses.

**Before and after** (Chapter 1 and the guide, 119 clips): 953 s → 873 s;
pace 90th percentile 0.157 → 0.124 s/letter. Flagged at the first limits
(0.6 s, 0.095): 66. At the final limits: 27 → 9. The 41 clips made
again: 474 s → 393 s; the worst, the keeper reading the letter
(`ch01_prebattle_91081df4`), 28.9 s → 15.1 s, and the retainer's death
line 19.6 s → 10.2 s. The full table is in the PR.

**Left flagged** (each a directed line whose direction asks for a slow,
broken delivery; the best of four takes is kept):

| Clip | Direction | Over |
| ---- | --------- | ---- |
| `ch01_death_retainer_630229ac` | weak, in a low voice, more tired than afraid | pause 0.98 s |
| `ch01_intro_b7d5aee5` | tired and offhand, not looking up | pause 0.84 s |
| `ch01_prebattle_43740f2e` | in a low voice, full of grief ("Aye. ...Burn it. Somebody. I can't.") | pace 0.237 s/letter (5.2 s for 18 letters; four one-word sentences) |
| `ch01_prebattle_91081df4` | reading it aloud, then her own voice, unsteady | pause 0.91 s |
| `ch01_retreat_keeper_d5cc5a69` | dazed, oddly calm | pause 0.93 s |
| `ch01_victory_77ff12e5` | in a low voice, full of grief | pause 0.85 s |
| `ch01_victory_7f43c8ee` | in a low voice, full of grief | pause 0.85 s |
| `ch01_victory_85aa9a96` | in a low voice, full of grief, and the question plainly | pause 1.01 s |
| `ch01_victory_d300878f` | in a low voice, full of grief, and the question plainly | pause 0.89 s |

For the next script pass (not changed here: dialogue writing is paused):
`ch01_prebattle_43740f2e` is four one-word sentences under a grief
direction, and no take reads it faster than 0.237 s a letter.

**For the Act 1 playtest (1064):** the agent's starting values to judge by
ear are the pause limit 0.8 s, the pace limit 0.17 s/letter, and the
punctuation table in `docs/voice.md` ("Punctuation the readers are
sent"); the nine lines above are the ones to listen to first.

Friction: estimate 210 production lines, measured 281 (1.3×), under 2×.
PR size: estimate 650, measured about 850.
