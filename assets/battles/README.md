# Battle files (`.ron`)

Each `*.ron` file here is one battle: everything needed to play it. They
are loaded by `trpg_content::battle` and validated by the all-assets test
(ADR-0005, ADR-0035), so adding a battle needs **only data**: a map, a
battle file and its scenes. The file stem is the battle's id
(`ch01.ron` → `"ch01"`). A chapter (`assets/chapters/`) names the battle it
plays; later the world map's battles and skirmishes (1007, 1008) will too.

## A full example

```ron
(
    id: "ch01",
    map: "ch01",                        // maps/ch01.map
    player_slots: [                     // roster members, placed here
        (character: "lead", pos: (3, 12)),
        (character: "bors", pos: (4, 13)),
    ],
    enemies: [
        (template: "brigand", level: Some(2), pos: (14, 4)),
        (template: "frost_elemental", pos: (15, 6), ai: Guard, id: "frost_1"),
        (
            character: "garth", pos: (20, 2), ai: Stationary, boss: true,
            loadout: Some((weapons: ["steel_axe"], armour: None, accessory: None)),
        ),
    ],
    reinforcements: [
        (turn: 3, unit: (template: "brigand", pos: (23, 8))),
    ],
    preparations: false,                // true: the Preparations screen first
    pack_cap: 3,
    default_pack: ["potion", "potion", "potion"],
    clear_gold: 1000,
    objective: Rout(),
    triggers: [
        (when: CombatStart(unit: "garth"), scene: "ch01_garth", once: true),
    ],
    battle_notes: [
        (text: "Frost Elemental: weak to Fire, absorbs Ice.", units: ["frost_1"]),
        (text: "Garth never leaves the gate.", units: ["garth"]),
    ],
    difficulty: Normal,
    music: Cue("battle_bright"),        // or Pool("skirmish")
    seed: 12345,
    light: "dusk",                      // optional: not its map's light
)
```

## Fields

