---
id: "1194"
title: Gate the game's skills with check-skills
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: ["engine#2559"]
nick_input: none
completed: 2026-10-25
---

# 1194 — Gate the game's skills with check-skills

## Context

1192 took Shuyi Engine's skill pack into `.claude/skills/` (ADR-0097) and
made the game's own skills pass `cargo shuyi check-skills .claude/skills`
on every rule but one: each of the game's 13 own skills (`ask-nick`,
`work-ticket`, …) fails "no worked example (example.ron)". A worked
example is a script the engine runs on a fresh game, so a skill about
this repository's workflow can't have one. engine#2559 makes the check ask
for one only from the pack's skills. Once the pin is past it, the check
passes here and can be a gate.

## Nick input

None.

## Scope

**In:**
- Moving the engine pin past engine#2559 (`engine-change` skill §7, with
  the `engine-pin` skill's review and ledger section), unless a pin move
  has already passed it.
- `cargo shuyi check-skills .claude/skills` in the `run-gates` skill's
  list and as a step in `.github/workflows/ci.yml` beside `cargo xtask
  skill-pack --check`.

**Out (do not do):**
- Editing a `shuyi-*` skill (the engine's).
- Giving the game's skills worked examples or a `waits_for`.

## Implementation steps

1. Check engine#2559 is in the engine's `tickets/done/` and the pin is
   past it (`work-ticket` §1); if not, move the pin.
2. Run `cargo shuyi check-skills .claude/skills`; fix anything it reports
   in the game's own skills.
3. Add the command to `run-gates` and to CI's `tickets` job.

## Acceptance criteria

- [x] `cargo shuyi check-skills .claude/skills` passes.
- [x] It is in the `run-gates` skill and in CI.
- [x] `cargo xtask ticket-lint` passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None new (no Rust code); the CI step above.

## Completion notes

- Moved the engine pin b4591a81 → 555bba71 (the engine's `main`, 2 PRs:
  #114 and #115, ticket 2559), with its section in `docs/engine/pins.md`.
  No breaking changes; `cargo xtask skill-pack` changed no pack skill.
- `cargo shuyi check-skills .claude/skills` now passes on all 28 skills
  with nothing to fix in the game's own. It is in the `run-gates` list
  after `skill-pack --check`, and a step after it in CI's `tickets` job.
- It notes that `waits_for` ids go unchecked without `--tickets`; only the
  pack's skills have `waits_for`, and they name engine tickets, so the
  game's `tickets/open` isn't passed.
- No follow-up tickets. Engine friction: none, no Rust code.
