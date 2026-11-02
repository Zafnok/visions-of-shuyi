# ADR-0109: Voice directions reach the voice design model, apart from the words

- **Status:** Accepted
- **Date:** 2026-11-02
- **Related tickets:** 0752, 0721, 0722, 0043, engine#3114
- **Amends:** ADR-0107's consequence that line directions have little effect
  with its readers, and its rule 2 for directed lines only. ADR-0046's rule 3 as
  ADR-0107 amended it stands: `spoken` still decides whether a clip plays.

## Context

0752 puts a direction on a script line ("whispering, frightened"), for a voice
tool or an actor, and never on screen. Nick asked that it be proven with the
real tools: "excitedly" must change the tone and must not be read aloud.

ADR-0107 found that neither reader takes a direction in words. Its public calls
don't, but Qwen3-TTS's `model.generate` has an instruct turn apart from the
spoken text. Tried on four characters, with two lines and four or more
directions each, measured and then heard by Nick in two rounds:

| Way | Measured | Nick |
| --- | -------- | ---- |
| Chatterbox's `exaggeration` from a table; or the Base or VoiceDesign weights with the character's clip as the reference take | Level, pitch and pace barely move: the reference take holds the delivery | "the direction doesn't change the delivery whatsoever" |
| **A.** VoiceDesign weights, the character's voice print only | The largest moves: a whisper 10 dB down, pitch doubled when excited | "too drastically alters the character's voice" |
| **B.** A, converted to the character's own clip | The voice stays; a whisper stays quiet | **Chosen** |
| **C.** A new performance from the voice's description, converted | About as good as B | Heard about level, and B was picked |

The speech-to-text check (faster-whisper `base.en`) heard no direction word in
any clip, and neither did Nick. It did find takes with the wrong words: 9 of 65
in round 2. "slowly, close to tears, voice breaking" broke the voice in every
way tried; Nick called it haunted.

## Decision

1. **The script.** `speaker (direction): text`,
   `speaker[expression] (direction): text`, `> (direction): text`, and
   `@delivery <direction>` / `@delivery clear` for a stretch
   (`assets/dialogue/README.md`, "Directions"). They parse in the engine's
   `.dlg` format (engine#3114). The direction is a field of the line, free text.
   The screen and the log read the text field, so they can't show it, and the
   line id doesn't hash it, so a translation doesn't go stale. `@delivery` makes
   no step of its own: the parser writes it on each line it covers.
2. **A direction is never in the text sent to a tool.** It goes in the model's
   instruct turn, the only place a model reads as instruction and not as words.
3. **Recipe for a directed line** (0721 builds it; the lab's scripts,
   `D:\voice-lab\scripts\r2_*.py`, ran it):
   - The voice print: `Qwen/Qwen3-TTS-12Hz-1.7B-Base`,
     `create_voice_clone_prompt(ref_audio=<the voice-refs clip>,
     x_vector_only_mode=True)`.
   - The take: `Qwen/Qwen3-TTS-12Hz-1.7B-VoiceDesign`,
     `model.generate(input_ids=<the spoken text>, ref_ids=[None],
     instruct_ids=[<the direction>], voice_clone_prompt=<that voice print>,
     temperature=0.75)`. No public method does this; `_build_instruct_text`
     and `_build_assistant_text` make the turns. `speaker_encoder` is on the
     Base weights only, which is why both load.
   - Keep the voice: `ChatterboxVC(<the multilingual model's s3gen>, "cuda")
     .generate(<the take>, target_voice_path=<the voice-refs clip>)`.
   - Check: transcribe the converted take, and keep the first whose words are
     the line's. Try up to four seeds. A line with no good take is reported.

   An undirected line stays with its reader, as ADR-0107 rule 2 says. Every
   character takes the same recipe when directed, whoever reads the rest of
   their lines.
4. **Level.** A directed clip keeps its distance from its character's level,
   within limits: matching every clip to one level would bring a whisper up to
   a shout. Nick heard the clips unmatched.
5. **Staleness.** A clip records its `direction`. `needs_remake` is true when
   the direction changed or the clip is stale. A changed direction leaves the
   clip playable, because it still says the words on screen and a silent line
   is worse.
6. **Words for sadness.** Writers use "in a low voice, full of grief", which
   Nick picked by ear over "sadly, on the verge of tears" and "quietly,
   heartbroken". "close to tears" and "voice breaking" stay out of the guides'
   examples.

## Consequences

- A directed line costs two model steps and up to four takes, so it is a few
  times slower than an undirected one. Writers are told to direct only where
  the words don't carry it.
- The recipe leans on private methods of Qwen3-TTS. Its revision is pinned by
  the lab's checkout (`assets-src/audio/voice-refs/README.md`), and 0721 keeps
  the calls in one reader implementation behind its `Synth` trait.
- An actor reads the same direction from `voice script`, and a future tool that
  takes directions gets them from the same field.
- Whether a direction suits a character, line by line, is left to the Act 1
  playtest (1064). Two lines per character were heard, not a whole scene.

## Alternatives considered

- **Direction words in the spoken text, with the audio cut afterward** ("She
  said excitedly: ..."). It reads the direction aloud by design and depends on a
  clean cut. Not tried, as the instruct turn worked.
- **A clip of each character per mood, as extra reference takes.** The mood
  reference (a directed take converted to the character's clip, as the
  character's reference) moved the delivery little in measurement, and it would
  mean a set of files per character.
- **A fixed list of directions mapped to settings.** It can't take free text, and
  the ticket asks for free text. Chatterbox's table showed settings barely move it.
- **Making a changed direction silence the clip.** That is what a stale clip
  does, but the words on screen are still right.
