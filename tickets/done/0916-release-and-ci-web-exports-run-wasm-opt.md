---
id: "0916"
title: Release and CI web exports run wasm-opt
type: infra
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-13
---

# 0916 — Release and CI web exports run wasm-opt

## Context

Filed by the engine pin move of 2026-11-10 (`docs/engine/pins.md`,
engine #331, engine ticket 3099, engine ADR-0094). A release `cargo
shuyi export web` now runs Binaryen's `wasm-opt -Oz --strip-debug
--strip-producers` and **fails with `SH1003`** when `wasm-opt` isn't
installed, unless `shuyi.ron`'s `export` part says `wasm_opt: optional`.
Before, it skipped `wasm-opt` silently when it was missing.

Neither of the game's runners installs Binaryen: CI's `export` job
(`.github/workflows/ci.yml`, `cargo shuyi export web --release
--without private-assets`) and `release.yml`'s `web` leg. So the pin
move set `export.wasm_opt: optional` in `shuyi.ron` to keep the
behaviour unchanged: the web package ships the Wasm as the build left
it, with a warning.

That leaves a smaller, stripped Wasm on the table: `-Oz` shrinks the
download for the web build Nick plays and the itch.io page (0901,
0915), and `--strip-producers` takes the build's names out of the
package (engine ADR-0094; the 0914 demo's reasons). The engine's own CI
web leg installs Binaryen since #331; copy how it does.

## Nick input

None. Verified by CI.

## Scope

**In:**
- Install Binaryen in CI's `export` job and in `release.yml`'s `web`
  leg (pinned version, as the other tools are pinned).
- Remove `wasm_opt: optional` (and its comment) from `shuyi.ron`'s
  `export` part, so a runner without it fails loudly.
- The `bench` job reads the exported web package (`needs: [changes,
  export]`); its web size numbers drop. Update `docs/targets.md`'s
  numbers if it records them; never raise a budget.
- `docs/releasing.md`: the release machine needs Binaryen for a local
  release web export.

**Out (do not do):**
- `pages.yml`'s `cargo shuyi web --release --debug-tools` build (a debug
  build; it only warns).
- Nick's machine: nothing is installed there by this ticket.

## Implementation steps

1. Read engine #331's workflow change (its CI web leg) and
   `docs/targets/web.md` or the book's *Targets and export* chapter for
   the install hint `SH1003` prints.
2. Add the install step to both workflows; remove the `optional` line.
3. Push and read the `export` job's log: `wasm-opt` ran, and the
   package's Wasm has no `name` or `producers` section.

## Acceptance criteria

- [x] CI's `export` job runs `wasm-opt` (its log says so) and passes
      without `wasm_opt: optional` in `shuyi.ron`.
- [x] `release.yml`'s `web` leg installs Binaryen the same way.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- CI: the `export` job (and `bench`, which reads its package).

## Completion notes

- A composite action, `.github/actions/binaryen`, installs Binaryen's
  `wasm-opt` at a pinned release (`version_133`), refused unless the
  tarball's SHA-256 matches, into `/usr/local/bin` (the engine looks on
  `PATH`; writing `GITHUB_PATH` is zizmor's github-env finding), like the
  `butler` action pins butler. The engine's own CI uses Ubuntu's `apt
  install binaryen`, which isn't pinned, so this copies its approach (a
  Binaryen on the runner) but not its install line.
- CI's `export` job and `release.yml`'s `web` leg use it; `shuyi.ron`'s
  `export` part no longer says `wasm_opt: optional`.
- **Deviation:** `demo.yml`'s `web` leg exports `--release --edition demo`
  too (on every PR touching `assets/`, `crates/content/` or `shuyi.ron`),
  so it would have failed with `SH1003`; it installs Binaryen the same way.
- `docs/releasing.md`'s *One package by hand* says a release web package
  needs `wasm-opt` on `PATH` (MSYS2's `mingw-w64-x86_64-binaryen`).
- `docs/targets.md`'s web package numbers: see the PR's `bench` run;
  updated if the size moved. No budget changed.
- Nothing for Nick: nothing is installed on his machine, and the game plays
  the same.