| Field | Meaning |
| ----- | ------- |
| `id` | The battle's id: must match the file name. |
| `map` | A map id (`assets/maps/<id>.map`). |
| `player_slots` | Where each roster member goes: `(character, pos)`. A slot whose character isn't in the army (e.g. dead in Classic) stays empty; roster members without a slot sit the battle out. The first slot's unit is unit 1, the next unit 2, … A slot with `after_enemies: true` is numbered after the enemies instead (and before the reinforcements): for adding a unit to a battle without renumbering the others. |
| `enemies` | The other units on the map at the start (numbered after the slots). Each is a generic `template` (from `characters.ron`'s `generics`) **or** a named `character`, written bare (`template: "brigand"`). Optional: `level: Some(n)` (templates only; a character's level is its own), `ai` (`Aggressive` default, `Guard`, `Stationary`, `Healer`), `loadout: Some((weapons: [...], armour: Some(..), accessory: Some(..)))` instead of the template's or character's, `boss: true`, `noncombatant: true` (a non-combat unit, such as a villager or a wild beast: it never uses Combat Arts or actives, `combat-arts.md`; an error together with `boss`), `defeat` (what becomes of it at 0 HP, `turn-structure.md` *Act 1's battle rules*: `Falls`, the default; `Retreats`, a boss who gets away, leaving the map alive and counting as defeated; `GoesDown`, a unit that can't be killed: it stays on its tile out of the fight, untargetable and blocking the tile, and counts as defeated), `name: Some("Garth")` (a display name; the map label becomes its first two letters), `id: "frost_1"` (written bare: a name for `battle_notes` to point at this unit), `faction: Ally` (a **green** unit, played by the AI in the Other phase; `Enemy` is the default; a green unit is neither a `boss` nor anything but `Falls`, and isn't an enemy for `DefeatUnit`), `raider: true` (an enemy that burns the houses of the `village` it ends its action on or beside, and goes for them, ticket 0313), `items: ["potion", "potion"]` (a green unit's **own consumables**, ticket 0513: consumable ids, one entry per item; it uses them like a player unit uses the pack, never the pack itself; they can't be traded or taken and are gone when the battle ends). Enemies and player slots carry no consumables (`weapons-and-items.md`): `items` on either is an error. |
| `reinforcements` | Units that arrive later: `(turn, unit: <an enemy entry>)`, numbered after the enemies. One on an occupied tile waits (`turn-structure.md`). |
| `preparations` | Whether the Preparations screen comes first (0408): the player changes the loadouts of the deployed units and fills the pack **from their own stock**, so such a battle has no `default_pack`. Default `false`. |
| `pack_cap` | How many consumables the pack may hold, written bare (`pack_cap: 3`). Default: `default_pack_cap` in `items.ron` (6). |
| `default_pack` | The consumables brought in without Preparations (given for free). Must be empty with `preparations: true`. |
| `solo_bench` | Characters in the army but not in the battle when it is played on its own (the Quick Battle): Preparations lists them so their gear can be traded. Ignored in the story. Default: empty. |
| `solo_stock` | The stock when the battle is played on its own (the debug Quick Battle): item ids, one entry per item. Ignored in the story, where the stock is the campaign's. Default: empty. |
| `solo_gold` | The party's gold when the battle is played on its own (the debug Quick Battle), to try the Preparations shop with. Ignored in the story, where the gold is the campaign's. Default 0. |
| `bots_army` | Optional, written bare: `bots_army: (chapter: "ch02", level: 35)`. The army the playtest bots field here: who is recruited by that chapter, at that level, with its ordinary gear (`docs/playtesting.md`, *Generating skirmishes*). A generated skirmish has one. Ignored in the story, where the army is the player's. Default: the characters of the player slots, as the character data has them. |
| `clear_gold` | Gold for winning. Default 0. |
| `clear_items` | Items for winning, into the stock, one entry per item (`clear_items: ["houndmasters_axe"]`: the Hound lieutenant's axe, Chapter 2); a weapon comes as a fresh copy. Default: none. |
| `objective` | `Rout()`, `DefeatUnit(unit: "garth")` (a character among the enemies or reinforcements), `Seize(pos: (x, y), by_lord: true)`, `Survive(turns: 8)`, `Escape(area: (x: 13, y: 2, w: 1, h: 6))` (every player unit still standing leaves through the exit area, one by one, with the `Escape` action; no turn limit: `turn-structure.md`, *Escape*). The first three take an optional `turn_limit: Some(n)`. `DefeatUnit` also takes `leftovers` (ticket 0845): `Stay`, the default, or `Flee` (`DefeatUnit(unit: "red_captain", leftovers: Flee)`, Chapter 1): when the unit is defeated, the battle is won and the enemies still on the map run, walking toward the nearest map edge as they fade, before the victory banner; they give no EXP and drop nothing. |
| `exit_name` | Optional, written bare (`exit_name: "the Brennish lines"`), only with `Escape`: the place the exit leads to, named on the objective line (`Get everyone to the Brennish lines (2/6 out)`), one line, translated by its English (`battles.<id>.exit`, `assets/lang/README.md`). Default: `Get everyone to the exit`. |
| `village` | Optional, written bare (ticket 0313, `turn-structure.md` *Save the village*): `village: (minimum: 3, rewards: [Gold(300), Item("elixir")])`. The houses are **every `house` tile** of the map; `minimum` is how many must be saved (1 to their number), and `rewards` gives one reward per house past the minimum (exactly that many; the first extra house standing at the win gives the first, …), each `Gold(n)` or `Item("id")` as a chest's loot. A player unit on a standing house saves it with the `Evacuate` action; a `raider` (below) that ends its action on or next to one burns it (its tile becomes `burnt` terrain, which gives no bonus); the battle is lost the moment the minimum can't be met, and on a win the houses still standing count as saved. Default: none. |
| `triggers` | The battle's story moments (0705, ADR-0030): `(when: …, scene: "id", once: true)`, with `when` one of `TurnStart(turn, phase)`, `UnitEntersArea(who: Character("id") \| Faction(Player), area: (x, y, w, h))`, `CombatStart(unit, against: Some("id"))`, `HalfHp(unit)`, `UnitFell(unit, mode: Some(Classic), recruit: true)`, `Talk(a, b)`. A trigger's scene plays for the characters whose units are on the map when it fires: its `@if` lines (`assets/dialogue/README.md`, "Who is still there"). |
| `battle_notes` | Strategy hints (0411, `magic.md` "Battle notes"): `(text: "…", units: [...])`. Shown in a `BATTLE NOTES` box when the battle starts and under the objective on the map menu's `Objective` page; the `units` (optional) blink on the map meanwhile. Each of `units` is an enemy's or reinforcement's `id`, or a character in the battle (every unit of it). A note changes no rule. Default: none, and no box. |
| `difficulty` | The map's tier: `Easy`, `Normal`, `Hard`, `Finale` → 2 / 3 / 5 / 8 rewind charges (`death-and-difficulty.md`). |
| `kind` | What the autobalance may change (`docs/playtesting.md`, *Autobalancing*; `docs/design/playtest-bots.md`): `Story` (the default), `FixedSkirmish`, `SideQuest` or `RandomSkirmish`. The first three let it change only the unnamed enemies' numbers (level, count, class, weapon); the map, named units and the objective stay as written. No rule of play reads it. |
| `music` | **Required.** What plays from the battle's start to its end, through both phases (`docs/design/audio.md`): `Cue("battle_bright")`, one music cue of `assets/audio/audio.ron` (a story battle's chosen theme), or `Pool("skirmish")`, one of its pools, from which a track is picked at random each time the battle starts (skirmishes). Restarting or retrying the battle picks again; rewinding never changes the track. |
| `seed` | Seed of the battle's random numbers. |
| `light` | Optional, written bare (`light: "dusk"`): the light this battle is fought in instead of its map's, one of `assets/data/lighting.ron`, so one map can be fought at two times of day. A look only (ADR-0058): no rule reads it, and it isn't in the battle the rules see. Default: the map's (`assets/maps/README.md`). |

## Rules checked by the loader

All reported at once, naming the entry (`player slot 2 ("bors")`,
`enemy 3`, `reinforcement 1`):

- the id doesn't match the file name; an unknown map;
- a `light` that isn't a light of `assets/data/lighting.ron` (with its line and column);
- `preparations: true` with a `default_pack`; an unknown `solo_stock` or `clear_items` item; a `solo_bench` character that is unknown, in the battle or listed twice;
- a `bots_army` level outside `1..=level_cap`;
- an unknown character or template; an enemy with both or neither;
- a level outside `1..=level_cap`, or a level given for a character;
- a loadout the unit can't carry;
- a unit outside the map, or on a tile its class can't stand on;
- two units on one tile at the start (a reinforcement may wait on one);
- a character placed twice;
- a reinforcement on turn 0;
- a `DefeatUnit` character that isn't an enemy here, a seize tile outside
  the map, an escape area that is empty or not all on the map,
  `Survive(turns: 0)`, a turn limit of 0;
- an `exit_name` on a map whose objective isn't `Escape`, or one that is
  empty or more than one line;
- a `village` whose minimum is 0 or more than the map's `house` tiles,
  whose `rewards` aren't one per house past the minimum, or with a
  reward of 0 gold or an unknown item; a `raider` that isn't an enemy, or
  in a battle with no `village`;
- a default pack over the cap, with an unknown item or a non-consumable;
- no `music`, or one that isn't a music cue (`Cue`) or a pool (`Pool`) of
  the audio manifest;
- two named units of one side with the same map label (ADR-0018; fix with a
  `map_label` in `characters.ron`); generic units may share one;
- every trigger problem `trpg_content::check_triggers` finds (unknown
  scene, a character not in the battle, an area off the map, a recruit
  who can't be killed, …);
- a unit `id` used twice, or that is also a character in the battle;
- a battle note with no text, more than one line or over 120 characters;
  more than 5 notes; a note unit that is neither an `id` nor a character in
  the battle, or that is listed twice.

`ch01.ron` and `ch02.ron` are Chapters 1 and 2's battles. `test.ron` (the test chapter's battle),
`quick.ron` (the debug Quick Battle) and `test_escape.ron`,
`test_retreat.ron` and `test_hold.ron` (one of Act 1's battle rules each,
0319) and `test_village.ron` (save the village, 0313) are test files, never part of the story: they are in `assets-test/battles/`, which a build that ships doesn't
have (ADR-0057).
