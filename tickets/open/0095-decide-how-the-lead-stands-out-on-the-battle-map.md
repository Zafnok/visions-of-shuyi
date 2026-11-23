---
id: "0095"
title: "Decide: how the lead stands out on the battle map"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0095 — Decide: how the lead stands out on the battle map

## Context

Nick, after watching a new player play Chapter 1 (2026-11-22):

> "The main character battle screen (not duel screen) sprite should be
> more distinctive, either a new sprite altogether or an icon to
> distinguish. Bc currently Tamsin looks more interesting than the bland
> main character"

Today (`crates/ui/src/map_view/sprite/units.rs`, the lookup around line
192): a unit's map sprite is its character's picture, else its class's.
The lead (`lead`, class `exile`) uses `lead_m` → `units/fighter_male.png`
or `lead_f` → `units/fighter_female.png` (private
`assets-private/game/tilesets/tiny_tales.ron`); the exile class also uses
`fighter_male`. Tamsin (`sergeant`, class `rider`) uses
`units/knight_m1.png`, a mounted knight. Neither map skin marks the lord:
`is_lord: true` is data only. The glyph look shows the two-letter
`map_label` (`El`). See `docs/design/look-and-feel.md` (map sprites,
around lines 249 and 340). The lead's look must work for both `lead_m`
and `lead_f` and survive renaming (0065).

Bought art only (memory: never draw character art; no commissions):
candidates come from the Tiny Tales bundle
(`assets-private/library/tiny-tales/`) and the other packs bought
(`THIRD_PARTY_ASSETS.md`).

## Nick input

**Decision** (`ask-nick`, with game screenshots made by `frame-png` of
Chapter 1's opening map, in the picture look and the glyph look, cursor
shown, per the memory on mockups covering both looks). Options (check
each game before citing):

- **A. A new map sprite** for the lead from the bundle (a hero with a
  cape or a distinct colour), several picks, male and female.
- **B. A small mark** over or beside the lead: a crown or star icon
  (Advance Wars marks a CO's unit with a badge; Fire Emblem Engage shows
  a mark on the main character's map icon), from the bundle's icon sheet
  (`C_Icons`).
- **C. A coloured outline or ground ring** under the lead only.
- **D. A and B together.** **E. Describe your own.**

Also ask: should the mark also show on the Playdate's one-bit map, and
should boss enemies get their own mark the same way (0488 keeps the seize
point visible; a boss mark is separate).

## Scope

**In:** the question with real-game pictures; the answer in
`docs/design/look-and-feel.md`; 1313's steps matched to it.

**Out (do not do):** any code (1313); drawing art.

## Implementation steps

1. Stage the pictures (throwaway `zz_` battle file + `frame-png`), and a
   sheet of candidate sprites as game screenshots (never a sheet of the
   raw files, ADR-0040).
2. Ask with `ask-nick`.
3. Record in `look-and-feel.md`; adjust 1313.

## Acceptance criteria

- [ ] `look-and-feel.md` says how the lead stands out in both looks and
      on the Playdate.
- [ ] 1313 matches.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None (a design document).

## Completion notes

