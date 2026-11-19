---
id: "0922"
title: "The title's campfire: a fire picture, its crackle and embers rising from it"
type: feature
milestone: M8 Release
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: decision
completed: 2026-11-18
---

# 0922 — The title's campfire: a fire picture, its crackle and embers

## Context

Ticket 0908 (polish of the title's sparks and the threat arrows,
2026-11-17) showed Nick four tunings of the title's sparks (count, speed,
glyph, colour). He wanted more than tuning: "we can probably enhance the
title screen some more in general, with additional sfx (fire burn)
ambience and a sprite of a campfire or smth which fades out when the
cinematic plays too". Asked whether that is its own ticket, with the
sparks redone here as embers coming from the fire: "Own ticket".

So the sparks' look is decided here, not in 0908.

Today's title: `docs/design/look-and-feel.md` (*Title screen*: the logo
over a backdrop, 46 sparks drifting up, `crates/ui/src/screens/title/glyph.rs`,
`SPARKS`, `spark`, `SPARK_KINDS`); `docs/design/title-screen.md` (the
title music, the intro cinematic the title goes idle into, ADR-0060);
ADR-0059 (title art). Pictures are sprite items from asset files
(ADR-0038); sounds and music are credited in `assets/audio/audio.ron`.

## Nick input

**Decision** (`ask-nick`, before building), on moving renders of the
real title (`cargo shuyi frame-png --gif`, the picture look with the
bought backdrops, and the Playdate's title): 3–4 variants of each of

- the campfire picture: where it sits (the foreground under the menu,
  beside the logo, …), its size, its flicker;
- the embers: how many, how they rise from the fire and fade, their
  colours (today's sparks among the options);
- the crackle: 2–3 candidate loops and how loud under the title song;
- the fade into the cinematic: how fast the fire and its sound go.

Name real games for each (title screens with a fire or ambience).

## Scope

**In:** the campfire picture as a sprite item (bought, or free under
ADR-0013, credited); the embers in place of today's sparks; a fire
ambience loop under the title music; the fade of all three when the
title goes idle into the intro cinematic; `look-and-feel.md` and
`title-screen.md` updated with Nick's words and picks; the Playdate's
title follows (`crates/ui/src/screens/title/playdate.rs`) as Nick picks.

**Out (do not do):** the threat arrows (0908); the cinematic's own shots;
a new title song; the move and attack ranges (0923).

## Implementation steps

1. Find candidates: the bought library (`assets-private/library/`,
   `cargo shuyi private-assets --library`) first, then free sites under
   ADR-0013's licences, for a pixel-art campfire (animated frames if
   possible) and a fire-crackle loop. Never draw character art; a fire
   sheet is fine to cut into frames (`shuyi-import-art`).
2. Render the variants (throwaway env-var switches in the title's skin,
   never committed) and ask Nick.
3. Build the pick: the picture through the title's view and skin
   (ADR-0054: the view says what, the skin paints it), the embers from
   the time alone (`shuyi-animate`), the sound through the audio module
   with its credit; the fade from the title's existing idle timer.
4. Record the picks in `look-and-feel.md` (*Title screen*) and
   `title-screen.md`; drop *Open sub-questions*' sparks line.

Expected size: about 250 production lines (one sprite and its frames on
the title view, the ember function replacing `spark`, one sound loop and a
fade), judged by 0811 (the title art).

## Acceptance criteria

- [x] Nick picked the fire, the embers, the crackle and the fade; recorded.
- [x] The title shows the campfire, its embers and plays the crackle; all
      three fade as the cinematic starts.
- [x] The new picture and sound are credited (`credits.ron` / `audio.ron`,
      `THIRD_PARTY_ASSETS.md` or the bought art's `private` list).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the embers' places at fixed times (as `sparks_are_where_they_were`);
  the fade's opacity over the idle time.
- Snapshot: the title at a fixed frame, glyph look and Playdate.
- Integration: the title going idle starts the fade and the cinematic.

## Completion notes

**Nick's picks** (two rounds of moving renders of the real title with the
bought backdrops, and recordings; his words are in `look-and-feel.md`,
*Title screen*, *The campfire, its embers and its crackle*):

- **Fire:** Idylwild's *Chromatic Campfires* (OpenGameArt, CC0), the
  orange fire with smoke, four frames: `assets/title/campfire.png`, named
  in `title.ron` (`campfire: Some((file, frames))`, a new optional field,
  checked by the loader: a whole number of frames wide).
- **Place:** the bottom-left corner, 4×.
- **Embers:** "Rise": 26 embers from the flames, fading 8 to 15 rows up.
  `ember` replaces `spark` (`title/glyph.rs`); the view's `sparks` field
  is now `embers` (and the cinematic's `Frame::Logo`).
- **Crackle:** Nick downloaded two Freesound recordings himself and asked
  for "less popping and a higher volume base sound", with a compressor;
  of four strengths he picked the light one (4:1) of HECKFRICKER's
  *Campfire 02* (CC0). `assets-src/audio/import.py` makes
  `assets/audio/sfx/campfire_02.ogg` from the HQ preview: the same chain
  he heard (compressor, limiter, ffmpeg's loudness normaliser), 40 s from
  1:00, its next 2 s crossfaded into its start so that it loops. Cue
  `title_campfire`, looped, volume 32: about 15 dB under the title song
  ("in between -12 and -18").
- **Fade:** "With the picture": the fire and embers go to black with the
  rest in 0.2 s and the crackle fades out over the same 0.2 s
  (`FIRE_FADE_MS`; the engine's `stop_sound` fade, no engine change).
- **Playdate:** "decide later, followup ticket": **0924**. Its bundle
  leaves `title/campfire.png` out, and a title with no campfire plays no
  crackle.

**The agent's starting rules** (for the Act 1 playtest, 1064):

- The fire's frame time (140 ms) and a gentle flicker (0.8 to 1.0
  brightness).
- With idle motion off (Options), the fire stands still on its first
  frame and no embers rise.
- The crackle starts with the title song (on the first press, as the
  song) and comes back with the picture: on the cinematic's logo shot,
  also in the silence after a skip, and when a press brings the menu back.
- Opening the game (New Game, Load Game, Continue, Quick Battle) fades the
  crackle out like the picture's fade; Options, Profile and Credits keep it
  playing, as they keep the music.

**Deviations:** the ticket's "3–4 crackle loops" became Nick's own two
recordings at four compressor strengths (his request). Freesound needs a
login, so Nick downloaded the originals; the importer uses the public HQ
preview of the one he picked.

**Also changed:** the credits (`credits.ron` gains the first free Art
credit, so a gate build's credits screen now has an Art section, and its
test says so), `THIRD_PARTY_ASSETS.md`, `assets/NOTICES-game.txt`, the
pack lists in `shuyi.ron`, the private audio manifest
(`Zafnok/visions-of-shuyi-assets`, merged first), the kept Quick Battle
session (its frames now carry the crackle) and nine snapshots (one sprite
line each: the fire). Tests that listed every sound now take only the
sounds played once (`one_shots` in `tests/it/main.rs`), or music only.

Engine friction: estimate 250, measured 218 (0.9×), under 2×.
