# Cinematic files (`.ron`)

A cinematic is a list of **shots** and when each starts (tickets 0817, 0818,
[ADR-0060](../../docs/adr/0060-cinematics-a-file-of-timed-shots-drawn-from-the-time-alone.md);
`docs/design/title-screen.md`, *Intro cinematic*). Each `*.ron` file here is
one cinematic, loaded by `trpg_content::cinematic` and validated by the
all-assets test. The file stem is its id. `trpg_ui::cinematic` plays it.

`title.ron` is the title screen's intro cinematic, which the title plays
in time with its song (ticket 0819); until ticket 0820 writes its real
shots it is a stand-in (one pan over the Chapter 1 map, then the logo). The
test one is `assets-test/cinematics/test.ron` (the debug menu's "Play test
cinematic" plays it).

```ron
(
    id: "title",
    length: Music("title"),
    shots: [
        (at: 0.0, shot: MapPan(battle: "ch01", from: (4, 9), to: (12, 3.5), zoom: 2)),
        (at: 15.0, shot: Talk(scene: "ch01_intro", first: 0, count: 3)),
        (at: 31.0, shot: Characters(who: ["lead", "retainer"], expression: "neutral")),
        (at: 117.0, shot: Logo),
    ],
)
```

| Field | Meaning |
| ----- | ------- |
| `id` | Must match the file name. |
| `length` | How long the cinematic is. `Music("cue")`: as long as that music cue (`assets/audio/audio.ron`, its `length_ms`), for a cinematic that plays with a track. `Seconds(10.0)`: a length of its own, above 0 and at most an hour. |
| `shots` | The shots, in order of time. At least one. |
| `at` | When the shot starts, in seconds from the start. The first is `0.0`; each is after the one before it and before the end. A shot lasts until the next one starts; the last one until the end. Kept to the millisecond. |
| `shot` | What is shown: one of the kinds below. |

Shots change with a **hard cut** (Nick, ticket 0036). There are no fades
in the file. After its end a cinematic starts again from `0.0`.

## Shot kinds

### `MapPan(battle, from, to, zoom)`

A battle's whole map (`assets/battles/`), with its units where they stand
when the battle starts, behind the whole screen. The middle of the screen
moves from `from` to `to` over the shot, slowly at both ends.

| Field | Meaning |
| ----- | ------- |
| `battle` | The battle's id. |
| `from`, `to` | The point of the map in the middle of the screen at the shot's start and end, in tiles, as `(column, row)`. A whole number is the **middle** of that tile: `(4, 9)` is the middle of the tile at column 4, row 9, and `(4.5, 9)` the edge between it and the tile to its right. Both must be on the map. Kept to a thousandth of a tile. |
| `zoom` | How many screen pixels a pixel of the map is: `1`, `2`, `3` or `4`. One size for the whole shot. |

The screen never shows past the map's edge: near an edge the picture stops
there, so a point closer to the edge than half of what the screen shows
moves nothing. Along a direction in which the whole map fits on screen, the
map is centred and doesn't move. The map is drawn by the map skin in use at
its art's own size, whatever zoom the player has the battle map at.

### `Logo`

The title's logo over the title's backdrop, with its sparks
(`assets/title/README.md`), drawn by the title screen's own code.

### `Talk(scene, first, count)`

Text boxes of a dialogue scene (`assets/dialogue/`) playing by themselves
in the dialogue screen's look: portraits, name plates, the caption, the
text typing out. Nothing waits for a key, so there is no `▼` and no key
name. The lines are whatever the script file says; the cinematic has no
text of its own.

| Field | Meaning |
| ----- | ------- |
| `scene` | The scene's id. |
| `first` | The first text box shown, counted from `0`. A text box is a speech or narration line; the scene is read with everyone there (every `@if` block plays its first part). |
| `count` | How many text boxes are shown, at least `1`. |

- The boxes must come **before the scene's first `@choice`**: a cinematic
  never shows a reply choice.
- Each box must fit **one page** of the text box (3 lines of 92), with the
  lead's name and every name at its longest: a cinematic turns no pages.
- Each box gets an equal share of the shot. Its text types out at the
  player's text speed (Options), faster if it would otherwise be whole for
  less than the last third of its share, and then holds. Rule of thumb:
  give a box about 4 seconds, more for three full lines.
- `{lead}` and the lead's pronouns are those of the lead the cinematic
  shows (below).

### `Characters(who, expression, from)`

Faces in the dialogue screen's portrait frames, each with its name under
it, crossing the screen one after another at one speed. The shot starts
with the first face just off the edge and ends when the last has just
left by the other; with a face in the middle of the screen, half of the
next one is in.

| Field | Meaning |
| ----- | ------- |
| `who` | The characters (`assets/data/characters.ron`), in the order they cross. At least one. |
| `expression` | The expression every face wears. Each character's portrait must have it; a character without a portrait gets an empty frame, and the expression must then be a standard one. |
| `from` | `Right` (the default; may be left out) or `Left`: the side they come in from. A face looks the way it is going. |

The name is the character's short name (`<id>.first` in
`assets/data/names.ron`, ticket 0712), or the whole name if it has none.

Six faces need about 19 seconds (the title's storyboard); fewer seconds
make them cross faster.

### The lead in `Talk` and `Characters`

Both show the lead of **the most recent save**: the name and the look the
player chose (Nick, ticket 0036). Without a save, the names table's name
for the lead and the male look, `lead_m`. The game notes which save was
written last under the storage key `last_save`.

More kinds come with tickets 0827 (units that march and fight) and 1010
(the overworld).
