---
id: "1028"
title: "Playdate: trpg-content builds without std (a generated file table instead of include_dir)"
type: infra
milestone: Post–Chapter 1
model: reasoning
effort: medium
status: done
blocked_by: ["1027"]
nick_input: none
completed: 2026-10-15
---

# 1028 — Playdate: `trpg-content` builds without `std`

## Context

Second `no_std` ticket of the Playdate port (planned by 1025).
[ADR-0070](../../docs/adr/0070-playdate-front-end.md) §3 lists what
`content` needs; the never-merged branch
[`spike/playdate`](https://github.com/Zafnok/visions-of-shuyi/tree/spike/playdate)
has a working version (+318 / −73 lines in 49 files). 1027 added the
`no-std` CI job and the pattern (default `std` feature, crate prelude).

## Nick input

None. Nothing a player sees changes.

## Scope

**In:**
- Default `std` feature on `crates/content`; without it,
  `#![no_std]` + `alloc`, the same prelude as `core` (1027).
- `include_dir` does not build without `std`: a `build.rs` writes a
  sorted table of `(path, include_bytes!(…))` for `assets/` and the
  embedded bundle reads that table when `std` is off. Same public
  functions (`load_embedded`, the bundle's lookups). With `std` on, keep
  `include_dir` as today (the spike's choice: two paths until the
  engine's bundle, 1103, replaces both). The test files (`assets-test/`)
  and the private layer (`private-assets`) stay `std`-only.
- `LazyLock` → `once_cell::race::OnceBox`; `f32`/`f64` `round` →
  `libm`; `Path::extension` → a string split; `ron` and `thiserror` with
  `default-features = false` (both support `no_std`), their `std`
  features forwarded by `content`'s.
- New dependencies `once_cell` and `libm` (MIT OR Apache-2.0; check with
  `cargo deny check licenses`).
- Add `trpg-content` to 1027's `no-std` CI job and the gate line.
- A test (with `std`) that the generated table and `include_dir` list the
  same files with the same bytes, so the two paths can't drift.

**Out (do not do):**
- `ui` (1029).
- Loading less data or a faster format (1036).
- Any change of behaviour with `std` on.

**Carve (1103–1118):** either order works. If 1103 (the bundle as layers)
lands first, give its bundle the generated-table path instead of
`include_dir`'s for `no_std`; the carved crates keep the `std` feature
(ADR-0070 §6).

## Implementation steps

1. Read ADR-0070 §3 and the spike's diff for `crates/content`
   (`git diff main...origin/spike/playdate -- crates/content`). Redo it on
   today's `main`.
2. Feature, prelude, path changes, `OnceBox`, `libm`.
3. `build.rs` file table: walk `assets/` in sorted order, skip what the
   bundle skips today (READMEs, per 0123 if it has landed), emit
   `include_bytes!` lines into `OUT_DIR`, and `cargo:rerun-if-changed` for
   `assets/`.
4. The drift test.
5. Add the crate to the CI job; run `cargo build -p trpg-content --no-default-features --target thumbv7em-none-eabihf`
   and `cargo test -p trpg-content`.

## Acceptance criteria

- [x] `cargo build -p trpg-content --no-default-features --target thumbv7em-none-eabihf` passes locally and in CI.
- [x] `cargo test -p trpg-content` passes with no test changed, plus the drift test.
- [x] `cargo deny check licenses` passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the generated table matches `include_dir` (paths and bytes).
- The CI `no_std` build.

## Completion notes

- `trpg-content` has a default `std` feature; without it the crate is
  `no_std` + `alloc` with the same prelude as `core` (plus `format!`).
  The spike's diff for `crates/content` was redone on today's `main`
  (one conflict, in `tip.rs`), without its `#[allow(unused_imports)]`s:
  the prelude is imported only where it is used, as in `core`.
- `build.rs` writes `OUT_DIR/bundle_files.rs`, a sorted
  `(path, include_bytes!(…))` table of every file under `assets/`, read by
  `src/bundle_table.rs` (the bundle without `std`, binary search, same
  public functions). With `std` the bundle is `include_dir` as before; the
  test files and the private layer stay `std`-only.
- Drift test `the_no_std_table_holds_the_same_files_as_include_dir` (in
  `bundle.rs`): the table module is compiled into the `std` tests and must
  list the same paths with the same bytes as `include_dir`'s `assets/`,
  and answer every public function the same.
- `ron`, `thiserror` are `default-features = false` at the workspace (as
  `serde` since 1027), `once_cell` and `libm` are new workspace
  dependencies (MIT OR Apache-2.0 / MIT); `content`'s `std` forwards their
  `std`. `core`'s dev-dependency on `ron` asks for `std` itself.
- CI's `no-std` job and the `run-gates` list build `trpg-content` too.
- **Deviation:** none in scope. READMEs are in the table, because
  `include_dir` embeds them today (0123 has not landed); when it does, both
  paths must skip them and the drift test will say so.
- Nothing a player sees changes.
