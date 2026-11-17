---
id: "0498"
title: "Short weapon names where a line is too narrow"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0480"]
nick_input: decision
completed:
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

- [ ] Nick's short names are recorded in `look-and-feel.md`.
- [ ] Every weapon in the game's data shows on a forecast line whole or by
      its short name, never cut, on the PC; on the Playdate where its
      short name fits 10 cells.
- [ ] A test fails if a weapon's short name is too wide for a column.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the content check (too wide, empty); the weapon line picks the
  full name, the short name or the cut.
- Snapshot: a forecast with a Steel Sword, PC and Playdate.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
