# Chapter files (`.ron`) and the New Game file

A chapter is a story beat (`docs/design/chapter-1.md`): scenes, a battle,
more scenes, then the next chapter. Each `*.ron` file here is one chapter,
loaded by `trpg_content::chapter` and validated by the all-assets test
(ADR-0035). The file stem is the chapter's id. The game flow
(`trpg_ui::flow`) plays them.

```ron
(
    id: "ch01",
    title: "Chapter 1: …",
    number: Some(1),
    intro_scenes: ["ch01_intro", "ch01_prebattle"],
    battle: "ch01",                     // assets/battles/ch01.ron
    victory_scenes: ["ch01_victory", "ch01_tbc"],
    next: Some("ch02"),                 // or None: "To be continued"
    world_map: None,                    // or Some("act1"): see below
)
```

| Field | Meaning |
| ----- | ------- |
| `id` | Must match the file name. |
| `title` | The chapter's title (for the title card, 0812, and the save slots: a save shows the title of the chapter it goes on with). |
| `number` | The chapter's number in the story, from 1: the Preparations shop sells what this chapter unlocked (`assets/data/basic_shop.ron`, 0443). Default: `None`, no shop (test chapters). |
| `intro_scenes` | Scene ids (`assets/dialogue/`) played in order before the battle. Default: none. |
| `battle` | The battle's id (`assets/battles/`). |
| `victory_scenes` | Scenes played in order after a victory. Default: none. They play for the army as the battle left it: a companion who died in Classic is gone, and their lines must be in `@if` blocks (`assets/dialogue/README.md`, "Who is still there"). |
| `next` | The next chapter's id, or `None`: the game shows "To be continued" and returns to the title. |
| `world_map` | The world map this chapter plays on (`assets/worldmaps/`, 1007), or `None` (the default): a linear chapter. |

### World-map chapters (1007)

A chapter with `world_map: Some("<map>")` lives on that world map: its
story battle is the map's place whose kind is `StoryBattle("<this
chapter>")` (each such chapter has exactly one, and the map names no
chapter that doesn't point back). When the chapter begins, the army is put
on the map (at its `start`, the first time on that map) and the map shows;
the chapter's `intro_scenes` play when the army **arrives** at its story
battle's place, then its battle; after a victory, its `victory_scenes`,
then the place is cleared and `next` begins. No "Save your progress?":
the player saves from the map's menu. A `next` that is a linear chapter
starts as linear chapters do; `None` after the act's last battle shows
"To be continued". The places a chapter opens are the map's business
(their `opens_when`, `assets/worldmaps/README.md`).

Battle notes (0411) are written with the chapter's content, but live in
the battle file (`battle_notes`, `assets/battles/README.md`): they name
the battle's units, and battles aren't one per chapter.

The flow: intro scenes → the battle → on a victory, the result is applied
to the army (`Campaign::apply_result`) → victory scenes → "Save your
progress?" (0802) → `next`. On a defeat: Game Over, with `Retry` (the battle again from
its first turn, every rewind charge back) or `Title`.

Checked: the id matches the file name; every scene exists; the battle
exists; `next` names a chapter.

## New Game (`assets/data/new_game.ron`)

```ron
(
    first_chapter: "ch01",
    roster: ["lead", "bors", "mira"],   // characters, in roster order
    gold: 0,
    stock: [],                          // item ids, one entry per item
)
```

Checked: the first chapter exists; every roster character exists, once,
and the lead (`"lead"`) is among them; every stock item exists. The lead's
unit takes the name the player gives it at New Game.

`ch01.ron` is Chapter 1, where New Game starts. `test.ron` (the test
chapter: the tests of the game flow start a new game there,
`NewGameDef::test_chapter`) and `quick.ron` (the debug Quick Battle, no
scenes) are test files, never part of the story: they are in
`assets-test/chapters/`, which a build that ships doesn't have (ADR-0057).
