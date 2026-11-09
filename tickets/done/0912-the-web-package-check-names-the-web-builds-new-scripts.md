---
id: "0912"
title: The web package check names the web build's new scripts
type: bug
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-08
---

# 0912 — The web package check names the web build's new scripts

## Context

0911's Release dry run ([run 42503479776](https://github.com/Zafnok/visions-of-shuyi/actions/runs/42503479776),
2026-11-08) failed `package (web)` at "Check the package"
(`scripts/check-release-package.sh`, from 0910). The export itself passed,
disclosure gate included. The checker's list of the web files the package
should hold doesn't name two files the package now has:

```
check: the package's files differ (< expected, > in the package):
9a10
> download.js
60a62
> sound.js
```

The last green dry run was 2026-11-02 (#525). The engine pin moved three
times since (#557, #572, #574), so the engine's web build most likely
added these two scripts. A real release would fail the same way.

## Nick input

None.

## Scope

**In:**
- Find which engine change added `download.js` and `sound.js` to the web
  build (`docs/engine/pins.md`, the engine's `web` export) and confirm both
  are meant to ship (what each is for, its license: engine's own or
  third-party per ADR-0013).
- Add both to the web case's file list in `scripts/check-release-package.sh`.
- Better if the engine can say it: if the engine names its web build's
  files somewhere the script can read (a report, `export-report.json`),
  read them from there instead so the next added script can't break the
  check. Otherwise the two names.

**Out (do not do):**
- Changing what the web build ships.
- The Linux program's executable bit (0913).

## Implementation steps

1. `git log -p` over `docs/engine/pins.md` since 2026-11-02; find the
   engine PR that added the scripts.
2. If a script is third-party, check ADR-0013 and its credit
   (`THIRD_PARTY_ASSETS.md`); a problem there is a new ticket.
3. Edit the `web)` case in `scripts/check-release-package.sh`.
4. Run the Release workflow by hand on the branch (`gh workflow run
   release.yml --ref <branch>`); `package (web)` passes. Record the run id.

## Acceptance criteria

- [x] A Release dry run's `package (web)` job passes (run id in the notes).
- [x] The notes say where `download.js` and `sound.js` come from and why
      they ship.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The Release dry run (the script has no unit tests).

## Completion notes

- `scripts/check-release-package.sh`'s web list now names `sound.js` and
  `download.js`. Release dry run
  [42551112067](https://github.com/Zafnok/visions-of-shuyi/actions/runs/42551112067):
  `package (web)` passed.
- **Where they come from:** both are Shuyi Engine's own web-shell scripts
  (`crates/cli/web/`, listed in `SHELL` in `web.rs`). They are the engine's
  code under its MIT OR Apache-2.0 license, not vendored, so they need no row
  in `THIRD_PARTY_ASSETS.md`. `NOTICES.txt` covers them under the engine's
  licenses.
  - `download.js`: engine PR #239 (engine ticket 2519, game pin row in
    `docs/engine/pins.md`). It offers the browser a file for the debug menu's
    "Save session". `index.html` always loads it, but only a build with debug
    tools calls it.
  - `sound.js`: engine PR #240 (engine ticket 3031). It is the Web Audio
    bridge for `shuyi-app`'s `in-house-sound` feature. `index.html` always
    loads it, and a build without the feature never calls it.
- **No engine report to read:** `export-report.json` (`ExportOutput`) lists
  the package, its sizes and `beside`, not the shell's files. So the two
  names are written into the script, with a comment pointing at the engine's
  `SHELL`. A future engine script still needs a line here.
- Only a shell script changed. Gates run: skill sync, `ticket-lint`,
  `check-mentions`, `typos` and `pr-size`. The Rust gates have nothing to
  check.
- Engine friction: none (two names).
