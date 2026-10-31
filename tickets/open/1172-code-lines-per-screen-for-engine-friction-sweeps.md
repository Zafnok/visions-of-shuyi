---
id: "1172"
title: "cargo xtask code-lines --tree: production lines per file and per screen of the whole game, for engine-friction sweeps"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1172 — Production lines per screen, for sweeps

## Context

`cargo xtask code-lines` (1151, ADR-0092) counts production lines per
crate in a diff. The first engine-friction sweep (1153) needed them per
file and per screen for the whole tree; `--base` with git's empty tree
gives only per-crate totals, so 1153 re-wrote the counting rules in a
throwaway script. Every later sweep (the `engine-friction` skill,
*Sweep*) needs the same numbers, counted the same way, to compare with
the rows in `docs/engine/friction.md`.

## Nick input

None.

## Scope

**In:**
- `cargo xtask code-lines --tree`: the production lines of every Rust
  file in the working tree (the same rules as today: no `tests/`,
  `tests.rs`, `*_tests.rs`, `#[cfg(test)]` items, blank or comment
  lines), grouped per **screen**: a file `x.rs` together with the
  folder `x/` beside it and everything in it (so `screens/save.rs`,
  `screens/save/view.rs` and `screens/save/glyph.rs` are one row), and
  per crate. Sorted largest first, with `--top N`; `--files` lists
  files instead of screens.
- The `engine-friction` skill's *Sweep* section names the command.

**Out (do not do):**
- Changing the diff mode's numbers.
- A gate on any number (ADR-0092: there is none).

## Implementation steps

1. In `crates/xtask/src/code_lines.rs`, reuse `test_ranges` and the
   production-file rule on whole files; walk `crates/`.
2. Group by screen as above; print a table like the diff mode's.
3. Tests with a small tree in a temporary folder.

## Acceptance criteria

- [ ] At commit `eb93fc89`, `cargo xtask code-lines --tree` counts
      `crates/ui/src/screens/battle/mod.rs` as 2009 and the
      `screens/save` screen plus `crates/ui/src/save.rs` as 675 (the
      1153 rows in `docs/engine/friction.md`), within a few lines.
- [ ] `--files` lists every file instead of screens, so a sweep can
      sum the files of a candidate that isn't one folder (the battle's
      panels, for example).
- [ ] A unit test: a file with a `#[cfg(test)] mod tests { … }` counts
      only the lines above it; `x.rs` and `x/y.rs` are one row.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit, as above.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
