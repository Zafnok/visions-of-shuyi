---
id: "1208"
title: Hold the release packages to a disclosure list
type: feature
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["0910", "engine#3081"]
nick_input: none
completed: 2026-11-02
---

# 1208 — Hold the release packages to a disclosure list

## Context

Engine PR #211 (3081), passed by the pin move of 2026-11-01: every `shuyi
export` writes `exposure-report.json`, listing each file of the final
package with its source and hash. A `disclosure` part in `shuyi.ron`
(`allowed_files`, `allowed_pack_entries`, `forbidden_paths`, `canaries`;
whole paths, no patterns) refuses an export that breaks it (`SH3012`), and
removes the package.

The game has things that must never reach a player: the test files in
`assets-test/` (ADR-0057), and the bought art's raw library (ADR-0040),
which only `game/` of the private repository may be built from. Today only
the build's features keep them out, and nothing reads the final package.
Once 0910 makes the release packages with `shuyi export`, the report exists
on every release and this gate costs one part of `shuyi.ron`.

The engine's check finds what is written plainly. It doesn't prove that
nothing unreleased is there, so the part guards against mistakes, not
against a player digging.

## Nick input

None. This holds packages to what they ship today; which chapters a demo
holds is not decided here.

## Scope

**In:**
- A `disclosure` part in `shuyi.ron`, from the `exposure-report.json` of a
  real export of each target: the files allowed beside the program,
  `forbidden_paths` for what must never ship, and a canary from a test file
  (the `test` language pack or a test character) that the program of a
  shipping build must not hold.
- The files allowed have to include the private build's and the music and
  voice beside it (0910), so a release passes; a test checks a build without
  them passes too, or says why it can't.
- `release.yml` keeps each `exposure-report.json` with the release's other
  reports.

**Out (do not do):**
- A demo's contents, or hiding unreleased chapters: that needs a decision on
  what a demo is, and its own ticket.
- Capture controls or watermarks (engine 3083, parked there).

## Implementation steps

1. Export each target, read each `exposure-report.json`.
2. Write the `disclosure` part from them, and check by hand that a file in
   `assets-test/` added to the package refuses the export with `SH3012`.
3. Keep the reports in `release.yml`.

## Acceptance criteria

- [x] `cargo shuyi export <target> --release` passes for every target the
      release ships, with the part on.
- [x] A file that breaks the list refuses the export, named with its source.
- [x] A canary from the test files is not found in a shipping build, and is
      found when the debug tools feature that embeds them is on.
- [x] A release's reports are kept with it.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: none (the engine tests its gate).
- Integration: the refusal above, run by hand or in CI's release job, with
  the result in the completion notes.

## Completion notes

`shuyi.ron` has a `disclosure` part: the files `export.beside` ships, the music
and audio licenses under both their names (a `.app` puts folders in
`Contents/Resources`), forbidden paths for the test files and the bought art's raw
library, and two canaries, from the `test` language pack and the test characters.
`docs/releasing.md` says what it does.

Results, all with the part on:

| Run | Where | Result |
| --- | ----- | ------ |
| windows, linux, macos, web, with the bought art | Release dry run [41400102118](https://github.com/Zafnok/visions-of-shuyi/actions/runs/41400102118) | Pass; each report `ok`, `declared`, features list `private-assets`, 0 pack entries |
| windows, web, with the bought art | This machine | Pass, and `check-release-package.sh` passes |
| windows, `--without private-assets` | This machine | Pass |
| windows, `assets-test/lang/test/ui.ron` added to `export.beside` | This machine | `SH3012`: `ui.ron (export.beside: ui.ron) isn't in disclosure.allowed_files`, and holds `canaries[0]`; only the report left |
| windows, `debug-tools` added to `export.features` | This machine | `SH3012`: the `.exe` and `.debug` hold both canaries and a forbidden path; only the report left |
| web, `debug-tools` added to `export.features` | This machine | `SH3012`: the `.wasm` holds both canaries and a forbidden path |

The local runs used a stand-in for cargo-about's licenses report, which has the
same file name. The refusal and canary runs were by hand, on Windows and the web
only: the engine's own tests cover the gate, and the list is the same on every
target.

Deviations:

- `release.yml` needed no change: 0910 already keeps each `exposure-report.json`
  with the release's other reports. The dry run's artifacts have them.
- A build without the extras: an allowed file a target doesn't ship is no error,
  so a build without the bought art or voice clips passes (above). A build without
  music can't be made, since `export.beside` requires `music/`. No automated test
  exports a package: that would cost a release build in every gate, so this is the
  ticket's "says why it can't".
- One small test the ticket didn't list, in `crates/xtask/src/game_tools/tests.rs`:
  every file of `music/` and the audio licenses is on the list under both names, so
  a new track fails the gates and not a release.

Worth knowing:

- The private repository has no voice clips at the pin. When the first arrive, each
  needs its two lines in the list, or the release export refuses them. That list
  will be long; no test covers it, because `voice/` is git-ignored and the gates
  never read the private repository.
- The Playdate isn't on the list because `release.yml` doesn't ship it. Its package
  has other names, so a release of it will need them added (the engine's ADR-0070).

No follow-up tickets. No gameplay rules decided.
