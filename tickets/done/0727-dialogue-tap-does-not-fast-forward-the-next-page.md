---
id: "0727"
title: "Dialogue: a Confirm tap doesn't fast-forward the next page"
type: bug
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: sign-off
completed: 2026-10-12
---

# 0727 — A Confirm tap doesn't fast-forward the next page

## Context

Nick, playing Chapter 1 (ticket 0804, 2026-10-12):

> "the hold F to speed up text is too eager, I press F to advance to next
> dialogue and already half the next text is displayed bc it interprets a
> tap as a hold too quickly"

`crates/ui/src/screens/dialogue.rs`: a Confirm press on a fully shown page
moves to the next page. On the very next frames the key is still down, so
`input.is_held(a)` for an `ADVANCE_KEYS` action is true and the new page
reveals at `FAST_FORWARD` (×6) until the finger lifts. A normal tap lasts
about 0.1 s, which at ×6 is a good part of a line.

Rule (`docs/design/controls.md`): Confirm, held, fast-forwards text. That
stays. What changes is when a press starts to count as a hold.

## Nick input

**Sign-off:** after merge, tap through the Chapter 1 opening on Pages and
say whether a tap still eats text, and whether holding still feels quick
enough to start.

## Scope

**In:**
- The dialogue screen's hold-to-fast-forward.

**Out (do not do):**
- The fight playback's hold (×4, `controls.md`, *Fight playback*) and other
  animations: Nick didn't report them. If the same thing shows there while
  testing, write a ticket.
- Text speed settings (`options.md`).
- A new key or action.

## Implementation steps

1. In `dialogue.rs`, fast-forward only when an advance key has been held
   for at least `FAST_FORWARD_DELAY` seconds **on the page being revealed**
   *(the agent's starting rule, tunable: 0.30 s)*. Keep a `held_s: f32` on the
   screen: add `input.dt` each frame an `ADVANCE_KEYS` action is held, set
   it to 0 when none is held **and whenever a new page starts** (the press
   that advanced, or a choice being confirmed). So the tap that turns the
   page never speeds up the next one, and a player who keeps the key down
   through page after page gets the short delay on each page.
2. A press while the page is still revealing shows the whole page at once,
   as today (unchanged).
3. No key is named anywhere (`keyboard-input` skill): only `Action`s.

## Acceptance criteria

- [x] Test: Confirm pressed on a shown page and held for 0.1 s more: the
      next page has revealed exactly what it would with no key held.
- [x] Test: Confirm held for longer than the delay: the page reveals at
      `FAST_FORWARD` from then on (`holding_confirm_fast_forwards` still
      passes, adjusted for the delay).
- [x] Test: the delay starts again on each new page.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration (`crates/ui/src/screens/dialogue/tests.rs`): the three tests
  above, reading how many characters are shown (state, not cells).

## Completion notes

- `DialogueScreen` counts how long an advance key (Confirm or End turn)
  has been held on the page (`held_s`), from zero at each new page and
  whenever the key is let go. The text goes at `FAST_FORWARD` only past
  `FAST_FORWARD_DELAY` (0.3 s); a frame that crosses the delay is fast only
  for the part past it, so the result doesn't depend on the frame rate.
- A press while the page is revealing still shows it all at once.
- Tests: `screens/dialogue/tests.rs` (tap, hold, delay per page, letting
  go) and `tests/it/dialogue.rs` (`a_tap_does_not_fast_forward`; the two
  hold tests now hold for 0.4 s on a long line).
- The `mid_reveal_snapshot` picture changed: it used to show the bug (the
  tap that turned the page had eaten five extra letters).
- Rule recorded in `docs/design/controls.md`, *Dialogue: a tap turns the
  page, a hold speeds up*.
- No deviations, no follow-up tickets. The fight playback's hold was not
  touched or tested here.

**the agent's starting rule** (Nick can veto): holding Confirm speeds the text
up only after 0.3 s on that page. Keep it held through several pages and
each page starts at the normal speed, then goes fast a moment later.

**For Nick (sign-off):** tap through the Chapter 1 opening. A tap should no
longer eat text. Then hold the key: does the speed-up start soon enough?
