# Audio manifest

`audio.ron` lists every sound and music cue the game can play, the music
pools, and a credit for every third-party work. Screens ask for a cue by id
(`ctx.audio.play_sound("menu_move")`); the game resolves the id here.
Why it works this way: [ADR-0026](../../docs/adr/0026-audio-cues-and-music-files.md).
Which cue is which track: [`docs/design/audio.md`](../../docs/design/audio.md).

## Where the files go

| What | Folder | Shipped how |
| ---- | ------ | ----------- |
| Sound effects | `assets/audio/` (e.g. `assets/audio/sfx/menu_move.wav`) | Embedded in the game binary |
| Music | `music/` at the repo root | A folder next to the exe; served next to the WASM on the web |

Formats: **OGG Vorbis** for anything recorded (all music, third-party
sounds). **WAV** (PCM or float) is allowed for our own short sounds. Nothing
else. Every file is **44.1 kHz, mono or stereo**: the native player plays
at 44.1 kHz and resamples anything else badly, and a file it can't read
crashes it, so the checks below read every file's header.

## Format

```ron
(
    music_fade_ms: 500,           // old track fades out this long, then the new one starts
    // How a volume's number (0-100) becomes the gain it plays at: Linear,
    // that percent of full (ticket 0882), or Decibels, 0.6 dB a step (the
    // engine's default; engine ADR-0157). Optional.
    volume_curve: Linear,
    groups: [
        // The volume a new player's group starts at on Options (0-100;
        // engine GroupDef). Optional: a group left out starts at the
        // engine's default.
        (id: "music", volume: 100),
    ],
    ducks: [
        // While a voice clip plays, lower a sound group to `to` on the
        // engine's decibel curve, whatever volume_curve says (each step
        // 0.6 dB: 90 is 6 dB under) over
        // attack_ms, and bring it back over release_ms after (engine
        // ADR-0099; ticket 0865). Optional.
        (when: "voice", lower: "music", to: 90, attack_ms: 150, release_ms: 600),
    ],
    sounds: [
        // files: relative to assets/audio/. Several files = variants, one
        // picked at random each time. volume: percent, 0-100.
        // Optional cue rules (engine ADR-0139, ticket 1349): pick
        // (Random, the default, NoRepeat, Shuffle, InOrder), volume_min,
        // and min_gap_ms: a play this soon after the cue's last one that
        // sounded is dropped (Harness::dropped_sounds lists it).
        (id: "menu_move", files: ["sfx/menu_move.wav"], volume: 100, min_gap_ms: 30,
         credit: Own),
        (id: "step_foot", files: ["sfx/step_1.ogg", "sfx/step_2.ogg"], volume: 80,
         pick: NoRepeat, credit: Credit("footsteps")),
    ],
    music: [
        // file: relative to music/. Loops unless `looped: false`.
        // length_ms: how long the file plays, in milliseconds.
        (id: "title", file: "new_sunrise_v1.ogg", volume: 100, length_ms: 133743,
         credit: Credit("new_sunrise")),
    ],
    pools: [
        // Music cues one is picked from (e.g. skirmish battles).
        (id: "skirmish", music: ["battle_1", "battle_2"]),
    ],
    credits: [
        // One per third-party work (credit even CC0: audio.md rule 3).
        (id: "new_sunrise", title: "New Sunrise", author: "nene",
         source: "https://opengameart.org/content/new-sunrise",
         license: "CC0-1.0", tags: ["music", "title", "calm"], note: "V1 file"),
    ],
)
```

- `credit` is `Own` (we made it), `Credit("<credit id>")`, or
  `Credits(["<id>", "<id>"])` when a sound's variants come from several
  works (e.g. `step_foot`).
- `license` is `"CC0-1.0"`, `"CC-BY-3.0"`, `"CC-BY-4.0"` or `"Own"`
  (ADR-0013, ADR-0027). A credit that isn't `Own` needs a `title`, an
  `author` and a `source` link. The license texts are in `licenses/`.
- `groups` sets where the Music (`music`), Sound (`sfx`) and Voice
  (`voice`) volumes start for a new player, and what `Restore defaults`
  puts them back to. The numbers are Nick's, from
  `docs/design/options.md` (*Volumes*): 100 for all three (0879). The
  game's own `ambience` group (1332) starts at 90, 10 under the music.
- `ducks` lowers a group while a voice clip plays: the music, and the
  `ambience` group by the same numbers (`audio.md`, rule 11 and
  *Ambience*).
- Every cue id (sounds, music and pools together) is unique.
- `length_ms` is the track's real length: screens that keep time with the
  music wrap their clock there
  ([ADR-0037](../../docs/adr/0037-music-clock.md)). Don't work it out by
  hand: write any number but 0, run `cargo test -p trpg-content`, and copy
  the value from the failure (`... it plays for 133743 ms ... write
  length_ms: 133743`). The same goes for a track that is re-cut.

## Ambience

The places' sound beds (ticket 1332, ADR-0132; `audio.md`, *Ambience*):
looped sound cues in `ambience/`, each with `group: "ambience"` and
volume 100 (the player's Ambience volume sets how loud they are), levelled
like the music by `assets-src/audio/import.py`. Which place plays which is
`assets/data/ambience.ron`:

```ron
(
    // Each ambience's beds, by light (data/lighting.ron). Every ambience
    // has a "noon" bed, which a light without its own plays.
    ambiences: {
        "meadow": {"noon": "bed_summer_meadow", "night": "bed_crickets"},
    },
    // The ambience a scene plays over `@place <map>`, by map id.
    places: {"harrowby_yard": "meadow"},
    // Camp's, if any.
    camp: Some("meadow"),
)
```

A battle file names its own (`ambience: "meadow"`, `assets/battles/`),
and a world map names the map's and each town's (`ambience: Some(...)`,
`assets/worldmaps/`); `@place battle` plays its battle's. A scene may
name its own instead, from a line on until its next `@place`:
`@ambience <id>`, or `@ambience none` for no bed
(`assets/dialogue/README.md`, *Places*; ticket 1383).
`cargo test -p trpg-content` refuses a bed that isn't a looped sound cue
of the `ambience` group, a light that isn't one, an ambience without a
`noon` bed, a place's map that isn't one, and a name of an ambience that
isn't one (a scene's `@ambience` too, with its file and line).

## Third-party files

Made by `assets-src/audio/import.py` (converted, trimmed and
loudness-matched; see [`assets-src/audio/README.md`](../../assets-src/audio/README.md)).
Don't edit them by hand: change the script and run it again. Add every new
work to `THIRD_PARTY_ASSETS.md` too.

## Checks

`cargo test -p trpg-content` refuses: a duplicate id, an unknown or missing
file, a wrong file format (by extension and by header; wrong sample rate or
channel count), a music `length_ms` of 0 or more than 20 ms from its
file's length, a volume over 100, a disallowed license, a credit
without title/author/link, an unknown credit (or an empty `Credits` list),
and a pool that is empty or names anything but a music cue. The game
checks the embedded sounds at start-up; the music folder is checked by the
tests only. Another test checks that every cue in `docs/design/audio.md`'s
tables is in the manifest.
