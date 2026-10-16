# Openings (`.ron`)

A conversation's short **moving opening** (ticket 0734,
[ADR-0076](../../docs/adr/0076-conversation-openings-a-cast-on-a-place-and-the-cinematic-walks.md);
`docs/design/look-and-feel.md`, *Places behind conversations*). A script
asks for one with `@opening <id>` after a `@place <map> <x>,<y>` line
(`assets/dialogue/README.md`, *Places*). Before the next text box, the
place is shown at full brightness while the opening's characters walk on
it; then it darkens over half a second and the talk comes up over it,
the characters standing where they stopped.

Each `*.ron` file here is one opening; the file stem is its id. They are
loaded by `trpg_content::cinematic::load_openings` and checked by the
all-assets test and `cargo xtask check-script`; the dialogue screen plays
them (`trpg_ui::cinematic::opening`).

```ron
(
    id: "ch01_sparring",
    map: "harrowby_yard",
    cast: [
        (who: "lead", at: (11, 8), facing: Right),
        (who: "retainer", at: (13, 8), facing: Left),
    ],
    action: [
        (at: 0.0, do: Walk(unit: "lead", path: [(11, 8), (12, 8)], secs: 0.2)),
        (at: 0.3, do: Walk(unit: "lead", path: [(12, 8), (11, 8)], secs: 0.2, facing: Right)),
    ],
)
```

| Field | Meaning |
| ----- | ------- |
| `id` | Must match the file name. |
| `map` | The place it plays on: a map of `assets/maps/` by its file name. A script's `@opening` must follow a `@place` with this map. |
| `cast` | Who is there, at least one. |
| `who` | A character of `assets/data/characters.ron`: a unit's character or a speaker. Each once. |
| `at` | The tile `(column, row)` they stand on when it starts: on the map, one they can stand on, nobody else's. |
| `facing` | Optional: `Down` (towards the camera, the default), `Left`, `Right` or `Up`, the way they face while they stand. |
| `action` | What they do: walks, at least one, as a cinematic's action has them (`assets/cinematics/README.md`, *Action*), with `unit` naming someone of the cast by their character. No strikes. |

- Who they are on the place: their map picture (the lead's by the look
  the player chose), or their two-letter label in the glyph look. No HP
  bar. Someone with a class walks by its movement type; a speaker (who
  has none) walks on foot.
- How long it plays: until its last walk ends. About **1.5 seconds**
  (Nick: half of the mockup's 3); the darkening adds half a second.
- A walker faces the way it goes; with `facing`, that way all along (a
  step back towards an opponent). After a walk it keeps facing the way it
  last faced, not the camera.
- Pictures, not a fight: nothing here runs the rules, and a rebalance
  never changes an opening.

Chapter 1's two:

| Opening | Scene | Place |
| ------- | ----- | ----- |
| `ch01_sparring` | `ch01_intro` | the yard at Harrowby: the lead and Hollis step in and back at each other |
| `ch01_pacing` | `ch01_tbc` | the study at Veyne Hall: Dace paces across the rug and half way back |
