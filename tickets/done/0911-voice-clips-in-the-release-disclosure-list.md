---
id: "0911"
title: Voice clips in the release disclosure list
type: bug
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-08
---

# 0911 — Voice clips in the release disclosure list

## Context

1208 (#525) held release packages to `shuyi.ron`'s
`disclosure.allowed_files`. An export that ships a file the list
doesn't name is refused with `SH3012`. The comment over the list says
"A new track or voice clip needs its two lines (an xtask test checks
`music/` and the audio licenses)". The test checks only `music/`.

Then 0722 (#559) made 94 Chapter 1 clips. `export.beside` copies
`(path: "voice", optional: true)` beside the program. Yet the list on
`main` has no `voice/` entry, and no open ticket mentions it. The retro
of 2026-11-08 found this. Its reading: the next real release with the
clips present is refused. No release has run since, so that is an
inference from `shuyi.ron` and the export code, not a failed run.

## Nick input

None.

## Scope

**In:**
- Confirm with a release dry run (the Release workflow by hand on the
  branch, as #521 and #525 did), with the clips present as the
  release has them.
- The list names each clip, twice for the `.app` bundle's Resources as
  the comment says. Or the xtask test that checks `music/` checks
  `voice/` too, against the voice manifest (`voice.ron`) when it is
  present. Prefer the test, so the next chapter's clips can't be
  forgotten.

**Out (do not do):**
- Changing which clips ship.

## Implementation steps

1. Find the test that checks `music/` against the list (`grep -rn
   "allowed_files" crates`) and extend it to `voice/`.
2. Add the entries the test asks for.
3. Dry-run the release and record the run id.

## Acceptance criteria

- [x] The list test fails when a clip in the voice manifest has no entry.
- [x] A release dry run with the Chapter 1 clips passes the disclosure
      gate (run id in the notes).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the extended list test.

## Completion notes

- **What shipped was more than the clips.** The private repository's
  `voice/` (at `assets-private.rev`) holds 122 files: 118 clips for
  Chapter 1's 94 lines (some lines have `.m`/`.f` pairs), `voice.ron`,
  `cast.ron`, and the voice tool's `say.ron` and `sent.ron`
  (`export.beside` copies the whole folder; ADR-0113 says `sent.ron`
  ships). `shuyi.ron` now lists all 122, twice (plain and the `.app`'s
  Resources): 244 lines.
- **The test** (`the_disclosure_list_names_the_music_voices_and_licenses_a_package_ships`,
  `crates/xtask/src/game_tools/tests.rs`) walks `voice/` at any depth,
  as the export copies it, rather than reading `voice.ron`: that also
  covers the manifest and the tool's files, which the manifest doesn't
  name. It skips `voice/` when the folder isn't here (CI's gates have no
  private repository), so it bites on a machine that ran `cargo shuyi
  private-assets`, as the session that makes the next chapter's clips
  (0757) does. Seen failing with the clips and no entries, passing with
  them, and passing without `voice/`.
- **Release dry run** [42503479776](https://github.com/Zafnok/visions-of-shuyi/actions/runs/42503479776)
  on the branch, with the bought art and the clips: the Export step
  (where the disclosure gate runs) passed on all four targets; Linux's
  package holds 178 files, voice included. The run is red for two
  other reasons, in the later "Check the package" step, neither about
  voice and both on `main` too: the web zip has `download.js` and
  `sound.js` the checker doesn't list (**0912**), and the Linux
  tarball's program isn't executable (**0913**). Both likely came with
  the engine pin moves since the last green dry run (2026-11-02).
- WASM, Playdate and `cargo doc` builds weren't rerun: the change is one
  xtask test and `shuyi.ron`'s list.
- Friction: estimate 25 production lines (the test), measured about 30
  in the test; under 2×.
- No gameplay rules decided.
