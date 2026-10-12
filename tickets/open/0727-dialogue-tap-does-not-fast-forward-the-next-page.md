---
id: "0727"
title: "Dialogue: a Confirm tap doesn't fast-forward the next page"
type: bug
milestone: M6 Story & dialogue
model: sonnet-5
effort: medium
status: todo
blocked_by: []
nick_input: sign-off
completed:
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
   *(Claude's starting rule, tunable: 0.30 s)*. Keep a `held_s: f32` on the
   screen: add `input.dt` each frame an `ADVANCE_KEYS` action is held, set
   it to 0 when none is held **and whenever a new page starts** (the press
   that advanced, or a choice being confirmed). So the tap that turns the
   page never speeds up the next one, and a player who keeps the key down
   through page after page gets the short delay on each page.
2. A press while the page is still revealing shows the whole page at once,
   as today (unchanged).
3. No key is named anywhere (`keyboard-input` skill): only `Action`s.

## Acceptance criteria

- [ ] Test: Confirm pressed on a shown page and held for 0.1 s more: the
      next page has revealed exactly what it would with no key held.
- [ ] Test: Confirm held for longer than the delay: the page reveals at
      `FAST_FORWARD` from then on (`holding_confirm_fast_forwards` still
      passes, adjusted for the delay).
- [ ] Test: the delay starts again on each new page.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration (`crates/ui/src/screens/dialogue/tests.rs`): the three tests
  above, reading how many characters are shown (state, not cells).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
