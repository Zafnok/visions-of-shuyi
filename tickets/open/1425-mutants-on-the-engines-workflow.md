---
id: "1425"
title: "The diff-mutation workflow is the engine's: mutants.yml becomes a call to the engine's, with the game's gate, save and cache as inputs"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#3547"]
nick_input: none
completed:
---

# 1425 — The diff-mutation workflow is the engine's

## Context

The engine's redundancy sweep of 2026-11-28 found the mutation-testing
pipeline written twice:

- the game's `.github/workflows/mutants.yml` (500 lines);
- the engine's (325 lines).

The two share the job graph: `changes`, `plan`, `baseline`, `pr`,
`mutants-result`, `weekly-baseline` and `weekly`. 104 non-comment lines
match verbatim. The game adds `gate` and `save` jobs and an outcome
cache.

Engine#3547 makes the engine's workflow one a game calls, as its
template's `playtest.yml` already calls
`Zafnok/shuyi-engine/.github/workflows/playtest.yml@<commit>`, with the
game's extra jobs as inputs.

**Expected size:** about 450 workflow lines removed and about 60 added.

## Nick input

None.

## Scope

**In:**
- `mutants.yml` becomes a short caller of the engine's workflow, pinned
  to the game's engine commit (`shuyi-engine.rev`), with the game's
  inputs: packages and features, system packages, the proptest seed, the
  outcome cache, the gate command and save.
- The engine-pin bump (the `engine-pin` lane) moves the caller's commit
  with the pin. Add the file to what the pin check compares.
- **Either order with 0134 and 0155:** if one is done, carry its change
  into the inputs or ask the engine for it (the `engine-change` skill).
  If not, it changes the engine's workflow through an engine ticket
  instead of the game's file, and its text says so.

**Out (do not do):**
- Changing what is mutated, the shard rule or the timeout (ADR-0100).

## Implementation steps

1. Read engine#3547's completion notes for the inputs.
2. Write the caller, and run it on this PR: the same jobs as the last
   run on `main`, and the same mutants found.
3. Add the caller's commit to the pin check and to the `engine-pin`
   skill's steps.

## Acceptance criteria

- [ ] The PR's mutation run finds the same mutants as the last run on
      `main` for the same diff (or the notes explain each difference).
- [ ] The caller's commit equals `shuyi-engine.rev`, and the pin check
      fails when they differ.
- [ ] zizmor passes on the caller.

## Tests required

- The PR's own mutation run is the integration test.

## Completion notes

*(Filled in by the session that completes the ticket.)*
