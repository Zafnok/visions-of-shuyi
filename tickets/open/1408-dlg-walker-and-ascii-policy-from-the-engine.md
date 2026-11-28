---
id: "1408"
title: "The .dlg checks on the engine's step walker and its ASCII text policy"
type: feature
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#2289"]
nick_input: none
completed:
---

# 1408 — The `.dlg` checks on the engine's walker and ASCII policy

## Context

The third engine-friction sweep (1397, `docs/engine/friction.md`) found
two pieces of engine-shaped code in `crates/content/src`:
- the same recursion over a scene's steps and their lines (zip `steps`
  with `lines.steps`, `blocks.next()`, recurse into replies and `@if`
  branches) in five files: `dialogue/deaths.rs`, `openings.rs`,
  `presence.rs`, `setting.rs`, `stand_in.rs` (plus a mutable copy over
  the text in `stand_in.rs`);
- ADR-0045's plain-ASCII rule, done by removing the engine's
  control-character errors by their wording in `dialogue.rs`
  `parse_dlg`, then a pass whose table copies the engine's
  `ascii_spelling` (`not_ascii`), which also needs its own
  `scripts_from_sources` and an `AsciiDlg` format (`formats.rs`).

Engine ticket 2289 adds `ParsedScene::walk`, `Scene::texts_mut` and a
`TextChars::Ascii` policy on `DialogueLimits`. Each item it replaces is
marked `// engine-friction: engine#2289, moves (game 1408 deletes it)`.

## Nick input

None.

## Scope

**In:**
- Move the pin past engine 2289 if it isn't (`engine-change` §7).
- The five checks on `walk`; `stand_in`'s text pass on `texts_mut`.
- The game's limits choose `TextChars::Ascii`; `not_ascii`, the
  `retain` on the message, `AsciiDlg` and the game's
  `scripts_from_sources` go, `Dlg<Stage>` is registered instead.
- ADR-0045 gets a line saying the engine enforces the rule now.

**Out (do not do):**
- What each check decides (deaths, openings, presence, setting, stand-in
  rules): it stays.
- Changing any script (dialogue writing is paused; only the check moves).

## Implementation steps

1. Estimate: about 200 lines removed, 30 added.
2. Move the pin if needed.
3. The policy first: the content tests' error messages may change
   wording (the engine's), which is allowed when the line and column
   stay; update their expected text.
4. The five walkers, one at a time, with the dialogue tests after each.

## Acceptance criteria

- [ ] `grep -n "engine#2289" crates/` finds nothing.
- [ ] A script with `’` still fails the content check at its line and
      column, suggesting `'` (a content test).
- [ ] The dialogue checks' tests pass with the same errors at the same
      lines.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The existing dialogue and content tests; one test of the ASCII rule
  through the engine's policy.

## Completion notes

*(Filled in by the session that completes the ticket.)*
