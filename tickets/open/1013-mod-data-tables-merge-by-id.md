---
id: "1013"
title: "Mods add and replace data entries by id instead of replacing whole tables"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: ["1012"]
nick_input: none
completed:
---

# 1013 — Mods add and replace data entries by id

## Context

After 1012, a mod's `data/items.ron` replaces the game's whole items
table. Two mods that each add one sword would then cancel each other out,
and every mod would carry a copy of the whole table, which `LICENSE`
section 4b discourages ("the minimum excerpts of data files needed for the
Mod to work"). 1011's ADR decides that a mod's data tables **add and
replace entries by id**, and may remove ids.

The tables are the `assets/data/*.ron` files loaded in
`crates/content/src/lib.rs` (`load_embedded`) through `ron_loader.rs`:
classes, items, skills, spells, arts, characters, names, supports, tips,
ai, terrain, palette, credits and the rest. Some are keyed by an `id`
field, some are maps, some are one record (`new_game.ron`, `keymap.ron`).

## Nick input

None.

## Scope

**In:**
- For each table in `assets/data/`, what its key is, written in the ADR's
  table (or a new section of `assets/data/README.md` if there is one; else
  create it) and used by the loader.
- Reading a table: the game's version, then each mod's version of the same
  file in load order, merged by key. A mod file may hold a `remove: [...]`
  list of keys. A one-record file (`new_game.ron`, `keymap.ron`) is
  replaced whole.
- Errors name the mod and file (via `bundle::display_path`).

**Out (do not do):** merging `.map`, `.dlg`, images or sounds (they replace
whole files, per the ADR); id clashes between mods beyond what the ADR
says; the mods folder (1014).

## Implementation steps

1. Read 1011's ADR, `ron_loader.rs` and each table's `load` function.
2. Add a bundle function that returns **every** layer's copy of a path in
   order (`bundle::all_versions(path) -> Vec<(display, &'static str)>`),
   game first, mods after.
3. In `ron_loader.rs`, add `load_merged<T, K>(path, key_of: fn(&T) -> K)`
   for list-shaped tables and a map version for map-shaped ones. A mod
   file is the same shape as the game's, plus an optional `remove` field;
   decide with the ADR whether a mod file wraps its entries
   (`(entries: [...], remove: [...])`) or uses the plain shape; document it.
4. Switch each table's `load` to the merged loader, one commit per few
   tables so a failure is easy to find. The cross-checks that already run
   after loading (`check_seals`, `check_spell_references`, ...) run on the
   merged tables unchanged.
5. A test mod under `crates/content/tests/mods/` (not in `assets/`): one
   that adds an item, one that changes an existing item's might, one that
   removes a tip. Tests build `Layer`s from it with 1012's test helpers.

## Acceptance criteria

- [ ] Each table's merge key is documented.
- [ ] A mod that adds an entry, replaces an entry and removes an entry
      gives the expected merged table (integration tests per shape: list,
      map, single record).
- [ ] Two mods changing the same entry: the later one wins (test).
- [ ] An error inside a mod's table names `mods/<id>/data/<file>.ron`
      (test).
- [ ] With no mods, every table loads exactly as before (the existing
      all-assets test passes unchanged).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `load_merged` on small hand-written RON.
- Integration: the test mods over the real `assets/` tables.
- Property: merging the game's table with an empty mod gives the game's
  table.

## Completion notes
