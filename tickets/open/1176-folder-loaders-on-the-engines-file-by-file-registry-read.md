---
id: "1176"
title: "The folder loaders on the engine's file-by-file registry read: the game's formats::load_in goes"
type: feature
milestone: "Engine: carve"
model: sonnet-5
effort: low
status: todo
blocked_by: ["engine#2219"]
nick_input: none
completed:
---

# 1176 — The folder loaders on the engine's file-by-file registry read

## Context

Ticket 1173 moved the game's loaders onto the engine's format registry
(`crates/content/src/formats.rs`). The maps, cinematics and dialogue
are folders of files, each checked after it reads. The engine's
`Formats::load_under` gives nothing when one file fails and reads the
folders below too, so 1173 wrote `formats::load_in`: each file of a
kind directly in one folder, read or with its errors. It is marked
`// engine-friction: engine#2219`. Engine ticket 2219 adds the same as
`Formats::load_each`.

## Nick input

None. Nothing a player sees changes.

## Scope

**In:**
- Move the pin past engine 2219 (`engine-change` skill, step 6).
- `map::load_all`, `cinematic::load_all` and `dialogue::load_scripts`
  call `formats::install().load_each::<Kind>(bundle::install(), DIR)`.
- Delete `formats::load_in` and its test
  (`load_in_keeps_the_files_that_read_when_one_fails`); the engine
  tests the same.

**Out (do not do):**
- Changing what any loader accepts or reports.
- The folder loops 1169 moves to `Bundle::load_each` (battles,
  battlers, chapters, openings, portraits, tilesets): they read their
  own formats, not the registry's. Either order works with 1169.

## Implementation steps

1. Write the estimate: about 15 lines removed, 3 changed.
2. Move the pin (`cargo xtask engine-pin`); build.
3. The three loaders, then delete `load_in` and its marker.

## Acceptance criteria

- [ ] `shuyi-engine.rev` is at or past the merge of engine 2219.
- [ ] `grep -rn "engine-friction: engine#2219" crates/` finds nothing.
- [ ] `cargo test -p trpg-content` passes unchanged, including
      `load_scripts_reports_a_broken_file_and_the_checks_of_the_others`.
- [ ] `cargo xtask ticket-lint --check-engine` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the loaders' existing tests, unchanged.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
