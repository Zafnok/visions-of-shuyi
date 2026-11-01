---
id: "1208"
title: Hold the release packages to a disclosure list
type: feature
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["0910", "engine#3081"]
nick_input: none
completed:
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

- [ ] `cargo shuyi export <target> --release` passes for every target the
      release ships, with the part on.
- [ ] A file that breaks the list refuses the export, named with its source.
- [ ] A canary from the test files is not found in a shipping build, and is
      found when the debug tools feature that embeds them is on.
- [ ] A release's reports are kept with it.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: none (the engine tests its gate).
- Integration: the refusal above, run by hand or in CI's release job, with
  the result in the completion notes.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
