# ADR-0091: The Playdate's screen skins: chosen by the skin set through `Skins`, the engine's screens painted by the game's, and the Options and Key bindings it has

- **Status:** Accepted
- **Date:** 2026-10-18
- **Related tickets:** 1045 (this), 1038, 1046 (the other Playdate screens)
- **Extends:** ADR-0054 (a screen's view and its skin), ADR-0083 (the
  skin set), ADR-0087 (the engine's screens take the game's rows),
  ADR-0079 (button chords)

## Context

ADR-0083 put a `SkinSet` in the context so the Playdate gets its own
skins, and said a screen's `draw` would read it "1038 onwards". Ticket
1045 is the first to do it, for New Game (mode, familiarity, lead with
its letter grid), Options, Key bindings and the Profile screen. Three
things were open:

- **How a `draw` chooses.** The ADR-0054 grep test wants `draw` to be one
  `glyph::paint(ctx, &self.view(ctx), buf);` line.
- **The engine's screens.** Options and Key bindings moved to the engine
  (1112, ADR-0087), whose `draw` calls the engine's glyph skin. The
  engine knows no `SkinSet`, which is this game's.
- **What the Playdate's Options and Key bindings have.** The Playdate has
  no keyboard, no window, a one-bit palette and one map skin; its pad has
  chords (ADR-0079) and the crank (ADR-0086).

## Decision

1. **`Skins<V>`** (`trpg_ui::screens`): a screen's two painters,
   `glyph` and `playdate`, and `paint(ctx, view, buf)`, which uses the one
   of `ctx.game.skin_set`. A screen with a Playdate skin has `pub const
   SKINS` beside its logic and its `draw` is `SKINS.paint(ctx,
   &self.view(ctx), buf);` (the grep test accepts it). The Playdate skin
   is `<screen>/playdate.rs`, beside `glyph.rs`.
2. **What the Playdate skins share** is `screens/playdate.rs`: the white
   ground, a window (white inside a double black frame, its title in the
   top of the frame and a line of its own in the bottom), the focus as a
   bar (white on black), grey dots for what can't be chosen (the
   palette's `window_text_off`), a list that scrolls with its focus with
   `▲`/`▼` in its frame, and the question and list windows. Inks come
   from the palette, as the map skin's do.
3. **The engine's screens take a painter**: `OptionsScreen::painted_by`
   and `KeyBindingsScreen::painted_by` (a `fn` of the context, the view and
   the buffer; the glyph skin until one is given). The game gives a `fn`
   that goes through its `SKINS`, so the choice is made when it draws.
4. **Key bindings with no keyboard** (`Keyboard::Absent`, the engine's)
   shows the controller side alone: no switch row, the view's `switch` is
   `None`, and leaving asks only for a button on every required action.
   The game lists `PLAYDATE_KEY_ROWS` then: its rows with the crank's two
   (Rewind back, Rewind forward) after Rewind, and without Zoom map, which
   the Playdate's one map skin can't do.
5. **Capturing a chord**: while a slot waits for a button, a second
   button that goes down while the first is held makes the slot the chord
   of the two (two together in one frame do too); it goes in when either
   is let go. Holding one button alone still backs out after
   `HOLD_TO_CANCEL_SECS`; a chord being held never does. A button of the
   capture still held when it ends does nothing when let go (the screen
   waits for it): the Playdate's B acts on release, since it waits for
   its chord partners, and would back out of the screen.
6. **What the Playdate's Options leave out**: `Layout` with no keyboard
   (`ctx.keyboard`), and, under the Playdate skin set, `PC_ONLY`:
   Fullscreen (no window), Colors (one bit has its own inks), Map look and
   Map zoom (one map skin, which can't zoom). This is the one place a
   screen's logic reads the skin set (ADR-0083 said none would): to leave
   out a setting the set's skins never use, never to decide a look. The
   PC keeps every row.

## Consequences

- 1038 and 1046 add a Playdate paint the same way: a `playdate.rs`, a
  `SKINS` const, the shared look from `screens/playdate.rs`.
- A second game using the engine's Options or Key bindings on its own
  device gives its painter with `painted_by`; the engine needs no skin
  set of its own.
- `cargo shuyi frame-png --playdate` starts as the device does (no
  keyboard, the Playdate's pad and buttons), so its pictures show the
  Playdate's rows and button names; steer it with `--pad`.
- Language rows (one build per language, 0061) and the portrait style
  setting (1042) don't exist yet; they come with their tickets.

## Alternatives considered

- **A skin trait in the context** (ADR-0054 waited for a second skin):
  a registry for two `fn`s a screen; `Skins` is the smallest thing that
  lets `draw` stay one line.
- **A wrapper screen in the game around the engine's**: `h.screen::<T>()`
  and every test that finds the screen by its type would stop finding it.
- **A `SkinSet` in the engine's context**: it is this game's notion of a
  device's looks; a `fn` painter is enough for the engine.
- **Hiding the rows by the console size or the palette**: implicit, and a
  Playdate-sized test console would lose the PC's rows (ADR-0083 refused
  the same for skins).
