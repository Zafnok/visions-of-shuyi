---
id: "1029"
title: "Playdate: trpg-ui builds without std (libm floats, hashbrown)"
type: infra
milestone: Post–Chapter 1
model: sonnet-5
effort: medium
status: todo
blocked_by: ["1028"]
nick_input: none
completed:
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

- [ ] `cargo build -p trpg-ui --no-default-features --target thumbv7em-none-eabihf` passes locally and in CI.
- [ ] `cargo test -p trpg-ui` passes; `cargo insta` shows no changed snapshot.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None new: the CI build and the unchanged tests and snapshots.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
