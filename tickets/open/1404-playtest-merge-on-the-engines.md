---
id: "1404"
title: "The playtest workflow joins its split battles with shuyi playtest --merge: the game's merge goes"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: low
status: blocked
blocked_by: ["engine#3447"]
nick_input: none
completed:
---

# 1404 — The playtest workflow's merge on the engine's

## Context

The third engine-friction sweep (1397, row C17) found that the
playtest workflow plays its slowest battles in two seed ranges and
joins them with the game's own `playtest --merge`
(`crates/tools/src/playtest.rs`, `merge_files` and `merge`, about 85
production lines, ticket 0173; `.github/workflows/playtest.yml` line
about 403). The engine's `shuyi playtest` has no merge, and that is the
last piece 1207 (the game's playtest command on the engine's) lacks.
Engine ticket 3447 adds `--merge`. Both items are marked
`engine-friction: engine#3447, moves (game 1404 deletes it)`.

1207 is blocked by this ticket: it deletes the rest of `playtest.rs`
once the workflow no longer needs the game's merge.

## Nick input

None.

## Scope

**In:**
- Move the pin past engine 3447 if it is behind (`engine-change` §7).
- In `playtest.yml`'s join step, the merge is the engine's:
  `cargo shuyi playtest --merge parts/<battle>-1.json parts/<battle>-2.json`
  with the same report and history flags. The parts are written by the
  engine's `--report` if the game's runs already use the engine's
  command. If not, the game's `--json` must be the same shape: check
  that first, and if it isn't, convert the parts' writer to the
  engine's report in this ticket.
- `merge_files`, `merge` and the `--merge` flag go from the game's
  command.

**Out (do not do):**
- The rest of the game's playtest command (1207).
- Changing which battles are split.

## Implementation steps

1. Write the engine-friction estimate: about 5 lines changed in the
   workflow, about 100 removed.
2. Move the pin, if needed.
3. On a branch, run the workflow on one split battle (`workflow_dispatch`)
   and compare its merged checks with the last `main` run.
4. Delete the game's merge and its tests.

## Acceptance criteria

- [ ] `grep -n "fn merge" crates/tools/src/playtest.rs` finds nothing.
- [ ] A dispatched playtest run of a split battle reports the same
      checks as before (its link is in the completion notes).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None new: the engine's tests cover the merge.

## Completion notes

*(Filled in by the session that completes the ticket.)*
