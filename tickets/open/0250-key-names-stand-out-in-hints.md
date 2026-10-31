---
id: "0250"
title: "Key names stand out from the words around them in every hint"
type: feature
milestone: M1 Engine
model: reasoning
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0250 — Key names stand out in every hint

## Context

Every screen names keys through the player's keymap (CLAUDE.md rule 7,
the `keyboard-input` skill): the battle's help bar ("f select · e info ·
s next unit…"), the dialogue box's corner ("b log   f ▼", 0729), tips and
lessons ("{Confirm}"), Options, Key bindings. Today a key's name is drawn
in the same style as the words beside it.

Chapter 1 playtest round 3 (0804, 2026-10-16), on the dialogue box's
"b log" hint:

> "generally looks fine, but I think all hotkeys need to be more easily
> discernable right now they blend in "b log" looks like some word... if
> it were "<B> log" or "[B] log" or bolded B log or something with a
> different color for the keybind it would be better"

So this is about every place a key is named, not one hint.

The help bar's layout is open ticket **0445** (either order works: if
0445 is done, keep its columns; if not, 0445 keeps this ticket's key
style). Controller button names in hints are 0220's (done or not, the
same style applies to button names).

**Waits for the engine carve and the Playdate port** (Nick, 2026-10-16),
so `blocked_by` has 1118 and 1050. After the carve, key-name text may sit
in engine code (1106/1107 move input to the engine): style it where the
game paints it, through a skin (ADR-0054), not in the engine's text
helper unless the engine already owns hints.

**Re-gated 2026-10-21 (0129):** the carve is done and 1050 was folded
into the Act 1 playtest (1064), so neither 1118 nor 1050 is in
`blocked_by` any more.

## Nick input

**Decision** (`ask-nick`) on real frames: the battle help bar, the
dialogue box corner, and a tip, each in every option. Options (his own
list plus references; check each game before citing):

- A. Brackets: "[B] log".
- B. Angle brackets: "<B> log".
- C. A key colour (the highlight colour, or one Nick picks), no brackets.
- D. A small key-cap box behind the letter, as many PC games draw keys
  (one glyph cell wide where it fits).
- E. Bold (the font has a bold face? check; if not, say so).
- F. Combinations (e.g. A + C), or describe your own.

Ask also: capital letters for keys ("B" rather than "b")? Nick asked for
uppercase labels before (0417).

## Scope

**In:** the question; one shared way every screen draws a key's name
(a text span with a "key" style that skins paint), used everywhere a key
or button is named; the keymap-name function gives keys as spans, not
plain text; `look-and-feel.md` records the answer; the
`keyboard-input` skill says to use it.

**Out (do not do):** changing which keys are bound; the help bar's
layout (0445).

## Implementation steps

1. List every place a key is named (grep the help-bar builders, the
   dialogue corner hint, tips/lessons `{Action}` expansion, Options, Key
   bindings).
2. Mockups; ask Nick.
3. Add the key style span; switch every place over; skins paint it.
4. Update `cargo xtask check-keys` if it needs to know the span.

## Acceptance criteria

- [ ] Nick picked; recorded in `look-and-feel.md`.
- [ ] Every key name on screen is drawn with the key style (a test lists
      each screen's hint spans and checks their style).
- [ ] `cargo xtask check-keys` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the keymap's name function returns key spans.
- Snapshot: help bar, dialogue corner, a tip; each reviewed.

## Completion notes

*(Filled in by the session that completes the ticket.)*
