---
id: "0911"
title: Voice clips in the release disclosure list
type: bug
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
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

- [ ] The list test fails when a clip in the voice manifest has no entry.
- [ ] A release dry run with the Chapter 1 clips passes the disclosure
      gate (run id in the notes).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the extended list test.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
