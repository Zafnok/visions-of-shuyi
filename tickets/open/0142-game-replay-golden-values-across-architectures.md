---
id: "0142"
title: Check game replay golden values on x64 ARM64 and wasm32
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: high
status: todo
blocked_by: ["engine#2116"]
nick_input: none
completed:
---

# 0142 — Game replay golden values across architectures

## Context

The 2026-11-01 audit found that engine PR #169 (2116), passed by game
#482, runs engine replay/asset golden tables across x64, ARM64 and wasm32.
Its fixtures do not execute `trpg-core`. This game's CI tests on host OSes
and builds its web and Playdate targets, but it has no shared game-state
golden table executed on the 32-bit wasm target. PR #497 removed floats
from AI scoring; that is useful but is not a cross-architecture replay
check. No existing desync is asserted here.

`crates/core/tests/it/replay.rs` already defines a small battle with
combat RNG, EXP, spells, terrain effects, triggers and a replay/save
script. Build on that fixture and the engine's stable `state_hash`, not
a new simulation or hash algorithm (ADR-0004, ADR-0007).

## Nick input

None. Automated checks only; no device, account, secret or check after
merge. This is validation infrastructure, independent of Act 1 content.

## Scope

**In:**
- A committed golden table of one representative game-rules replay: initial
  state and each command's resulting state hash, event hash and final
  outcome (including an explicit nonterminal outcome if that is the
  existing script's result).
- The same fixture and table executed natively on x64 and ARM64, and under
  Wasmtime using `wasm32-wasip1`, with no device backend linked.
- A deliberate wrong expected value demonstrating failure, and an explicit
  reviewed update procedure for intentional rules changes.

**Out (do not do):**
- Promising that this one fixture proves every game rule or AI choice.
- Content importing, baked packs, new public hashing APIs or benchmarks.
- Renderer/browser checks (0141), UI scenes (1202), bot record stamps
  (1195) and changes to gameplay. Each can land independently: this test
  constructs its existing rules fixture and does not read `PlayRecord`.

## Implementation steps

1. Verify engine 2116 is done and at the pin, as it was after #482. Read
   the engine's determinism job and Hello, Grid's golden test for the
   pattern; no pin-only ticket is needed.
2. Add `crates/core/tests/it/golden.rs` and its module in `it/main.rs`,
   following this repository's single integration-test program rule.
   Reuse `replay::setup(seed)` and `replay::script()` (make only necessary
   helpers `pub(super)`). Fix the seed in the fixture. Use the existing
   `shuyi_core::state_hash` for the serializable `BattleState` and events.
   Read the expected table with `include_str!`; no runtime asset path,
   clock, device crate or simulation dependency is added.
3. Check initial and per-command values with a command index in every
   failure. Keep the ordinary same-run replay/save tests too: a golden
   expectation tests stability across builds, not a replacement for them.
   An intentional hash change prints expected/actual and is reviewed as a
   rules or fixture change; CI must never bless automatically.
4. Run this named golden test in CI on an explicit x64 runner, an ARM64
   runner (reuse the current Apple-silicon host leg if verified ARM64)
   and `wasm32-wasip1` under the pinned Wasmtime version used by the
   engine. Print the actual target triple/architecture; a missing runner
   or skipped target cannot count as a pass. Only this pure-core test
   target runs under WASI, not the window/xtask workspace.
   First compile the entire `it` test program for WASI: a name filter
   selects tests to run, not modules or dev-dependencies to compile.
   Audit `proptest`'s current fork/timeout defaults and the integration
   harness against engine 2116's feature setup. Make scoped dev-feature
   or target-specific test-harness fixes where required; preserve native
   property-test coverage and do not skip the golden on a compile failure.
5. Change one expected hash in a disposable working copy, run the named
   test natively and under Wasmtime, record both nonzero results, then
   restore it. Document the local command and reviewed update procedure
   in the existing testing guide and `run-gates`.

Expected size: about 80–150 test lines, a small golden table and 40–80
workflow lines; under 900 changed lines including docs and ticket move.

## Acceptance criteria

- [ ] One checked-in fixture/table is used unchanged on x64, ARM64 and
      wasm32; initial state, every accepted command's state and events,
      and final outcome match. CI records the actual architecture.
- [ ] The fixture includes the existing script's combat RNG, EXP,
      spells/terrain effects and triggers; the test asserts those paths
      were exercised so an empty replay cannot pass.
- [ ] A wrong expected hash fails with the command index and values on
      native and Wasmtime runs; normal CI cannot regenerate expectations.
- [ ] Existing replay/save tests and the Playdate no-std build still
      pass. No device backend is linked into the WASI test target.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the same named golden test on all three architectures.
- Negative integration: the deliberate wrong-value check above; reuse
  the engine's hash function rather than testing a second hasher.
- Existing replay/save integration tests remain the behavioral reference.

## Completion notes

*(Filled in by the session that completes the ticket: fixture coverage,
actual target triples, failure proof and reviewed update instructions.)*
