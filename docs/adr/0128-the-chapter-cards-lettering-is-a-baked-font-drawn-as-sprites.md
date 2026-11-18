# ADR-0128: The chapter card's lettering is a baked font drawn as sprites

- **Status:** Accepted
- **Date:** 2026-11-17
- **Related tickets:** 0812; engine 2224 and 2334
- **Amends:** ADR-0018's "one bitmap font" (`assets/fonts/README.md`):
  text stays in the cell font everywhere except the chapter title card.
  ADR-0038 §3 (menus, text boxes and help bars stay glyph cells) stands.

## Context

Nick picked the chapter title card's look in ticket 0812: the chapter's
name in Cinzel, between two rules (`docs/design/look-and-feel.md`, *The
chapter title card*). He said a title card may break from one font for
everything. Cinzel is a TrueType font (OFL-1.1) with letters of different
widths, so it can't be drawn as the console's 8×16 cells.

The engine reads only BDF bitmap fonts into one atlas of fixed cells.
Proportional fonts (TTF baked to atlases, text placed in pixels) are
engine tickets 2224 and 2334, after its MVP. The game never waits on the
engine (ADR-0092).

## Decision

- `cargo xtask lettering` rasterizes Cinzel Bold at 40 px and Cinzel
  Medium at 20 px with `fontdue` (a dev-tool dependency of `xtask` only;
  macroquad already depends on it). It writes two glyph sheets to
  `assets/title/` (`chapter_name` and `chapter_line`). Each sheet is a PNG
  of white glyphs whose alpha is the font's coverage, plus a
  `FontAtlasDef` (the engine's atlas file) beside it. The atlas gives each
  printable ASCII glyph's cell and, in its `ink`, the glyph's inked
  columns. A test fails if the committed files differ from what the tool
  makes.
- The sources are static instances of Cinzel's variable font, made once
  with fontTools' instancer and committed in `assets-src/fonts/`
  (`Cinzel-Bold.ttf` at weight 700, `Cinzel-Medium.ttf` at 500).
- `title.ron` names the sheets (`lettering`). `trpg_content::lettering`
  loads them with the engine's own checks plus one more: every glyph must
  have its ink.
- The card's skin lays a line out by each glyph's ink columns with a
  fixed gap, a space a quarter of a cell. Each glyph is a sprite from its
  sheet painted in the line's colour (`Paint::Solid`), so the theme still
  picks the colour (ADR-0038: a picture is a sprite item). A line with a
  character the sheet lacks (a language beyond ASCII) is printed in the
  cell font in its row.
- The Playdate's skin draws the card in the cell font.

## Consequences

- The game ships a second font: `THIRD_PARTY_ASSETS.md`, the credits and
  `assets/fonts/Cinzel-LICENSE.txt`.
- The ink layout ignores the font's own kerning and advances; the gap is
  tuned by eye against the mockups Nick chose.
- When engine 2224 and 2334 land, the card moves to the engine's
  proportional text and `lettering` is deleted (the friction log has a
  row for it).

## Alternatives considered

- **Bake each chapter's name into a picture.** That is less code, but
  every new chapter, renamed chapter and translated title would then need
  a new picture.
- **Make the engine change first** (2224 is a large ticket with its own
  ADR). Waiting on it would block the card for a single screen's need.
- **Big block letters from the cell font's half blocks** (shown to Nick).
  He wanted another font instead.
