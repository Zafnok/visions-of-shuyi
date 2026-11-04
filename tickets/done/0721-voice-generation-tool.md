---
id: "0721"
title: "`cargo xtask voice`: generate the missing and stale clips, and print recording scripts"
type: infra
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["0043", "0238", "0754"]
nick_input: none
completed: 2026-11-03
---

# 0721 — `cargo xtask voice`: generate the missing and stale clips, and print recording scripts

## Context

Voice clips are made outside the game by a tool and kept in the private
assets repository ([ADR-0046](../../docs/adr/0046-voice-clips-by-line-id.md)
rules 5 and 7). Nick picked the tools and each character's voice in 0043
(`docs/design/voices-languages-and-script.md`, *Voices*), and
[ADR-0107](../../docs/adr/0107-free-local-voice-tools-with-designed-voices-and-a-reader-per-character.md)
says how they run: designed reference clips in
`assets-src/audio/voice-refs/`, read by Qwen3-TTS or Chatterbox per
character. Scripts will keep changing (and a hired writer may rewrite
them), so the tool must redo only what changed, every time.

## Nick input

None. Both tools are free and need no account or key. They run on Nick's
machine; ADR-0107 says where they are installed, and the agent does it.

## Scope

**In:** the `voice` xtask with `status`, `generate` and `script`
subcommands; the cast file; loudness matching; the manifest written.

**Out (do not do):** generating Chapter 1 for real (0722); anything in
the game crates beyond reusing `trpg_content`; running in CI; calling
the tool from the game.

## Implementation steps

