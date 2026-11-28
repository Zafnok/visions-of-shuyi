---
id: "1392"
title: "Content's Raw structs that only wrap strings into ids deserialize the core type directly"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["1237"]
nick_input: none
completed: 2026-11-27
---

# 1392 — Content's Raw structs that only wrap ids deserialize the core type

## Context

Step 6 of 1237 (shared test drivers and fixtures), split off by its size
rule: 1237 measured 1,747 changed lines before this step (`cargo xtask
pr-size`), past the 1,500 its *Scope* set for doing it in the same PR.

From the 2026-11-17 friction sweep
([report](../../docs/research/friction-2026-11-17.md), bucket B3): adding
one field to a definition edits the core type, its `Raw*` mirror in
`trpg-content` and the conversion between them. `trpg-content` has about
63 `Raw*` serde types (`grep -rn "struct Raw\|enum Raw" crates/content/src`:
`item.rs` 10, `battle.rs` 8, `worldmap.rs` 7, `spell.rs` 5, `lesson.rs`
4, `character.rs` 4, …). Some only turn a `String` into an id
(`ClassId`, `SkillId`, `SpellId`, `ItemId`, `ArtId`, `CharacterId`, …),
which the core id types can deserialize themselves: they derive
`Deserialize`.

## Nick input

None. Loading only; nothing a player sees changes.

## Scope

**In:**
- Each `Raw*` type of `crates/content/src` whose conversion only wraps
  strings into core ids (no lookup, no check, no default worked out from
  other fields): deserialize the core type directly, and delete the
  `Raw*` type and its conversion.
- A list, in the completion notes, of every `Raw*` type kept and why.

**Out (do not do):**
- A `Raw*` type whose conversion checks something, looks something up, or
  works a default out from other fields: it stays.
- The serde mirror enums of `content/src/enums.rs`: 1230 deletes them.
  Either order: skip any `Raw*` type 1230 already removed; if 1230 is
  not done, leave the enums to it.
- Changing a data file's format: the `.ron` files read the same before and
  after.
- Deriving `Default` or `Deserialize` on a core type whose fields don't
  all read from data as they are (ADR-0004: `core` stays pure; a serde
  derive is fine, I/O is not).

## Implementation steps

1. List the `Raw*` types (`grep -rn "struct Raw\|enum Raw"
   crates/content/src`) with each one's conversion (`fn into_*`, `From`,
   `TryFrom`, or the loader's `map`). Mark each "ids only" or "keeps"
   with its reason.
2. For each "ids only" type: give the core type `Deserialize` (with
   `#[serde(deny_unknown_fields)]` and the same `#[serde(default)]`s the
   `Raw*` type had), read it directly in the loader, and delete the
   `Raw*` type and its conversion.
3. Content errors keep their file and line: run `cargo test -p
   trpg-content`, whose error tests cover them, and add one for a type
   whose errors no test reads yet.
4. Measure with `cargo xtask code-lines` and `cargo xtask pr-size`.

## Acceptance criteria

- [x] Every `Raw*` type left in `crates/content/src` is named in the
      completion notes with the check or lookup that keeps it.
- [x] No data file under `assets/`, `assets-test/` or
      `assets-private/content/` changes.
- [x] `cargo test -p trpg-content` passes, its error tests unchanged in
      what they check (file and line included).
- [x] The same tests pass, none removed (`cargo test --no-fail-fast`).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The existing content loading and error tests, unchanged in what they
  check; a new error test only where a removed `Raw*` type's errors had
  none.

## Completion notes

**Done.** Nine of the 63 `Raw*` types are gone; the loaders read the
core (or content) type itself and run the same checks on it:

| Removed | Now read as | Ids |
| ------- | ----------- | --- |
| `art::RawArt` | `trpg_core::ArtDef` | `id` through `bare_id::one` |
| `camp::RawEvent` | `trpg_core::CampEvent` | `units: Vec<CharacterId>` (already a bare string) |
| `chapter::RawNewGame` | `NewGameDef` (content's) | `stock` through `bare_id::list` |
| `character::RawLoadout` | `trpg_core::LoadoutDef` (characters, generics, battle enemies) | `bare_id::list` / `bare_id::option` |
| `worldmap::RawCondition` | `trpg_core::NodeCondition` (its existing derive) | `Alive(CharacterId)` |
| `cinematic::RawSide` | the engine's dialogue `Side` | none; `#[serde(default = "right")]` |
| `spell::RawKind` | `trpg_core::SpellKind` | none |
| `item::RawRules` | `trpg_core::WeaponRules` | none |
| `item::RawPassive` | `trpg_core::WeaponPassive` | none |

Each new derive has `#[serde(deny_unknown_fields)]` and the `Raw*` type's
`#[serde(default)]`s. No data file changed. `cargo test -p trpg-content`
and `-p trpg-core` pass unchanged; a new test
(`character::tests::a_loadout_that_does_not_parse_names_its_line`) checks
that a loadout that doesn't parse (an id that isn't a string, an unknown
field) still reports `file:line`; no test read those errors before.

