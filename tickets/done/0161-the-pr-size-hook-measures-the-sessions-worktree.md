---
id: "0161"
title: The pr-size hook measures the session's worktree, not the main checkout
type: bug
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-11
---

# 0161 — The pr-size hook measures the session's worktree, not the main checkout

## Context

ADR-0101 (ticket 0135) added `cargo xtask pr-size` and a Claude Code
`PreToolUse` hook in `.claude/settings.json` that runs `pr-size --hook`
before `git commit`, `git push` and `gh pr create`.

On 2026-11-11 the engine pin PR (Zafnok/visions-of-shuyi#641, branch
`engine-pin/85e8d5c5`, 64 changed lines) got "this PR is at 4,127 changed
lines, over the limit of about 3,000" on every Bash call. The same warning,
with the same number, fires in every session that works in a worktree under
`.claude/worktrees/`.

Root cause:

1. **Wrong repository.** The session runs in a worktree, but the hook
   process runs in the main checkout `D:\tactical-rpg`. `cargo xtask` there
   builds that checkout's xtask, and `repo_root()` (`CARGO_MANIFEST_DIR`)
   is the main checkout. Its branch is `main`, level with `origin/main`, so
   what gets counted is its untracked files: `test-1261.log` (3,854 lines),
   `pin-report-1261.md` (135), an open ticket (126) and `.codex/config.toml`
   (12). That adds up to 4,127.
2. **Wrong branch, so no exemption.** The branch is read in that same
   checkout (`main`), so an `engine-pin/` or `sonar/` session never gets
   its exemption (ADR-0101 §5), even though `Verdict::of` handles it.
3. **Fires too often.** The hooks' `if` rules (`Bash(git commit*)` …) also
   match commands that Claude Code can't analyze: variables, `$(…)` and
   brace groups. So all three hooks fire on such a call, and each prints
   the warning.

The diff itself is right: the merge base, the three-dot base and the
`linguist-generated` pathspec are all correct, and the existing
`changed_files_count_a_real_diff` test covers them.

## Nick input

None. Repository tooling; verified by xtask's tests.

## Scope

**In:**
- `pr-size --hook` reads the hook's JSON input from stdin. It measures the
  git repository that contains the input's `cwd` (the session's directory)
  and reads the branch there.
- With `--hook`, it stays silent unless `tool_input.command` really runs
  `git commit`, `git push` or `gh pr create`.
- A test that rebuilds the miscount: a "main checkout" with a large
  untracked file, and a worktree whose diff is small, measured from a hook
  input naming the worktree.

**Out (do not do):**
- Turning the warning into a failure, or changing the limits.
- Changing CI's call (`--base HEAD^1 --branch "$HEAD_REF"`): it runs in
  the PR's own checkout and was right.
- Rewriting the hook as a shell script (ADR-0101 rejected that).

## Implementation steps

1. `crates/xtask/src/pr_size.rs`: add `HookInput { cwd, command }` and
   `hook_input(json: &str) -> HookInput`, which reads `cwd` and
   `tool_input.command` and tolerates missing fields. Add
   `runs_a_pr_step(command) -> bool`, true when the command contains
   `git commit`, `git push` or `gh pr create`.
2. Add `repo_of(dir) -> Result<PathBuf, String>`: `git rev-parse
   --show-toplevel` run in `dir`.
3. `crates/xtask/src/main.rs` `pr_size`: with `--hook`, read stdin. If the
   command is not a PR step, print nothing. Otherwise measure
   `repo_of(cwd)` (fall back to `repo_root()` when there is no `cwd`).
   Read the branch in that same repository. If git fails, print nothing
   and exit 0 (the hook never gets in the way).

Expected size: about 60 production lines and 80 test lines.

## Acceptance criteria

- [x] `pr_size::tests::the_hook_measures_the_sessions_worktree` builds a
      main checkout with a 3,001-line untracked file and a worktree on an
      `engine-pin/` branch, and one on a ticket branch, each with a 3-line
      change. Measured from a hook input naming the worktree, it counts 3
      lines; the `engine-pin/` one is exempt.
- [x] `hook_input` and `runs_a_pr_step` have unit tests (missing fields,
      `cd x && git commit -m y`, `git status`, `gh pr create`).
- [x] Without `--hook`, `pr-size` behaves as before; CI's call is unchanged.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: hook input parsing, command matching.
- Integration: throwaway git repository with a worktree (as
  `changed_files_count_a_real_diff` does).

## Completion notes

- `pr-size --hook` now reads the hook's JSON on stdin
  (`pr_size::hook_answer`). It measures the repository of the input's
  `cwd` (`git rev-parse --show-toplevel`) and reads the branch there, so
  a worktree session counts its own diff and an `engine-pin/` or
  `sonar/` worktree keeps its exemption. It answers only when the command
  runs `git commit`, `git push` or `gh pr create`. With no input it
  measures xtask's own checkout as before. A git error with `--hook` is
  silence and exit 0. Without `--hook` nothing changed, including CI's
  call.
- Checked by hand with the built xtask and real hook input. The main
  checkout's `cwd` gives 4,127, as before. The engine pin worktree
  (`engine-pin-4f3c20`) gives nothing (exempt). The same worktree with
  `--branch t1` warns.
- **Takes effect once the main checkout has this commit.** The hook
  process runs in the main checkout (`D:\tactical-rpg`) and builds that
  checkout's xtask, so the old one keeps reporting 4,127 until `main` is
  pulled there. Sessions started before that still get the old warning.
- Deviation: step 4 (a note in ADR-0101) was dropped. Accepted ADRs
  aren't edited (work-ticket *Don'ts*), and ADR-0101's text ("runs
  `pr-size --hook` before `git commit`, `git push` and `gh pr create`")
  is still accurate.
- The `if` rules in `.claude/settings.json` stay. Per the Claude Code
  hooks docs, a rule that names more than the command name also fires on
  commands with `$VAR`, `$(...)` or ones it can't analyze. That is
  best-effort filtering; the check in xtask makes it exact.
- Engine friction: none (repository tooling, no game code).
