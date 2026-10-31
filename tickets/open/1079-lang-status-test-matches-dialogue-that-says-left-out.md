---
id: "1079"
title: "lang-status test fails on any dialogue line that says \"left out\""
type: bug
milestone: Post–Chapter 1
model: implementation
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1079 — lang-status test fails on any dialogue line that says "left out"

## Context

`crates/xtask/src/game_tools/tests.rs`,
`the_test_pack_keeps_the_test_text_and_lists_a_files_lines`, checks
that a `lang-status` report limited to dialogue leaves no test text out
with `assert!(!dialogue.contains("left out"))`. The report prints every
missing line's English text, so the check also fails on any dialogue
line that happens to contain the words "left out". Ticket 0745 hit it
with a Chapter 3 line ("He just left out the rest.") and reworded the
line to get past the gate (a test is never weakened to pass, so the
line changed, not the test). The next writer who uses the phrase will
hit it again.

The report's own marker is the summary's "; N test texts left out"
(the engine's `crates/cli/src/lang_status.rs`, where the first line of
the report is written).

## Nick input

None.

## Scope

**In:**
- Make the test look at the report's first line (the summary), or at
  the exact marker "test texts left out", instead of the whole report.
- Check the other `lang-status` tests in the same file for the same
  substring trap.

**Out (do not do):**
- Changing what `lang-status` reports (the engine's).
- Rewording any script line.

## Implementation steps

1. Read the test and the report format in the engine's `lang_status.rs`
   at the pin.
2. Narrow the assertion; run `cargo test -p xtask`.

## Acceptance criteria

- [ ] A dialogue line containing "left out" no longer fails the test
      (add one to the test fixture, or assert on the summary line only).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The changed unit test itself.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
