---
id: "0722"
title: Generate Chapter 1's voices
type: content
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["0716", "0720", "0721", "0826", "0728", "0737", "0754", "0758"]
nick_input: none
completed: 2026-11-06
---

# 0722 — Generate Chapter 1's voices

## Context

The first real use of AI voices (`docs/design/voices-languages-and-script.md`,
[ADR-0046](../../docs/adr/0046-voice-clips-by-line-id.md)): every
Chapter 1 line that 0043 says is voiced, in the voices Nick picked,
made with 0721's tool. 0716 finishes Chapter 1's English scenes, so this
waits for it. Not on the Chapter 1 playtest's critical path.

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> Nick plays Chapter 1 on Pages with voices
> on and says which lines or characters sound wrong (a misread name, a flat
> or odd reading, a voice that doesn't fit). His comments become a
> follow-up ticket; they don't block this PR.

## Scope

**In:** the clips and manifest for `ch01.dlg` in the private assets
repository; `say.ron` entries for Chapter 1's names; the pin
(`assets-private.rev`) in this repository.

**Out (do not do):** rewording script lines so they read better aloud
(write a ticket); other chapters (0757); Japanese and Chinese voices (0755,
0756); Vix's lines (0476); any clip committed to this repository.

## Implementation steps

1. `cargo xtask voice generate --dry-run`: note the line and character
   counts. The tools are free, so there is nothing to approve.
2. Add `say.ron` respellings for every name in Chapter 1; generate one
   line per name first and listen to each.
3. `cargo xtask voice generate`. Listen to every clip once (or, if
   listening isn't possible in the session, transcribe each clip with a
   speech-to-text tool and compare with `spoken`; list mismatches).
   Regenerate bad ones (remove the clip's entry from `voice.ron` and run
   `generate` again, `docs/voice.md`); a line that won't come out right after three
   tries is listed in the notes and left silent (remove its clip).
4. Commit to the private repository, `cargo xtask private-assets --pin`,
   commit `assets-private.rev` here.
5. Check the Pages build's size and first-scene loading on the web
   build; write both in the notes.

## Acceptance criteria

- [x] `cargo xtask voice status` reports no missing or stale clip for `ch01` (besides lines listed as left silent).
- [x] Every clip is OGG Vorbis, 44.1 kHz, mono, loudness-matched (0721's `generate` makes them so; it has no separate check command, so measure a sample with ffmpeg's `ebur128` as `assets-src/audio/import.py --check` does).
- [x] The web build plays the first scene's voices without a gap longer than the text box appearing (checked in the browser).
- [x] `THIRD_PARTY_ASSETS.md`'s two entries under *AI voice tools* become rows of its table, marked AI-generated and *private*, each with a credit in `assets/data/credits.ron` (the credits test needs both), shown only in a build that has clips.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None new; 0238's and 0721's tests cover the code. The manifest is validated when the game loads it.

## Completion notes

**What was done.** Chapter 1 is voiced: 94 clips for the 21 scenes of
`ch01.dlg` in the private repository's `voice/en/` (pinned
in `assets-private.rev`), 6.3 MB, with its `cast.ron` (the cast table of
`docs/design/voices-languages-and-script.md`), `say.ron`, `voice.ron` and
`sent.ron`. The voice tools are credited: rows in `THIRD_PARTY_ASSETS.md`
(AI-generated, private) and two entries in `assets/data/credits.ron`, kept
off the general list (`hidden`); the credits screen shows them under a
**Voices** heading after the art, with the decided lines "Voices generated
with AI: Qwen3-TTS, Chatterbox." and "We hope to replace these with a human
cast.", only when the game found clips (`Ctx::has_voices`).

- **Dry run:** 96 clips, 62 undirected (5,438 characters) and 34 directed
  (3,866 characters).
- **Names (step 2):** every name Chapter 1 says (Hollis, Tamsin, Rook,
  Aske, Rue, Dace, Harl, Red Harl, Maud, Veyne, Veyne Hall, Harrowby, the
  Thornmarch, Greywater Abbey, the Red Company, the Ash Peace) was read in
  one line by both readers (Qwen3 with the narrator's voice, Chatterbox
  with Hollis's) and transcribed. Only Aske was misread (Qwen3 said
  "Askey"), so `say.ron` has one entry, `"poacher": "Ask"`.
- **Checking the clips (step 3):** no one listened to them. The tool
  transcribes every take and keeps it only if it says the line's words
  (0721), so every saved clip's transcript matched its `spoken` (or the
  respelled words). All 94 are Vorbis, 44.1 kHz, mono (ffprobe). A sample
  of 10 measured with `ebur128`: undirected clips −18.0 to −18.9 LUFS; the
  directed death and retreat lines quieter (−20.7, −23.5), as 0721's
  directed rule keeps them.
- **Left silent:** `ch01_prebattle_a4c1d405`, the lead's "Dace's." (both
  `.m` and `.f`). The readers said it right, but the transcript spells the
  name "Dases", too far from a six-letter line; three tries (the first
  run, then `say.ron` spellings "Daces" and "Dayces") gave a female take
  but no male one, and a line with one gendered clip makes the game refuse
  the whole manifest, so neither is kept. Follow-up: **0761**.
- **Web build (step 5):** `cargo shuyi web --release --debug-tools
  --private-assets` is 91 MB: the wasm 10.5 MB, music 75 MB, voices
  6.3 MB. In the browser the game fetched `voice.ron` with no warning,
  asked "Play with AI-generated voices?" (only asked when clips were
  found), and the first scene fetched its first four clips (42–93 KB) as
  it opened, each in 5–7 ms locally, then one more per line, three lines
  ahead. Audio wasn't heard in the session; the clips are loaded long
  before their box appears, under the 0.3 s skip.

**Deviations.**

- Voiced scene by scene (`--scene` for each of the 21), since the tool's
  default is every scene of every chapter; removals are limited to the
  scenes voiced, so nothing of another chapter was touched.
- `cast.ron` doesn't list Vix (`guide`): the tool refuses a cast entry for
  a speaker with no lines, and her lines are 0476's.
- The ticket said `cargo xtask private-assets --pin`; it is `cargo shuyi
  private-assets --pin` now.

**Follow-up tickets:** 0761 (short lines the transcript check can't hear).

**The agent's starting rules** (for the Act 1 playtest, 1064, by ear):

- Names are said as spelled, Aske as "Ask", Dace to rhyme with "face",
  Veyne as "Vain", Rue as "Roo": no pronunciation guide exists, these are
  how the readers say them.
- The voices' credits come right after the art's.

For the playtest checklist (1064): Nick plays Chapter 1 on Pages with
voices on and says which lines or characters sound wrong (a misread name,
a flat or odd reading, a voice that doesn't fit).
