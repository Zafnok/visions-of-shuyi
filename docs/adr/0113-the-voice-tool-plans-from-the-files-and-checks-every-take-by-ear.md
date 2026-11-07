# ADR-0113: The voice tool plans from the files and checks every take by ear

- **Status:** Accepted
- **Date:** 2026-11-03
- **Related tickets:** 0721, 0722, 0760, 0757, 0761
- **Extends:** ADR-0046 rule 7, ADR-0107 rules 2 and 3, ADR-0109 rule 3.

## Context

`cargo xtask voice` (0721) makes the clips that are missing or stale, again and
again as scripts change. The manifest the game reads (`voice.ron`) has a fixed
set of fields, checked by the engine with no room for more: a clip's line,
variant, words, direction, voice and maker. Three changes leave all of them
equal and still call for a new clip: a new spelling for the readers (0721 step
6), a reference clip replaced under the same name, and a reader swapped for a
voice. The readers are Python programs in their own environments that take
half a minute to load their models.

## Decision

1. **The plan is a pure function** of the scripts as written, the names, the
   cast, `say.ron`, the manifest and `sent.ron`: the clips to make and why, the
   recorded clips to record again, the clips to remove
   (`crates/xtask/src/voice/plan.rs`).
2. **`sent.ron` beside the manifest** records, per clip file, the words sent
   to the reader (after `say.ron`) and a fingerprint (FNV-1a 32) of the voice's
   reference clip. A clip whose record differs is made again. A clip with no
   record (one placed by hand) is judged by the manifest alone. The game never
   reads it.
3. **A reader change is seen in `made_by.tool`**, and a voice change in the
   manifest's `voice`, the reference clip's name.
4. **`cast.ron` takes ADR-0107's form**: each entry names its reference clip
   and reader. The engine's older form (speaker to voice id) stays only where
   0760 removes it.
5. **The readers run in batches.** One child process per reader per seed
   reads every pending line (`tools/voice/lab.py`), so a model loads once per
   batch, not once per line. A reader, the transcriber and the audio step are
   each behind a trait (`Synth`, `Ear`, `Audio`), with fakes for the tests.
6. **Every take is transcribed**, directed or not, and kept when its words are
   the line's (or the respelled ones) to 85 % by characters; otherwise the next
   of four seeds is tried. ADR-0109 asked this of directed takes; round 2 of
   its listening found wrong words in undirected takes too.
7. **Levels by ffmpeg**, as the music's import does: trimmed to 0.1 s of
   silence at either end, then brought to −18 LUFS integrated with the true
   peak at most −1 dBTP. A directed clip keeps its take's distance from its
   reference clip's level, between −12 and +6 dB. All four numbers are
   tunable.
8. **A clip of a line that is gone is removed**, a recorded one too: the game
   refuses a manifest that names a line it doesn't have, and the recording
   stays in the private repository's history.

## Consequences

- A rename, a new spelling, a new direction or a new voice remakes exactly the
  clips it touches, and the tests prove each.
- Over a few hundred lines, `sent.ron` adds a small file to `voice/`; it ships
  with the folder and is harmless.
- A take that says the right words badly passes the check. A person still
  listens (0722), and removing a clip's entry makes it again.
- Speech-to-text spells invented names its own way, so the 85 % margin lets a
  short line with one misheard name fail where a long one passes. The four
  seeds and the report cover it.

## Alternatives considered

- **Extra fields in `voice.ron`.** The engine's reader denies unknown fields,
  and the game has no use for them.
- **Comparing `say.ron` with a saved copy.** Knows which word changed but not
  which clip used it once names change too; the per-clip record does.
- **One process per line.** Simple, and a model load per line.
- **Loudness in Rust.** Testable without ffmpeg, but a second measure that
  could disagree with the music's.

## Follow-ups

- **A short line is also checked by sound** (0761). In 0722 the lead's
  "Dace's." was read right by both voices, but the transcriber spelled the
  name "Dases" or "Daces", 0.67 to 0.8 alike, and no take of twelve passed.
  A line of at most four words (`SHORT_LINE_WORDS`, *tunable*) now also
  passes when the transcript has as many words and each sounds the same:
  a Metaphone-like key of the word (`synth::sound`: `c` before `e`, `i`
  or `y` and `z` are `s`, `ph` is `f`, vowels after the first letter
  dropped, a doubled sound once). Every other line keeps the 85 % rule.
  The cost: a short line read as a sound-alike word ("held" for "hold")
  passes; a person listens to it in the Act 1 playtest (1064). Rejected:
  `generate --keep <clip>`, which saves an unchecked take for someone who
  listened: it needs a listener on every such line of every chapter
  (0757), where the key needs none.
