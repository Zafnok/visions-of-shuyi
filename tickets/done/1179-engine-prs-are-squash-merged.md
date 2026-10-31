---
id: "1179"
title: "Engine PRs are squash-merged, and engine-pin takes only a commit on the engine's main"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-10-21
---

# 1179 — Engine PRs are squash-merged

## Context

Nick, 2026-10-21: squash merging is better for the engine's PRs, and he
kept being reminded to use a merge commit. The reminder came from this
repository: the `engine-change` skill (step 3.5 and the brief in §6),
`docs/engine/README.md`, and the message `cargo xtask engine-pin` prints.

Their reason was that the pin must be a commit on the engine's `main`. A
squash merge also leaves one commit on `main` per PR, and the skill
already pins `main` after the merge (step 6), so the rule bought nothing.
The engine's repository now allows only squash merges, titled with the
PR's title.

What a squash merge does change: the PR branch's commits never reach
`main`, and once the branch is deleted they can't be fetched. So
`engine-pin` with no commit must check that the checkout's commit is on
the engine's `main`, not just on some pushed branch.

## Nick input

None (his request above).

## Scope

**In:**
- The `engine-change` skill and `docs/engine/README.md` say engine PRs
  are squash-merged.
- `cargo xtask engine-pin` with no commit refuses a commit that isn't on
  `origin/main` of the checkout, and no longer says "merge commit".

**Out (do not do):**
- The engine repository's own settings (done there, not a file here).
- `engine-pin <commit>`: a commit given by hand is still taken as it is.

## Implementation steps

1. Reword step 3.5 and §6 of `.claude/skills/engine-change/SKILL.md`.
2. Reword `docs/engine/README.md`.
3. In `crates/xtask/src/engine.rs`, check `origin/main` contains the
   checkout's commit; fix the usage text and the success message.

## Acceptance criteria

- [x] Nothing outside `tickets/done/` asks for a merge commit on an
      engine PR.
- [x] `engine-pin` with no commit fails on a commit that is only on a
      branch.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None new: the check runs `git` on the local checkout, which the tests
  don't have (as with the "isn't pushed" check it replaces).

## Completion notes

Done as scoped. Engine PRs are squash-merged, and `engine-pin` with no
commit takes only a commit on the engine's `main`.
