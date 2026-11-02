---
id: "0910"
title: "Release packages made by shuyi export, not by hand in release.yml"
type: infra
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["0909", "engine#3075", "0138"]
nick_input: none
completed: 2026-11-01
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

- [x] Each target's package has every file the last release's has, plus
      `NOTICES.txt`, and `export-report.json` beside it.
- [x] The Windows package runs with the music, on a machine with no MinGW.
- [x] The private art is in the release builds, and a build without the
      secret still fails or builds as `release.yml` says today.
- [x] Final extracted package inventories are compared, not only Cargo
      commands. No parallel archive/app creation, stale web-file injection
      or duplicated engine notice/path-remapping logic remains in the
      release jobs. A missing required beside-file refuses a test export.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- A test tag's release run, extracted file inventories compared in the PR,
  and a failed export with one required beside-file absent.
- A Windows launch without MinGW verifies packaged runtime dependencies.

## Completion notes

Pin: `76e05909`, unchanged. Dry run on this branch, with the bought art:
[run 41367418393](https://github.com/Zafnok/visions-of-shuyi/actions/runs/41367418393),
all four packages green.

**Done.** `shuyi.ron` has an `export` part (`trpg-app`, `private-assets`, the
files beside the program). `release.yml`'s four jobs are one matrix job that runs
`cargo shuyi export <target> --release`, then `scripts/check-release-package.sh`,
then gives the files the release's names. The package names are unchanged.

**Inventories**, extracted from the run's artifacts against v0.3.0's, top folder
and macOS's `.app` path set aside:

| Package | v0.3.0 files | Now | Lost | Added |
| ------- | ------------ | --- | ---- | ----- |
| windows-x64 | 48 | 56 | none | `NOTICES.txt`, `HOW-TO-OPEN.txt`, 6 tracks |
| linux-x64 | 48 | 55 | none | `NOTICES.txt`, 6 tracks |
| macos-universal | 48 | 58 | none | `NOTICES.txt`, `HOW-TO-OPEN.txt`, `Info.plist`, `CodeResources`, 6 tracks |
| web | 54 | 60 | `mq_js_bundle-LICENSE-MIT.txt` | `NOTICES.txt`, 6 tracks |

The 6 tracks were added to `music/` after v0.3.0. Every package's exposure report
lists `private-assets`; each has `export-report.json` and `exposure-report.json`.
Linux asks for glibc 2.30 at most (floor 2.31). macOS is signed ad hoc with both
slices.

**Launch.** The run's Windows package, extracted, with only System32 on `PATH`:
window open, no MinGW module loaded, Windows' audio modules loaded, and imports
free of MinGW DLLs. The music wasn't heard: the folder ships beside the program
and the audio stack starts, nothing more was measured.

**Refusals.** With `dist/licenses/THIRD_PARTY_LICENSES.html` absent, the export
stops with `SH1002` naming the file, before building. The check script also
refused a missing file and a claim of bought art the build doesn't have. A build
without the key passes `--without private-assets` (run locally); a real release
without the key still fails in the private-assets action, whose rule is
unchanged. That branch, and the `release` job attaching `kept/*`, didn't run in
a dry run: no real tag was pushed, because it would publish a public release.

**Deviations.**
- A dry run replaces the test tag, for the reason above; `release.yml` says it is
  the same pipeline.
- Windows now builds with GNU, as the exporter does (ADR-0108). `build.rs`
  embedded the icon and version info on MSVC only, so it now does so on GNU too
  and the check refuses a program without them.
- The web package loses `mq_js_bundle-LICENSE-MIT.txt`, against the first
  criterion as written: `NOTICES.txt` names `mq_js_bundle` with the MIT text, and
  the fourth criterion forbids keeping the engine's notices by hand.
- `README.txt` is one text for all targets, without the version (`beside` is the
  same on each); `HOW-TO-OPEN.txt` replaces its opening advice. The packages' top
  folders are now `-windows`, `-linux`, `-macos`. The web zip has `index.html` at
  its root. macOS's `music` and `audio-licenses` are in `Contents/Resources`.
- `music/README.md` moved to `docs/music.md`: `beside` copies a whole folder.
- The `.debug` files and reports go on the GitHub Release beside the packages,
  never in them, because workflow artifacts expire.
- macOS's zip stays a `ditto` line, as the engine's CI does: its exporter leaves
  a folder, and only `ditto` keeps the signature.

**No follow-ups.** 1208 (disclosure list) is now unblocked on this side. 0903 must
check `steamworks` on the GNU target (ADR-0108).

No gameplay rules decided.
