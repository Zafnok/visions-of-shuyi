---
id: "1273"
title: "The content-change skill keeps only the game's rules on top of shuyi-private-files"
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1273 — The content-change skill keeps only the game's rules on top of shuyi-private-files

## Context

The engine pin move of 2026-11-15 passed engine PR #426 (engine ticket
2593): the engine's skill pack now has `shuyi-private-files`, copied
into `.claude/skills/` by `cargo xtask skill-pack`. It says how a game
changes its private files: a branch in the private checkout made with
`--no-track`, the private repository's pull request first, then the
game's that moves the pin with `private-assets --pin`, two pull
requests that both move it, and what never goes in public.

The game's `content-change` skill does the same job for its own layout
(`assets-private/content/` and `story/`, `assets-private.rev`, ADR-0117)
in 181 lines of its own. `keyboard-input` and `run-gates` already hold
only the game's rules on top of `shuyi-input` and `shuyi-gates`
(ADR-0097); `content-change` should do the same, so a fix to the
engine's steps reaches the game at the next pin.

The two use different names (the template's `private/` and
`private.rev`, the game's `assets-private/` and `assets-private.rev`):
check how far `shuyi-private-files` reads them from `shuyi.ron`, and
keep the game's names in `content-change`.

## Nick input

None.

## Scope

**In:**
- `content-change` points to `shuyi-private-files` for the steps they
  share and keeps what is the game's: its paths and pin file, the
  bought art left out (`game/`, `test/`), `check-leaks` (ADR-0118),
  the story documents and the dialogue pause.
- Its description says when to read which, so an agent changing the
  game's content never follows the template's paths alone.
- `pwsh -File scripts/sync-agent-skills.ps1` and `cargo shuyi
  check-skills .claude/skills`.

**Out (do not do):**
- Editing `shuyi-private-files` (a pack skill, never edited here).

Estimate: about 120 lines changed, most of them removed.

## Implementation steps

1. Read both skills side by side and list each step of `content-change`
   that `shuyi-private-files` states.
2. Replace those with a pointer; keep the game's names and rules.
3. Sync the loaders and `run-gates`.

## Acceptance criteria

- No step of `shuyi-private-files` is restated in `content-change`.
- `check-skills` and the sync check pass.

## Tests required

None beyond the gates (`check-skills` and the sync check).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
