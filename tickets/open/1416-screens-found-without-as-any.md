---
id: "1416"
title: "Delete every screen's as_any and as_any_mut now that the engine finds any screen on the stack"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: low
status: blocked
blocked_by: ["engine#3255"]
nick_input: none
completed:
---

# 1416 — Screens found without as_any

## Context

The third engine-friction sweep (1397) counted 31 copies of
`fn as_any(&self) -> Option<&dyn Any> { Some(self) }` in 28 files and 7
copies of `as_any_mut`: about 150 lines that exist only so that
`ScreenStack::find`, `Game::screen_mut` and the tests can find a
screen. A screen that forgets them is silently not found. Engine ticket
3255 gives every screen `as_any` through a blanket supertrait and
removes the trait methods. That change is breaking, so this ticket goes
with the pin move that takes it.

**No code markers.** The sweep marks no copy: a marker above each
one-line method on 28 screens would make `cargo xtask pr-size` warn on
every PR that touches a screen. To find the copies, use
`grep -rn "fn as_any" crates`.

## Nick input

None.

## Scope

**In:**
- Move the pin past engine 3255 (`engine-change` skill, §7) if it isn't
  there yet. Its break is what makes the copies fail to compile.
- Delete every `fn as_any` and `fn as_any_mut` in an `impl Screen`
  across the game's crates, and the `use core::any::Any` lines that only
  they used.
- If some other code reads a screen through `Screen::as_any`, make it
  use `ScreenStack::find` or `AsAny` instead.

**Out (do not do):**
- Anything else in those screens.

## Implementation steps

1. Write the engine-friction estimate first: about 150 lines removed and
   0 added.
2. Move the pin, then run `cargo check --workspace --all-targets`: every
   copy fails to compile, which lists them.
3. Delete them, then run the gates.

## Acceptance criteria

- [ ] `grep -rn "fn as_any" crates` finds nothing.
- [ ] Every test that finds a screen on the stack still passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None new: the existing tests that find screens.

## Completion notes

*(Filled in by the session that completes the ticket.)*
