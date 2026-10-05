---
id: "0120"
title: "The private-assets fetch test fails now and then on macOS"
type: bug
milestone: M0 Foundation
model: sonnet-5
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-10-04
---

# 0120 — The private-assets fetch test fails now and then on macOS

## Context

On PR #193 (ticket 0805, which doesn't touch `xtask`) the `test
(macos-latest)` job failed once in
`private_assets::tests::a_fetch_puts_the_checkout_at_the_pinned_commit`
(`crates/xtask/src/private_assets.rs`), while Linux and Windows passed and
the same test passed on `main`:

```
git ["clone", "--quiet", "--bare", "src", "remote.git"]: fatal: failed to
copy file to 'remote.git/objects/2e/2635ca…': No such file or directory
```

The test's `Scratch::new` builds a small repository in a temp folder,
commits twice, then makes its "remote" with a local `git clone --bare`. A
local clone copies (or hard-links) the files under `objects/`. The likely
cause, not yet proven: git's automatic maintenance (`gc --auto`, started
in the background by the commits just before) packs and removes loose
objects in `src` while the clone is copying them. Each test already uses
its own folder (its name and the process id), so two tests colliding is
unlikely.

A red check that isn't the PR's fault costs a re-run and a look each time
(ADR-0042: a PR merges on green).

## Nick input

None.

## Scope

**In:**
- Make the scratch repositories of the `private_assets` tests safe from
  background git work, and confirm the cause if it can be reproduced.

**Out (do not do):**
- Changing what `cargo xtask private-assets` does for real checkouts.
- Retrying the test or marking it ignored on macOS.

## Implementation steps

1. Try to reproduce on a Mac runner (a loop of the one test in a draft PR's
   job is enough) and read `GIT_TRACE=1` output for a `gc`/`maintenance`
   process during the clone.
2. Fix where the tests run git (`AS_TESTER` and `g` in the test module):
   pass `-c gc.auto=0 -c maintenance.auto=false` on every scratch command,
   and make the bare clone with `--no-local` (or `--no-hardlinks`) so it
   goes through the pack protocol instead of copying files.
3. If step 1 shows another cause, fix that instead and say so in the
   completion notes.

## Acceptance criteria

- [x] The test passes 50 times in a row on `macos-latest` (a temporary
      loop in CI; removed before merge).
- [x] No test in `private_assets` is skipped, retried or ignored.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the existing `private_assets` tests, unchanged in what they check.
- Property: none.
- Snapshot / integration: none.

## Completion notes

**Cause, confirmed on a Mac runner.** A temporary CI job looped the
`private_assets` tests 150 times on `macos-latest` with `GIT_TRACE` on.
Before the fix, 1 run in 150 failed, this time with `fatal: hardlink
different from source at 'remote.git/objects/info/packs_…'`. The trace
shows the runner's git (2.55, Homebrew) starting `git maintenance run
--auto --quiet --detach` after each commit, and that running `git repack
-d -l --cruft … --write-midx` in the background (7 times in one test run)
while the local `git clone --bare` copied the files under
`src/.git/objects`. The ticket's guess was right.

**Fix** (test module of `crates/xtask/src/private_assets.rs` only):

- `Scratch::new` writes `gc.auto=0` and `maintenance.auto=false` into the
  scratch `src` repository, and gives the same two settings to the bare
  clone with `--config`, so they are stored in `remote.git` too.
- The bare clone is made with `--no-local`, so it goes through git's pack
  protocol instead of copying files.

With the fix the same loop gave 0 failures in 150 runs, and the trace has
no `repack` at all. The loop job was removed before merge.

**Deviation from step 2.** The settings are stored in the two scratch
repositories instead of passed as `-c` on every scratch command. Git does
not hand `-c` settings to the other side of a local push or clone, so
`-c` on the test's `git push` would not stop upkeep in `remote.git`;
settings stored in the repository cover every command run there.

**Left alone (out of scope).** The checkout that the command itself makes
in the tests (`public/assets-private/`) still gets git's normal upkeep,
as a real checkout does. Nothing copies files out of it, and the trace
shows no repack there.

No follow-up tickets. Nothing changes in the game or in what
`cargo xtask private-assets` does. No gameplay rules were decided.
