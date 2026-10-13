---
id: "0451"
title: "Options: the Tile glow cursor choice when the map is pictures"
type: tuning
milestone: M3 Battle UI
model: sonnet-5
effort: low
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0451 — The Tile glow choice when the map is pictures

## Context

Ticket 0444 (2026-10-13): Nick dropped the Tile glow cursor on picture
tiles. "For tile glow imo we deprecate it for the 2D style it's simply
unsustainable to keep it around for this style but for glyph we might
consider keeping it."

So the picture skin draws corner marks whatever the cursor style
(`crates/ui/src/map_view/sprite.rs`, `CursorMarks::of`). The Options menu
was left alone: its `Cursor` row still steps through `Corners`,
`Large corners` and `Tile glow` (`crates/ui/src/screens/options.rs`,
`CURSOR_STYLES`), and with the map look set to pictures `Tile glow` looks
the same as `Corners`. That is *Claude's starting rule*, written in
`docs/design/look-and-feel.md` (*Cursor and selection*); nothing has
decided what the menu should do.

The map look is a setting too (`Map look`, ticket 0824), so a player can
have picked `Tile glow` on glyphs and then switch to pictures.

## Nick input

**Decision** (`ask-nick`). What the `Cursor` row does while the map look
is pictures. Offer at least:

- **A. Leave it:** three choices, `Tile glow` draws corner marks on
  pictures (today).
- **B. Two choices on pictures:** `Tile glow` is not offered; a player who
  had it sees `Corners` until they go back to glyphs, where their glow
  returns.
- **C. Drop the glow everywhere:** two choices in both looks (he said the
  glyph look "might" keep it).
- **D.** "Describe your own."

## Scope

**In:**
- The `Cursor` row of the Options screen, as Nick picks.
- `look-and-feel.md` records the answer.

**Out (do not do):**
- The cursor's look in either skin (0444, 0416).
- A new cursor style.

## Implementation steps

1. Ask Nick and record the answer in `look-and-feel.md` (*Cursor and
   selection*); remove the item from *Open sub-questions*.
2. If the row changes: `screens/options.rs` (the row's choices depend on
   `ctx.settings().map_look` and `has_pictures`, as the `Map look` row
   already does). A saved `glow` setting must still load.
3. If the glow goes everywhere: remove `CursorStyle::TileGlow`, the glyph
   skin's glow and its text keys, and map a saved `glow` to `corners`.

## Acceptance criteria

- [ ] `look-and-feel.md` records Nick's answer, with the date.
- [ ] The `Cursor` row offers what he picked in each map look, with a
      test.
- [ ] A settings file saved with `glow` still loads.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (`screens/options/tests.rs`): the row's choices in each map look.
- Integration (`crates/ui/tests/it/options.rs`): stepping the row.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
