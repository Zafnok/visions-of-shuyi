---
id: "1131"
title: "Move the carved engine code to the Shuyi Engine repository with its history; the game pins it by revision"
type: infra
milestone: "Engine: carve"
model: opus-5.5
effort: high
status: todo
blocked_by: ["1118"]
nick_input: none
completed:
---

# 1131 — Move the engine code to Shuyi Engine

## Context

ADR-0063 §3 and §8. The engine's repository,
[`Zafnok/shuyi-engine`](https://github.com/Zafnok/shuyi-engine), has
existed since 2026-10-14. It is public, dual-licensed MIT OR
Apache-2.0, and holds the engine's ADRs, tickets, decisions and skills.
The code is carved in this repository's `engine/` folder (1102–1118).
When the first example runs on engine crates alone (1118), the code moves
there with its history, and the game uses it like any other dependency.

Moving the code is what makes it MIT OR Apache-2.0: Nick owns it and
chose that license (the engine repository's `docs/decisions.md`). This
repository's `LICENSE` is not changed.

The engine repository's MVP tickets wait for this one (`game#1131` in
their `blocked_by`): 2002, 2201, 1129, 1133, 1134 and 1132.

## Nick input

None. The repository exists, and Nick's `gh` login can push to it.

## Scope

**In:**
- **Measure first.** Over the last 30 days, how many merged PRs here
  changed a file under `engine/`? Use `gh pr list --state merged
  --search "merged:>=<date>"` and `gh pr view --json files`. Write the
  count in the Completion notes. If it is 1 in 5 or more, also do the
  local-checkout step below (ADR-0063 §8).
- **Move with history.**
  1. Run `git filter-repo --subdirectory-filter engine` on a fresh
     clone.
  2. Push the result to a branch of the engine repository.
  3. Merge that branch into the engine repository's `main` with
     `--allow-unrelated-histories`, through a PR there.
  4. Resolve any clash with the files already there (`README.md`,
     `docs/`): the engine repository's versions win, and the moved
     files' content is merged into them.
- **CI for the engine repository.** Add the code gates to the CI that
  engine ticket 2002 started:
  - fmt;
  - clippy pedantic;
  - tests and docs;
  - the web build of the examples;
  - `cargo deny` and machete;
  - the mutation gate in shards (ADR-0047);
  - Pages for the examples.

  Update its `run-gates` skill.
- **This repository:**
  - Delete `engine/`.
  - Make the game's crates depend on the engine by git URL and revision.
    `shuyi-engine.rev` holds the pinned revision (like
    `assets-private.rev`, ADR-0040), and `cargo xtask engine-pin`
    updates it.
  - Retire `check-engine-boundary` (1102): the boundary is now a
    repository wall.
  - Update `CLAUDE.md` and the `work-ticket` skill: a ticket that needs
    an engine change is an engine PR first, then a game PR that moves
    the pin.
- **The local checkout** (only if the measure says so):
  - `cargo xtask engine` checks the engine repository out at the pinned
    revision into a git-ignored `shuyi-engine/` folder.
  - A Cargo `[patch]` in `.cargo/config.toml` points the game at that
    folder, so one working copy edits both.
  - CI never uses the patch.

**Playdate:** the Playdate port (1025's tickets, 1027–1053) builds
`core`, `content` and `ui` without `std` and adds a second front end,
`crates/playdate`. If those tickets have landed, code this ticket moves
keeps its default `std` feature and stays in the `no-std` CI build, and
`cargo xtask playdate` keeps building (ADR-0070 §6).

**Out (do not do):**
- the engine's MVP tickets (they are the engine repository's);
- publishing to crates.io. Write an engine ticket for the first release
  if none exists.

## Implementation steps

1. Measure.
2. Filter, push, and open the merge PR in the engine repository. Its CI
   must be green.
3. Switch the game to the pinned dependency and delete `engine/`. The
   game's gates must pass.
4. Add the local checkout if needed.
5. Update `CLAUDE.md`, the skills, `docs/engine/README.md`, and
   ADR-0063's status line ("code moved on …").

## Acceptance criteria

- [ ] The engine repository's CI is green with the code, and history is
      kept (`git log` on a moved file shows commits from before the
      move).
- [ ] The game builds from the pinned revision, native and web, and its
      gates pass.
- [ ] `shuyi-engine.rev` and `cargo xtask engine-pin` work.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None beyond both repositories' gates.

## Completion notes
