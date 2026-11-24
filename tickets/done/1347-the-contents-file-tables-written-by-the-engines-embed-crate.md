---
id: "1347"
title: "The content's file tables written with the engine's embed crate, not a walk of the game's own"
type: infra
milestone: M1 Engine
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-23
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

- [x] Every table lists the same paths as before (the test).
- [x] Two paths differing only by case in a layer fail with the engine's
      error.
- [x] Lines removed, measured, in the completion notes.
- [x] The Playdate's three `no_std` builds and the web build pass.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the build script's walk on a fixture folder.
- Integration: the tables' paths test above.

## Completion notes

- **What was done.** `build.rs` walks every layer with `shuyi_embed::walk`
  (through `src/layer_files.rs`, which `build.rs` and the test include by
  path), skipping `**/README.md` and the content's `tests/**` as engine
  patterns, and writes every table with `shuyi_embed::table_source` and
  the Playdate's left-out list with `paths_source`. Its own walk, the
  sort, the hand-written `include_bytes!` lines and the `Source` pair are
  gone. Each entry now names its file by an absolute path, as the
  engine's table source writes it, in place of `concat!(env!(…), …)`.
  `shuyi-embed` is a build-dependency (and a dev-dependency, for the
  test), declared at the pin like the other engine crates, and added to
  `cargo xtask engine`'s crate list so the local checkout patches it too.
- **Stays the game's:** the one-bit pictures, the demo's cut copies, the
  editions' cfgs and lists, the `rerun-if-changed` lines (the folder
  lines cargo already scans whole), and the `playdate-bundle.txt` rule
  matcher: its rules are read across `assets/` and the content together
  (a rule matching nothing in either is an error), which isn't one
  folder's skip, and switching it to the engine's glob would change the
  list's format, which is out of scope.
- **Same tables, checked.** Before the change every table's paths and the
  file each entry reads were dumped in three builds (default, with
  `private-assets`, and the `story-trial` edition with `private-assets`,
  which covers the editions' lists and the demo), with the content
  checked out; after it, all 33 were identical, path for path and source
  for source. **Deviation:** that expected output isn't committed: it
  lists the private content's file names (ADR-0118) and would break at
  every new file. The committed test instead checks today's rules on the
  tables a test build has (`bundle.rs`, `every_table_is_sorted_once_…`:
  sorted, each path once, no `README.md`, no case pair, and the
  Playdate's table is `assets/` without its list, in order), beside the
  existing test that each layer is its folder byte for byte.
- **Case check.** `tests/it/layer_files.rs` makes a fixture with
  `portraits/Hero.png` and `portraits/hero.png` and expects the engine's
  `SameButCase` naming both. Windows and macOS disks can't hold the pair,
  so there the test has nothing to refuse and returns; it runs on CI's
  Linux, like the engine's own test.
- **Lines** (`cargo xtask code-lines`): 66 added, 75 removed, so 9 net
  removed (`build.rs` 591 → 572 lines, plus the 21-line
  `layer_files.rs`), against the ticket's "about 150". Under 50 is the
  ticket's trigger: the `engine-friction` review found the estimate was
  wrong, not the engine (outcome 4, row in `docs/engine/friction.md`).
  The walk was 22 lines and the writers about 20; the rest of the script
  is the game's own conversions.
- **Follow-ups:** none.
