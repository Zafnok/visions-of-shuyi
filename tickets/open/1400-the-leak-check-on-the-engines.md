---
id: "1400"
title: "The leak check on the engine's: check-leaks and the workflow guard go, shuyi check-leaks runs in CI"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#3443"]
nick_input: none
completed:
---

# 1400 — The leak check on the engine's

## Context

The third engine-friction sweep (1397,
`docs/research/2026-11-27-engine-shaped-code-audit.md`, row C1) found
that the game's private-files checks name nothing of the game. The
checks, and the files they live in:

- No tracked file quotes a line of a `.dlg` script or a row of a `.map`
  (`crates/xtask/src/leaks.rs`, ticket 0157, ADR-0118, about 325
  production lines). The game's layer order is in `content_layer.rs`,
  about 65 lines.
- No workflow job with the private files caches `target/` or uploads
  an artifact (`crates/xtask/src/workflow_guard.rs`, 388 lines, compiled
  only for its tests).

Engine ticket 3443 (`Zafnok/shuyi-engine`) adds `shuyi check-leaks`,
whose settings live in `shuyi.ron`'s `private_assets`. Each file this
ticket deletes is marked
`// engine-friction: engine#3443, moves (game 1400 deletes it)`.

## Nick input

None.

## Scope

**In:**
- Move the pin past engine 3443 if it is behind (`engine-change` §7).
- Set `private_assets.leaks` in `shuyi.ron` to today's values:
  - phrase 7, short 4, row 12;
  - unsearched `tickets/`;
  - the allowed uploads of `workflow_guard::ALLOWED_UPLOADS`.
- In CI, `ci.yml` runs `cargo shuyi check-leaks` in place of
  `cargo xtask check-leaks`.
- Delete `leaks.rs`, `content_layer.rs` and `workflow_guard.rs`, their
  tests, and their lines in `crates/xtask/src/main.rs` and `lib.rs`.
  `content_layer` has no other user (`grep -rn content_layer crates`).
- Fix the docs that name the old command: `AGENTS.md`, the
  `content-change` skill, ADR-0118's "how it is checked" line and
  `run-gates`.

**Out (do not do):**
- Changing what counts as a leak. If the engine's rules differ from the
  game's, say so in the completion notes, and keep the stricter rule
  through the settings.

## Implementation steps

1. Write the engine-friction estimate: about 15 lines added (config, CI
   line), about 800 removed.
2. Move the pin, if needed. Set the config.
3. Run `cargo shuyi check-leaks` locally with the private files checked
   out (`cargo shuyi private-assets`). It must pass on `main`'s files.
4. Plant a 7-word quote of a private line in a scratch file. It must
   fail, naming the place, not the words. Then remove the scratch file.
5. Delete the old code and update the docs.

## Acceptance criteria

- [ ] `grep -rn "check-leaks\|workflow_guard\|content_layer" crates .github`
      finds only the engine command in `ci.yml`.
- [ ] `cargo shuyi check-leaks` passes in CI on the PR.
- [ ] The planted quote of step 4 fails it locally (noted in the
      completion notes).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None new: the engine's tests cover the check. The game's CI run is
  the proof.

## Completion notes

*(Filled in by the session that completes the ticket.)*
