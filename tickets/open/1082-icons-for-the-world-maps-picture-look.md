---
id: "1082"
title: "Bought or free icons for the world map's picture-look plates"
type: content
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["1007"]
nick_input: decision
completed:
---

# 1082 — Icons for the world map's picture-look plates

## Context

Ticket 1007's picture look of the world map (`docs/design/look-and-feel.md`
*The world map*) draws each place as the glyph look's symbol plate: a
coloured square with a font symbol (`‼` story battle, `♥` side quest,
`⌂` town), with a dark edge (`crates/ui/src/screens/world_map/picture.rs`).
Nick (2026-10-29): "we can also use itch io icons for the pixel look
rather than in house renders... the symbol plates specifically, but it can
be a follow up".

## Nick input

**Decision, before building:** which icon set. Search itch.io (and the
free Japanese sites, `feedback-art-candidates`) for small pixel icon sets
that match the bought Tiny Tales world-map tiles; check each licence
against ADR-0013 (no non-commercial, nothing copyleft; a paid pack needs
Nick's purchase, ADR-0040, and then lives in the private repository);
render each candidate on the world map at 1× and 2× zoom (local mockups
from the bought art stay uncommitted, ADR-0040) and ask with `ask-nick`.
The plates must stay as distinct as today: story battle, fixed skirmish,
random skirmish, side quest (1008), town, capital.

## Scope

**In:**
- The chosen icons as image files (public `assets/` if free and
  permissive, `assets-private/game/` if bought, with its credit's
  `private` list, ADR-0051), credited (`assets/data/credits.ron`,
  `THIRD_PARTY_ASSETS.md`).
- `picture.rs` draws a place's icon sprite (ADR-0038: a sprite item from
  the asset file) in place of the coloured square and its symbol; the
  glyph look keeps its plates.

**Out (do not do):**
- The glyph look's plates (Nick picked them).
- Drawing icons ourselves.

## Implementation steps

1. Search, check licences, render candidates, ask Nick.
2. Add the files and credits; a small `.ron` naming each mark's icon.
3. `picture.rs`: the icon sprite per `PlaceMark`; tests read the sprites
   (a look test, `screens/world_map/tests.rs`).

Expected size: about 60 production lines (an icon table and its sprites).

## Acceptance criteria

- [ ] Nick's pick is in `look-and-feel.md` *The world map*.
- [ ] Every mark draws its icon in the picture look (a test of the
      sprites), and the glyph look is unchanged.
- [ ] Credits and `THIRD_PARTY_ASSETS.md` list the set.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Look test: the picture look's sprites per mark.
