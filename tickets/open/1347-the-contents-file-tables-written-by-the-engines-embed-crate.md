---
id: "1347"
title: "The content's file tables written with the engine's embed crate, not a walk of the game's own"
type: infra
milestone: M1 Engine
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1347 — The content's file tables written with the engine's embed crate

## Context

The engine pin move of 2026-11-23 passed engine PR #527 (engine ticket
2215): the `shuyi-embed` crate writes a game's file table at build time on
every target, `std` or not, and `include_dir` is gone from the engine. Its
build-script library (`shuyi_embed::walk`, `matches`, `table_source`,
`paths_source`, and the `Embed` builder's `dir`, `skip` and `write_to`)
walks a folder, skips by pattern, sorts, and fails the build for a file
name that isn't UTF-8 or two paths that differ only by case (which
`include_dir` embedded both of, one lost on Windows and macOS).

`crates/content/build.rs` (591 lines) does that job itself for every
layer (ADR-0070 §3, ADR-0082, ADR-0090, ADR-0105): it walks `assets/`,
`assets-test/`, the private folders and the content, leaves out the
`README.md`s and the lists of `playdate-bundle.txt`, `demo/bundle.txt`,
`story-preview/bundle.txt` and `story-trial/bundle.txt`, and writes each
table as `(path, include_bytes!(…))` lines. It has no check of names that
differ only by case.

## Nick input

None.

## Scope

**In:**
- The walk, the sort, the skip lists and the table and path-list source
  of `build.rs` on `shuyi_embed` (a build-dependency), with every table
  holding the same paths in the same order as today (a test compares).
- The case and UTF-8 checks the engine's walk makes, on every layer.
- What stays the game's: the one-bit conversion of the Playdate's
  pictures (`OUT_DIR/onebit*`, ADR-0085), the demo's cut copies
  (`OUT_DIR/demo/`), the editions' cfgs, and the `rerun-if-changed`
  lines, unless `shuyi_embed` gives them.
- Measure: production lines before and after (`cargo xtask code-lines`);
  expected about 150 lines removed. Under 50 is a finding for
  `engine-friction` (the engine's piece almost fits).

**Out (do not do):**
- Changing which files a build embeds, or the bundle's layers' order.
- `bundle.rs`'s reading of the tables (ADR-0105's `Layer::table`).

## Implementation steps

1. A test (in `crates/content/tests/it/`) listing each table's paths at
   today's `build.rs`, kept as the expected output.
2. Move the walk and the writing onto `shuyi_embed`; keep the game's
   conversions.
3. The test passes unchanged; a fixture folder with two names differing
   by case fails the build script's function (a unit test of it).

## Acceptance criteria

- [ ] Every table lists the same paths as before (the test).
- [ ] Two paths differing only by case in a layer fail with the engine's
      error.
- [ ] Lines removed, measured, in the completion notes.
- [ ] The Playdate's three `no_std` builds and the web build pass.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the build script's walk on a fixture folder.
- Integration: the tables' paths test above.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
