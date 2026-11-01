---
id: "0138"
title: "Declare target profiles and the rules crate in shuyi.ron"
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0138 — Declare targets and rules in shuyi.ron

## Context

1008's verification ran the engine skill pack's extra checks. The game's
existing `run-gates` commands work, but `cargo shuyi check --target all`
returns SH3003: `shuyi.ron lists no targets`. `cargo shuyi lint --json`
returns no findings while reporting that its rules fence was not checked,
because `shuyi.ron` names no `rules` package. These are configuration
limits, not passes of those checks. The game already verifies its own
builds, crate boundaries, text and input in CI.

The pinned engine supports `Project::targets` (a map of target settings)
and `Project::rules` (a package name). Wire those existing capabilities
into the game. 1199's desktop-target identity check and 0910's exports
need the profiles first; their other changes remain their own jobs.

## Nick input

None. This declares the existing desktop, browser and Playdate targets and
existing pure core; it changes no controls, display layout or game rules.
Tests and the command reports verify it.

## Scope

**In:**
- Valid profiles/settings for the game's existing targets in `shuyi.ron`.
- `rules: Some("trpg-core")`, using the exact syntax of the pinned engine.
- Required project-check/lint commands in the game's gate list and CI,
  with honest remaining limits.

**Out (do not do):**
- App identity and icons (1199), export packaging (0910), new platforms,
  screen redesigns, or weakening the existing core fence or other gates.
- Any 1008 gameplay change or new engine feature.

## Implementation steps

1. Read `.claude/skills/shuyi-gates/SKILL.md`, the pinned engine's
   `crates/cli/src/project.rs`, target contracts and shipped profile
   examples; read the game's current `shuyi.ron` and core boundary tests.
2. Declare the existing targets with the correct profiles/backends,
   frame sizes and controls. Use the engine's supported settings, without
   pretending the custom Playdate front end is an engine backend it isn't.
3. Name `trpg-core` as the rules package. Run lint and inspect findings
   and limits; fix actual wiring errors without loosening the fence.
4. Add the working checks to `run-gates` and CI. Document any capability
   that must still wait for 1199. 1199 and 0910 consume these profiles;
   do not implement their app/export work here.

Estimate: about 40 production lines and 500 total changed lines, based on
existing project settings and xtask command tests; no new screen.

## Acceptance criteria

- [ ] `cargo shuyi check --target all` reaches every declared target,
      without SH3003 or a limit about missing target settings.
- [ ] `cargo shuyi lint --json` runs the rules fence and reports no
      missing-rules-package limit or findings.
- [ ] A test of the project wiring fails when the rules package or a
      declared profile is missing, rather than accepting a limited check.
- [ ] The checks run in CI and the game gate list.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit/integration: project wiring and command reports, in the existing
  xtask test modules (no new root integration-test binary).
- Property/snapshot: none; no game or look changes.

## Completion notes

*(Filled in by the session that completes the ticket.)*
