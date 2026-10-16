---
id: "1056"
title: "Playdate: the debug build's pdc step crashes (0xc0000005); make cargo xtask playdate pack again"
type: bug
milestone: Post–Chapter 1
model: opus-5.5
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1056 — Playdate: the debug build's `pdc` step crashes

## Context

Found while working 1103 (2026-10-16), on `main` too (`66842dc3`), so it
is not 1103's change. On Nick's machine (Windows, SDK 3.1.2 in
`%USERPROFILE%\Documents\PlaydateSDK`), `cargo xtask playdate` (the
`playdate-dev` profile, `crates/xtask/src/playdate.rs:138`) builds and
links both `pdex.elf` and `pdex.dll`, then the SDK's `pdc.exe` exits
with `0xc0000005` (an access violation) and no message:

```
playdate: pdc failed (exit code: 0xc0000005): "...\PlaydateSDK\bin\pdc.exe" "-sdkpath" "...\PlaydateSDK" "...\target\playdate\Source" "...\target\playdate\VisionsOfShuyi.pdx"
```

Running `pdc.exe` by hand on the same `target\playdate\Source`, without
MSYS2 on `PATH`, crashes the same way. `cargo xtask playdate --release`
(the `playdate` profile, what CI runs) packs fine. In the debug build
`pdex.elf` is about 74 MB and `pdex.dll` about 17.7 MB; in the release
build about 39 MB and 6.2 MB. A guess, not checked: `pdc` cannot handle
an ELF or DLL this large, or this much debug info.

ADR-0070 §1 (toolchain), `crates/playdate/README.md` (how to build),
`Cargo.toml` (`[profile.playdate-dev]`).

## Nick input

None.

## Scope

**In:**
- Find what makes `pdc` crash on the debug build and make
  `cargo xtask playdate` (no `--release`) produce a `.pdx` again.
- If the fix is a profile or flag change, say why in a comment beside it.

**Out (do not do):**
- Changing the release (`playdate`) profile or what CI builds.
- Device speed or stack work (1055, ADR-0070 §4).

## Implementation steps

1. Reproduce: `cargo xtask playdate` with `PLAYDATE_SDK_PATH` set; note
   the sizes of `target/playdate/Source/pdex.elf` and `pdex.dll`.
2. Find which file `pdc` chokes on: copy `Source/` and pack it with
   only `pdex.elf` and `pdxinfo`, then only `pdex.dll` and `pdxinfo`.
3. Shrink the culprit and retry, one change at a time, in
   `[profile.playdate-dev]`: `debug = "line-tables-only"` (or `false`),
   `opt-level = 1`; for the DLL, a `-s`/`--strip-debug` link flag in the
   Simulator link in `crates/xtask/src/playdate.rs`. Keep the first
   change that packs, and keep enough line info to read a crash address
   if possible (the release profile keeps line tables for that).
4. If no build setting fixes it, make the debug run say so in plain
   words (and how to use `--release` instead) rather than print only the
   exit code, and note the SDK version in the README.
5. Update `crates/playdate/README.md` if the debug build changes.

## Acceptance criteria

- [ ] `cargo xtask playdate` (debug) produces
      `target/playdate/VisionsOfShuyi.pdx` with `pdex.bin` and `pdex.dll`
      on Nick's machine, or (step 4) fails with a message that names the
      cause and the `--release` way round it.
- [ ] `cargo xtask playdate --release` still packs; CI's `playdate` job
      passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: if `xtask`'s plan changes (flags, profile), its plan test in
  `crates/xtask/src/playdate.rs` covers the new flag.
- Integration: the manual run above, written in the completion notes.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
