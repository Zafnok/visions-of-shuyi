---
id: "1102"
title: "The engine folder and a gate that keeps game code out of it"
type: infra
milestone: "Engine: boundary"
model: sonnet-5
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1102 — The engine folder and the boundary gate

## Context

ADR-0063 splits the project into an engine and the game, first inside
this repository (an `engine/` folder in the same Cargo workspace), later
as two repositories. Its §4 makes the boundary a gate so nothing of the
game leaks into the engine while code moves (1103–1117). This ticket
creates the folder and the gate before any code moves, so it conflicts
with no open work and can run now, during the Chapter 1 playtest work.

Read: ADR-0063, `docs/engine/boundary.md`, `crates/xtask/src/main.rs`
(how subcommands are registered), `crates/xtask/src/check_keys.rs` (a
similar check with path lists and tests), `.github/workflows/ci.yml`
(where `ticket-lint`, `check-keys` and `check-text` run).

## Nick input

None.

## Scope

**In:**
- `engine/README.md`: what the engine is (ADR-0063 §1), the working name
  (§3), the five crates planned and what each owns (§2), the allowed
  third-party crates (start with `serde`, `ron`, `thiserror`,
  `include_dir`, `macroquad`, `quad-snd`, `gilrs`, `quad-storage`,
  `proptest`, `insta`), and that its files stay under `LICENSE` until
  1130.
- `engine/crates/` (empty, with a `.gitkeep`), and the workspace
  `members` extended to `["crates/*", "engine/crates/*"]` so a crate
  added there is built.
- `cargo xtask check-engine-boundary` (`crates/xtask/src/engine_boundary.rs`):
  1. Reads `cargo metadata --format-version 1 --no-deps` and, for every
     package whose manifest is under `engine/`, fails if a dependency is
     a workspace package outside `engine/`, or a third-party crate not
     on the README's allowed list (parse the list from a fenced
     `engine-deps` block in `engine/README.md`).
  2. Scans every file under `engine/` and fails on the strings `trpg_`,
     `trpg-`, `visions-of-shuyi`, `assets-private`, `assets-test/`, a
     path starting with `assets/` outside `engine/`, and on any id
     listed in this game's data: class, character, terrain, item, spell,
     skill and art ids, and audio cue ids (read from `assets/data/*.ron`
     and `assets/audio/*.ron` with the existing loaders in
     `trpg-content`; skip ids of three letters or fewer and common words
     listed in an `ALLOW` constant, with a comment saying why each is
     there).
  3. Prints every finding as `file:line: message`, exits non-zero if any.
- The check runs in CI next to `check-keys`, and is added to the
  `run-gates` skill's list.

**Out (do not do):** moving any code (1103 onward); a license for the
engine (1130); renaming anything.

## Implementation steps

1. Add the folder, README and workspace member glob. Confirm `cargo
   build --workspace` and `cargo metadata` still work with an empty
   `engine/crates/`.
2. Write `engine_boundary.rs` with pure functions over the metadata JSON
   and file contents (unit-testable without running cargo), plus the
   thin command that runs `cargo metadata` and walks `engine/`.
3. Register the subcommand; add it to `ci.yml` and `run-gates`.
4. Tests: a fake metadata JSON with an engine package depending on
   `trpg-core` fails; one depending on an unlisted crate fails; one
   depending only on listed crates passes; a file containing
   `trpg_core::` fails with its line; a file naming a class id fails; a
   clean file passes.

## Acceptance criteria

- [ ] `cargo xtask check-engine-boundary` passes on `main` with the
      empty folder.
- [ ] Its unit tests cover each rule above (passing and failing case).
- [ ] CI runs it; `run-gates` lists it.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each rule over fixtures.
- Integration: `crates/xtask/tests/cli.rs` runs the command on the real
  repository and expects success.

## Completion notes
