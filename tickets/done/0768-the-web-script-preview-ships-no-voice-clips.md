---
id: "0768"
title: "The web script preview ships no voice clips"
type: bug
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: low
status: done
blocked_by: ["0767"]
nick_input: none
completed: 2026-11-15
---

# 0768 — The web script preview ships no voice clips

## Context

Story kit run 43679214465, the first since 0767 merged: the `story-preview
(web)` job's export was refused with SH3012. The web build copies
`shuyi.ron`'s `web.voice` folder (`voice/`, which the private-assets
action checks out with the bought art) beside the page, and the
`story-preview` edition's `disclosure` lists no voice clip, rightly: 0764
leaves voice out, and the clips are the script's lines spoken, named by
scene id. An edition can replace `export.beside` but not `web.voice`, so
the edition alone can't leave them out. 0767's local export never reached
the check (its `wasm-opt` stalled first).

## Nick input

None.

## Scope

**In:** the preview job doesn't check the voice clips out, so no
target's preview holds a clip.

**Out:** voice in the preview (0764 leaves it out); an engine way for an
edition to drop `web.voice` (engine friction, noted below).

## Implementation steps

1. Give the private-assets action a `voice` input, and pass `voice:
   "false"` from `story-kit.yml`'s `preview` job.

## Acceptance criteria

- [ ] The Story kit's `story-preview (web)` job exports and passes the
      disclosure check.

## Tests required

- None beyond the workflow's own run (the export's disclosure check is
  the test).

## Completion notes

**What was done:** the private-assets action has a `voice` input
(default `"true"`, so every other job is unchanged): with `"false"` the
sparse checkout leaves `voice/` out and nothing is copied beside the
game. `story-kit.yml`'s `preview` job passes `voice: "false"`, so the
clips are never downloaded, rather than downloaded and deleted (Nick
asked for that in review). The web build skips `web.voice` when the folder
isn't there; the desktop exports never copied it.

**Verification:** run 43679214465 refused 126 files, the voice folder's
126 (20 named and "106 more"), and nothing else, so the folder was the
only problem. The release Wasm built locally holds none of the edition's
`forbidden_paths`. A local release export can't finish here (MSYS2's
`wasm-opt` stalls, as in 0767), so the next Story kit run is the check:
**the acceptance box stays open until that run is green.**

**Engine friction:** a workaround for an engine assumption. An edition
replaces `export.beside` but not `web.voice` or `web.music`, so a web
edition can't leave the voice out. Not filed in the engine: an engine
ticket needs Nick's OK; noted for him in the PR.

**Starting rules:** none. **Follow-ups:** none filed.
