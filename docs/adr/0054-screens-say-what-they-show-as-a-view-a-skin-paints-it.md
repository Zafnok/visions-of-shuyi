# ADR-0054: A screen says what it shows as a view; a skin paints it

- **Status:** Accepted; extends ADR-0038
- **Date:** 2026-10-04
- **Related tickets:** 0805, 0239, 0240, 0241

## Context

ADR-0038 made the battle map's graphics a skin: the battle screen builds a
`MapScene` (plain data) and a `MapSkin` paints it, so the glyph look could
be swapped for bought sprites without touching the battle's logic.

Menus and other screens never got that split. Each `Screen::draw` decides
what to show and how it looks in the same function: it reads the screen's
state, picks cells, box characters and colours, and prints. Nick (ticket
0805, 2026-10-04) wants the Options screen, and after it the other
screens, ready for the same swap: "if we want to swap out the lo-fi retro
look with a purchased pack later (as we did with sprite art) then it is
simple to swap it out without affecting logic".

## Decision

A screen is three parts, each in its own module:

1. **Logic** (`screens/<name>.rs`): the screen's state, what keys do, and
   `fn view(&self, ctx: &Ctx) -> <Name>View`. It names no cell, glyph,
   colour or position, and doesn't import `UiColor`, `Rect`, `BoxStyle` or
   `Cell`.
2. **View** (`screens/<name>/view.rs`): plain data saying everything the
   screen shows this frame, in the player's language and with their keys'
   names already filled in: titles, rows, values, which one is focused,
   messages, open questions, the help line. Values are meanings, not
   drawings: a volume is `Volume { level, max }`, never a string of block
   characters; "the cursor's keys change this" is a flag, never two
   arrows.
3. **Skin** (`screens/<name>/glyph.rs`): `pub fn paint(ctx, &view, buf)`.
   Everything about the look lives here: where things go, box characters,
   bars, highlights, colours. It decides nothing about what is shown.

`Screen::draw` is one line: `glyph::paint(ctx, &self.view(ctx), buf)`.

**Tests.** A test of what happened reads the view, the settings or the
state (in a Harness test: `h.game().screen::<NameScreen>()` and its
`view`, so the screen opts into `Screen::as_any`). Only the skin's own
tests and the snapshots read cells and colours.

**Swapping the look** is writing another `paint` for the same view (drawing
a pack's panels and widgets as sprite items, ADR-0038) and choosing between
them where `draw` calls the glyph one. No trait or registry until a second
skin exists: the function is the seam. The view may gain a field a new
skin needs; the logic fills it and the glyph skin ignores it.

The Options screen (0805) is the first, and the worked example:
`screens/options.rs`, `options/view.rs`, `options/glyph.rs`. The other
screens follow in tickets 0239 (title, key bindings, layout picker), 0240
(the other full-screen menus) and 0241 (the battle screen's panels and the
dialogue screen).

## Consequences

- A bought UI pack touches only `glyph.rs` files (or adds `sprite.rs`
  beside them); no game rule or key handling is at risk.
- Behaviour tests stop breaking when a label moves a cell.
- One more type per screen, built every frame. The views are a few dozen
  short strings: nothing to measure.
- Until 0239–0241 are done the screens are mixed: new screens follow this
  ADR; old ones are converted by those tickets, not piecemeal.
- Shared widgets (`widgets::Menu`) both hold state (the focus) and draw.
  Their state stays; their drawing becomes a skin function the screens'
  skins call (0239 does `Menu`).

## Alternatives considered

- **A `ScreenSkin` trait in `Ctx` now, like `MapSkin`:** a registry for
  one implementation. The map needed it because two skins exist and a
  debug tool switches them; menus have one. Add it with the second skin.
- **Theme the existing draw code** (swap box characters and colours
  through a table): covers a recolour, not a pack with picture panels,
  icons or different layouts, and leaves tests reading cells.
- **Retained widget tree** (a small UI toolkit): far more than the game's
  dozen fixed screens need, and it would couple every screen to the
  toolkit's layout rules.
