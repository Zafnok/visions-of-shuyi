---
id: "1273"
title: "The content-change skill keeps only the game's rules on top of shuyi-private-files"
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-15
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

- `content-change` now opens by sending the reader to `shuyi-private-files`
  for every shared step (the checkout, the `--no-track` branch, never
  pushing `main`, pushing by full name, the two pull requests, `--pin`
  after the merge, two pull requests moving the pin, never in public) and
  keeps only the game's: a table mapping the template's names to the
  game's, the bought art left out, the ticket in branch, commit and title,
  the pull requests' links, the pin on the private branch while both are
  open (the game's own rule: the template's CI waits on the old pin),
  what the pin carries (`-- content story`, the `[NNNN]` check), a
  `story/`-only change moving the pin, `check-leaks` (ADR-0118), the
  artifact rule, no bare mention, and the dialogue pause.
- Names: `private-assets` reads `checkout` and `pin_file` from
  `shuyi.ron`'s `private_assets`, so the pack's commands work here as
  written; only its prose says `private/` and `private.rev`, which the
  table maps.
- Beyond the scope list: `story-import` pointed at `content-change` §5
  and §6–§7, which no longer exist; it now names the sections. AGENTS.md's
  map row lists `content-change` with `keyboard-input` and `run-gates`.
- `check-skills` (the description had to fit 1,024 characters) and
  `sync-agent-skills.ps1 -Check` pass. No follow-up tickets. No gameplay
  rules decided.
