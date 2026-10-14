---
id: "1130"
title: "Apply the engine's license and name, as Nick chose them"
type: infra
milestone: "Engine: public"
model: sonnet-5
effort: medium
status: todo
blocked_by: ["0059", "1118"]
nick_input: none
completed:
---

# 1130 — Apply the engine's license and name

## Context

ADR-0063 §3 keeps the engine's files under `LICENSE` until Nick chooses
the engine's license and name. That choice is 0059, recorded in
`docs/engine/decisions.md`. `LICENSE` itself is never changed: it stays
the game's (`CLAUDE.md` hard rule 6).

## Nick input

None here. The words come from 0059.

## Scope

**In:**
- The license files Nick chose go in `engine/` only. For "MIT OR
  Apache-2.0" those are `engine/LICENSE-MIT` and
  `engine/LICENSE-APACHE`, with the copyright line from 0059. Each
  engine crate's `Cargo.toml` gets `license = "<SPDX>"` instead of the
  workspace's `license-file`.
- `deny.toml` treats the engine crates as what they now are, and the
  game's own crates stay private.
- A line in the root `README.md` and `LICENSE` section 6 is **not**
  edited. Instead `engine/README.md` states that the folder is under its
  own license. Ask Nick, in the PR, whether he also wants a sentence
  added to `LICENSE` naming the folder. Do it only in his words.
- Rename the packages (`engine-*` → `<name>-*`), the binary, the
  `engine.ron` file, the JS plugin prefix and the docs, per 0059.
- `cargo-about`'s third-party report for the game still covers the
  engine crates.

**Out (do not do):**
- moving to a new repository (1131);
- publishing to crates.io (1131).

## Implementation steps

1. Add the license files and the SPDX fields.
2. Rename the packages and everything that names them, using
   `check-engine-boundary` and `typos` to find them.
3. Do the full build and test, native and web.

## Acceptance criteria

- [ ] `cargo metadata` shows each engine crate with the chosen SPDX
      license.
- [ ] `LICENSE` is unchanged unless Nick wrote the change.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None beyond the gates.

## Completion notes
