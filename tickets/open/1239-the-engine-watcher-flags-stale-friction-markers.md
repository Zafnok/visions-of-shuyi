---
id: "1239"
title: "The pin check flags an engine-friction marker whose engine ticket is done at the pin and that no open ticket deletes"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1239 — The pin check flags stale engine-friction markers

## Context

A workaround for an engine gap is marked `// engine-friction: engine#NNNN`
(`engine-change` §2, step 2). The engine watcher (ADR-0096, ticket 1190)
reads the markers (`fetch::friction_markers`, `crates/engine-watch/src/fetch.rs:285`)
only to report a PR **in the gap** that completes a marker's ticket
(`unblocks`, `lib.rs:346-366`). Once the pin passes that PR, nothing
looks at the marker again.

The game/engine boundary audit of 2026-11-03 (finding records#4) found
both of today's markers stale: `crates/xtask/src/game_tools.rs` (lines
77, 126, 280) cites engine#2540 and `crates/ui/src/screens/camp/music.rs:97`
cites engine#2341, and both engine tickets are in the engine's
`tickets/done/` at the pin. Here both happen to have a game ticket (1182
deletes the first; 1080 rewrites the second to cite engine#2398), but
only a sweep would have noticed a marker with none.

This ticket adds the check to `cargo xtask engine-pin --check` (its
online half, which already asks GitHub, `engine.rs:245`): a marker whose
engine ticket is done at the pin fails the check unless an open game
ticket names `engine#NNNN`. It trips only in the PR that moves the pin
past the ticket, or that adds such a marker: the PR that should deal
with it.

## Nick input

None.

## Scope

**In:**
- A pure `stale_markers` in `crates/engine-watch/src/lib.rs`, markers
  with their places, and the engine's done tickets at the pin.
- The check in `engine-pin --check` (not `--offline`), and one line in
  the `engine-pin` issue's checklist and the `engine-pin` skill.

**Out (do not do):**
- Deleting or rewriting today's markers (1182 and 1080 do).
- Markers in this crate's own tests (`fetch::friction_markers` already
  skips `crates/engine-watch`).
- A pin move.

## Implementation steps

Expected size: about 80 production lines (`lib.rs` 25, `fetch.rs` 35,
`engine.rs` 15, `render.rs` 2), plus about 60 test lines.

1. `crates/engine-watch/src/lib.rs`: a `pub struct Marker { path: String,
   line: usize, id: String }`, `friction_markers` returning them (the
   line from the text before the match), and
   `pub fn stale_markers(markers: &[Marker], done: &HashSet<String>, open_texts: &[String]) -> Vec<String>`:
   one message per marker whose `id` is in `done` and that no open
   ticket's text names (`engine#{id}` anywhere in it), for example
   `` `crates/ui/src/screens/camp/music.rs:97` marks a workaround for engine#2341, which is done at the pin, and no open ticket names it: delete the workaround here, or write the game ticket that does (`engine-change` §2) ``.
   `unblocks` keeps working on the ids.
2. `crates/engine-watch/src/fetch.rs`: `friction_markers(dir)` returns
   `Vec<Marker>` with the path relative to the repository's root;
   `watch` maps them to ids. Add `pub fn done_at(pin: &str) -> Result<HashSet<String>, String>`:
   the four-digit ids of `gh api repos/{ENGINE_REPO}/contents/tickets/done?ref={pin} --jq '.[].name'`
   (the Contents API lists at most 1,000 files; the engine has 158 done
   tickets, so say in its doc comment to switch to the Git Trees API
   past that), and `open_ticket_texts(root)` (the files of
   `tickets/open/`, from the existing `markdown_files`).
3. `crates/xtask/src/engine.rs`: `fn marker_problems(root)` joins those
   (`#[mutants::skip]`, like `watch_report`: it only calls `gh` and the
   tested pure function), and `check_with_gains` adds its problems to
   `gain_problems`'s.
4. `crates/engine-watch/src/render.rs`, the issue's *The work* list:
   `- [ ] each marker an *Unblocks* row names: the open ticket that
   deletes it, or delete it in this PR`.
5. `.claude/skills/engine-pin/SKILL.md`: one line where it runs
   `--check`, saying what the new problem means and the two ways out.

## Acceptance criteria

- [ ] `stale_markers` returns a message for a done id no open ticket
      names, and nothing for an open id or a done id an open ticket names
      (unit tests).
- [ ] `friction_markers` reports the path and line of each marker
      (unit test over a fixture string).
- [ ] On this branch, `cargo xtask engine-pin --check` passes: 1182
      names engine#2540 and 1080 names engine#2341 (or, if either is
      done, its marker is gone).
- [ ] The issue body's *The work* list has the marker line (render test).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `stale_markers` (three cases), `friction_markers` with places,
  the render line.
- Integration: `the_real_repository_reads` also reads the markers with
  places; no test reaches the network.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
