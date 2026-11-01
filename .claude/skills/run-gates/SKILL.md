---
name: run-gates
description: This game's local quality gates before pushing — every command in order, including its own (content validation, the WASM, Playdate and no-std builds, the engine pin and skill pack checks, the private-assets tests) — and its CI's mutation testing. Use before every commit you intend to push, and whenever CI fails and you need to reproduce it. Not for what the gates every Shuyi game runs mean, lint's findings or snapshots in general (shuyi-gates), or for writing tests (shuyi-test).
metadata:
  version: "1.0"
---

# Run the local gates

Verify shared skill discovery before the other gates:

```powershell
pwsh -File scripts/sync-agent-skills.ps1 -Check
```

**Read [`shuyi-gates`](../shuyi-gates/SKILL.md) first.** It holds what a
game on Shuyi Engine's gates are and mean: the order (fast to slow, stop
at the first failure), never weakening a gate, `lint`'s findings and
problem codes, the rules crate's fence, snapshots, and a CI failure this
machine doesn't reproduce. This skill holds this game's: the full list of
commands, which mirror CI (ADR-0014), and its own environment.

## Environment facts

- Nick's Windows machine uses the **GNU** Rust host toolchain
  (`x86_64-pc-windows-gnu`); **MSVC is not installed**. Don't add anything
  that needs MSVC locally. CI builds MSVC. Put MSYS2's `mingw64/bin` on
  `PATH` for the tests.
- `cargo-binstall` doesn't build on this machine; use `cargo install
  --locked <tool>`.
- The `no_std` build needs the Playdate's target once:
  `rustup target add thumbv7em-none-eabihf`.
- `cargo shuyi <command>` is Shuyi Engine's tools (`shuyi-cli`, ADR-0089,
  run through `cargo xtask shuyi` with this game's), on this repository's
  `shuyi.ron`; `cargo xtask` keeps the repository's own commands.
- The engine is pinned by revision (`docs/engine/README.md`). With the
  local checkout on (`cargo xtask engine`), the gates build on
  `shuyi-engine/` and `--locked` fails because `Cargo.lock` names the
  checkout: run `cargo xtask engine --off` before the gates you push on.
  The engine's own gates are its repository's `run-gates` skill, run
  inside `shuyi-engine/`.

## Order (fast → slow; stop at first failure and fix)

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo shuyi check-keys                    # no hard-coded keys (keyboard-input skill)
cargo shuyi check-text                    # no new screen text written as a literal (ADR-0045)
cargo shuyi check --target all            # the frame, text, controls and one-bit patterns fit desktop, web and Playdate (0138, 0140; identity: 1199)
cargo shuyi lint                          # the rules crate's fence on trpg-core: its clippy.toml's bans, floats among them, and its dependencies (0138, 0139)
cargo xtask engine-pin --check           # the engine's pin agrees in shuyi-engine.rev, Cargo.toml, Cargo.lock (ADR-0063 §8)
cargo xtask skill-pack --check           # .claude/skills/shuyi-* is the pinned engine's skill pack (ADR-0097)
cargo shuyi check-skills .claude/skills   # every skill here keeps the engine's skill rules (ADR-0097)
cargo xtask ticket-lint                   # tickets/ follows tickets/README.md
cargo xtask pr-size                       # the PR against about 3,000 changed lines; warns, never fails (ADR-0101)
cargo doc --workspace --no-deps          # with RUSTDOCFLAGS="-D warnings"
cargo build -p trpg-app --target wasm32-unknown-unknown
cargo build -p trpg-core --no-default-features --target thumbv7em-none-eabihf  # Playdate, no std (ADR-0070), with the engine crates under it
cargo build -p trpg-content --no-default-features --target thumbv7em-none-eabihf
cargo build -p trpg-ui --no-default-features --target thumbv7em-none-eabihf
cargo clippy -p trpg-playdate --features sdk --target thumbv7em-none-eabihf -- -D warnings   # the Playdate front end's SDK glue (1031)
cargo clippy -p trpg-playdate --features sdk,bench --target thumbv7em-none-eabihf -- -D warnings   # and its bench build (1055)
cargo deny check
cargo machete
typos
```

A change to the engine pin also runs `cargo xtask skill-pack` (after the
pin has moved: it builds with the new pin) and commits what it writes in
`.claude/skills/`; then run `pwsh -File scripts/sync-agent-skills.ps1`
and commit the Codex loaders too. The `engine-pin` skill says when.

**The gates never read the bought art** (ADR-0040): run them exactly as
above, without `--features private-assets`, whether or not
`assets-private/` is on the machine. A ticket that changes files in
`assets-private/game/` also runs the one test that opts in, and looks at
the result in a build:

```bash
cargo test -p trpg-content --features private-assets --test it private_assets::
cargo run -p trpg-app --features private-assets
```

## Snapshots (insta)

`shuyi-gates` *Snapshots* holds the rule. Here a failing snapshot test
writes `*.snap.new`: read it, then `cargo insta accept` if the change was
meant. `cargo insta review` is interactive and won't work in this
environment.

## Mutation testing (CI only)

CI's `mutants (diff)` check (`.github/workflows/mutants.yml`, ADR-0073,
ADR-0100) runs `cargo mutants` on every PR's diff, split over up to 20
`mutants (diff, shard N)` jobs of at most about 60 mutants each; the `MISSED` lines are in the log of each red
shard. `mutants (diff, baseline)` runs the tests once on the unmutated
tree; when it is red, the tests fail under nextest without any mutant.
CI runs the tests with `cargo-nextest`, which does not run doc tests: a
mutant only a doc test would catch counts as missed, so cover it with a
unit test. A new test that takes 5 s or more goes in the slow list of
`.config/nextest.toml` (ADR-0073), so caught mutants don't wait for it.

Don't run it locally (Nick's machine): run the gates above, push, open
the PR and watch CI. Only to reproduce a CI mutants failure, scope it to
the mutants in question, diffing against `origin/main` (the local `main`
may be stale). On PowerShell, write the diff to a file first:
`git diff origin/main > $env:TEMP\pr.diff; cargo mutants --in-diff $env:TEMP\pr.diff -f <file>`.

- `MISSED` mutants in code you wrote = your tests don't check that
  behaviour. Add an assertion that would fail under that mutant.
- `#[mutants::skip]` only with a comment explaining why the code is
  untestable (e.g. `Debug` formatting), never to make the gate pass.