**Deviation: the ticket's premise about ids.** The game's own ids
(`ItemId`, `ArtId`, `SpellId`, `SkillId`, `ClassId`) derive `Deserialize`
as newtype structs, so RON reads them only as `("iron_sword")`. That is
the form saves and play records write (`crates/core/tests/fixtures/
save_v2_*.ron`, the winning fixtures), and the data files write the bare
`"iron_sword"`. Only the engine's `CharacterId` is `#[serde(transparent)]`.
Making the ids transparent would change the save format (a
`SAVE_VERSION` bump that refuses every player's saves) and the records,
which is not "loading only". So a new core module, `trpg_core::bare_id`
(`one`, `list`, `option`: pure serde, ADR-0004), reads a bare string into
any id with `From<String>` (added for `ItemId` and `ArtId`). A definition
read straight from data names it on its id fields with
`deserialize_with`; the ids' own `Deserialize` stays the saves'.

The ticket's rule read as: a `Raw*` type goes when its fields map one to
one onto a type that can be read as written, and its checks can run on
that type after reading. Checks that sat in the same loop as the
conversion (empty id, duplicate, cost 0, negative might) stay, now
reading the core value.

**Kept, and why** (54):

- *File wrappers* (no core type has the file's shape: a list under a
  field, sometimes with file-level fields): `ambience::RawFile`,
  `art::RawFile`, `basic_shop::RawFile`, `camp::RawFile`,
  `character::RawFile`, `class::RawFile`, `item::RawFile`,
  `lesson::RawFile`, `skill::RawFile`, `spell::RawFile`,
  `support::RawFile`, `support::RawPart`, `terrain::RawFile`. Their
  inner types are named below.
- `ambience::RawFile` also: its maps become id sets and lookups.
- `basic_shop::RawRow`: the core keeps rows as `(ItemId, Unlock)` pairs;
  no core row type, and each row is checked for duplicates and chapter 0.
- `battle::RawBattle`, `RawEnemy`, `RawSlot`, `RawReinforcement`,
  `RawObjective`: units looked up by template or character, ids numbered,
  positions checked against the map, an objective's unit resolved.
- `battle::RawVillage`, `RawLoot`: rewards checked against the item
  table, houses counted from the map, `burnt` looked up; and core `Loot`'s
  own `Deserialize` reads the save form `Item(("x"))`, not the file's
  `Item("x")`.
- `battle::RawNote`: unit names resolved to unit ids.
- `battler::RawSet`: pictures looked up by name.
- `character::RawCharacter`, `RawGeneric`: classes looked up, stats
  worked out, the `enums.rs` mirrors (1230's).
- `cinematic::RawShot`, `action::RawAction`, `RawAct`, `RawActionRef`,
  `opening::RawOpening`, `RawCast`: units and actions resolved by name,
  points built, combats checked against their battle.
- `class::RawClass`: `enums.rs` mirrors (1230), movement type checked,
  spells, skills and promotions looked up.
- `item::RawKind`: rows become a map, every kind exactly once.
- `item::RawWeapon`: range split into min and max, the special part built
  from `special`/`wielder`/`passive`, its id is the table's key.
- `item::RawBonus`: becomes `Stats` (`from_growable`), another shape.
- `item::RawArmour`, `RawAccessory`, `RawConsumable`, `RawSeal`: their
  `id` is the table's key, not a field of the core def (a flattened row
  can't keep `deny_unknown_fields`); armour also maps an `enums.rs`
  mirror.
- `lesson::RawDone`, `RawItem`, `RawLesson`: action names looked up
  (`Action::from_name`).
- `skill::RawSkill`: `family` defaults to the id (a default worked out
  from another field).
- `spell::RawSpell`: range split into min and max, terrain effect
  resolved. `RawTerrainEffect`, `RawLasts`: terrain names looked up.
- `support::RawPair`: `SupportPair::new` orders the pair; the pair is
  the key.
- `terrain::RawTerrain`: glyphs and colours parsed, move costs mapped to
  movement types.
- `worldmap::RawLater`: its shop index comes from its position;
  `RawShop`: core `Shop`'s own `Deserialize` reads the save form of
  `ItemId`; `RawKind`, `RawNode`, `RawRoad`, `RawWorldMap`: places,
  levels, roads and names worked out and looked up.

The `content/src/enums.rs` mirrors are 1230's, untouched.

Friction: 2.7×, accepted because the estimate (40 added) left out the
bare-id helper and its tests; measured 109 added, 227 removed (net −118).
Row in `docs/engine/friction.md`. PR size: estimate 500, measured about
750.

Follow-ups: none.
