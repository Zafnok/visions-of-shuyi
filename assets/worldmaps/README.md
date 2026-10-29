# World map files (`.ron`)

One file per act's world map (ticket 1007, ADR-0099,
`docs/design/world-structure.md`). The file stem is the map's id, the one
chapters name in `world_map` (`assets/chapters/README.md`). Loaded and
checked by `trpg_content::worldmap`; the rules are `trpg_core::worldmap`.

There is no real map here yet (Act 1's is ticket 1070). The test maps are
in `assets-test/worldmaps/` (`test_world`, then `test_world_2`), which the
debug menu's "Test world map" starts.

```ron
(
    act: 1,
    levels: (1, 6),              // the act's enemy levels, lowest and highest
    music: "world_map",          // the map's track (audio.ron)
    start: "home",               // where the army starts
    final_node: "bridge",        // the act's final story battle
    next: Some("act2"),          // the next act's map, or None (the default)
    size: (70, 30),              // in cells, at most (70, 30)
    picture: None,               // an image for the picture look (below)
    nodes: [
        (id: "home", name: "Millbrook", pos: (14, 20), music: Some("village"),
         kind: Town(shops: [
             (kind: Armoury, stock: ["iron_sword"]),
             (kind: Vendor, stock: ["potion"]),
         ])),
        (id: "ford", name: "Stone Mill", pos: (34, 13), kind: StoryBattle("ch04")),
        (id: "pass", name: "Wolf Pass", pos: (30, 5),
         kind: SideQuest((battle: "sq_pass", intro_scenes: ["sq_pass_intro"],
                          victory_scenes: ["sq_pass_won"])),
         opens_when: [Cleared("ford"), Alive("tamsin")]),
        (id: "cross", name: "Crossroads", pos: (24, 16), kind: Empty),
    ],
    roads: [
        (from: "home", to: "cross", via: [(14, 16)]),
        (from: "cross", to: "ford", via: [(24, 13)]),
    ],
    ground: [ /* `size.1` rows of `size.0` characters, below */ ],
)
```

| Field | Meaning |
| ----- | ------- |
| `act` | The act, from 1. |
| `levels` | The act's enemy levels; random skirmishes are capped against it (1008). |
| `music` | The music cue the map plays: to its end, a fade, 30 s of quiet, again (`audio.md`, *The world map*). |
| `start` | The place the army starts at: open from the start (no `opens_when`) and not a battle. |
| `final_node` | The act's final story battle. Winning it moves the army to `next`'s start, with nothing cleared; this map can't be reached again. |
| `next` | The next act's map, or `None`. |
| `size` | Across and down, in cells; the screen shows the whole map (no scrolling yet). |
| `picture` | An image path in the bundle for the picture look (the bought World Map tiles, made into one picture per act, ticket 1070): drawn over the map's cells, stretched to `size`. Without one the picture look draws the glyph look. |
| `ground` | The glyph look's ground: one character a cell (`' '` `.` `,` plain, `f` forest, `t` thicket, `^` mountain, `A` peak, `w` river, `~` sea). Empty: all plain. |

**A place** (`nodes`):

| Field | Meaning |
| ----- | ------- |
| `id` | Unique on the map. |
| `name` | Its name as the panel shows it (translatable: `worldmaps.<map>.<id>.name`). |
| `pos` | Its cell. At least one cell from the left and right edges (its plate is three cells wide) and one row from the top (its level marker). |
| `kind` | `StoryBattle("<chapter>")`: that chapter's battle (the chapter says `world_map: Some("<this map>")`). `SideQuest((battle, intro_scenes, victory_scenes))`: a battle of its own. `Town(shops: [...], capital: false)`: shops as on battle maps (`kind`, `stock`), checked as they are (a Tailor sells cloth and leather only, a Blacksmith sells nothing, no Basic shop). `Empty`: a place to pass through. Fixed and random skirmishes come with 1008. |
| `music` | A cue played while the army stands there (a town's `village`, a capital's `capital`). |
| `opens_when` | Every condition must hold for the place to show: `Cleared("<place>")` (its battle won), `Flag("<flag>")`, `Alive("<character>")` (in the army). A story battle also waits for its chapter to begin. A cleared place stays open. |

A battle's **level marker** is worked out from its battle file: the
average level of its enemies, rounded.

**A road** joins two places (`from`, `to`) and runs straight across or
down, turning at each cell of `via` in order. A road is open when both
its places are.

**Checked** (`cargo test -p trpg-content`): roads join places and run
straight; every place can be reached from `start` once every condition
holds; `start` and `final_node` are right; story battles name chapters on
this map, side quests battles and scenes that exist, conditions places
and characters that exist; town shops; music cues; `size`, `ground` and
where places sit; `next` names a map; a chapter's `world_map` names a map
with its story battle.
