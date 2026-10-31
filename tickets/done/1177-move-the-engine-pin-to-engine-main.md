---
id: "1177"
title: "Move the engine pin to the engine's main (engine 2304, 2305 and 3006)"
type: infra
milestone: "Engine: carve"
model: implementation
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-10-22
---

# 1177 — Move the engine pin to the engine's main

## Context

The game builds with the Shuyi Engine commit in `shuyi-engine.rev`
(`6d961093` on 2026-10-21). Three engine tickets are done but past that
pin (checked 2026-10-21 with `work-ticket` §1's commands: each is
`behind`):

- **engine#2304**, the grid scene says where the map goes on past the
  view: 0449 (arrows at the map's edges) needs it.
- **engine#2305**, the text box paints narration from the left when the
  game asks: 0740 (narration left-aligned) needs it.
- **engine#3006**, one-bit output palette roles to patterns and a
  distinctness check: the Playdate work (lane D and E of
  `docs/ROADMAP.md`'s *Act 1 critical path*) builds on it.

Moving the pin once, on its own, keeps 0449 and 0740 small and gives
every later ticket the current engine. Both wait for this ticket instead
of their engine ticket.

## Nick input

None.

## Scope

**In:**
- `cargo xtask engine-pin <commit>` to the engine's `main` (the newest
  commit when the ticket is worked; at least `8617026c`, which closed
  2305).
- Whatever the game needs to build and pass the gates on the new pin:
  compile fixes, new fields added to the lists that name every field
  (for example the grid scene's in `every_field_is_listed`), snapshot
  updates that the engine change explains. Keep every behaviour the same.
- The pin check: `cargo xtask engine-pin --check`.

**Out (do not do):**
- Using the new engine features: the arrows are 0449's, the narration
  alignment 0740's, the one-bit patterns the Playdate tickets'.
- Engine changes. If the new pin breaks something the game can't fix
  on its side, stop and follow the `engine-change` skill.

## Implementation steps

1. `cargo xtask engine --off` if a local checkout is patched in.
2. `cargo xtask engine-pin <engine main's commit>`.
3. Build and run the gates (`run-gates` skill); fix what the new engine
   needs, game side only. If a snapshot changes, say why in the PR (the
   engine ticket that changed it).
4. In 0449 and 0740, check their pin step still reads right (they say
   1177 moves the pin) and note anything this ticket already did for
   them.

## Acceptance criteria

- [x] `shuyi-engine.rev` is at or past the commits that closed engine
      2304, 2305 and 3006 (`gh api .../compare/<sha>...<pin>` says
      `ahead` or `identical` for each; the commands are in the PR).
- [x] `cargo xtask engine-pin --check` passes.
- [x] No behaviour changes: snapshots and play records unchanged, or
      each change explained by an engine ticket.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None new; the existing suite on the new pin.

## Completion notes

Moved the pin to engine main `8d5055ca` with `cargo xtask engine-pin`. By the time this was worked, the pin (`384f7961`, moved by 1181 and earlier tickets) was already ahead of the commits that closed engine 2304, 2305 and 3006 (compare said `ahead` for each), and 0449 and 0740 were already done, so step 4 had nothing to do. No game-side fixes were needed: all gates pass, no snapshot or play record changed. Nothing for Nick; no starting rules; no follow-ups.
