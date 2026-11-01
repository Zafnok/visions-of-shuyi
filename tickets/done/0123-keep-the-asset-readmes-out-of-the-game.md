---
id: "0123"
title: Keep the asset READMEs out of the game
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-01
---

# 0123 — Keep the asset READMEs out of the game

## Context

`trpg_content::bundle` embeds every file under `assets/` with
`include_dir!` (ADR-0005). That includes the `README.md` of each asset
directory: about 67 KB of notes for people who write assets, which no code
reads. Ticket 0122 (ADR-0057) found them while checking what a release
build holds: the sold game's `.exe` contains them, with their mentions of
test files, tickets and tools.

They are harmless, but they are not the game's, and the build that ships
should hold only what it reads.

## Nick input

`None.`

## Scope

**In:**
- No `README.md` in the embedded bundle of any build: `assets/`,
  `assets-test/` and, with `private-assets`, `assets-private/game/` and
  `assets-private/test/`.
- The READMEs stay where they are in the repository, beside the files they
  describe.

**Out (do not do):**
- Moving or rewriting any README.
- Changing what any other file embeds.

## Implementation steps

1. Read `crates/content/src/bundle.rs` and ADR-0005, ADR-0040, ADR-0057.
2. `include_dir!` has no filter. Pick one way and write it down in a short
   ADR (`write-adr` skill) or as an amendment note in ADR-0057's style:
   - the bundle's reads and listings skip files named `README.md` (the
     layering is Shuyi Engine's since 1103, so this is an engine change:
     the `engine-change` skill; and the bytes stay in the binary, so this
     alone does **not** meet the acceptance criterion); or
   - embed from a filtered copy that `crates/content/build.rs` writes to
     `OUT_DIR` (ADR-0040 rejected a *merging* build script; a filter is
     simpler, but read its reasons first); or
   - move the READMEs out of the embedded directories (e.g. to
     `docs/assets/<kind>.md`). This is under **Out**: if it turns out to
     be the best way, stop and rewrite this ticket's scope first.
3. Private `game/README.md` needs no credit: the engine's credits check
   (Shuyi Engine's since 1104) excuses a file of that name. Keep that
   working for whatever still reads the README.
4. Update `crates/content/README.md`.

## Acceptance criteria

- [x] `bundle::files_under("")` lists no `README.md`, with and without the
      test files (`bundle::story_only`).
- [x] A release build of `trpg-app` does not contain the text of
      `assets/maps/README.md` (check the binary for a sentence of it, in a
      test or by hand with the command in the PR).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the bundle lists and reads no README; every other file as before.
- Integration: the all-assets test still passes.

## Completion notes

Every bundle layer is now a table `build.rs` writes, and its walk skips
`README.md`, so no build embeds one (ADR-0105). `include_dir` is gone from the
game's own crates; it stays in `Cargo.lock` through the engine.

Deviations from the steps: none of the ticket's three ways was taken as
written. The nearest is the filtered copy, but the tables that already existed
for the Playdate needed no copy. The dialogue tests that run the README's
examples now read it with `include_str!`. `lang.rs` lost its README filter,
and `playdate-bundle.txt` its `**/README.md` line and `**/` form, which nothing
else used.

With `private-assets`, a missing `assets-private/test/` now gives an empty
table where it used to stop a debug build from compiling (ADR-0105).

Binary check, on `cargo build --release -p trpg-app`: a sentence each of the
maps, dialogue and lang READMEs is found 0 times. There was no older release
build to compare with, so the search itself was only controlled with a word
the game's data does hold.

The engine's credits check still excuses a private `README.md`; the bundle
just no longer hands it one. No gameplay rule was decided.

No line estimate was written before coding. Measured: 39 production lines
added, 23 removed.

Nothing for Nick to check.
