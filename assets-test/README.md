# Test files (`assets-test/`)

The files only the **tests** and the **debug tools** use
([ADR-0057](../docs/adr/0057-test-files-in-their-own-embedded-directory.md)).
They are laid out like `assets/` and loaded with it, but only a debug build
or a build with the debug tools (the Pages build) embeds them. **A build
that ships has none of them.**

| Path | What |
| ---- | ---- |
| `data/characters.ron` | The test characters (`test_lord`, `test_knight`, `test_archer`, `test_scout`, `test_rogue`, `test_mage`) and generic templates (`test_brigand`, `test_raider`, `test_frost_elemental`) |
| `data/names.ron` | The test characters' names |
| `data/supports.ron` | The two test support pairs |
| `data/camp.ron`, `dialogue/test_camp.dlg` | The test camp event (`test_campfire`, ticket 1003): listed at camp once the test chapter's battle is won, with the lord and the knight in the army |
| `data/lessons.ron` | The test lesson (`test_cursor`): the cursor's four directions on the first turn of the test chapter and the Quick Battle (ticket 0456) |
| `chapters/test.ron`, `battles/test.ron` | The test chapter: the tests of the game flow start a new game there (`NewGameDef::test_chapter`) |
| `chapters/quick.ron`, `battles/quick.ron` | The debug Quick Battle (title screen, where the debug tools are on) |
| `battles/test_escape.ron`, `battles/test_retreat.ron`, `battles/test_hold.ron`, `chapters/test_hold.ron` | One of Act 1's battle rules each (ticket 0319): an escape map, a boss who retreats, a boss who can't be killed in a hold whose closing scene (`dialogue/test_act1.dlg`) kills the knight with `@dies`. The bots play them (`cargo xtask playtest <id>`) |
| `battles/test_green.ron` | A green villager with two Potions of its own (ticket 0513): it drinks one when wounded. The bots play it (`cargo xtask playtest test_green`) |
| `battles/test_village.ron`, `maps/test_village.map` | Save the village (ticket 0313): five houses, save at least three, two raiders who burn them. The bots play it (`cargo xtask playtest test_village`) |
| `maps/test_small.map` | The map of both battles |
| `cinematics/test.ron` | The test cinematic: a pan over the test battle's map, on which the lead walks up to the brigand and fells it, two text boxes of the test scene, three faces crossing, then the logo. The debug menu's "Play test cinematic" plays it |
| `dialogue/test*.dlg` | The test scenes: the dialogue test, the test chapter's scenes, the battle triggers' and the supports' |
| `portraits/test_lord*`, `portraits/test_knight*` | Placeholder portraits. The bought ones are in the private repository's `test/` folder (`assets-private/test/`), which is laid over this one the way `assets-private/game/` is laid over `assets/` |
| `lang/test/` | The language pack the tests use: English in capitals. Never offered to players |
| `tilesets/test*` | The generated test tilesets (`cargo xtask test-tileset`; `assets/tilesets/README.md` describes them) |
| `images/test_card.png` | The sprite test card (`cargo xtask test-card`) |

Each kind's format is described in the README of its directory under
`assets/`.

## Rules

- A file goes here when it exists for tests or debug tools and nothing in
  the story refers to it. Its id starts with `test` (the Quick Battle's
  files are `quick`).
- **No story file may refer to a test file** (a test character, scene,
  map…): the story's files must load without this directory, and a test
  fails if they don't.
- **No path may be in both `assets/` and here**, apart from the five data
  tables in `data/`. Those hold only the test entries, which are added to
  the story's table by id. An id (or, for names, a name) that the story's
  file has too is an error.
- `tilesets/` and `images/test_card.png` are generated: never edit them by
  hand.
- The licence rules of `assets/` apply here too (ADR-0013,
  `THIRD_PARTY_ASSETS.md`).
