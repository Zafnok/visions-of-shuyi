---
id: "1123"
title: "Animation clips drawn from the time alone, with Aseprite and TexturePacker sheets imported"
type: feature
milestone: "Engine: real time"
model: sonnet-5
effort: high
status: todo
blocked_by: ["1119"]
nick_input: none
completed:
---

# 1123 — Animation clips; Aseprite and TexturePacker import

## Context

ADR-0065 §2–§3. Today the only animation is the 3×4 walk sheet
(ADR-0049). An `AnimationClip` is a list of frames with a direction and
a repeat rule. The frame shown is a pure function of the clip and the
time since it started (as ADR-0060 does for cinematics), so a tick sim
stores only `(clip, start_step)`.

Aseprite's CLI exports a sheet plus JSON with frame tags. That is the
common route and needs no extra crate. Reading `.aseprite` directly goes
through `asefile` (MIT), only if it stays small.

## Nick input

None.

## Scope

**In:**
- `engine_assets::anim`: `Atlas` (named rectangles in an image),
  `AnimationClip`, `AnimationSet`, `frame_at(clip, elapsed_ms)`.
- Importers report errors with the file and the JSON path:
  - Aseprite JSON (`json-array` and `json-hash`, `frameTags` with
    direction and repeat, per-frame `duration`);
  - TexturePacker JSON (hash and array).
- Our RON sidecar form of the same data, and `engine-cli import --print`
  to turn an import into it.
- Measure `asefile` behind an `aseprite` feature. Keep it only if it adds
  under 300 KB to the WASM build (ADR-0065 §2). Write down the number
  either way.
- Fixtures: a small sheet made for the tests (CC0, ours).

**Out (do not do):** skeletal animation; tweens on sprites (the cinematic
easing covers them).

## Implementation steps

1. Write the types and `frame_at` with property tests (the frame index
   is periodic for loops; ping-pong never repeats the end frame twice).
2. Write the two importers with fixture files and error tests.
3. Do the `asefile` measurement.

## Acceptance criteria

- [ ] `frame_at` property tests pass.
- [ ] Both importers read their fixtures, and the bad fixtures give errors
      naming the JSON path.
- [ ] The `asefile` size result is recorded in the Completion notes.
- [ ] `cargo deny check` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit, property, and fixture tests, as above.

## Completion notes
