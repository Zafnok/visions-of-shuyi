---
id: "0250"
title: "Key names stand out from the words around them in every hint"
type: feature
milestone: M1 Engine
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: decision
completed: 2026-11-16
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

- [x] Nick picked; recorded in `look-and-feel.md`.
- [ ] Every key name on screen is drawn with the key style (a test lists
      each screen's hint spans and checks their style). **Moved to 0254**:
      it needs engine#2347 and engine#3215 first (completion notes).
- [x] `cargo xtask check-keys` passes (no code changed).
- [x] All gates in the `run-gates` skill pass (docs and tickets only:
      the ticket lint, typos and the doc checks).

## Tests required

- Unit: the keymap's name function returns key spans.
- Snapshot: help bar, dialogue corner, a tip; each reviewed.

## Completion notes

Done 2026-11-16: the decision, and the build split off.

**The decision** (Nick, five rounds of mockups on real Chapter 1 frames,
both looks, only the key names repainted with the real font and palette;
the mockups stay local, the picture look shows bought art): key caps.
Recorded in `docs/design/look-and-feel.md`, *Key names in hints*, with
his words. In short: a small rounded box like a keyboard key, the name
in bold, single letters in capitals, longer names as written (`Space`,
`Shift+Space`), 2 px between the letters and the box on all four sides,
a `.` or `,` after a cap drawn whole; the arrow keys as one picture of
the arrow cluster (an upside-down T of four 8 × 8 px caps); a cursor on
other keys as four caps. Rounds: brackets, angle brackets, a colour, a
key cap, bold and brackets + colour, lowercase and capitals → bold +
key cap, both cases → even margins, longer names as written or in
capitals → the arrow cluster at 8 and 10 px a key, a rebound cursor as a
small lettered T, four caps or one wide cap.

**Why the build is not here:** the engine fills key names into text
(`Ctx::text_with`, `help_keys`) and draws every text as plain cells; it
has no styled spans, no bold and no pixel box around a word. Engine
ticket 2347 (already open) gives key names a role and colours; it lists
bold, boxes and pictures as Out. With Nick's OK (2026-11-16) this
ticket wrote engine#3215 (Zafnok/shuyi-engine#450, the ticket alone)
for the key-cap style, the bold face and the arrow cluster, and the
game's half is 0254, blocked by both. Per `engine-change` §2 (more than
one PR's worth: a ticket and a follow-up).

**Follow-up tickets:** 0254 (the game turns the key caps on; added to
1064's `blocked_by` and to the roadmap in 0250's place), 0255
(controller buttons: coloured buttons or the platform's icons, Nick's
decision), 0256 (the Playdate's one-bit hints, Nick's decision). Both are milestone *Later* and block nothing (Nick: "I only want
the web/PC keycaps to block anything substantial, all other systems
designs and implementations can wait for much later"). 0463's
key hint now points at 0254.

*Claude's starting rules (in the design doc, Nick can veto):* an action
with no key reads `! not mapped` in plain text, no cap; on the Key
bindings screen the caps are the slot columns' key names, and a slot
being captured keeps its prompt as plain text.

Nothing for the Act 1 playtest from this ticket: 0254 carries the
look.
