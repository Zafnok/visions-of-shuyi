---
id: "0444"
title: "The cursor on picture tiles: easier to see, if Nick wants it"
type: tuning
milestone: M3 Battle UI
model: sonnet-5
effort: medium
status: done
blocked_by: ["0437"]
nick_input: sign-off
completed: 2026-10-13
---

# 0444 — The cursor on picture tiles

## Context

Ticket 0437 drew the battle map's terrain with the bought tiles
(ADR-0052). The cursor on those tiles was left as it was: 1-pixel corner
marks just inside the tile, 3 pixels long (0433's placeholder; nothing has
decided it). The glyph look's marks sit in the gaps the font leaves around
letters, on a dark ground. Pictures have no such gaps and the ground is
bright: on grass the marks are hard to see at 1×. Claude saw it in 0437's
frames; **Nick hasn't said it bothers him.**

(This ticket first also asked how strong range tints should be on picture
tiles. Nick was shown 75%, 50% and 35% and answered on 2026-10-04: "75%
looks ok". The tint stays as it is; `look-and-feel.md`, *Colour*.)

Code: `crates/ui/src/map_view/sprite.rs` (`paint_own_cursor`,
`corner_arms`); the glow style tints the tile in
`crates/ui/src/map_view/sprite/ground.rs` (`tints`).

**Nick has now said it (Chapter 1 playtest, ticket 0804, 2026-10-12):**
"the cursor is also a bit hard to see, I think there's an open ticket for
this." So the first question below is answered: **he wants it easier to
see.** Go straight to rendering options. He also found the player's blue
outline hard to see (ticket 0447); the two can be shown to him in the same
frames, but each is its own ticket and PR.

## Nick input

**Sign-off, with a question first** (answered 2026-10-12, see above: skip
to the options). Ask Nick whether the cursor is easy
enough to see on the bought tiles (he will have played the Quick Battle on
Pages after 0437 merged).

- If he says it is fine: change nothing, write that in the Completion
  notes and in `look-and-feel.md`, and move this ticket to `done/`.
- If he wants it easier to see: render options first (`ask-nick`, with
  frames from `cargo xtask frame-png` built with
  `--features private-assets`, sent to him and never committed): for
  example thicker marks, marks with a dark edge, or a frame round the
  tile. Then build what he picks.

## Scope

**In:**
- The cursor on tiles painted as pictures, as Nick picks, in all three
  `CursorStyle`s.
- `docs/design/look-and-feel.md` records his answer.

**Out (do not do):**
- The glyph look's cursor, or the mixed skin's (sprite units on glyph
  terrain: it uses the glyph look's).
- Range tints: decided, 75% in both looks.
- Lighting (0438) and zoom (0439).
- A player setting beyond the three cursor styles that exist.

## Implementation steps

1. Ask Nick (above). Record his answer in `docs/design/look-and-feel.md`
   (*Cursor and selection*), and remove the item from *Open
   sub-questions*. If nothing is to change, stop here.
2. `sprite.rs`: change `paint_own_cursor` / `corner_arms` to the look he
   picked. Keep the cursor inside its tile, or cut it at the map area
   (`sprite_skin_paints_only_the_area` must still pass).
3. Update `the_cursor_marks_the_corners_inside_its_tile_or_glows`
   (`sprite.rs`) and review the changed sprite-skin snapshots.
4. Render the Quick Battle (`frame-png`, private assets), look at it, and
   send it to Nick.

## Acceptance criteria

- [x] `look-and-feel.md` records Nick's answer, with the date.
- [x] If he wanted a change: the cursor on picture tiles is the look he
      picked, in every `CursorStyle`, with a test naming its rectangles.
- [x] Every glyph-skin snapshot is unchanged.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the cursor's rectangles for each style.
- Property: the existing `…_paints_only_the_area` properties still pass.
- Snapshot / integration: the sprite-skin snapshots, reviewed.

## Completion notes

**Nick's answer** (2026-10-13), shown four looks cut from Quick Battle
frames with the bought tiles (thick marks; thin marks with a dark edge;
thick marks with a dark edge; a frame round the tile), on a unit, grass,
forest, road, water and the fort, at both zooms:

> "can we use C style for 1x and B style for 2x? and for tile glow imo we
> deprecate it for the 2D style it's simply unsustainable to keep it
> around for this style but for glyph we might consider keeping it"

**Done:**

- Picture tiles, map at 1× zoom: corner marks 2 px thick and 4 px long
  with a 1 px near-black edge. At 2× zoom: marks 1 px of the art thick and
  3 long with an edge of 1 px of the art. All inside the tile. The marks
  pulse; the edge doesn't. Large corners are a pixel longer.
  (`sprite.rs`: `paint_own_cursor`, `CursorMarks`, `corner_marks`.)
- Tile glow is no longer drawn on picture tiles: the ground's tints no
  longer take the cursor (`sprite/ground.rs`, `tints`).
- `look-and-feel.md`, *Cursor and selection*, records it.
- The glyph look and the mixed skin (sprite units on glyph terrain) are
  untouched: no glyph-skin snapshot changed. Four sprite-skin snapshots
  changed, in the cursor's rectangles only.
- Looked at with `frame-png` and the private assets (Quick Battle, both
  zooms; not committed).

**Deviations:**

- `at_twice_the_zoom_everything_is_twice_the_size` no longer has a
  cursor: its marks now have a size for each zoom, which
  `the_cursor_is_dark_edged_corner_marks_inside_its_tile` names.
- ADR-0052 (section 4) still lists the cursor's glow among the tints over
  a picture. It is accepted, so not edited; the rule that changed is a
  look, recorded in `look-and-feel.md`.

**Claude's starting rules** (Nick to agree or veto):

- With the map as pictures, a player who has picked `Tile glow` sees the
  normal corner marks.
- The Options menu still offers `Tile glow` there. What the menu should do
  is ticket **0451**.

**Follow-up:** 0451 (the `Tile glow` choice in Options when the map is
pictures).
