---
id: "1191"
title: "The engine-pin issue lists each Breaking: line once"
type: bug
milestone: "Engine: carve"
model: implementation
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-10-25
---

# 1191 — The engine-pin issue lists each Breaking: line once

## Context

The engine watcher (1190, ADR-0096) filed its first `engine-pin` issue,
Zafnok/visions-of-shuyi#423, on 2026-10-25. Every row of its *Breaking*
table shows its text twice. For example, engine PR #88's cell is the
list for `shuyi-app`, `shuyi-ui`, `shuyi-assets` and `shuyi-cli`, then
the same list again.

The cause is in `breaking_lines` (`crates/engine-watch/src/lib.rs`). It
reads the `Breaking:` lines of the PR body and of the engine ticket's
Completion notes, and the engine writes the same lines in both: its
`work-ticket` copies the ticket's notes into the PR body. Nothing removes
the repeats. The long cells (engine 2024's is about 900 characters
before doubling) become hard to read in the review the `engine-pin`
skill asks for.

## Nick input

None.

## Scope

**In:**
- `breaking_lines` keeps each line once, in the order first seen (body
  first, then the notes).
- A unit test.

**Out (do not do):**
- Other changes to the report or the issue. The next sync of #423 picks
  up the fix by itself.

## Implementation steps

1. In `breaking_lines`, drop a line already collected (compare the
   trimmed text after `Breaking:`).
2. In `crates/engine-watch/src/tests.rs`, a test with the same line in
   the body and the notes, plus one line only in the notes, expects each
   once, body order first.

## Acceptance criteria

- [x] `breaking_lines_are_listed_once_across_the_body_and_the_notes`
      passes; the existing `breaking_lines_*` tests still pass.
- [x] `cargo run -p trpg-engine-watch --locked` against today's gap
      prints engine PR #88's *Breaking* cell once.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the test in step 2.

## Completion notes

`breaking_lines` now keeps each `Breaking:` line once, in the order
first seen (body, then the ticket's notes), with a comment saying why
the same line is usually in both. New test:
`breaking_lines_are_listed_once_across_the_body_and_the_notes`. Run
against the day's gap (pin `3f2256d4`, engine `main` `b4591a81`), engine
PR #88's *Breaking* cell lists each crate once (591 characters).
Issue #423 picks this up at the first engine merge after this PR
merges. The hourly run doesn't redraw it while the engine's `main` is
unchanged.

Gates: fmt, clippy, the workspace tests, `engine-pin --check`, the docs
of `trpg-engine-watch`, `cargo deny`, `cargo machete`, `typos` and
`ticket-lint`. The WASM and Playdate builds were skipped: no crate they
build depends on `trpg-engine-watch`.

Engine friction: estimate 5, measured 6 (1.2×), under 2×.

Nothing for Nick; no starting rules; no follow-ups.
