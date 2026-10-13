# Cinematic files (`.ron`)

A cinematic is a list of **shots** and when each starts (ticket 0817,
[ADR-0060](../../docs/adr/0060-cinematics-a-file-of-timed-shots-drawn-from-the-time-alone.md);
`docs/design/title-screen.md`, *Intro cinematic*). Each `*.ron` file here is
one cinematic, loaded by `trpg_content::cinematic` and validated by the
all-assets test. The file stem is its id. `trpg_ui::cinematic` plays it.

There is none here yet: the title's comes with tickets 0819 and 0820. The
test one is `assets-test/cinematics/test.ron` (the debug menu's "Play test
cinematic" plays it).

```ron
(
    id: "title",
    length: Music("title"),
    shots: [
        (at: 0.0, shot: MapPan(battle: "ch01", from: (4, 9), to: (12, 3.5), zoom: 2)),
        (at: 6.0, shot: Logo),
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

More kinds come with tickets 0818 (characters, conversation snippets),
0827 (units that march and fight) and 1010 (the overworld).
