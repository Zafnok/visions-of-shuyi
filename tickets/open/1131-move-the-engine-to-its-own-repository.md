---
id: "1131"
title: "Move the engine to its own public repository; the game pins it by revision"
type: infra
milestone: "Engine: public"
model: opus-5.5
effort: high
status: todo
blocked_by: ["1130"]
nick_input: setup
completed:
---

# 1131 — Move the engine to its own repository

## Context

ADR-0063 §8 sets three conditions for the move:

1. the license and name are applied (1130);
2. an example runs on the engine alone (1118);
3. the API has settled: over the last 30 days, fewer than 1 in 5 merged
   game PRs had to change a file under `engine/`.

Check condition 3 first, by listing the merged PRs and the files they
touched with `gh pr list --state merged` and `gh pr view --json files`.
If it doesn't hold, stop. Write in the Completion notes what the count
was and set `status: blocked` with that reason.

## Nick input

**Setup.** Nick creates the empty public repository
`github.com/Zafnok/<name>` (no README, no license; the history brings
them) and gives the session push access, with the same settings as this
repository's free tier:

- branch protection on `main`, requiring the CI checks;
- Dependabot, CodeQL and secret scanning on.

## Scope

**In:**
- `git filter-repo --subdirectory-filter engine` on a fresh clone, so
  the history of every engine file is kept. Push it to the new
  repository.
- A CI for the engine repository, copied from this one's and cut to the
  engine's crates and examples. It includes the mutation gate in shards
  (ADR-0047) and Pages for the examples.
- This repository:
  - delete `engine/`;
  - the game's crates depend on the engine by git URL and revision, and
    `engine.rev` holds the pinned revision (like `assets-private.rev`,
    ADR-0040);
  - add `cargo xtask engine-pin` to update it;
  - `CLAUDE.md` and the `work-ticket` skill say how a ticket that needs
    an engine change runs: engine PR first, then a game PR that moves
    the pin.
- Publishing to crates.io: the engine crates get `publish = true` and
  version `0.1.0`. Publish only if Nick said "yes" to going public in
  0059.

**Out (do not do):**
- community files and the docs site (1132);
- a crates.io release before 0059 allows it.

## Implementation steps

1. Check condition 3.
2. Do the filter-repo, push, and CI for the new repository, until it is
   green.
3. Switch the game to the pinned dependency and delete `engine/`. The
   game's CI must be green.
4. Update `CLAUDE.md`, the skills, `docs/engine/README.md` and ADR-0063's
   status line ("moved to its own repository on …").

## Acceptance criteria

- [ ] The engine repository's CI is green, with history kept
      (`git log` on an engine file shows commits from before the move).
- [ ] The game builds from the pinned revision, native and web, and its
      gates pass.
- [ ] `engine.rev` and `cargo xtask engine-pin` work.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None beyond both repositories' gates.

## Completion notes
