---
id: "0910"
title: "Release packages made by shuyi export, not by hand in release.yml"
type: infra
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["0909", "engine#3075", "0138"]
nick_input: none
completed:
---

# 0910 — Release packages made by shuyi export

## Context

0138 declares the project target profiles first; this ticket uses them.

Engine PR #183 (ticket 3004), reached by the pin move of 2026-10-30, adds
`shuyi export <target> --release`: it runs `check --target`, refuses on an
error, builds, and packages for `windows`, `linux`, `macos` and `web`. It also
does what `.github/workflows/release.yml` doesn't:

- `NOTICES.txt`, and the player note on opening an unsigned program;
- the build's paths remapped, and the program searched for the home folder,
  so a release can't carry a developer's paths;
- a stripped program, with the debug file kept beside the package;
- the Linux build in Valve's sniper SDK, so its glibc floor is known and old;
- a universal macOS `.app`, signed ad hoc.

`release.yml` does each package by hand in about 200 lines of shell.
Engine PR #212 (3075) has now shipped feature selection, game package
selection and files beside the program. That removes the two former
blockers: `private-assets` (ADR-0040), and external music/voice/license
files (ADR-0046). The pin move of 2026-11-01 reached it; `engine#3075` stays as the
minimum-revision dependency. Game 0909 and 0138 are already done.

Engine PR #205 (ticket 3080), passed by the pin move of 2026-11-01: `shuyi
web` builds in `dist/web.staging` and replaces `dist/web` whole, and `export
web` refuses a folder holding a file the build didn't make. So the web job
must add nothing to `dist/web` by hand. `release.yml`'s Package step copies
the licenses into its own folder today; keep them out of `dist/web` when it
moves.

## Nick input

None.

## Scope

**In:**
- `shuyi.ron` names the release's features and the files beside the program,
  as 3075 made it possible, and explicitly selects `trpg-app` (the project's
  tool package is `xtask`). `notices` is 0909's.
- `release.yml`'s Windows, Linux, macOS and web jobs call `cargo shuyi export
  <target> --release`, and upload what it makes, under the release's names.
- The program's debug files are kept as release artifacts.
- `docs/` that describes the release is updated.

**Out (do not do):**
- itch.io (0901) and Steam (0903).
- The Playdate's package (1052).
- The Playdate command/exporter cleanup (1206). Either order works:
  this ticket changes desktop/web release jobs, while 1206 changes only
  the Playdate build command; neither changes the other's package policy.
- Changing what a player gets, other than the additions above: the music and
  voice stay beside the program, and Shironejiya's text stays bundled (0901).

## Implementation steps

1. ~~Move the pin past engine#3075 if it isn't (`engine-pin`).~~ (done by the pin move of 2026-11-01)
2. Run `cargo shuyi export windows --release` locally, and compare the file
   list with the last release's zip.
3. Change `release.yml` target by target; run it from a test tag, and compare
   each package's files.

Expected size: about 150 lines of `release.yml` removed, a small manifest
and package verification script; under 1,200 changed lines including tests,
docs and the ticket move.

## Acceptance criteria

- [ ] Each target's package has every file the last release's has, plus
      `NOTICES.txt`, and `export-report.json` beside it.
- [ ] The Windows package runs with the music, on a machine with no MinGW.
- [ ] The private art is in the release builds, and a build without the
      secret still fails or builds as `release.yml` says today.
- [ ] Final extracted package inventories are compared, not only Cargo
      commands. No parallel archive/app creation, stale web-file injection
      or duplicated engine notice/path-remapping logic remains in the
      release jobs. A missing required beside-file refuses a test export.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- A test tag's release run, extracted file inventories compared in the PR,
  and a failed export with one required beside-file absent.
- A Windows launch without MinGW verifies packaged runtime dependencies.

## Completion notes

*(Filled in by the session that completes the ticket: pin, package
inventories, launch/refusal evidence, deviations and follow-ups.)*
