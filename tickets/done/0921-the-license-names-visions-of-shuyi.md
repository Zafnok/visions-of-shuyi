---
id: "0921"
title: The license names Visions of Shuyi
type: infra
milestone: M8 Release
model: haiku-4.5 / gpt-6-luna
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-16
---

# 0921 — The license names Visions of Shuyi

## Context

Ticket 0905 renamed the game and the repository from `tactical-rpg` to
Visions of Shuyi, but left `LICENSE` naming `tactical-rpg` because only
Nick edits the license. Nick (2026-11-16) asked for the remaining
mentions of the old name to be fixed, the license included.

## Nick input

None. The change is Nick's request above.

## Scope

**In:**
- `LICENSE`: the title becomes "Visions of Shuyi Source-Available
  License, version 1.0" and the attribution in section 3c becomes
  "Visions of Shuyi by Ibasho Corp.". Nothing else in it changes.
- Two ADR lines that state the present, not history: ADR-0004's binary
  name (`visions-of-shuyi`) and ADR-0008's repository name
  (`Zafnok/visions-of-shuyi`).

**Out (do not do):**
- The storage name `tactical-rpg` (`shuyi.ron`, `app_info.rs`, their
  tests and docs): changing it loses every player's saves (ADR-0116,
  ticket 1199).
- The SonarCloud project key `Zafnok_tactical-rpg` (README badges,
  `sonar-project.properties`, `tools/sonar-triage/`): it must be renamed
  in SonarCloud first.
- `D:\tactical-rpg` paths: the real folder on Nick's machine.
- Done tickets and other ADR text: history.

## Implementation steps

1. Edit the two lines of `LICENSE`.
2. Edit the ADR-0004 and ADR-0008 lines.

## Acceptance criteria

- [x] `grep -n tactical-rpg LICENSE` finds nothing.
- [x] Every other `tactical-rpg` left in the repository is one of the
      "Out" items above.
- [x] `cargo xtask ticket-lint` and `cargo xtask check-mentions` pass; no
      code changed, so the workspace build and tests are left to CI.

## Tests required

- None: no code changes, and no test reads `LICENSE`.

## Completion notes

- `LICENSE` title and the 3c attribution now say Visions of Shuyi.
- ADR-0004 names the binary `visions-of-shuyi`; ADR-0008 names the
  repository `Zafnok/visions-of-shuyi`.
- Filed and done in the same PR, on Nick's word.
- No gameplay rule decided.
