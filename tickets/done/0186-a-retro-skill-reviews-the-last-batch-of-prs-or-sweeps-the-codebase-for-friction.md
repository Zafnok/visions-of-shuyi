---
id: "0186"
title: A retro skill reviews the last batch of PRs or sweeps the codebase for friction
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-17
---

# 0186 — A retro skill reviews the last batch of PRs or sweeps the codebase for friction

## Context

Two reviews were run by hand on 2026-11-17, each with about 20 agents:

- The retro of merged PRs #616–#735
  (`docs/research/retro-2026-11-17-prs-616-735.md`, PR #744): ten readers
  of 10 PRs each, six cross-cutting lenses, a synthesizer, three skeptics,
  then tickets filed for what survived.
- A friction sweep of the codebase (`docs/research/friction-2026-11-17.md`):
  ten lens finders (touchpoints, engine candidates, built-but-not-backfilled,
  process, tooling, tests, roadmap vs effort, hotspots), a skeptic per
  lens, and a synthesizer.

Nick asked for both to become a repeatable `/retro` skill: go back over the
previous batch of PRs and say what went well and what went wrong. Each run
was set up from nothing; the prompts, the evidence rules and the
"don't re-report what an open ticket covers" step were rediscovered both
times.

## Nick input

None.

## Scope

**In:**
- `.claude/skills/retro/SKILL.md`: two modes, *PRs* (default: the merged
  PRs since the last retro's last PR) and *sweep* (the codebase through
  fixed lenses). Finding where the last retro stopped, gathering PR data
  with `gh`, the fan-out shape, the lens prompts, the evidence rule, the
  skeptic pass, the report's layout and file name, and turning surviving
  findings into tickets through `write-ticket` (or amendments of open
  tickets).
- The sweep's item 5 (`docs/research/friction-2026-11-17.md`), as Nick
  changed it on 2026-11-17: check the previous retro's tickets first, file
  every surviving action item (no cap), grouped into logical buckets, and
  remove a rule for each rule added. The line budgets themselves are
  0189's.
- The `.agents/skills/retro` loader from `scripts/sync-agent-skills.ps1`.
- The skill named in `AGENTS.md`'s map.

**Out (do not do):**
- No xtask command or scheduled run: the skill is run by hand when Nick
  asks.
- No change to the retro or sweep already written.

## Implementation steps

1. Write `.claude/skills/retro/SKILL.md` from the two runs' setups.
2. Run `pwsh -File scripts/sync-agent-skills.ps1`, then `-Check`.
3. Add `retro` to the skills row of `AGENTS.md`'s map.

## Acceptance criteria

- [x] The skill says how to find the PR range, which data each reader
      gathers, the lenses for both modes, the skeptic verdicts, the report
      path and layout, and how findings become tickets.
- [x] `pwsh -File scripts/sync-agent-skills.ps1 -Check` passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: none (a skill document).
- Snapshot / integration: `sync-agent-skills.ps1 -Check`.

## Completion notes

- `.claude/skills/retro/SKILL.md`: PRs mode (the default) and sweep mode,
  with the fan-out, the lenses, the evidence rule, the skeptic verdicts,
  the report layout and filing.
- Deviation: the sweep proposed a cap of 5 new tickets per retro. Nick
  chose no cap (2026-11-17): file every surviving item, grouped into
  logical buckets. The skill says so.
- The same PR carries the first sweep (`docs/research/friction-2026-11-17.md`)
  and its tickets, filed by bucket: new 0187-0191, 0258, 0259, 0335,
  0877, 0878 and 1294, plus amendments to 31 open tickets.
- Follow-up: 0189 (line budgets for the rules text).
- Engine friction: none: no code.
