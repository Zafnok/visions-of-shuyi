---
id: "1317"
title: "Every ticket says what the lesser targets get"
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["1314"]
nick_input: none
completed:
---

# 1317 — Every ticket says what the lesser targets get

## Context

Nick (2026-11-22) wants the weaker targets (Playdate, a future GBA)
deprioritized but never forgotten: "several tickets including skills
and build processes to ensure all tickets have lesser build target
considerations to spin off into engine changes and fallbacks etc." 1314
builds the graphics tiers and the fallback check; this ticket makes the
process use them.

Today no non-engine skill but `run-gates` and `keyboard-input` mentions
the Playdate: `write-ticket`, `work-ticket`, `engine-change`,
`ascii-art` and `ask-nick` don't. The tickets 0336, 1311–1316 and 1318
already carry a trial `## Lesser targets` section.

## Nick input

None (process).

## Scope

**In:**
- `tickets/TEMPLATE.md`: a `## Lesser targets` section (what the
  Playdate gets — the same, a fallback from 1314, or nothing with a
  `TODO`; GBA: the `Minimal` tier row or "no target").
- `write-ticket` skill: fill it for every feature, content and tuning
  ticket that touches the look, sound, input or performance; when the
  answer needs an engine change, the ticket follows `engine-change`.
- `work-ticket` skill §3/§4: build the fallback or its `TODO`; check
  with `frame-png` at the lower tier (1314's override) and `cargo shuyi
  bench` for the Playdate where it applies.
- `engine-change` skill: an engine feature that draws or moves says how
  each tier degrades.
- `ascii-art` and `ask-nick`: mockups say what the Playdate shows (or
  that it shows the fallback).
- `cargo xtask ticket-lint`: a **warning**, never a failure (memory:
  warn, don't fail, on process rules), for an open feature/content/
  tuning ticket without the section. Existing tickets are not
  back-filled; the warning applies to tickets created after this one.
- `pwsh -File scripts/sync-agent-skills.ps1` after the skill edits.

**Out (do not do):** back-filling every open ticket; failing the lint.

## Implementation steps

1. Edit the template and the five skills.
2. The lint warning in `crates/xtask` (by ticket number above 1317, or
   by creation date), with a test.
3. Sync the skill loaders; run `-Check`.

## Acceptance criteria

- [ ] The template and skills name the section and 1314's tiers.
- [ ] `cargo xtask ticket-lint` warns (exit 0) on a new feature ticket
      without the section (a test).
- [ ] `scripts/sync-agent-skills.ps1 -Check` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the lint warning in `crates/xtask`.

## Lesser targets

This is the process for them.

## Completion notes

