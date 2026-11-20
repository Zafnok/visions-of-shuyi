---
id: "0192"
title: "The playdate CI job installs the ARM compiler from a cache, not a slow mirror"
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-20
---

# 0192 — The playdate CI job installs the ARM compiler from a cache

## Context

Ticket 0132 gave the `playdate` job in `.github/workflows/ci.yml` a
20-minute limit and its step *Install the ARM compiler* one of 8
minutes, so a stalled mirror fails fast instead of holding the run for
an hour. It still fails: between 2026-11-19 and 2026-11-20 the step hit
its 8 minutes on PRs #776, #778 and #784, turning `ci-result` red on
changes that had nothing to do with it, and each needed the failed job
re-run by hand while Nick was merging a chain of fixes.

On a good run the step takes 13 to 90 seconds. The first attempt of run
44680674840 (#778) shows where a bad one goes:

```
05:24:25 Get:2 http://us-east-2.ec2.archive.ubuntu.com/ubuntu resolute/universe … binutils-arm-none-eabi … [3265 kB]
05:24:37 Get:3 http://us-east-2.ec2.archive.ubuntu.com/ubuntu resolute/universe … gcc-arm-none-eabi … [67.3 MB]
05:32:21 ##[error]The action 'Install the ARM compiler' has timed out after 8 minutes.
```

`apt-get update` finished in 15 seconds; the EC2 Ubuntu mirror then sent
3 MB in 12 seconds and never finished the 67 MB compiler. The re-run,
on the Azure mirror, fetched all 119 MB in 2 seconds. So the time goes
to whichever mirror the runner draws, every run, for the same four
packages.

## Nick input

None. Nothing a player sees changes; CI's `playdate` job is the check.

## Scope

**In:**
- Keep the four `.deb` files the step installs (`binutils-arm-none-eabi`,
  `gcc-arm-none-eabi`, `libnewlib-dev`, `libnewlib-arm-none-eabi`) in
  the Actions cache with `actions/cache`, keyed on the runner's Ubuntu
  release, so a run that finds them installs them with `dpkg -i` and
  downloads nothing (no `apt-get update` either).
- A run that misses the cache downloads them as before
  (`--no-install-recommends`, 0132), with apt's retries on, into the
  cached folder, then installs the same way.
- If `dpkg -i` can't install them (a new runner image lacking a
  dependency), the step falls back to apt to finish the install, so a
  stale cache never fails the job.

**Out (do not do):**
- Caching anything built: only Ubuntu's public packages go in this
  cache, never `target/` or a build that embeds the private content
  (ADR-0117 §4, ticket 0171; `crates/xtask/src/workflow_guard.rs`).
- Swapping Ubuntu's packages for Arm's own toolchain tarball: a
  different compiler version is a change to the device build, not to
  how it is installed.
- The `docs` and `test` jobs' apt installs: they haven't stalled.

## Implementation steps

1. In the `playdate` job, before *Install the ARM compiler*, a step
   writes the runner's Ubuntu codename (`lsb_release -cs`) to its
   outputs, and an `actions/cache` step (the pinned SHA the job already
   uses for the SDK) caches `~/arm-debs` under a key naming that
   codename and a version to bump by hand.
2. *Install the ARM compiler*: if `~/arm-debs` has no `.deb`, run
   `apt-get update` and `apt-get install --download-only` with
   `Dir::Cache::archives` pointed at it; then `dpkg -i` the files, and
   on failure `apt-get update` and `apt-get install -f`. Print
   `arm-none-eabi-gcc --version`.
3. A comment says why (this ticket, the mirror).

## Acceptance criteria

- [x] On this ticket's PR, a `playdate` run that restores the cache
      spends well under a minute in *Install the ARM compiler*, and the
      device build and both pdx checks pass.
- [x] `cargo test -p xtask workflow_guard` passes (the new cache names
      no `target`).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: none new; the workflow guard's tests cover the cache's path.
- CI: the `playdate` job itself, on a run that misses the cache and one
  that hits it.

## Completion notes

- Done as planned in `.github/workflows/ci.yml`. A step names the
  runner's Ubuntu codename. `actions/cache` (the pin the SDK's cache
  already uses) keeps `~/arm-debs` under
  `arm-none-eabi-debs-<codename>-v1`, and *Install the ARM compiler*
  runs `dpkg -i` on what it holds. It downloads first only when the
  folder is empty, and falls back to `apt-get install -f` if `dpkg`
  can't finish.
- Measured on PR #785, run 44789867303. Attempt 1 missed the cache: the
  install took 51 s on a normal mirror and saved the cache (3 s).
  Attempt 2, re-running only `playdate`, hit
  `arm-none-eabi-debs-resolute-v1`: 2 s to restore and 4 s to install,
  with no `apt-get update` and no mirror. The device build and both pdx
  checks passed both times.
- The cache is per branch: a PR first restores `main`'s, so after this
  merges, the first `main` run of `playdate` fills it once for every PR.
  A new Ubuntu release on `ubuntu-latest` changes the codename, and its
  first run downloads once.
- Gates: `cargo test -p xtask` unit tests pass, the workflow guard among
  them. Locally, `tests/cli.rs` `a_moved_command_answers_through_xtask`
  failed only because a running `trpg-tools.exe` (the shuyi MCP server)
  locks the file it rebuilds; CI's `test` job runs it. No Rust changed.
