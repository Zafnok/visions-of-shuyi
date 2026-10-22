---
id: "0514"
title: "Playtest CI plays only when a balance lever changed"
type: infra
milestone: M4 Enemy AI
model: sonnet-5
effort: low
status: done
blocked_by: ["0506"]
nick_input: none
completed: 2026-10-22
---

# 0514 — Playtest CI plays only when a balance lever changed

## Context

0506 added `.github/workflows/playtest.yml`: the persona bots play every
battle in `assets/battles` 100 times each, and the job fails only when a
battle is blocked. It played on any pull request touching `assets/`,
`crates/core/`, `crates/bots/` or the engine pin, and on every push to
`main`. A full run is about 25 minutes on CI's runners, and most of
`assets/` (dialogue, art, audio, fonts, text, credits, tips, palettes)
can't change a battle's result.

Nick (2026-10-22): "playtest CI should watch only game balance levers
(maps, numbers, equipment, spells, etc) before deciding if it should run
or not, it should not run on every PR."

Game side: which files are balance levers is this game's knowledge. (An
engine-provided playtest workflow that takes such a list is part of the
engine-tickets work that followed 0506.)

## Nick input

None.

## Scope

**In:**
- `.github/playtest-levers.sh`: the list of balance levers; reads changed
  paths and prints `play=true|false`.
- `detect` in `playtest.yml` uses it for pull requests **and** pushes to
  `main`; a `workflow_dispatch` ("Run workflow") always plays.

**Out (do not do):**
- Changing what the playtest does or how it reports.
- A workflow-level `paths` filter (ADR-0014: the workflow always runs and
  reports; the job is skipped by `if:`).

## Implementation steps

1. Write `.github/playtest-levers.sh` with the levers: `assets/battles/`,
   `assets/maps/`, the number tables in `assets/data/` (`ai`, `arts`,
   `characters`, `classes`, `items`, `new_game`, `skills`, `spells`,
   `supports`, `terrain`), `crates/core/src/`, `crates/content/src/`,
   `crates/bots/`, `shuyi-engine.rev`, and the playtest itself (its
   command, workflow and this script). Tests and Markdown never play.
2. `detect`: check out with history; diff the PR's merge commit against
   its base, or the pushed commits (`github.event.before` to `HEAD`; a
   new branch or a lost old tip plays); pipe the paths through the
   script.
3. `docs/playtesting.md`: say when CI plays.

## Acceptance criteria

- [x] A change to dialogue, fonts, audio, art, text, credits, tips,
      palettes, docs, tests or the UI doesn't play.
- [x] A change to a battle, a map, a number table, the rules, content
      loading, the bots, the pin or the playtest itself plays.
- [x] Pushes to `main` follow the same rule; "Run workflow" always
      plays.
- [x] The workflow always reports (ADR-0014).

## Tests required

- The script, fed sample paths and the real diffs of recent merges
  (below). No Rust code.

## Completion notes

Done 2026-10-22, stacked on 0506's PR.

- Checked locally by feeding the script paths: dialogue, fonts,
  credits, tips, palettes, docs, tickets, test files and UI code give
  `play=false`; battles, maps, `classes`/`ai` tables, `core`, `content`,
  `personas.ron`, the pin, the script and the playtest command give
  `play=true`. 0506's own diff plays; `main`'s last five merges (design
  and docs PRs) would not have.
- *Claude's starting rule:* `crates/content/src/` counts as a lever
  (it builds each battle's army and setup); `crates/ui/` and
  `crates/app/` don't. The list is one file to edit if a lever is
  missing.

Engine friction: none, no Rust code.
