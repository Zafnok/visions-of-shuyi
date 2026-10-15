---
id: "1029"
title: "Playdate: trpg-ui builds without std (libm floats, hashbrown)"
type: infra
milestone: Post–Chapter 1
model: sonnet-5
effort: medium
status: done
blocked_by: ["1028"]
nick_input: none
completed: 2026-10-15
---

# 1029 — Playdate: `trpg-ui` builds without `std`

## Context

Third `no_std` ticket of the Playdate port (planned by 1025). The screens
and their views run unchanged on the Playdate (ADR-0054); only the look is
new skins. [ADR-0070](../../docs/adr/0070-playdate-front-end.md) §3 lists
what `ui` needs; the never-merged branch
[`spike/playdate`](https://github.com/Zafnok/visions-of-shuyi/tree/spike/playdate)
has a working version (+401 / −109 lines in 120 files, ~195 of them the
prelude import).

## Nick input

None. Nothing a player sees changes.

## Scope

**In:**
- Default `std` feature on `crates/ui`; without it, `#![no_std]` +
  `alloc`, the same prelude as `core` (1027). Everything `std`-only
  (snapshot helpers that write files, `insta`) behind `std` or `cfg(test)`.
- Float calls (`round`, `floor`, `sin`, `cos`, `hypot`, `rem_euclid`;
  26 in the spike): through a small `libm`-backed trait used only
  without `std`, so `std` builds keep calling the standard methods and
  give the same numbers as today.
- `HashMap` (2 uses) → `hashbrown`; one `LazyLock` → `OnceBox`.
- New dependency `hashbrown` (MIT OR Apache-2.0); `cargo deny check licenses`.
- Add `trpg-ui` to the `no-std` CI job and the gate line.

**Out (do not do):**
- Any new screen, skin or Playdate code (1032 onwards).
- Any change of behaviour or snapshot with `std` on.

**Carve (1103–1118):** either order works. Modules that moved to
`engine/` crates get the same `std` feature and CI line (ADR-0070 §6).

## Implementation steps

1. Read the spike's diff for `crates/ui`
   (`git diff main...origin/spike/playdate -- crates/ui`) and redo it on
   today's `main`.
2. Feature, prelude, float trait, `hashbrown`, `OnceBox`.
3. `cargo build -p trpg-ui --no-default-features --target thumbv7em-none-eabihf`;
   `cargo test -p trpg-ui` (no snapshot may change).
4. Add the crate to the CI job.

## Acceptance criteria

- [x] `cargo build -p trpg-ui --no-default-features --target thumbv7em-none-eabihf` passes locally and in CI.
- [x] `cargo test -p trpg-ui` passes; `cargo insta` shows no changed snapshot.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None new: the CI build and the unchanged tests and snapshots.

## Completion notes

- `crates/ui` has a default `std` feature; without it the crate is
  `#![no_std]` + `alloc` with the same prelude as `core` and `content`
  (`use crate::prelude::*;` in the ~105 files that need it, `std::` paths
  now `core::`/`alloc::`). Redone on today's `main` from compiler errors
  rather than copying the spike, so no `#[allow(unused_imports)]` anywhere.
- Floats: `src/float.rs` holds a `libm`-backed `Float` trait (`f32`: `round`,
  `floor`, `sin`, `cos`, `hypot`, `rem_euclid`) and `Round` (`f64::round`,
  used once by `cinematic::millis`). It is in the prelude only without
  `std`; the seven float-only files import it under the same `cfg`. With
  `std` every call is still the standard method.
- `HashMap` in `input.rs` and `snapshot.rs` → `hashbrown` 0.17 (MIT OR
  Apache-2.0; brings `foldhash` 0.2, Zlib). Only lookups, never iterated
  for output, so nothing depends on its order. `Ctx::effective_lang`'s
  `LazyLock` → `once_cell::race::OnceBox`, as `content` did.
- `trpg-ui` added to the `no-std` CI job and the `run-gates` list.
- Deviation: three small tests in `float.rs` (built under `cfg(test)` too)
  hold the `libm` versions to `std`'s numbers: bit-equal for rounding and
  remainders, within 1e-6 for `sin`/`cos`/`hypot`. Without them CI's
  mutation testing would see the `no_std`-only code as untested.
- No snapshot changed. Nothing a player sees changes; nothing for Nick.
