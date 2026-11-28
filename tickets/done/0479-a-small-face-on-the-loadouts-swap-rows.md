---
id: "0479"
title: "Decide and build: who holds an item, at a glance, on the Loadouts swap rows"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0478"]
blocks: ["1064"]
nick_input: decision
completed: 2026-11-16
---

# 0479 — Who holds an item, at a glance, on the Loadouts swap rows

## Context

A friend's playtest of the Act 1 build, passed on by Nick (2026-11-07):

> "There should probably be a mini icon of character portrait in the
> loadout stock screen for swapping"

On Preparations → Loadouts, the list beside a slot shows what the other
units hold in such a slot (`docs/design/weapons-and-items.md`, *Loadout
(per unit)*: "Steel Bow 25/25 from Test Scout"). The rows are text only
(`unit_rows`, `crates/ui/src/screens/preparations.rs` ~:432; `ItemRowView`,
`preparations/view.rs` ~:130).

**Nick dropped a mini portrait once.** `docs/design/look-and-feel.md`
~:77: he saw "the 16×16 panel mini-portrait" and said it "looks like a
memeface so we can just forego the battle portrait"; ~:779: portraits are
"**Not** in the battle side panel or hover … (Nick dropped the mini
portrait)". So the look is his call, on real frames, before building.

What exists to draw a small face with:

- Portraits: `assets/portraits/*.ron` sidecars, 256×256 frames
  (`trpg_content::portrait::FRAME_PX`); the real art is in the private
  repository (ADR-0040; `cargo run -p trpg-app --features
  private-assets`). `crates/ui/src/portrait.rs`: `draw_portrait_in` draws
  one in any area at whole-number scales.
- The dialogue log already shows a face at 8×4 cells (0729;
  `screens/dialogue/log/glyph.rs`), and tips show Vix's
  (`tips/glyph.rs`).
- The units' bought map sprites, 16×20 (`map_view/sprite/units.rs`).

## Nick input

**Decision** (`ask-nick`, before building), on `frame-png` pictures of
the Loadouts screen with the private art, one swap list with three or
more holders. Options:

- A. The dialogue log's small face (8×4 cells) at the start of each swap
  row.
- B. The unit's map sprite (the one he sees on the battle map) at the
  start of each row.
- C. No picture: the holder's name first and in the unit's colour
  (`Test Scout: Steel Bow 25/25`).
- D. Combinations, or describe your own.

Show the glyph look too (memory: mockups cover both looks): with sprites
off, A and B need a glyph stand-in (the unit's map glyph). If you cite a
real game's trade or convoy screen, check what it shows first: claim
only what the game really does.

## Scope

**In:**
- The question and the answer, recorded in `weapons-and-items.md`
  (*Loadout (per unit)*).
- What he picks: a holder id on each swap row in the view (e.g.
  `ItemRowView.holder: Option<CharacterId>`), and the skin drawing the
  picture (a sprite item from an asset file, ADR-0038: never per-pixel
  cells), with a glyph stand-in when sprites are off.
- The Playdate skin of Preparations (1046): if it is done, it shows the
  holder by its own rule (text is fine); if not, a line in 1046.

**Out (do not do):**
- The item numbers line (0478).
- Faces anywhere else (the side panel, the forecast).
- Moving the lists onto the engine's panels (1238).

## Implementation steps

1. After 0478, render the options with `cargo shuyi frame-png` (private
   art on) and ask Nick (`ask-nick`).
2. Record the answer; add the holder id to the swap rows in
   `unit_rows`.
3. Paint the pick in `preparations/glyph.rs` (and the picture skin, if
   the screen has one), as a sprite item.
4. Tests and snapshots.

## Acceptance criteria

- [x] Nick's pick is recorded in `weapons-and-items.md`.
- [x] Each swap row in the view names its holder (test).
- [x] The picked look is painted on the swap rows, glyph and picture
      looks (`frame-png` pictures in the PR).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the swap rows' holder ids.
- Snapshot: the Loadouts swap list.

## Completion notes

**Nick's pick** (2026-11-16, on `frame-png` pictures of the knight's
armor swap list, four holders, with the bought art): A, the dialogue
face, "but I think it probably needs to be scaled smaller". Of half size,
three quarters and a crop of the head at full size, he took three
quarters ("looks best"), and asked for "arrows for scrolling if more
characters live off screen". Recorded in `weapons-and-items.md`
(*Loadout (per unit)*) and the portrait list of `look-and-feel.md`.

**Done:**
- `ItemRowView.holder: Option<HolderView>` on each swap row: the
  character, its portrait's id (the lead's by gender) when the game has
  one, and its map letters. The stock's own rows have none.
- The glyph skin paints a swap row with a face three rows tall: the
  neutral face as one sprite under the glyphs, 48×48 (a 64×64 face at
  three quarters; any other size fitted to 48 keeping its shape), the
  item on the middle row. A holder without a face (the build without the
  bought art, or a unit with no portrait) gets its map letters in the
  player's colour on a one-row row.
- The stock list scrolls by its rows' heights and shows `▲` / `▼` at
  its right edge while rows are hidden above or below (the credits'
  marks). Only this list: the other Preparations lists are unchanged.
- 1046 (Playdate Preparations, not done) gets a line: the Playdate shows
  the holder by its own rule; the `from Test Scout` text is enough.

**Deviations:**
- The ticket's "glyph stand-in when sprites are off": Preparations has
  no picture skin and the map look setting is the battle map's, so the
  stand-in is chosen by whether there is a face to draw (the public
  build's units have none but the test lord and knight), not by that
  setting, as the dialogue log's faces are.
- Scaling to 48×48 is not a whole scale; `draw_portrait_in` only does
  whole scales, so the skin adds the sprite itself (a few lines).

Engine friction: estimate 90, measured 141 (1.6×), under 2×; signals:
workaround no · third copy no · budget no.
PR size: estimate 400, measured 387 before these notes.

Follow-up tickets: none.

### For the Act 1 playtest (1064)

**Starting rules** (the agent's, for the Act 1 playtest, 1064):
- The face is the portrait's neutral expression.
- The row keeps `from Test Lord` beside the face, as in the pictures
  Nick picked from.
