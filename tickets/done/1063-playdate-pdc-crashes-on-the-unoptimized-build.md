---
id: "1063"
title: "Playdate: pdc crashes packing the unoptimized build's pdex.elf"
type: bug
milestone: Post–Chapter 1
model: implementation
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-10-22
---

# 1063 — pdc crashes on the unoptimized build

## Context

Found while working 1061 (2026-10-18). `cargo xtask playdate` without
`--release` (the `playdate-dev` profile) builds both halves, then the
SDK's `pdc` (3.1.2, Windows) exits with an access violation
(`0xc0000005`) and no `.pdx` is made. With or without
`--private-assets`, so it is not the bought art. Packing a `Source/`
folder with only `pdxinfo` and that `pdex.elf` (about 80 MB, unoptimized
with debug info) crashes it; the same folder with only `pdex.dll` packs.
`--release` (a 45 MB `pdex.elf`) packs fine, and CI only builds that.

`crates/playdate/README.md` says the unoptimized build is "fine in the
Simulator", which it can't be while `pdc` cannot pack it.

## Nick input

None.

## Scope

**In:**
- Find why `pdc` crashes on that `pdex.elf` (its size, its debug
  sections, something in the device link of `playdate-dev`) and make the
  unoptimized build pack: e.g. strip or limit its debug info in the
  `playdate-dev` profile or the device link, or build only the Simulator
  half without `--release` by default.
- Keep `crates/playdate/README.md` true.

**Out (do not do):**
- Changing the `playdate` (release) profile's output.

## Acceptance criteria

- [ ] `cargo xtask playdate` (no `--release`) makes a `.pdx` on Nick's
      machine, or refuses the device half without `--release` with a
      message saying why.
- [ ] An xtask test covers whatever changes in the plan.
- [ ] All gates in the `run-gates` skill pass, and the CI `playdate` job.

## Tests required

- Unit: `xtask playdate`'s plan for the changed build.
- Manual: `cargo xtask playdate` on Windows packs.

## Completion notes

**Closed as a duplicate of 1056, not worked (2026-10-22).** 1056 (#400)
fixed this very crash: not the size or the debug info, but the
per-function `.ARM.exidx.*` sections of an unoptimized build pushing the
ELF past 65,280 sections, which `pdc` 3.1.2 can't read; the debug device
build now strips them after the link (`crates/xtask/src/playdate.rs`,
`device_steps`), and `crates/playdate/README.md` says so. Found by the
retro of the last 50 PRs: 1056's notes said "no follow-up tickets", and
this ticket stayed open in 1064's `blocked_by`, in the roadmap's Lane D
and in 1170. Removed from all three in the same PR.
