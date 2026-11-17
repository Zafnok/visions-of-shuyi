---
id: "0498"
title: "Short weapon names where a line is too narrow"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0480"]
nick_input: decision
completed: 2026-11-17
---

# 0498 — Short weapon names where a line is too narrow

## Context

Ticket 0480 put a weapon's uses after its name in the attack forecast
(`Iron Sword 20`, `look-and-feel.md`, *Attack forecast*, *HP after the
fight, and the weapon's uses (0480)*). A forecast column is 13 cells
(`crates/ui/src/screens/battle/glyph/forecast.rs`, `COLUMN_W`), and 6 of
the game's 14 weapons don't fit with their uses: Steel Sword 25, Steel
Spear 25, Iron Gauntlets 20, Steel Gauntlets 25, Houndmaster's Axe 25,
Fordwarden's Spear 20 (`assets-private/content/data/items.ron`). The
Playdate's columns are 10 cells (`playdate/forecast.rs`, `arms_text`), so
there most names don't fit.

0480 shipped a stopgap: the name is cut with `…` (`Steel Swo… 25`, the
Playdate `Iron S… 20`). Nick didn't want that (2026-11-16, asked during
0480):

> "use shortened names rather than truncated i.e. Stl Swo / probably need
> some mapping from long to short name"

So each weapon gets a **short name**, written by hand, shown where its full
name doesn't fit.

## Nick input

**Decision** (`ask-nick`, before building): the short name of each weapon
that needs one, shown as `frame-png` pictures of the forecast (PC and
Playdate). Draft them from Nick's example (`Stl Swo` for Steel Sword) and
offer one or two styles, e.g. vowels dropped (`Stl Swd`, `Hndmstr Axe`) or
first word cut (`Stl Sword`). Ask also whether a weapon whose full name
fits keeps it (the recommendation) or every weapon always shows its short
name in narrow lines.

## Scope

**In:**
- An optional short name per weapon in the item data (`items.ron`, public
  stand-in and private content), checked by the content's validation (not
  empty, at most as wide as the forecast's column minus its uses).
- Its translation: a key beside `items.<id>.name` in the language packs
  (the `test` pack's too, `assets-test/lang/test/data.ron`), and a
  `Words` method for it (`crates/ui/src/words.rs`, next to `item`).
- The forecast's weapon line, glyph and Playdate: the full name when it
  fits with its uses, else the short name, else (no short name) the cut
  0480 does today.
- The content change goes through the private repository (`content-change`
  skill) and this PR moves `assets-private.rev`.

**Out (do not do):**
- The Playdate's Equip and spell lists (1075 decides those; tell 1075 the
  short names exist so it can use them).
- Spells (their names all fit with their uses today).
- Any other screen's weapon names.

## Implementation steps

1. Read 0480's completion notes and `glyph/forecast.rs` (`paint_weapon`),
   `playdate/forecast.rs` (`arms_text`).
2. Draft the short names, render them, ask Nick; record his answer in
   `look-and-feel.md` under 0480's section.
3. Add the field and its check in `trpg-content` (items' format and its
   `README.md`), the language key, the `Words` method.
4. Use it in both skins' weapon line.
5. Fill the data in the private repository and the public stand-in.

## Acceptance criteria

- [x] Nick's short names are recorded in `look-and-feel.md`.
- [x] Every weapon in the game's data shows on a forecast line whole or by
      its short name, never cut, on the PC; on the Playdate where its
      short name fits 10 cells.
- [x] A test fails if a weapon's short name is too wide for a column.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the content check (too wide, empty); the weapon line picks the
  full name, the short name or the cut.
- Snapshot: a forecast with a Steel Sword, PC and Playdate.

## Completion notes

Nick picked from `frame-png` pictures of three forecasts (Ellery's Steel
Sword against Fordwarden's Spear, Steel Gauntlets and Houndmaster's Axe,
in a throwaway staged battle, PC and Playdate): **B, the first word
shortened and the kind kept**, and a weapon whose full name fits keeps it.
Not picked: A, vowels dropped (`Stl Swd`, `Hndmst Axe`); C, seven letters
for every weapon so the Playdate never cuts. Recorded in
`look-and-feel.md`, *Short weapon names (0498)*.

| Weapon | Short name |
| ------ | ---------- |
| Steel Sword | Stl Sword |
| Steel Spear | Stl Spear |
| Iron Gauntlets | Iron Gntlt |
| Steel Gauntlets | Stl Gntlt |
| Houndmaster's Axe | Hound Axe |
| Fordwarden's Spear | Ford Spear |

Built:

- `WeaponDef::short_name` (core, beside `name`); `short_name:
  Some("..")` in the items file, documented in its header (the items
  format has no `README.md`; its header is where the format lives),
  checked on load: not blank, and at most `WEAPON_LINE_CELLS` (13) less a
  space and the durability's digits (`item::short_name_room`). A test
  checks every weapon in the game's data fits by name or short name, and
  one that the UI's `COLUMN_W` is the content's 13.
- The language key `items.<id>.short` (`Lang::short_name`, the README's
  table, the `test` pack's `items.steel_sword.short`) and
  `Words::short_name`.
- The forecast view carries the weapon's short name
  (`ForecastSideView::short`); both skins pick the name with one function,
  `glyph/forecast.rs` `weapon_name`: the full name if it fits with its
  uses, else the short name, else cut with `…`.
- Data: the private repository (Zafnok/visions-of-shuyi-assets#9) and the
  public stand-in.
- 1075's Out section now says the short names exist.

**The Playdate:** none of B's names fits its 7 cells before `25`, so
there the short name is cut (`Stl Sw… 25`, `Hound … 25`). The acceptance
criterion "where its short name fits 10 cells" holds trivially. If Nick
wants the Playdate's forecast uncut, the short names would need a second,
7-cell form (option C was that); 1075 decides the Playdate's lists and
may revisit it.

*The agent's starting rules (Nick can veto; for the Act 1 playtest,
1064):*

- Where even the short name doesn't fit (the Playdate), the short name is
  cut, not the full name (`Stl Sw… 25` rather than `Steel … 25`: more of
  the weapon shows). The pictures Nick saw cut the full name there.
- A weapon with no uses after it (an art spending them, broken) gets the
  same rule against the whole column, so a long name shows its short name
  rather than being clipped without `…`.

The private pull request is based on the game's pin, not on the private
`main`: `main` also carries 0484's content commit, whose game PR (#720)
hadn't merged. After both merge, the pin moves to the private `main`.

Engine friction: estimate 70 production lines (a field, its check, a
language key, a `Words` method and one picker for two skins, judged by
0480's `paint_weapon`), measured 112 added (1.6×), under 2×.
