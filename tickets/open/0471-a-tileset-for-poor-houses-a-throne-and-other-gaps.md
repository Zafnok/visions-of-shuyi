---
id: "0471"
title: "Find a tileset for poor village houses, a throne, a better gate and the picture look's other gaps"
type: research
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0322"]
nick_input: decision
completed:
---

# 0471 — A tileset for poor houses, a throne and other gaps

## Context

Ticket 0037 (2026-10-22) added house, gate and throne tiles
(`docs/design/terrain.md`, *Building tiles*). The bought Tiny Tales
bundle (ADR-0040, `look-and-feel.md`) doesn't cover them well. Nick:

> "imo the tileset look for the house is too modern/nice looking, these
> should be poor peasant people getting raided they need shittier houses.
> Probably need to buy additional assets for this so we can have some
> ticket scope for good tilesets for this that might bundle a few other
> gaps in our current design." … [gate:] "The tileset here is
> serviceable, it's not great but not as bad as house." … [throne:] "we
> def need a new tileset to complement our existing one if all we got is
> a chair bruh"

0322 draws stand-ins: the World Map set's small house, the Fortress
archway, a chair. This ticket finds art that fits the Tiny Tales look
(16×16 tiles, its palette and outline style) for:

1. **Poor peasant houses** for a raided village (Chapter 2, Kell's Ford):
   thatched, shabby, one tile each.
2. **A throne** (one tile, on the indoor floor).
3. **A better gate** (one tile in a wall), optional.
4. **Other gaps of the picture look**, bundled if one pack covers them:
   at least *Burning* (`look-and-feel.md`'s terrain table: a recoloured
   forest stands in), and any other stand-in listed there.

## Nick input

**Decision:** show Nick the candidates (store previews rendered next to
our map in a scratch mockup, never committed, `CLAUDE.md` *Bought art*)
with prices and licences; he picks and buys. Nothing is bought without
him.

## Scope

**In:**
- Search, in this order: Mega Tiles' other packs (Tiny Tales' artist, the
  best match), itch.io and other stores, then Japanese free-material
  sites (memory: Nick accepts hand-made art; search those early); AI
  art only sparingly. Never a commission (Nick vetoed them).
- Check each licence against ADR-0013 and ADR-0040 (commercial use in a
  sold game, no redistribution of the raw files is fine).
- Mockups of our map with each candidate's houses, throne, gate, in the
  scratchpad; ask Nick with the `ask-nick` skill.
- After he buys: add the files to the private repository's `library/`
  and `game/`, swap the stand-ins in `assets-src/tilesets/tiny_tales.ron`,
  credits (`assets/data/credits.ron`, the `private` list, `work-ticket`
  step 3), `cargo shuyi private-assets --pin`, `look-and-feel.md`'s
  table. If the import is more than a swap of tiles, write a follow-up
  ticket for it instead.

**Out (do not do):**
- The glyph look (decided in 0037, built in 0322).
- Unit sprites, portraits, battle pictures.

## Implementation steps

1. List every stand-in in `look-and-feel.md` and the three tiles above.
2. Search; shortlist 2–4 packs with prices, licences, what each covers.
3. Scratch mockups; ask Nick.
4. When bought: import, swap, credit, pin; `frame-png` the test map from
   0322 for the PR.

## Acceptance criteria

- [ ] Nick picked a pack (or chose to keep the stand-ins); his words are
      in `look-and-feel.md`.
- [ ] If bought: the house and throne (and what else the pack covers)
      are drawn from it; credits list it; `assets-private.rev` is pinned.
- [ ] `cargo xtask ticket-lint` and the `run-gates` skill pass.

## Tests required

- The tileset tests (`crates/content/src/tileset.rs`) still pass with the
  swapped tiles; nothing new unless the import adds code.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
