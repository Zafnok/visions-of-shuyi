---
id: "1121"
title: "Pointer (mouse and touch) and analog stick input in the engine, rebindable and deterministic"
type: feature
milestone: "Engine: real time"
model: opus-5.5
effort: high
status: todo
blocked_by: ["1119"]
nick_input: none
completed:
---

# 1121 — Pointer and analog input

## Context

ADR-0064 §8. The engine reads keys and controller buttons only. Sticks
are turned into four digital directions (`engine-ui::input::pad`, from
`ui/src/input/pad.rs`), and there is no mouse or touch at all
(`docs/engine/boundary.md`, *app*). Free movement wants analog speed.
Editors and many 2D games want a pointer.

Read the `keyboard-input` skill. Every new input stays an action the
player can rebind.

## Nick input

None. Our game doesn't use these unless a later ticket says so.

## Scope

**In:**
- `engine-app` reads the mouse position, buttons and wheel, touches, and
  stick axes (gilrs on native, the web pad plugin) into `RawInputEvent`s.
- `engine-ui::input` gets four things:
  - `Axis` actions (two-axis and one-axis) bound to sticks or to keys
    (keys give full deflection);
  - a dead zone setting;
  - pointer state in console pixels, and in world pixels for the top
    world layer's camera (1120);
  - pointer buttons as bindable inputs.
- `TickInput` carries axes as `i16` and the pointer in world pixels, so
  records replay exactly (1119).
- The Harness scripts pointer moves, clicks and stick positions.
- The rebinding screen (1112) lists axes.

**Out (do not do):** gestures (pinch, swipe); on-screen touch controls.

## Implementation steps

1. Add the raw events in `engine-app`, on native and web.
2. Add the axis actions and the bindings format (saved bindings get a
   new version with a migration, per ADR-0053's pattern).
3. Add pointer state, the Harness script commands and the tests.

## Acceptance criteria

- [ ] A Harness test moves a dot with a scripted stick at half deflection
      and gets half speed.
- [ ] A Harness test clicks a menu row with the pointer.
- [ ] Saved bindings from before load unchanged (fixture).
- [ ] `check-keys` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: dead zone, axis from keys, pointer mapping.
- Integration: the Harness tests above.

## Completion notes
