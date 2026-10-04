---
id: "0444"
title: "The cursor on picture tiles: easier to see, if Nick wants it"
type: tuning
milestone: M3 Battle UI
model: sonnet-5
effort: medium
status: todo
blocked_by: ["0437"]
nick_input: sign-off
completed:
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

## Nick input

**Sign-off, with a question first.** Ask Nick whether the cursor is easy
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

- [ ] `look-and-feel.md` records Nick's answer, with the date.
- [ ] If he wanted a change: the cursor on picture tiles is the look he
      picked, in every `CursorStyle`, with a test naming its rectangles.
- [ ] Every glyph-skin snapshot is unchanged.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the cursor's rectangles for each style.
- Property: the existing `…_paints_only_the_area` properties still pass.
- Snapshot / integration: the sprite-skin snapshots, reviewed.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