1. `voice/<lang>/cast.ron` (in the private repository's `voice/`):
   character id → reference clip and reader (`Qwen3` or `Chatterbox`), from
   0043's cast table. Narration has its own entry (`>`), and the lead's two
   voices are `lead_m` and `lead_f`.
2. `cargo xtask voice status [--lang en]`: loads the scripts, cast and
   manifest; prints per scene the lines that are voiced by 0043's rule
   and have no clip, a stale clip, or a clip whose cast voice changed;
   and clips whose line no longer exists.
3. `cargo xtask voice generate [--scene <id>] [--dry-run]`: for each line
   `status` lists, send `spoken_text` (0238) to the tool with the
   speaker's voice, save `voice/<lang>/<scene>/<line id>[.m|.f].ogg`
   (OGG Vorbis, 44.1 kHz, mono), and add or replace the manifest entry
   with `made_by: Generated { tool, model, date }`. Delete clips whose
   line is gone. `--dry-run` prints the count of lines and characters
   (what a service bills by) and makes nothing.
4. The tool is called through one small trait (`Synth: text + voice →
   audio`), with one implementation per reader behind it, so swapping tools
   later is one file. The readers are Python programs in their own
   environments (ADR-0107); the trait runs them as child processes. A fake
   implementation (a tone per line) is used in tests.
5. Loudness: match every undirected clip to one level, the way 0214's
   import matched the music (reuse its code), and trim silence at both ends
   to at most 100 ms (*tunable*). A directed clip keeps its distance from
   its character's level, within a floor and a ceiling (*tunable*), so a
   whisper stays quiet (ADR-0109 rule 4). Nick heard them unmatched in 0752.
6. Pronunciation: `voice/<lang>/say.ron`, name id or word → how to spell
   it for the tool (invented names are often misread). `spoken` in the
   manifest stays the real text; the respelling is only what is sent.
   A change to `say.ron` marks the lines using that name for redoing.
7. `cargo xtask voice script --character <id>`: prints a recording
   script for a human actor: per line the id, scene, the line before it
   (who said what), the words, and the file name to deliver. This is how
   a cast replaces the generated clips later (ADR-0046 rule 10).
8. Never overwrite a `Recorded` clip; `generate` skips them and `status`
   lists them as stale for a human to re-record.
9. **Voice directions (0752, done):** `Line::direction` goes through the
   `Synth` trait, is printed in `voice script`, and is written to each
   clip's `direction` in the manifest. `status` and `generate` redo a clip by
   `trpg_content::voice::needs_remake`. A line with no direction goes to its
   reader as in step 4. A directed line takes the recipe of ADR-0109, whoever
   reads the character: Nick picked it by ear, Hollis and Vix included.
   `generate` keeps the first take whose transcript is the
   line's words and retries the rest; a line with no good take is reported,
   not saved. `--dry-run` counts directed lines apart: each costs more.
9b. **Spoken words (0754):** send what the line says aloud, with the
   stand-in for the player's name, not what the box shows.
10. `docs/voice.md`: how to run all three, where the files go, and the
   commit-and-pin steps for the private repository (`cargo xtask
   private-assets --pin`).

## Acceptance criteria

- [x] With the fake synth: `generate` on `test.dlg` makes one clip per voiced line; a second run makes none (test).
- [x] Rewording a line, renaming a name in it, changing its direction, or changing its speaker's voice makes exactly that line's clip regenerate (tests).
- [x] A `Recorded` clip is never overwritten (test).
- [x] `--dry-run` writes nothing and prints the counts.
- [x] `script --character` output for a test character is snapshot-tested.
- [x] One real line generated with the chosen tool plays in the game (say which, in the notes).
- [x] No key or bought file is committed to this repository.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the planning (what to make, redo, delete) as a pure function of scripts, cast, manifest and `say.ron`.
- Integration: a temp folder run with the fake synth.

## Completion notes

**What was built.** `cargo xtask voice status | generate | script`
(`crates/xtask/src/voice.rs`, `voice/{files,plan,synth}.rs`), the readers in
`tools/voice/lab.py`, `docs/voice.md` (how to run it, the files, the
commit-and-pin steps), and ADR-0113 for the tool's choices.

- **The plan** is a pure function of the scripts as written, the names, the
  cast, `say.ron`, the manifest and `sent.ron` (`voice/plan.rs`). Voiced:
  every speech and narration line of the scenes in `assets/dialogue/` (or
  one `--scene`, test scenes too); replies are silent. The lead's lines get
  both of the lead's voices (`.m`/`.f`); a line with a pronoun gets a pair
  in one voice.
- **Redo reasons:** no clip; the words changed (`is_stale`); the direction
  changed (`needs_remake`); the voice changed (the cast's clip, its reader,
  or the reference clip's contents); `say.ron` spells a word of it
  differently. A `Recorded` clip is never made; `status` lists it for
  re-recording. Clips of lines that are gone, or of a line that changed
  between one clip and a gendered pair, are removed.
- **`sent.ron`** (new, beside the manifest) records the words sent and the
  reference clip's fingerprint per clip: the engine's manifest reader denies
  extra fields, so this is how a `say.ron` change or a replaced reference
  clip is seen (ADR-0113).
- **Readers:** one batch per reader per seed, as child processes in the
  lab's environments: Qwen3 (Base clone), Chatterbox, and ADR-0109's
  recipe for every directed line (VoiceDesign take with the instruct turn,
  converted by ChatterboxVC). Every take, directed or not, is transcribed
  (faster-whisper `base.en`) and kept if it says the line's words (85 % by
  characters, the lab's margin); else the next of four seeds; a line with
  no good take is reported and not saved.
- **Loudness:** ffmpeg as the music's import does: trimmed to at most
  0.1 s of silence at either end, −18 LUFS integrated, true peak at most
  −1 dBTP; a directed clip keeps its take's distance from its reference
  clip's level, within −12 to +6 dB. All four are marked *tunable* in
  `voice/synth.rs`.

**The real line (acceptance).** The test scene's 16 clips were generated
with the real tools into a scratch folder: Qwen3-TTS for the narrator, the
lead and `test_lord` (cast as `rival`), Chatterbox for `test_knight`
(`retainer`) and `test_archer` (`poacher`), and the directed recipe for the
two directed lines. All 16 passed the transcript check on the first seed.
`status` then reported nothing to do. Every file is Vorbis, 44.1 kHz, mono;
undirected clips measured −17.9 to −19.4 LUFS (a few held back by the peak
ceiling), the directed ones −20.3 ("slowly, tired") and −21.5 ("warmly,
teasing"). The game's own manifest check, staleness filter and OGG length
reader accepted all 16 as playable. No agent listened to them.

**Deviations.**

- `cast.ron` takes ADR-0107's form (clip and reader per entry), not the
  engine's speaker-to-voice-id form that `trpg_content::voice::cast_from_source`
  reads. The game doesn't read the cast, and this ticket keeps out of the
  game crates, so the test folder's cast keeps the old form for now:
  **0760** removes it.
- The transcript check runs on undirected takes too, not only directed
  ones: ADR-0109's round 2 found wrong words in them as well.
- Running the transcriber needed PyTorch's CUDA libraries on the `PATH`
  (`lab.py`'s `hear`); the lab's own check had run without the GPU path.
- 0722 amended: how to force a clip again (remove its manifest entry), and
  that the tool has no separate loudness-check command.

**Follow-up tickets:** 0760 (one form for `cast.ron`).

**The agent's starting rules** (all tunable; the Act 1 playtest, 1064,
judges them by ear): voices at −18 LUFS, a little over the music's −20,
like the sounds; at most 0.1 s of silence before and after a line; a
directed line stays between 12 dB quieter and 6 dB louder than its voice's
level; Chatterbox's undirected lines at `exaggeration` 0.5 (its default).

Engine friction: estimate 750 production lines (`voice_test.rs` plus
`assets-src/audio/import.py`'s loudness part, and the planning), measured
1,203 (1.6×), under 2×; signals: workaround no · third copy no · budget
no. PR size: estimate 1,900, measured 2,727, under 3,000; the tests and
rustfmt's wrapping ran longer than planned. The work doesn't split cleanly
(the plan, the files and `generate` are tested together).

