---
id: "0141"
title: Check the game's real window and browser build with the engine's capture mode
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["engine#2018", "engine#2019"]
nick_input: none
completed:
---

# 0141 — The game's real window and browser checked

## Context

The 2026-11-01 audit of the last 60 PRs in each repository found that
engine PRs #171 and #173 shipped native capture plans and Chromium boot
and capture checks. Game PR #482 passed both, but its ledger classified
them as nothing for the game. This game's CI still builds the web version
and uploads it without opening it. Raster and scene tests cannot prove
that this game's loader boots or its real window shows the same frame.
No actual rendering defect or broken current build is asserted here.

The facilities are already at the game's pin. Reuse
`shuyi-app/capture`, `shuyi_platform::capture::Plan` and the pinned
engine's `tools/web-check/check.py`; do not fork their implementation.
The engine's ADR-0052 describes the comparison and tolerance.

## Nick input

None. Tests and CI verify the result; no new accounts, secrets, bought
art or check after merge. This is validation infrastructure, independent
of the remaining Act 1 content.

## Scope

**In:**
- An opt-in `capture` feature of `trpg-app`, forwarding
  `shuyi-app/capture`, and the corresponding allowed web-build flag in
  `shuyi.ron`. Ordinary builds leave it off.
- Short deterministic plans for the title, a battle and the existing test
  world map, using public placeholder assets and the debug-tools build.
- Linux native captures under Xvfb/Mesa and web boot/capture checks in
  Chromium, failing on the engine checker's errors and mismatches.
- A checkout of engine tooling at exactly `shuyi-engine.rev` in the CI
  job's temporary folder, without enabling the local engine patch.
- Failure artifacts, a bounded timeout and the game's gate instructions.

**Out (do not do):**
- New renderer, browser checker, screenshot comparator or capture protocol.
- Bought assets, real Act 1 maps, audio correctness and Playdate captures.
- Changing game content, gameplay or the engine's tolerance to hide a miss.
- Cross-architecture replay hashes (0142) and scene-data expansion (1202).
  Either can land independently; these plans use existing input and images.

## Implementation steps

1. Verify engine 2018 and 2019 are done and present at the pin; they were
   included by game #482. If the checkout has moved backwards, move the
   pin first with `engine-pin`, including its review and ledger section.
2. Read the engine's native `render` and `web-render` jobs and Hello,
   Grid's capture feature/plans. Add only the game's forwarding feature,
   web flag and plans. Reach the test world map through its current debug
   menu; do not require unfinished story content.
3. In `.github/workflows/ci.yml`, use the existing code-change predicate
   and an always-reporting result job. Resolve `shuyi-engine.rev`, check
   out that exact engine commit below a job-temporary path with no saved
   credentials, and verify its HEAD. Run its checker directly and install
   its hash-pinned Python requirements. Do not call `cargo xtask engine`,
   change the lock, read tools from moving main or copy the checker here.
4. Build native with `--features capture,debug-tools` on `trpg-app` and
   set `SHUYI_CAPTURE`/`SHUYI_CAPTURE_OUT` for each plan under Xvfb. Boot a
   normal web build first, then build with `--capture --debug-tools` and
   place the chosen plan as `capture.ron` in a disposable capture folder.
   Use the engine checker's boot and capture modes at the game's console
   size. Never add a plan to a public export or weaken clean web staging.
5. Keep logs and screenshots on failure. Exercise a deliberately broken
   loader and a failing capture in disposable test output, requiring a
   nonzero result; record the commands and failure evidence in the PR.
6. Add the local commands and CI-only dependencies to `run-gates` and the
   appropriate existing development guide; use the `document` workflow
   if the repository has acquired one by then.

Expected size: about 100–180 workflow/feature lines and three small plans,
under 1,000 changed lines including tests, docs and this ticket's move.

## Acceptance criteria

- [ ] CI boots the game's ordinary web build in Chromium and fails on a
      console/page error, failed request or blank canvas as the engine's
      checker specifies; a deliberately broken loader proves failure.
- [ ] Native and web capture plans reach title, battle and test world
      map, compare their actual pixels with the same-frame raster, and
      pass within the engine's documented tolerance. A failing capture
      proves the gate returns failure without an allowlist exception.
- [ ] Checker files come from exactly `shuyi-engine.rev`; the job fails
      if that revision cannot be checked out. No divergent checker copy
      or local Cargo engine patch is introduced.
- [ ] A normal game/export has no capture feature or `capture.ron`.
      CI saves useful logs/pictures even when a check fails or times out.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the real native and browser runs above on public assets.
- Negative integration: broken loader and failing capture must be caught;
  use the pinned checker's fixtures as well, rather than retesting its
  parsing or pixel arithmetic in game unit tests.
- Gate wiring: the new jobs are included in the CI result, with their
  skip condition and timeouts exercised as existing CI jobs require.

## Completion notes

*(Filled in by the session that completes the ticket: commands, actual
captures and negative checks, deviations and follow-ups.)*
