---
id: "1027"
title: "Playdate: trpg-core builds without std (a default std feature) and CI checks it"
type: infra
milestone: Post–Chapter 1
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-10-15
---

# 1027 — Playdate: `trpg-core` builds without `std`

## Context

First build ticket of the Playdate port of Chapter 1 (planned by 1025;
Nick, 2026-10-15: "I wanna rush a playdate port!!!"). Playdate Rust has
no standard library, so the rules must build with `#![no_std]` + `alloc`.
Ticket 1026 did this on the never-merged branch
[`spike/playdate`](https://github.com/Zafnok/visions-of-shuyi/tree/spike/playdate);
[ADR-0070](../../docs/adr/0070-playdate-front-end.md) §3 lists what it
takes. This ticket brings the `core` part of the spike to `main`, cleanly,
with the CI job that keeps it building.

## Nick input

None. Nothing a player sees changes.

## Scope

**In:**
- A default `std` feature in `crates/core/Cargo.toml`; with it off the
  crate is `#![no_std]` + `extern crate alloc`
  (`#![cfg_attr(not(feature = "std"), no_std)]`).
- `std::` paths to `core::` / `alloc::` (`BTreeMap`, `Arc`, `Rc`, `fmt`,
  `iter`, `mem`, `cmp`, `cell`, `error::Error` all exist there).
- A crate prelude (`crate::prelude::*`: `Vec`, `String`, `Box`,
  `ToString`, `ToOwned`, `vec!`, `format!`) imported where needed.
- `serde` in the workspace `Cargo.toml` with
  `default-features = false, features = ["derive", "alloc"]`; `trpg-core`'s
  `std` feature turns on `serde/std` (and the same for any other
  dependency of `core` that has a `std` feature).
- A CI job `no-std` in `.github/workflows/ci.yml`: installs the
  `thumbv7em-none-eabihf` target and runs
  `cargo build -p trpg-core --no-default-features --target thumbv7em-none-eabihf`.
  Add it to `ci-result`'s `needs` and to the `changes` filter the other
  Rust jobs use. 1028 and 1029 add their crates to the same job.
- The same command in the `run-gates` skill's *Order* list
  (`.claude/skills/run-gates/SKILL.md`), next to the WASM build, with
  `rustup target add thumbv7em-none-eabihf` under *Environment facts*.

**Out (do not do):**
- `content` and `ui` (1028, 1029).
- Any change of behaviour: with `std` on, every test passes unchanged.
- Float maths changes: `core` uses no float functions that need `libm`
  (the spike needed none; if the build finds one, use `libm` as 1028
  does).

**Carve (1103–1118):** either order works. If `core`'s modules have moved
into `engine/` crates (1114) first, those crates get the same `std`
feature and CI line (ADR-0070 §6: carved crates keep it).

## Implementation steps

1. Read ADR-0070 §3 and the spike's diff for `crates/core`
   (`git diff main...origin/spike/playdate -- crates/core`). Redo it on
   today's `main` (the code has moved since; don't cherry-pick blindly).
2. Add the feature, the prelude and the path changes.
3. Workspace `serde` without default features; `std` features forward it.
4. Check: `cargo build -p trpg-core --no-default-features --target thumbv7em-none-eabihf`
   and `cargo test -p trpg-core` (with `std`).
5. Add the CI job and the gate line.

## Acceptance criteria

- [x] `cargo build -p trpg-core --no-default-features --target thumbv7em-none-eabihf` passes locally and in the new CI job.
- [x] `cargo test -p trpg-core` passes with no test changed.
- [x] Nothing else in the workspace changes behaviour (all tests pass).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None new: the CI build is the test; existing tests prove nothing changed.

## Completion notes

- `trpg-core` has a default `std` feature (forwarding `serde/std`); with it
  off the crate is `#![no_std]` + `alloc` and builds for
  `thumbv7em-none-eabihf`. Redone from `spike/playdate` on today's `main`
  (one conflict in `save.rs`, from 0835's doc lines).
- Workspace `serde` is `default-features = false, features = ["derive",
  "alloc"]`. `ron` and `thiserror` are left as they are: `core` doesn't use
  them outside its tests, and 1028 changes them for `content`.
- Deviations from the spike: no `#[allow(unused_imports)]` on the prelude
  imports. The prelude is imported only in modules that use it (dropped
  from `stats`, `magic`, `weapon`). The prelude has no `format!`: `core`'s
  library code doesn't use it (tests run with `std`); add it when needed.
- No float function needed `libm`.
- CI: new `no-std` job (gated on `changes`, in `ci-result`'s `needs`);
  the `run-gates` skill runs the same build and says to add the target.
- Tests still use `std::` paths: they always build with `std`.
- Nothing a player sees changed; no game rules decided.
