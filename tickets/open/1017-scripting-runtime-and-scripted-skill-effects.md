---
id: "1017"
title: "Scripting in core: a deterministic, sandboxed script runtime and script-backed skill effects"
type: feature
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["1011", "1013", "engine#2101"]
nick_input: none
completed:
---

# 1017 — Scripting runtime and scripted skill effects

## Context

Nick asked for "a scripting DSL or something for easy modding support".
Data covers most mods: a new item, class or skill made from existing effect
kinds. A **new kind** of effect today needs Rust code: ADR-0021 made skill
effects a closed set of data variants, "no scripting, no per-skill code".
1011's ADR amends ADR-0021: a skill may name a script, and the script
returns data `core` already understands.

The hard rules: `core` stays pure and deterministic (ADR-0004, ADR-0019).
A script reads; it never changes battle state. It returns effects, and the
battle applies them through the same code paths as data effects, so rewind,
saves (a battle is its history, ADR-0039) and the bots keep working.

**The runtime is the engine's: engine#2101** (in
[Shuyi Engine's tickets](https://github.com/Zafnok/shuyi-engine/tree/main/tickets)).
A deterministic, sandboxed script runtime is engine material, and
visual scripting (engine 1135) compiles to it (engine ADR-0005 §5). 2101
builds it as an engine module, `shuyi-script`, with an engine ADR that
picks the language (default: Rhai, integer-only, with an operation
limit; research for 1101 confirmed Rhai builds for
`wasm32-unknown-unknown`, and that `mlua` supports only emscripten on
the web, which our build doesn't use). Scripts compile once at load and
report errors with file and line; a script reads a view the game hands
it and returns plain values; errors and limits give an error value,
never a panic. This ticket is the game's half: this game's view types,
`PassiveEffect::Script`, turning the values into effects, and wiring it
into the battle, all in `trpg-core` and `trpg-content`.

## Nick input

None.

## Scope

**In:**
- Move the pin to an engine `main` that has engine#2101 (`cargo xtask
  engine-pin <commit>`, the `engine-change` skill §7), and add the
  engine's `shuyi-script` module to the root `Cargo.toml`'s
  `[workspace.dependencies]` beside the other engine crates (same git
  URL and `rev`) and to `trpg-core` (or the crate 1011's ADR says).
  Add it to `CRATES` in `crates/xtask/src/engine.rs` too, so
  `engine-pin`, `engine-pin --check` and `cargo xtask engine`'s patch
  know it. `trpg-core` also builds without `std` for the Playdate
  (ADR-0070): if engine#2101's module doesn't, put scripting behind a
  `trpg-core` feature the Playdate's build leaves off, and say so in the
  completion notes.
- The game's use of the runtime in `crates/core/src/script.rs`: the
  bundle's scripts compiled once at load, with the engine's limits
  unless 1011's ADR sets others (the engine gives no clock, no host
  randomness, no file or network access and integer maths only).
- What a script sees: a read-only view (the attacker and defender's
  stats, weapons, HP, positions, terrain, the turn number, nearby units).
  Exact fields: the ADR's list.
- What a script returns: existing `PassiveEffect` / `ActiveEffect` values
  or `CombatMods` additions (ADR-0021), nothing else.
- A new effect variant (for example `PassiveEffect::Script { script }`)
  that the battle evaluates where it gathers other passives.
- Loading `scripts/*.rhai` (or the ADR's extension) from the bundle, so
  mods can ship them; a script that fails to compile is a content error
  with file and line.
- A script that hits the operation limit or errors at run time gives no
  effect and a logged warning; it never panics or desyncs.

**Out (do not do):** the runtime itself (engine#2101); scripted battle
events and objectives (1018); any
script in the shipped game's own content (scripts are for mods; the game's
skills stay data, per ADR-0021's reasoning about tests); the editor (1019).

## Implementation steps

1. Read ADR-0004, 0019, 0021, 1011's ADR, `crates/core/src/skill*` and
   where the battle gathers passives into `CombatMods`.
2. Move the pin and add the engine's module (Scope); confirm the WASM
   build (`cargo build -p trpg-app --target wasm32-unknown-unknown`) and
   `cargo deny check` still pass.
3. Build the view type and the conversion from the script's return value
   into effects, with every failure turned into "no effect".
4. Wire the new variant into passive gathering. Forecast and resolution
   must still agree (ADR-0021 point 2): the script runs once per gathering,
   with the same inputs, so both see the same mods.
5. A test mod skill: "+10 hit when the defender stands in a forest", written
   as a script, compared against the same rule written as data.

## Acceptance criteria

- [ ] A scripted passive changes the forecast exactly as the equivalent
      data passive does (test with the real combat code).
- [ ] The same script on the same state always returns the same effects
      (property test).
- [ ] A script that loops forever stops at the limit and gives no effect
      (test).
- [ ] A script that fails to compile is reported as a content error with
      its file and line (test).
- [ ] Rewind and save/load of a battle that used a scripted skill give the
      same state as playing straight through (test).
- [ ] The WASM build, `cargo deny check` and all gates in the `run-gates`
      skill pass.

## Tests required

- Unit: the view, the return conversion, limits.
- Property: determinism.
- Integration: the test mod's scripted skill in a battle, with rewind.

## Completion notes
