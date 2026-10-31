---
id: "0132"
title: "The playdate CI job stops after a time limit and installs only the ARM compiler parts it uses"
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-10-25
---

# 0132 — The playdate CI job times out and installs less

## Context

On 2026-10-26 the `playdate` job of the engine pin PR (#434) hung for
over 70 minutes in its step *Install the ARM compiler* and had to be
cancelled by hand and re-run. On `main` the whole job takes about 4
minutes. The step's log shows why:

```
Get:6 … libnewlib-arm-none-eabi all 4.6.0.20260123-1 [48.5 MB]
Get:7 … libstdc++-arm-none-eabi-newlib all 15:14.2.rel1-1+29 [487 MB]
```

The Azure Ubuntu mirror served the 48.5 MB package in 12 minutes and was
still on the 487 MB one when the job was cancelled. Two things made a
slow mirror cost over an hour:

1. **No time limit.** The `playdate` job in `.github/workflows/ci.yml`
   has no `timeout-minutes`, so GitHub's default of 360 minutes applies.
   No job in `ci.yml` has one; `mutants.yml` (15 and more) and
   `playtest.yml` (150) do.
2. **More than it needs.** `apt-get install -y gcc-arm-none-eabi` also
   installs its *Recommends*, among them
   `libstdc++-arm-none-eabi-newlib` (487 MB), the C++ standard library.
   The device build (`cargo xtask playdate --release --device`, ADR-0070,
   `crates/playdate/README.md`) links a Rust `staticlib` with the C shim
   `crates/playdate/shim/shim.c` and the SDK's C files: C and newlib, no
   C++. So about 90% of each run's download is unused.

## Nick input

None. Nothing a player sees changes; CI's `playdate` job is the check.

## Scope

**In:**
- A `timeout-minutes` on the `playdate` job in `.github/workflows/ci.yml`,
  and a shorter one on its *Install the ARM compiler* step, so a stalled
  download fails fast and the PR's author can re-run it.
- The install takes only what the device build uses: `gcc-arm-none-eabi`
  and `libnewlib-arm-none-eabi` with `--no-install-recommends`, if the
  build and both pdx checks still pass. If that isn't enough for the
  build (a missing library at link time), add the one package the
  linker names, never all the Recommends back.
- A comment in the step saying why recommends are left out (the C++
  library is 487 MB and the build is C).

**Out (do not do):**
- Time limits on the other jobs of `ci.yml` or other workflows (a ticket
  of its own if wanted; the stall seen was this step's).
- Caching the apt packages or the toolchain with `actions/cache`, or
  switching to Arm's own toolchain tarball: only if the smaller install
  still takes more than 2 minutes in CI (then write it in the Completion
  notes and file a follow-up with `write-ticket`).
- Retrying the download in a loop.
- The Playdate SDK's own download and cache (they already work).
- Anything in `crates/playdate` or `cargo xtask playdate`.

## Implementation steps

1. Read the `playdate` job in `.github/workflows/ci.yml` (from the
   comment above `playdate:` to the step *Check pdc packed the converted
   sounds and music (1047)*), and `crates/playdate/README.md` for what
   the device build links.
2. Look up how long the job and the install step take on `main`'s last
   few runs: `gh run list --workflow CI --branch main --limit 5`, then
   `gh run view <id> --json jobs` and the job's steps
   (`gh api repos/Zafnok/visions-of-shuyi/actions/jobs/<job id>`). Write
   them down for the Completion notes.
3. Add `timeout-minutes: 20` to the job (under `runs-on:`; about five
   times its usual 4 minutes, with room for a cold Rust cache) and
   `timeout-minutes: 8` to *Install the ARM compiler*. If step 2 shows
   a cold-cache run near 20 minutes, take the job's limit to twice the
   longest run seen and say so in the notes.
4. Change the step's command to
   `sudo apt-get update && sudo apt-get install -y --no-install-recommends gcc-arm-none-eabi libnewlib-arm-none-eabi`,
   with the comment above it.
5. Push and read the `playdate` job's log: the install step's
   `Need to get … MB` line, its time, and that *Build the device game*
   and both checks pass. If the link fails, add the package it names
   (Scope, In, second bullet) and push again.
6. Run `zizmor .github/workflows/ci.yml` (the `zizmor` job of
   `security.yml` checks workflows) and the `run-gates` gates.

## Acceptance criteria

- [ ] `grep -n "timeout-minutes" .github/workflows/ci.yml` shows the
      job's limit and the install step's.
- [ ] The install step has `--no-install-recommends`, and its log in the
      PR's CI run has no `libstdc++-arm-none-eabi-newlib`.
- [ ] The PR's `playdate` job passes, *Build the device game* and both
      pdx checks included.
- [ ] The Completion notes give the install's download size and time
      before and after.
- [ ] The `zizmor` check passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit / property / snapshot: none (a workflow change; no Rust changes).
- Integration: the PR's own CI run of the `playdate` job is the test.

## Completion notes

- The `playdate` job in `.github/workflows/ci.yml` has
  `timeout-minutes: 20`, and its *Install the ARM compiler* step
  `timeout-minutes: 8`. The install is now
  `apt-get install -y --no-install-recommends gcc-arm-none-eabi libnewlib-arm-none-eabi`,
  with a comment saying why the Recommends are left out.
- **Before** (`main`'s last four green runs, 2026-10-25/26): the job took
  3 m 40 s, 3 m 52 s, 4 m 44 s and 4 m 27 s; the install step 21 s, 24 s,
  63 s and 47 s. The 4 m 27 s run's log: `Need to get 607 MB of archives`,
  `Fetched 607 MB in 12s`, then about 30 s unpacking, most of it
  `libstdc++-arm-none-eabi-newlib` (487 MB). No cold-cache run came near
  20 minutes, so the job's limit stayed at 20.
- **After** (this PR's run, job 121383195662): the job took 3 m 38 s, the
  install step 21 s. `Need to get 119 MB of archives`, `Fetched 119 MB in
  2s`, 997 MB on disk instead of 3235 MB. Four packages: binutils, gcc,
  `libnewlib-arm-none-eabi`, `libnewlib-dev`.
  `libstdc++-arm-none-eabi-newlib` appears only in apt's list of
  recommended packages it skipped; nothing named libstdc was downloaded
  or unpacked. *Build the device game* and both pdx checks passed with no
  extra package. The linker's `.note.GNU-stack` warning on `setup.o` is
  in `main`'s runs too.
- The install step was already short on most runs (21–63 s). What this
  change fixes is the stall, not the usual time: the download is 80%
  smaller, so a slow mirror has much less to send.
- `zizmor .github/workflows/ci.yml` at CI's `min-severity: medium`: no
  findings. Its one low finding (`self-repository`, line 22) is on `main`
  already and untouched here.
- No deviations, no follow-up tickets, no Nick input.
