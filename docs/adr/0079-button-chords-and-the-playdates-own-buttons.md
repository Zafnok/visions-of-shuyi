# ADR-0079: Button chords, and the Playdate's own buttons

- **Status:** Accepted
- **Date:** 2026-10-17
- **Related tickets:** 1034 (1033, 1035, 1045)
- **Extends:** ADR-0034 (controller input) and ADR-0053 (the player's
  controller buttons): a pad binding may now be two buttons together. The
  saved format keeps version 2.

## Context

The Playdate has the D-pad, A, B, the crank and the system Menu button.
Nick chose "Menu + chords" (`docs/design/playdate.md`, *Buttons and the
crank*): A + B together ends the turn, hold B + left / right picks the
previous / next ready unit. Pad bindings were single buttons, and every
button acted the moment it went down. A chord needs a button to hold back
its own action until it knows whether its partner is coming, and the PC's
pads must not change at all.

## Decision

1. **`ButtonChord`** (`shuyi_assets::keymap`): a pad binding is one
   `Button` or two, written `"South+East"` in `keymap.ron` and the saved
   bindings. The two are kept in `Button::ALL` order, so `"East+South"` is
   the same chord. A chord and its buttons on their own are different
   bindings. `PadKeys`, `PadBindings`' slots, `Keymap::buttons_for` and
   `primary_button` hold `ButtonChord`s; `bind` and `with_pad` take
   anything `Into<ButtonChord>` (a `Button`). Saved bindings without chords
   read as before.
2. **A `playdate` table** beside `pad` in `keymap.ron` (optional in the
   format; a file without one uses `pad`), validated like `pad`.
   `KeymapDef::for_playdate()` makes it the default buttons, and the
   Playdate front end calls `Ctx::use_playdate_buttons()` before the
   player's bindings load. The PC never sees it.
3. **How a chord fires** (`shuyi_ui::input::state`, its module docs and
   `Keymap::chord_wait`). Which buttons hold back follows from the
   bindings, so the file needs no second syntax:
   - A button in no chord, or whose own action repeats (the D-pad on the
     cursor), acts on press, as before. So the cursor never lags.
   - A button with a partner that acts on press (B, partner of ← and →)
     holds back until it is let go: the partner may come any time while it
     is held.
   - Any other chord button (A, whose only partner B also holds back)
     holds back for the chord window (`chord: (window_ms: 80)` in
     `keymap.ron`) or until let go, whichever is first.
   - A chord fires when one of its buttons goes down while the other is
     down and still holding back: the chord's actions instead of either
     button's own. A button a chord used gives nothing when let go, but
     stays a partner while held (B held, ← ← → gives three picks).
   - A chord pressed with a button that repeats repeats as that button
     would (B held + ← held cycles units like the cursor).
   - The window counts from the first update after the press, so a long
     frame on the device never eats it.
4. **Holding back changes two `Game` details.** A pad action can now come
   in a frame with no button going down, so `FrameInput::pad_pressed`
   means "this frame's input came from a pad". And the press that ends the
   title's wait is taken (`InputState::pad_taken`), so it gives nothing
   when let go either.
5. **Help text** names a chord as both buttons with the language's
   `keys.chord_join` between them (`A+B`, `B+←`), through
   `PadKind::chord_name`, the one place buttons get names.

## Consequences

- On the Playdate, A's Confirm comes up to 80 ms later (or on release),
  and B's Cancel on release; held Confirm (fast-forward) works once the
  window has passed. Pads without chords behave exactly as before.
- The Key bindings screen can show chords in slots; capturing one on the
  Playdate is 1045's job. `PadBindings`' pure API binds them already.
- The rule is a property of the bindings: a player who binds a chord on a
  PC pad gets the same rules there.

## Alternatives considered

- **Two chord syntaxes** (`"A+B"` together, `"hold B, ←"`): explicit, but
  a second notation in data and saved files for a difference the
  bindings already show.
- **Order in the file decides** (first button held, second pressed):
  makes `A+B` either delay Confirm until release (breaking hold-to-fast-
  forward) or lose "either order"; and the file order would matter in a
  way nothing shows.
- **A delay on every button of a chord pad:** makes the cursor lag 80 ms
  on the Playdate.
