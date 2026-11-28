---
id: "1424"
title: "UI tests stop re-testing the engine's Game loop, Ctx and Options screen: game.rs, screen.rs and options/tests.rs"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1424 — UI tests stop re-testing the engine's Game loop, Ctx and Options screen

## Context

This is a sibling of 1232, under the same rule, found by the engine's
redundancy sweep of 2026-11-28: **a test goes only when an engine test
checks the same case and it reaches no game code the kept tests don't.**
1232 names the map-skin copies, the key-binding, picker and profile
screens and the scene camera. It does not name these three files:

- `crates/ui/src/game.rs`: 1,083 lines, its tests from line 170. 18
  test names match the engine's `crates/ui/src/game/tests.rs` (some are
  helper functions). The exact-name matches are about 212 lines.
- `crates/ui/src/screen.rs`: 1,296 lines, its tests from line 504. 25
  names match the engine's `crates/ui/src/ctx/tests.rs`, about 242
  lines.
- `crates/ui/src/screens/options/tests.rs`: 725 lines. 6 tests repeat
  the engine's `crates/ui/src/screens/options/tests.rs`, about 160
  lines.

More near-name matches (pad events, the music fade, fullscreen,
`play_voice`, preload and a failed save) may also go.

**Expected size:** 0 production lines. About 600–700 test lines removed,
nothing added.

## Nick input

None.

## Scope

**In:**
- In each file, delete the tests that check the same case as an engine
  test and reach no game code.
- Keep every test that runs through the game's own keymap, palette,
  voice manifest, settings migration or data.
- Keep one smoke test per kind (the loop, the context, the options
  screen), so the mutation job finds no new survivor.

**Out (do not do):**
- 1232's files (either order: the two tickets touch different files).
- Changing a production line.

## Implementation steps

1. For each test in the three files, find the engine test of the same
   case at the pin. Use the names first, then read the bodies.
2. Delete those that pass the rule. List the deleted tests and their
   engine twins in the completion notes.
3. Run the ui crate's tests and the diff-mutation job's local check on
   the three files.

## Acceptance criteria

- [ ] Every deleted test is listed in the completion notes beside the
      engine test that checks the same case.
- [ ] No production line changes (`git diff --stat` touches test code
      only).
- [ ] The ui crate's tests pass, and the PR's mutation job reports no new
      survivor in `game.rs`, `screen.rs` or the options screen.

## Tests required

- None new. The kept tests and the engine's tests are the regression
  suite.

## Completion notes

*(Filled in by the session that completes the ticket.)*
