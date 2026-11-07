# The `voice/` folder

Voice clips for dialogue lines ([ADR-0046](adr/0046-voice-clips-by-line-id.md)).
The folder sits **beside the game** (next to the executable, or next to
`index.html` on the web) and is never embedded. Without it the game runs
the same, silent, and says nothing about it.

In this repository `voice/` is git-ignored: real clips live in the private
assets repository, in its top-level `voice/` folder, and `cargo xtask
private-assets` copies them here. The only clips in this repository are the
test tones in `crates/app/tests/voice/`, laid out the same way.

## Layout

```
voice/
  en/                         one folder per language; only `en` is read today
    voice.ron                 the manifest: every clip
    cast.ron                  which voice says each speaker's lines
    ch01_intro/               one folder per scene id
      ch01_intro_1a2b3c4d.ogg     <line id>.ogg
      ch01_intro_5e6f7a8b.m.ogg   said of a male lead
      ch01_intro_5e6f7a8b.f.ogg   said of a female lead
```

- A clip is named after its **line id** (`cargo xtask lines [scene]` lists
  them). Rewording a line changes its id, so the old clip no longer
  belongs to anything.
- Files are OGG Vorbis, 44.1 kHz, mono.
- A line whose words change with the lead's gender (`{they}`, `{their}`…)
  has two clips, `.m.ogg` and `.f.ogg`. Every other line has one.
- A line that says the lead's name (which the player types) writes what the
  voice says instead, `{lead|the exile}` (ADR-0107 rule 4; how to write it:
  `assets/dialogue/README.md`, "What the voice says for the name"). The text
  box shows the name and the clip says the stand-in, so the clip's `spoken`
  has "the exile" where the box has the name. A bare `{lead}` can't have a
  clip; the content checks allow one only in a reply, which is silent.

## `voice.ron`

```ron
(
    clips: [
        (
            line: "ch01_intro_1a2b3c4d",
            spoken: "The king rides for Harrow Keep at dawn.",
            voice: "hollis_a",
            made_by: Generated(tool: "…", model: "…", date: "2026-10-03"),
        ),
        (
            line: "ch01_intro_5e6f7a8b",
            variant: M,
            spoken: "He left his sword with the king.",
            voice: "hollis_a",
            made_by: Recorded(actor: "A. Person"),
        ),
        (
            line: "ch01_intro_5e6f7a8b",
            variant: F,
            spoken: "She left her sword with the king.",
            voice: "hollis_a",
            made_by: Recorded(actor: "A. Person"),
        ),
    ],
)
```

| Field | Meaning |
| ----- | ------- |
| `line` | The dialogue line id. |
| `variant` | `M` or `F` for a gendered pair; left out otherwise. |
| `spoken` | The words the clip says: the line's text with every `{n:…}` name and pronoun filled in, as it was when the clip was made. |
| `direction` | How the line was to be said when the clip was made (`assets/dialogue/README.md`, "Directions"); left out if it had none. |
| `voice` | The cast voice that says it. |
| `made_by` | `Generated(tool, model, date)` or `Recorded(actor)`. The credits and store pages say which voices are generated while any are. |

The game refuses the whole manifest (a logged warning, no voices) when:

- a `line` is not the id of a dialogue line,
- a line has the same variant twice,
- a line has `M` without `F` or the other way round, or both a plain clip
  and a gendered pair,
- `spoken` or `voice` is empty, or the file isn't valid RON.

### Stale clips

A clip is **stale** when its `spoken` is not what its line says today, for
example after a rename in `assets/data/names.ron`. A stale clip is not an
error: the game leaves it out and that line is silent until the clip is
made again. The same goes for a plain clip on a line that needs a gendered
pair, and for any clip on a line with a bare `{lead}`. Changing a line's
stand-in for the name (`{lead|the exile}` to `{lead|the heir}`) keeps its id
and its translations, and makes its clip stale. The game checks clips against
the lines as written (`trpg_content::dialogue::written`), so the stand-ins
are read.

A clip whose line's `direction` changed still says the right words, so it
keeps playing. It is not stale, but it **needs remaking**
(`trpg_content::voice::needs_remake`), and the voice tool makes it again. How
a direction reaches the tools, so that it changes the delivery and isn't read
aloud, is [ADR-0109](adr/0109-voice-directions-reach-the-voice-design-model-apart-from-the-words.md).

## `cast.ron`

```ron
(
    voices: {
        ">": (clip: "narrator", reader: Qwen3),
        "lead_m": (clip: "lead_m", reader: Qwen3),
        "lead_f": (clip: "lead_f", reader: Qwen3),
        "retainer": (clip: "retainer", reader: Chatterbox),
    },
)
```

Each speaker's voice ([ADR-0107](adr/0107-free-local-voice-tools-with-designed-voices-and-a-reader-per-character.md)
rule 2), from the cast table of `docs/design/voices-languages-and-script.md`:
a character id, `>` for narration, and `lead_m` and `lead_f` for the lead's
two voices. `clip` is the voice's reference clip,
`assets-src/audio/voice-refs/<clip>.flac`, and the manifest's `voice` names
it. `reader` reads its undirected lines: `Qwen3` or `Chatterbox`. Every
directed line takes ADR-0109's recipe, whoever the reader. The game doesn't
read this file; `cargo xtask voice` does.

## `say.ron`

```ron
{
    "aske": "Ask",
    "Harrow": "Harro",
}
```

