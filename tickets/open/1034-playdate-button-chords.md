---
id: "1034"
title: "Playdate: button chords (A + B ends the turn, hold B + left/right picks the previous/next unit)"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: ["1033"]
nick_input: none
completed:
---

# 1034 — Playdate button chords

## Context

The Playdate has only the D-pad, A, B, the crank and the system Menu
button. Nick chose "Menu + chords" (`docs/design/playdate.md`, *Buttons
and the crank*, Q4 B):

| Control | Does |
| ------- | ---- |
| A + B together | End turn |
| Hold B + left / right | Previous / next ready unit |

Everything else on that table is either already how the game works (B
with nothing to cancel opens the map menu, A on an empty tile opens it)
or another ticket (crank and menus: 1035). 1033 feeds the buttons in as
a controller of `PadKind::Playdate` with the default `pad` table; pad
bindings today are single buttons (`PadBindings`,
`crates/ui/src/input/bindings.rs`), while keyboard bindings already have
chords (`Chord`).

## Nick input

None. Nick decided the chords in 0061.

## Scope

**In:**
- **Button chords** in the pad bindings: a binding may be one button or
  two (e.g. `South + East` for End turn; `East + DpadLeft` for Previous
  unit). Saved and loaded with the player's bindings (ADR-0053) without
  breaking saved bindings that have none.
- **The Playdate's defaults:** a `playdate` table in
  `assets/data/keymap.ron` (beside `pad`), used for `PadKind::Playdate`
  pads: D-pad = cursor, A = Confirm, B = Cancel, A + B = End turn, hold
  B + left = Previous unit, hold B + right = Next unit. A test pins it
  against `docs/design/playdate.md`. The PC's `pad` table is unchanged.
- **How a chord fires** (technical, record in the crate README and a doc
  comment; tunable values in `keymap.ron` next to `repeat`):
  - A + B: the two pressed within a short window (start at 80 ms) fire
    End turn and neither A's nor B's own action. To allow that, A's and
    B's single actions wait for the window or fire on release, whichever
    comes first, **only on buttons that start a chord** (others keep
    firing on press).
  - Hold B + left/right: a D-pad press while B is held fires the chord's
    action and cancels B's own action for that hold; B released with no
    D-pad press fires Cancel as usual. Left/right held repeat as the
    cursor does.
- **Help text names chords** through the keymap: `key_name` for an action
  bound to a chord shows both buttons (e.g. "A+B"), using
  `PadKind::Playdate`'s names (`keyboard-input` skill). The words joining
  them come from the language file.
- `check-keys` passes: no button named outside the allowed files.

**Out (do not do):**
- The crank, the system menu, map menu entries (1035).
- The Key bindings screen's Playdate look (1045); this ticket makes
  chords bindable in `PadBindings`' pure API only (bind, clear,
  defaults, `unmapped_required`), with tests.
- Any PC change: pads of other kinds keep single-button bindings and
  fire on press exactly as today.

**Carve (1106, 1107):** either order works. If input moved to the engine
first, add chords to the engine's pad bindings there (an engine feature
any game can use) and keep the `playdate` table in the game's
`keymap.ron`.

## Implementation steps

1. Read the `keyboard-input` skill, ADR-0034, ADR-0053,
   `crates/ui/src/input.rs`, `input/bindings.rs`, `input/pad.rs`,
   `crates/content/src/keymap.rs`.
2. Chord type for buttons in `content` (keymap file format) and `ui`
   (bindings, `InputState`).
3. The `playdate` table and its test.
4. The firing rules above in `InputState`, with unit and property tests
   (e.g. any order and timing of A and B within the window gives exactly
   one End turn and no Confirm/Cancel; B tapped alone always gives one
   Cancel).
5. Help-text names for chords.
6. Simulator check: end a turn with A + B and cycle units with B +
   left/right in Chapter 1's first battle.

## Acceptance criteria

- [ ] A + B ends the turn (its question shows, as on the PC) and hold B +
      left/right cycles ready units, in the Simulator.
- [ ] B tapped alone cancels; A tapped alone confirms.
- [ ] Unit and property tests for the firing rules pass.
- [ ] PC pad and keyboard behaviour unchanged (existing input tests pass
      unchanged).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit + property: chord firing, the default table, bindings save/load
  with and without chords.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
