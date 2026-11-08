---
id: "0146"
title: "Dependabot and green PRs don't wait unseen: a dependency-upkeep skill and a daily list"
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0146 — Dependabot and green PRs don't wait unseen: a dependency-upkeep skill and a daily list

## Context

The retro of 2026-11-08 found PRs that waited with nobody looking:

- **#561 (the actions group)** had been blocked for 2 days. Its
  `playdate` job timed out after 8 minutes at "Install the ARM compiler"
  (an `apt-get` step the bump doesn't touch), and nobody re-ran it.
  #487, the previous actions group, was all green and sat unmerged for 7
  days until Dependabot closed it as superseded. #560 (ron) was merged in
  passing, with no record.
  - The game has no `dependency-upkeep` skill.
  - Its `AGENTS.md` has no exception for Dependabot PRs. The engine has
    both (engine ADR-0037).
- **Green ticket PRs were skipped** in a merge batch with nothing to say
  why. #570 was green and free of conflicts from 2026-11-07. #569 was
  green, then conflicted twice in `game_tools/plays.rs` as other PRs
  merged around it.

The engine's twin is engine#2057.

## Nick input

None. Whether green PRs merge themselves is engine#2060's question for
Nick, asked for both repositories.

## Scope

**In:**
- A `dependency-upkeep` skill for the game, from the engine's
  (`github.com/Zafnok/shuyi-engine/blob/main/.claude/skills/dependency-upkeep/SKILL.md`),
  with the game's gates. It re-runs a job once when the failure is in a
  step the bump doesn't touch. `AGENTS.md` gets the exception in hard
  rule 1.
- A scheduled workflow (daily) keeping one issue labelled `pr-waiting`.
  It lists Dependabot PRs open more than 24 hours, and ticket PRs green
  for more than 12 hours with no `held:` comment, with their failing
  jobs or merge state. It closes the issue when the list is empty.
- `work-ticket`'s "choose the ticket" step reads that issue first.

**Out (do not do):**
- Auto-merge (engine#2060).
- The ARM install fix (0150).

## Implementation steps

1. Copy and adapt the engine's skill into `.claude/skills/dependency-upkeep/`,
   then run `pwsh -File scripts/sync-agent-skills.ps1`.
2. Write `.github/workflows/pr-waiting.yml` with `schedule` and
   `workflow_dispatch`, least permissions, and zizmor clean.
3. Update `AGENTS.md` and `work-ticket`.

## Acceptance criteria

- [ ] `.claude/skills/dependency-upkeep/SKILL.md` exists, and its loader
      passes `-Check`.
- [ ] A dispatch run writes the issue listing the waiting PRs (link in
      the notes).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Workflow: one dispatch run.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