How to spell a name or a word for the readers, which often misread
invented names: a name id of `assets/data/names.ron` (for that name as it
is today) or a word, and the spelling to send. Only whole words change.
The manifest's `spoken` keeps the real words; the respelling is only what
the reader is sent. Changing an entry makes the clips of the lines that use
it again. Optional.

## `sent.ron`

What the tool sent to make each clip, by clip file: the words as spelled
for the reader, and a fingerprint of the voice's reference clip. The tool
writes it; nothing else reads it. A clip is made again when the tool would
send other words (`say.ron` changed) or the reference clip was replaced.

## Making the clips: `cargo xtask voice`

On a developer's machine with the voice lab (ADR-0107 rule 3:
`D:\voice-lab`, its Python environments in `envs\qwen` and
`envs\chatter`, its models in `hf\`; another place by `--lab` or the
`VOICE_LAB` variable) and `ffmpeg`. Never in the game or in CI. It reads the
scripts the game is built with, so build after editing a script (any
`cargo xtask` command does).

```bash
cargo xtask voice status
```

Per scene: the clips to make and why (no clip, the words changed, the
direction changed, the voice changed, `say.ron` spells it differently), the
recorded clips a person must record again, the clips to remove (their line
is gone, or it now needs a gendered pair instead of one clip or the other
way round), and the lines whose speaker has no voice in `cast.ron`.

```bash
cargo xtask voice generate --dry-run
```

```bash
cargo xtask voice generate
```

Removes the clips `status` would remove, then makes the rest. Undirected
lines go to their voice's reader; directed lines to ADR-0109's recipe. Every
take is transcribed (faster-whisper) and kept when it says the line's words
(or the respelled ones), 85 % alike letter by letter, or, for a line of at
most four words, word for word by sound, since a transcriber spells an
invented name its own way ("Dases" for "Dace's.", ADR-0113's follow-ups);
otherwise the next seed is tried, up to four. A
line with no good take is reported and not saved. Each clip is trimmed to at
most 0.1 s of silence at either end and brought to −18 LUFS (true peak at
most −1 dBTP); a directed clip keeps its distance from its voice's level,
from 12 dB below to 6 dB above, so a whisper stays quiet. These numbers are
in `crates/xtask/src/voice/synth.rs`, marked *tunable*. The manifest and
`sent.ron` are written after every batch, so a stopped run keeps what it
made. A `Recorded` clip is never made or replaced. `--dry-run` counts the
clips and characters it would send, directed ones apart, and makes nothing.

A clip that transcribes right but sounds wrong is made again by removing its
entry from `voice.ron` (and its file) and running `generate`: the next take
is a new one.

```bash
cargo xtask voice script --character retainer
```

A recording script for an actor (ADR-0046 rule 10): per line, the file to
deliver, the scene, the line before it, the direction and the words. A
character id, `>` for the narrator, or `lead_m` / `lead_f`.

Every command takes `--lang <code>` (default `en`), `--scene <id>` (one
scene, a test scene too; by default every scene of `assets/dialogue/`) and
`--dir <folder>` (default `assets-private/voice`, the private repository's
voice folder). The readers are `tools/voice/lab.py`, one command per step,
each run in its lab environment.

### Committing the clips

1. `cargo shuyi private-assets` (the checkout must have `voice/`).
2. `cargo xtask voice generate`, then `cargo xtask voice status` to see
   what is left.
3. In `assets-private/`: commit `voice/` and push.
4. Here: `cargo shuyi private-assets --pin`, and commit
   `assets-private.rev` in the ticket's PR.

## How the game plays them

- At start-up `app` reads `voice/en/voice.ron` (from beside the executable,
  else the working directory; fetched on the web) and hands the text to
  `Ctx::set_voice_manifest`, which validates it and keeps the clips that
  aren't stale.
- A screen says a line with `ctx.play_voice(&line_id)`. Nothing happens if
  voices are off (`ctx.voices_on`), or the line has no playable clip. One
  voice plays at a time: a new one stops the old. `ctx.stop_voice()` stops
  it.
- When a scene starts, a screen calls `ctx.preload_voices(&line_ids)` with
  its lines in script order. `app` keeps the next 3 clips loaded and frees
  those passed.
- A clip that isn't loaded within 0.3 s of being asked for is skipped
  rather than played late.
- Clips play at `ctx.voice_volume` (0–10, default 8).
- A missing or unreadable clip file is a logged warning and silence.

The dialogue screen (ticket 0720) says each line's clip as its text box
comes up, speech and narration but never a reply the player picks. The
first press fills in the box and leaves the voice playing; moving on to
another line, a choice or out of the scene stops it. After each line it
preloads the next 3, and at a choice the first line of each reply's
reaction. The dialogue log's cursor says the line it moves onto unless
*Voice on hover in the log* is off. The debug menu's **Voice test** says
the test scene's voiced lines, one per press, without the screen.

## Trying it with the test clips

```bash
cp -r crates/app/tests/voice voice
```

```bash
cargo run -p trpg-app
```

Open the debug menu and choose "Voice test". For the web build, run
`cargo shuyi web` after the copy: it puts `voice/` in `dist/web/voice/`.

The test clips are tones made by `cargo xtask voice-test-clips` (it needs
`ffmpeg`), one per entry of `crates/app/tests/voice/en/voice.ron`.

## Shipping

- `cargo shuyi web` copies `voice/` (its `.ron` and `.ogg` files) into
  `dist/web/voice/` when the folder exists.
- `cargo shuyi export` copies `voice/` into each package when the folder
  exists (`shuyi.ron`'s `export` part).
- CI builds get the folder from the private repository
  (`.github/actions/private-assets`). Gates never read it.
