# Cinematic files (`.ron`)

A cinematic is a list of **shots** and when each starts (tickets 0817, 0818,
0827 and 0832,
[ADR-0060](../../docs/adr/0060-cinematics-a-file-of-timed-shots-drawn-from-the-time-alone.md),
[ADR-0061](../../docs/adr/0061-cinematic-action-authored-walks-and-strikes-in-the-map-scene.md),
[ADR-0062](../../docs/adr/0062-cinematic-combat-shot-a-fight-the-rules-work-out.md);
`docs/design/title-screen.md`, *Intro cinematic*). Each `*.ron` file here is
one cinematic, loaded by `trpg_content::cinematic` and validated by the
all-assets test. The file stem is its id. `trpg_ui::cinematic` plays it.

`title.ron` is the title screen's intro cinematic, which the title plays
in time with its song (ticket 0819): Nick's storyboard (tickets 0820 and
0833), cut where the song changes, its battle played as turns with the
combat scene for each fight. The
test one is `assets-test/cinematics/test.ron` (the debug menu's "Play test
cinematic" plays it); `assets-test/cinematics/combat.ron` is a test of the
combat shot alone.

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
| `actions` | Optional. Action lists by name, for map shots to share: `actions: { "duel": [ … ] }` ([below](#action-units-that-march-and-fight)). Every list must be used by a shot. |
| `shots` | The shots, in order of time. At least one. |
| `at` | When the shot starts, in seconds from the start. The first is `0.0`; each is after the one before it and before the end. A shot lasts until the next one starts; the last one until the end. Kept to the millisecond. |
| `shot` | What is shown: one of the kinds below. |

Shots change with a **hard cut** (Nick, ticket 0036). There are no fades
in the file. After its end a cinematic starts again from `0.0`.

## Shot kinds

### `MapPan(battle, from, to, zoom, action_from, action)`

A battle's whole map (`assets/battles/`), with its units where they stand
when the battle starts (or doing what the shot's `action` says), behind
the whole screen. The middle of the screen
moves from `from` to `to` over the shot, slowly at both ends.

| Field | Meaning |
| ----- | ------- |
| `battle` | The battle's id. |
| `from`, `to` | The point of the map in the middle of the screen at the shot's start and end, in tiles, as `(column, row)`. A whole number is the **middle** of that tile: `(4, 9)` is the middle of the tile at column 4, row 9, and `(4.5, 9)` the edge between it and the tile to its right. Both must be on the map. Kept to a thousandth of a tile. |
| `zoom` | How many screen pixels a pixel of the map is: `1`, `2`, `3` or `4`. One size for the whole shot. |
| `action` | Optional. What the units do during the shot: a list written in place, or the name of one in the file's `actions`. |
| `action_from` | Optional, only with `action`: where on the action's clock the shot starts, in seconds. `0.0` if left out. |

The screen never shows past the map's edge: near an edge the picture stops
there, so a point closer to the edge than half of what the screen shows
moves nothing. Along a direction in which the whole map fits on screen, the
map is centred and doesn't move. The map is drawn by the map skin in use at
its art's own size, whatever zoom the player has the battle map at.

#### Action: units that march and fight

```ron
(
    id: "title",
    length: Music("title"),
    actions: {
        "bridge": [
            (at: 1.0, do: Walk(unit: "sergeant", path: [(6, 11), (11, 11), (11, 8)], secs: 6.0)),
            (at: 8.5, do: Strike(unit: "sergeant", target: "red_captain", hp_left: 40)),
            (at: 9.5, do: Strike(unit: "sergeant", target: "red_captain", hp_left: 0)),
        ],
    },
    shots: [
        (at: 0.0, shot: MapPan(battle: "ch01", from: (6, 11), to: (10, 9), zoom: 3, action: "bridge")),
        (at: 7.0, shot: MapPan(battle: "ch01", from: (11, 8), to: (11, 8), zoom: 4,
            action_from: 7.0, action: "bridge")),
        (at: 12.0, shot: Logo),
    ],
)
```

(An example of the form only: the title's cinematic is ticket 0820's.)

An action is a list of things units do, each with the time it starts.
They are **pictures, not a battle**: the combat rules are never run, so a
strike does what the file says whatever the units' stats are, and nothing
here changes when the game is rebalanced. An action makes no sound.

The action has its own clock, in seconds. A shot starts it at its
`action_from` and it then runs with the cinematic. So the two shots above
show one fight: the first from its start, the second (seven seconds on,
from another place, closer) from where the first left off. A shot may
start anywhere on the clock, also after the list's last entry: it then
shows how it all ended. Whatever the clock, each unit is where the
entries before that time put it.

| Field | Meaning |
| ----- | ------- |
| `at` | When the entry starts, in seconds on the action's clock. Entries are in order of time (two may start together). |
| `do` | `Walk` or `Strike`. |

A **unit** is named as the battle file names it: a player slot by its
`character`, an enemy by its `id`, and an enemy that is a character by
that character. Only units on the map when the battle starts
(reinforcements are not there).

`Walk(unit, path, secs)`: the unit walks along `path` at an even speed,
taking `secs` seconds (more than 0), and stays at its end.

- `path` is tiles as `(column, row)`, at least two. The first is where
  the unit is then: where it starts the battle, or the end of its walk
  before. Each next tile is a straight run along the row or the column
  from the one before it (a neighbour or further), so a path only needs
  its corners.
- Every tile on the way is on the map and one the unit's movement type
  can enter.
- A unit's walk starts no sooner than its walk before ends.

`Strike(unit, target, hp_left)`: the unit leans a few pixels toward the
target and back (0.4 s). Half way, the target flashes and its HP bar drops
to `hp_left` percent of its full HP (`0` to `100`, never more than it had
left). At `hp_left: 0` the target then fades out over half a second and is
gone.

- A unit doesn't strike in the middle of its own walk, nor itself.
- After a strike left it with `0`, a unit can't be named again.

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

### `Combat(battle, attacker, target, from, seed)`

The battle's own **combat scene** for one attack: the two fighters'
full-body pictures over the backdrop of the ground the defender stands on,
their plates with name, HP and numbers under them, the strikes, the HP
draining and a fall, exactly as a battle shows an attack with combat
animations on (`crates/ui/src/screens/battle/stage.rs`). Nothing is drawn
from the file: **the rules work the fight out.** The battle is set up as
it starts (the player's side as a battle played on its own has it), the
attacker is put on `from`, and it attacks the target from there through
the game's own rules, so the numbers, hits, misses and HP are the game's.

```ron
(at: 50.0, shot: Combat(battle: "ch01", attacker: "lead", target: "road_brigand", from: (9, 11))),
```

| Field | Meaning |
| ----- | ------- |
| `battle` | The battle's id. |
| `attacker`, `target` | Two units on the battle's map at its start, named as an [action](#action-units-that-march-and-fight) names them (a player slot by its `character`, an enemy by its `id` or its character). |
| `from` | Optional. The tile `(column, row)` the attacker strikes from; where it starts if left out. On the map, a tile the attacker's class can stand on, and not where another unit starts. Only the attacker moves: the target fights from where it starts, on its own ground. |
| `seed` | Optional. A whole number that seeds the fight's luck; the battle's own `seed` if left out. The same seed always gives the same fight. To show a miss, a critical or a fall, try seeds until the fight goes that way. |

- The rules must allow the attack: the two are enemies, and the target is
  in range of what the attacker carries from `from`. An enemy attacker
  attacks in its own phase. The attacker uses what it has equipped, or
  else the first other weapon the rules allow. No art, active or item.
- The content check runs every combat shot's fight. A file whose fight
  the rules refuse fails it with the reason, so a rebalance that puts a
  target out of reach is found by the tests, not on the title.
- A rebalance can change how a fight goes (its HP, a hit that becomes a
  miss). It is the game's fight, so it follows the game.
- The fight plays at the battle's normal speed from the shot's start
  (2½ to 4 seconds for two or three strikes, half a second more with a
  fall). A
  shot longer than its fight holds its last moment; a shorter one is cut
  off at the next shot, like any shot.
- No sound: a cinematic plays its music alone. The scenes an attack may
  set off in a battle (a boss's lines, a death quote) are not shown.
- The lead's name and picture are the cinematic's lead (below).

### The lead in `Talk`, `Characters` and `Combat`

All three show the lead of **the most recent save**: the name and the look the
player chose (Nick, ticket 0036). Without a save, the names table's name
for the lead and the male look, `lead_m`. The game notes which save was
written last under the storage key `last_save`.

More kinds come with ticket 1010 (the overworld).
