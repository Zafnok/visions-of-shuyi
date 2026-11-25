# ADR-0132: A place's ambience is data, played by one bed player every screen shares

- **Status:** Accepted
- **Date:** 2026-11-25
- **Related tickets:** 1332 (decided by 1322); each chapter's ambience is
  1335–1342

## Context

1322 decided that a place has a sound bed under its music (`audio.md`,
*Ambience*): a loop levelled like the music, with its own volume on
Options starting 10 under the music's; heard on the battle map, in a
scene over its place, on the world map, in an open town and in camp; cut
while a fight's stage shows and back after, from where its loop is;
dipped under a voiced line as the music is; one bed per light where a
place has several, the noon bed in a light without its own.

What existed:

- **The engine's sound groups** (engine ADR-0091): a game declares its
  own group with a starting volume in its audio manifest's `groups`, puts
  a cue in it with `group:`, and lowers it under a voice with a `ducks`
  rule (engine ADR-0099). The Options screen builds a volume row for any
  group (`RowValue::volume`). The player's volumes are saved as a map by
  group, so a new group needs no settings version.
- **Looped sounds with a handle** (`AudioQueue::start_sound`, `stop_sound`
  with a fade, `set_sound_volume`), owned by the screen that started them
  and stopped at once when it leaves the stack (the title's crackle,
  0922).
- **Look data beside the rules**: a battle file's `light` is read on the
  side (`battle::load_lights`), never put in the `BattleDef`; a world
  map's `music` and each town's sit beside its rules in `WorldMapDef`.
- The engine's parked ticket 2370 (weather: particles, an overlay and a
  loop sound) is blocked, so it can't carry this.

## Decision

### 1. The engine's group and dip, nothing new in the engine

The beds play in a game group, `ambience` (`trpg_content::ambience::
AMBIENCE`), declared in `audio.ron` at volume 90, with a `ducks` rule that
copies the music's numbers. Its Options row is `RowValue::volume` on that
group, under the Sound volume. No engine change.

### 2. Ambiences are a data file; places name them where they are written

`assets/data/ambience.ron` (`trpg_content::ambience`) holds the ambiences,
each a map from light to a sound cue, and names the ambience of each map a
scene's `@place` shows (`places`, by map id) and of camp (`camp`). A
battle file names its own (`ambience: "meadow"`, read on the side like its
`light`, into `AmbienceTable::battles`), and a world map names the map's
and each town's (`ambience:` beside `music:`). A bed is a looped cue of
the `ambience` group; every ambience has a `noon` bed, which a light
without its own plays (`AmbienceTable::bed`). Loading checks every cue,
light, map and name. The private content's `audio.ron` declares the same
group, dip and cues; the bed files are public (CC0).

### 3. One bed player, on the game's context

`Shuyi::ambience` (`trpg_ui::ambience::AmbiencePlayer`) holds the one bed
playing. A screen with a place asks for its bed every frame
(`ambience::play`): the same bed goes on; another fades the old out over
0.5 s (the engine's stop fade) and starts at silence, fading in over 0.5 s
by `set_sound_volume`; none fades it out; `hushed` fades it to silence and
back while the loop goes on. A screen leaving the stack calls
`ambience::leave`, which fades out only the bed it started (by screen
name), so a scene over its own battle keeps the battle's bed and doesn't
stop it as it closes. A bed whose screen left without fading it was
stopped by the stack; the player notices (`sound_is_live`) and starts
again. A screen with no place (a menu, Options) asks nothing, and the bed
goes on under it.

**Rejected:** one bed per screen, like the title's crackle. A scene over
the battle at the battle's place would play the bed twice, or stop and
restart it, and a fight couldn't bring it back from where its loop was.

### 4. The Playdate plays none for now

`use_playdate_skins` turns the player off (`AmbiencePlayer::off`), its
build bakes no `ambience`-group cue, its bundle leaves out
`audio/ambience/`, and its Options has no Ambience row (`PC_ONLY`), until
the bench says beds run well within its `Reduced` tier (ADR-0131).

## Consequences

- A chapter's ticket sets its beds with data alone: an entry in
  `ambience.ron`, a battle file's `ambience`, a world map's.
- A bed's light is its screen's: the battle's look, a scene's map's, noon
  on the world map and in camp. A light change during a battle would ask
  for the new bed on the next frame, crossfading.
- The rules, the bots and play records never hear it (ADR-0004).
