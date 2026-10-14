---
id: "1128"
title: "Optional rigid-body physics: an engine-physics crate over rapier2d with cross-platform determinism"
type: feature
milestone: "Engine: more styles"
model: opus-5.5
effort: high
status: todo
blocked_by: ["1122"]
nick_input: none
completed:
---

# 1128 — Optional rigid-body physics (rapier2d)

## Context

This is ADR-0065 §5. Most of the 2D styles Nick named are better served
by the pixel-stepping kit (1122). Some games want rigid bodies: crates
to push, ropes, things that tumble.

`rapier2d` (Apache-2.0, pure Rust) gives bit-for-bit identical results
on every platform, the web included, with its `enhanced-determinism`
feature. That feature can't be combined with `simd8`.

## Nick input

None.

## Scope

**In:**
- A crate `engine/crates/physics` (`engine-physics`), not a dependency of
  any other engine crate. It depends on `rapier2d` with
  `default-features = false` and `["dim2", "enhanced-determinism"]`.
  `cargo deny` must pass.
- A `PhysicsWorld` stepped from a `TickSim::step` at the fixed rate.
  Bodies and colliders are created from data. Positions cross between
  `Fx` and `f32` at this crate's edge. Events come back as engine events.
- A `hash()` over body positions and velocities (their bits), so replays
  and networking (ADR-0067) can check agreement.
- A test that runs the same scene natively and as `wasm32` and gets the
  same hashes, against a checked-in table if running wasm in CI is
  impractical.
- Measure the size it adds to a WASM build and record it.

**Out (do not do):**
- making physics the default;
- joints beyond what the test scene needs.

## Implementation steps

1. Write the crate, the stepping and the conversion.
2. Write the determinism test and the size measurement.
3. Add a small example scene (boxes falling on a slope) in the crate's
   tests.

## Acceptance criteria

- [ ] The same hashes come out natively and on wasm, or match the
      checked-in table.
- [ ] `cargo deny check` passes.
- [ ] The size is recorded in the Completion notes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the determinism test.

## Completion notes
