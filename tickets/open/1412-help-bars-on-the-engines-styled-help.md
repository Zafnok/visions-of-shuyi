---
id: "1412"
title: "Help bars on the engine's styled help: the styled wrap, joined hints, HelpBar and frame widths go"
type: feature
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#3251"]
nick_input: none
completed:
---

# 1412 — Help bars on the engine's styled help

## Context

The third engine-friction sweep (1397, `docs/engine/friction.md`) found
that `crates/ui/src/widgets/help.rs` is 193 production lines for about
30 of the game's own: a 6.4× ratio.
- It wraps styled text keeping key caps whole and joins hints.
- It builds a help bar from the hints a screen's state allows.
- It shows key caps on a keyboard and plain text on a pad.
- Its `frame_widths` copies the engine's `pub(crate)`
  `KeyCaps::frame_widths`.

Engine ticket 3251 (`Zafnok/shuyi-engine`) adds all of it to
`shuyi_ui`. The marked items carry
`// engine-friction: engine#3251, moves (game 1412 deletes it)`.

## Nick input

None. No hint's text or look changes.

## Scope

**In:**
- Move the pin past engine 3251, if it isn't already (`engine-change`
  skill §7).
- These go to the engine's, and their callers use it directly or
  through a re-export from `widgets::help`:
  - `for_player`, `key_styled`, `join_hints`, `HelpBar`;
  - `styled_lines`, `wrap_styled`, `hint_column`, `pad_last_hint`;
  - `frame_widths` (now `Ctx::frame_widths`, 9 callers);
  - `key_spans` and `spans_of`.
- `text_with_keys` becomes a call of `Ctx::text_styled_with_spans`.
- What stays: the `HelpKeys` alias over the game's `Action`,
  `action_key` and `key_pair`.

**Out (do not do):**
- rewording any hint;
- the one-bit skins' help rows (engine#2371, game 1184).

## Implementation steps

1. Write the estimate: about 170 lines removed, 20 added.
2. Move the pin; replace the marked items; keep `widgets/help/tests.rs`
   green. Tests the engine now covers may go: say which in the
   completion notes.

## Acceptance criteria

- [ ] `grep -n "engine#3251" crates/ui/src` finds nothing.
- [ ] Every screen's help-bar snapshot is unchanged.
- [ ] The engine-friction line is in the PR body.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The existing help and snapshot tests, unchanged in what they show.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
