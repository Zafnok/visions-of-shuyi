---
id: "0168"
title: The writer's mirror gets its first sync, its kit and a test pull request
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["0152"]
nick_input: setup
completed: 2026-11-13
---

# 0168 — The writer's mirror gets its first sync, its kit and a test pull request

## Context

Ticket 0152 built the writer's mirror (ADR-0122): the sync
(`.github/workflows/story-mirror.yml`), the kit
(`.github/workflows/story-kit.yml`), the mirror's own check
(`tools/story-mirror/check.yml`) and `cargo xtask story-import`. Its
workflows run only from `main` and need Nick's setup (0152's *Nick
input*: the private repository `Zafnok/visions-of-shuyi-story`, its
workflow permissions, the `STORY_MIRROR_TOKEN` secret, the ruleset), so
0152 merged with its by-hand runs not done. This ticket does them.

## Nick input

**Setup:** 0152's five steps, if not done yet (the session checks with
`gh repo view Zafnok/visions-of-shuyi-story` and `gh secret list`). Then
one click: **Approve** on the test pull request (step 4), or the session
proves only the refusal.

## Scope

**In:**
- One run of each workflow by hand and the test pull request below, with
  fixes to them if a run fails (in this ticket's PR).

**Out (do not do):**
- Adding a writer (0766), the preview program (0764), the guide (0765).
- Any change to a script's words: the test pull request is thrown away.

## Implementation steps

1. Check the setup is done (above). If the ruleset or the workflow
   permissions can't be read with `gh api`, ask Nick.
2. `gh workflow run story-mirror.yml`; when it is green, check the mirror
   on GitHub: `dialogue/`, `story/`, `design/`, `README.md`,
   `.github/workflows/check.yml`, and the last commit's `Game-Commit:` and
   `Assets-Commit:` trailers.
3. `gh workflow run story-kit.yml`; check the release `kit` holds
   `story-check-linux`, `story-check-windows.exe` and
   `THIRD_PARTY_LICENSES.html`, and its notes name the commit.
4. In a scratch clone of the mirror: a branch that puts a curly quote in a
   line of `dialogue/ch01.dlg`, pushed, and a pull request. The mirror's
   check fails with an annotation on that line. Push a second commit that
   takes it out: the check passes.
5. `cargo xtask story-import <n> --dry-run`: without Nick's approval it
   refuses; with it, the change shown is that line in
   `content/dialogue/ch01.dlg`. Close the pull request and delete its
   branch; nothing is imported.

## Acceptance criteria

- [x] The mirror holds `dialogue/`, `story/`, `design/`, `README.md` and
      its `check.yml`; its last commit has `Game-Commit:` and
      `Assets-Commit:` trailers (linked in the completion notes).
- [x] The release `kit` holds `story-check` for Linux and Windows and
      `THIRD_PARTY_LICENSES.html`.
- [x] A mirror pull request with a curly quote fails the mirror's check
      with an annotation on its line; without it, it passes.
- [x] `story-import --dry-run` refuses it unapproved and shows the line
      when approved (or the refusal only, said so).

## Tests required

- By hand, in the completion notes: the runs above, with links.

## Completion notes

**Done (2026-11-13).**

- **Setup:** all five of 0152's steps were done. The mirror is private
  and was empty, its workflow permissions read only with pull-request
  approval off (`gh api .../actions/permissions/workflow`), the secret
  `STORY_MIRROR_TOKEN` is set, and the ruleset `main` is active on the
  default branch (update, deletion, non-fast-forward) with Repository
  admin as its bypass.
- **Sync:** [run 43548585103](https://github.com/Zafnok/visions-of-shuyi/actions/runs/43548585103),
  green. The mirror holds `.github/workflows/check.yml`, `README.md`,
  `design/`, `dialogue/` (ten scripts and its README) and `story/`. Its
  first commit,
  [eda2106](https://github.com/Zafnok/visions-of-shuyi-story/commit/eda21068effcc10f532605e16de0f7f512e1b5d0),
  has `Game-Commit: aa1aee75…` and `Assets-Commit: 5d2ad7f1…`.
- **Kit:** [run 43548650742](https://github.com/Zafnok/visions-of-shuyi/actions/runs/43548650742),
  green. The release [`kit`](https://github.com/Zafnok/visions-of-shuyi-story/releases/tag/kit)
  holds `story-check-linux`, `story-check-windows.exe` and
  `THIRD_PARTY_LICENSES.html`, and its notes carry the same two trailers.
- **Test pull request:** [mirror PR 1](https://github.com/Zafnok/visions-of-shuyi-story/pull/1)
  put `You’ll` in line 80 of `dialogue/ch01.dlg`.
  - **The first run found a bug.** The check went red
    ([run 43549060455](https://github.com/Zafnok/visions-of-shuyi-story/actions/runs/43549060455))
    but printed no report and set no annotation. A `run:` step's shell
    is `bash -e`, and `set -uo pipefail` leaves `-e` on. So when
    `story-check` exited 1, the step stopped before `cat` and the `sed`
    that writes the annotations. **Fixed** in `tools/story-mirror/check.yml`
    by catching the code (`code=0; ./story-check … || code=$?`).
  - A pull request runs the workflow file from its own branch, so the
    test branch carried the fixed `check.yml`. With the quote, the check
    fails ([run 43549380913](https://github.com/Zafnok/visions-of-shuyi-story/actions/runs/43549380913))
    with one annotation, `dialogue/ch01.dlg:80`, "non-ASCII punctuation '’'; use ' instead".
    A second commit took the quote out and the check passes
    ([run 43549439108](https://github.com/Zafnok/visions-of-shuyi-story/actions/runs/43549439108)),
    with "0 problems in 10 scripts".
  - A local `story-check` built with the private content gave the same
    single problem. Built without it (no `assets-private/`), it also
    lists every story name and character as unknown. The kit is built
    with the content checked out, so the mirror's check doesn't hit this.
- **Import:** `cargo xtask story-import 1 --dry-run` refuses with
  "pull request 1 isn't approved by Zafnok (no review): Nick approves it
  on GitHub first". **Refusal only:** the test pull request was opened
  with Nick's own `gh` login, and GitHub doesn't let an author approve
  their own pull request. No other account had access. The approved path
  stays covered by 0152's tests (fake `gh` answers and the three-way
  apply in throwaway repositories). It runs for real on the first writer's
  pull request (0766). The pull request is closed, its branch deleted,
  nothing imported, and the mirror's `main` is still at `eda2106`.

**Deviations.**

- The mirror's `main` still has the old `check.yml` until this PR
  merges. The merge's push touches `tools/story-mirror/**`, which starts
  the sync and copies the fix there. A sync from this branch would have
  stamped the mirror with an unmerged game commit, so it wasn't run.
- My first test commits came from a Windows clone with
  `core.autocrlf=false` set on the command line. They rewrote every file
  with CRLF, so the PR diff showed every line. I rebuilt the branch with
  LF endings and reran both checks: the runs linked above are from the
  clean branch, whose diff was the one line (and `check.yml`).

**Follow-up tickets:** 0169 (`playtest.yml`'s *Play the battle* has the
same `-e` shape, so a blocked battle loses its report upload).

**For Nick:** nothing to click. No game rule was decided here.
