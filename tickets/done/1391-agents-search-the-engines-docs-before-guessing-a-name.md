---
id: "1391"
title: "Agents search the engine's docs with cargo shuyi docs before guessing a name"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-27
---

# 1391 — Agents search the engine's docs before guessing a name

## Context

Engine PR #606 (engine ticket 3412, engine ADR-0154), in the pin since the
pin move of 2026-11-27 (`docs/engine/pins.md`), adds `shuyi docs <words…>
[--limit N] [--kind K]` and the MCP tool `docs`: a search of the engine's
book, its skills, its examples and its problem codes, as the pinned engine
has them, with each hit's title, path and excerpt and the "how do I" rows
that match. The MCP server's instructions now say to call `docs` before
guessing a name. Engine PR #626 (3413) adds the names other engines use as
aliases (`docs tween`, `docs prefab`).

A new game made with `shuyi new` gets a row for it in its `AGENTS.md`
(the engine's `templates/shuyi-default/base/AGENTS.md`: "run it before
guessing a type's or a function's name"). This game's `AGENTS.md` and
skills don't name it. Agents here read the engine's source in
`~/.cargo/git/checkouts/` or a `shuyi-engine/` checkout to find a type,
and a harness without the MCP server (Codex) never sees its instructions.

## Nick input

None.

## Scope

**In:**
- `AGENTS.md`: one line under *Shared instructions and skills* or the
  repo map's `shuyi.ron` row: `cargo shuyi docs <query>` searches the
  pinned engine's docs, skills and examples; run it before guessing an
  engine type's or function's name, and before reading the engine's
  source.
- The game's skills that send an agent into the engine's code
  (`engine-change`, `engine-friction`, `engine-pin`'s step 4, "Does the
  game already have code doing this job?" and its engine side): the same
  pointer, once each, where they look an engine piece up.
- `scripts/sync-agent-skills.ps1` after the skill edits.

**Out (do not do):**
- The engine's own skill pack (`.claude/skills/shuyi-*`): copied at the
  pin, never edited here (ADR-0097).
- Any change to the MCP server's setup (`.mcp.json`).

## Implementation steps

1. Run `cargo shuyi docs play a sound` and `cargo shuyi docs MapLook` to
   see what it answers at the pin.
2. Add the line to `AGENTS.md`, and the pointer to the three skills.
3. Run `pwsh -File scripts/sync-agent-skills.ps1`, then `-Check`, and
   `cargo shuyi check-skills .claude/skills`.

Expected size: about 15 lines of docs.

## Acceptance criteria

- [x] `AGENTS.md` names `cargo shuyi docs <query>` and says to run it
  before guessing an engine name.
- [x] `engine-change`, `engine-friction` and `engine-pin` point at it where
  they look an engine piece up.
- [x] `sync-agent-skills.ps1 -Check` and `cargo shuyi check-skills
  .claude/skills` pass.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None beyond the gates: a docs change.

## Completion notes

- `cargo shuyi docs play a sound` and `cargo shuyi docs MapLook` both
  answer at the pin (the settings Learn page and `hello-grid`'s sound,
  settings and skin files), so the pointer names them as examples.
- `AGENTS.md`: a paragraph under *Shared instructions and skills* (it
  reaches Codex too, which has no MCP server instructions).
- `engine-change` step 1 (check the engine doesn't have it already),
  `engine-friction` step 5 (outcome 1 or 2 turns on whether the engine has
  the piece) and `engine-pin` step 4 (the engine side of "Does the game
  already have code doing this job?"): one pointer each.
- The Codex loaders didn't change (they read the shared source);
  `sync-agent-skills.ps1 -Check` and `check-skills` pass.
- No code, no follow-up tickets, nothing for Nick.
