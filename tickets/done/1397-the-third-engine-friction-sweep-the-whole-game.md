---
id: "1397"
title: "The third engine-friction sweep: measure the whole game against the engine, and file the fixes"
type: research
milestone: "Engine: carve"
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: []
nick_input: none
completed: 2026-11-27
---

# 1397 — The third engine-friction sweep: the whole game

## Context

Nick asked (2026-11-27) whether the game still holds engine-shaped
code, either left over from the carve (1102–1118, 1131) or added
since. He wanted to know whether anything generic enough to move to the
engine would save lines in the game. No trigger in the `engine-friction`
skill's *Sweep* started this one. It runs on request, and its first
part was done in the same session as a research report
(`docs/research/2026-11-27-engine-shaped-code-audit.md`, 16 readers).
The 11xx block is full (1199), so this ticket is 1397 and its
follow-ups are 1400–1422.

The earlier sweeps were 1153 (the 15 largest screens) and 0506's
(the playtest area). This one covers every game crate, the tools and
the process crate included.

## Nick input

None. Nick asked for the sweep and said "File the sweep" (2026-11-27).

## Scope

**In:**
- Read every game crate against the pinned engine (230080d), the
  engine's `main` and their tickets. Write one row per slice in
  `docs/engine/friction.md`, plus a section for the sweep under
  *Sweeps*.
- File the findings:
  - engine tickets, in one ticket-only engine PR;
  - one game follow-up per engine ticket;
  - a game ticket for copies of pieces the engine already has;
  - a ticket for the bug found on the way;
  - notes on the engine tickets that already hold a finding.
- Mark the code each engine ticket replaces (`engine-change` §2, step 3).
- Correct the open game tickets the sweep found wrong (1180, 1207,
  1274, 1219, 1288).

**Out (do not do):**
- Any refactor or engine change: the filed tickets do them.
- Unparking parked engine tickets. The notes on 2359 and 2231 say their
  condition is met; Nick unparks them.
- Putting a filed ticket on the Act 1 critical path.

## Implementation steps

1. Slice the game into 16 parts and read each for its engine-shaped
   code, with file and line evidence.
2. Check each claim against the engine by hand: the ticket blockers by
   script, the engine functions by grep.
3. Write the report, the log rows and the sweep section.
4. File the engine tickets in `Zafnok/shuyi-engine`, and the game
   tickets and markers here.

## Acceptance criteria

- [x] `docs/engine/friction.md` has a `1397 (sweep)` row for each of the
      16 slices, and a *Sweeps* section with the findings ranked.
- [x] Every finding has an outcome:
  - outcome 1 or 2 with a filed ticket;
  - outcome 4 with a reason in its row;
  - or a note on the engine ticket that already holds it.
- [x] Every filed engine ticket passes the leak test, and every game
      follow-up has its `engine#NNNN` in `blocked_by`.
- [x] Each filed finding's code carries an `engine-friction:` marker. The
      exception is 1416's one-line `as_any` copies, and that ticket says
      why.
- [x] `cargo xtask ticket-lint` passes.

## Tests required

- None (research: docs, tickets and comments only).

## Completion notes

**What was done.** 16 readers, one slice each, read about 120k
production lines of the game against the pinned engine. Their claims
were checked by hand before filing. The headline:

1. **About 6,000 lines are ready to delete now**, under filed tickets
   whose blockers the engine has all shipped and that nobody has
   started:
   - 1170: the Playdate front end, about 4,800 lines;
   - 1180, then 1207: the bots' and playtest's copies;
   - 1274: `clean_targets.rs`, which copies `trim-targets --merged`;
   - 1219, 1222, 1228 and 1182.
2. **About 250 lines copy engine functions** that exist today: 1421.
3. **About 5,000 lines need 22 new engine pieces.** See the ranked
   table in `docs/engine/friction.md`, *Sweeps*, 1397.

**Filed.**
- Engine, in one ticket-only PR:
  - 2151 and 2152 (the tactics kit's movement and threat queries);
  - 2286–2291 (assets: corner recipes, image edits, stills by tier, the
    `.dlg` walker and text policy, map look checks, cell-width text);
  - 3250–3257 (ui: the sprite piece painter, styled help bars, the
    dialogue backlog, sound scheduling, profile export and import,
    `AsAny`, enemy-phase playback, the question and language screens);
  - 3443–3447 (tools: leak checks, an edition's cut list, private pin
    files, GIF options, `playtest --merge`);
  - notes on 2359, 2231, 2370, 2723 and 2725.
- Game: 1400–1420 (one per engine ticket); 1421 (the engine's
  existing pieces); 1422 (the arts menu's columns, a bug).
- Changed: 1180 (also `seize_field`), 1207 (blocked by 1404), 1274
  (the whole of `clean_targets.rs` goes), 1219 (either order with
  1409), 1288 (1401 moves its parser).

**Deviations.**
- The sweep was cut by slice, not by screen. Each row's estimate is
  "what the slice keeps", so ratios are per slice. The writers' own
  per-ticket numbers are in each engine ticket's Context.
- 3255 (`AsAny`) breaks the API (every `Screen` loses two methods).
  1416 has no markers, because one on each of 38 screens would warn
  every screen PR.
- The engine's `main` (d5c23358) is ahead of the game's pin. Each
  engine ticket was checked against `main`, not only the pin.
- 1170's worktree from 4 weeks ago
  (`playdate-frontend-engines-backend-ab9b86`) holds only an edit of
  its ticket. It was left alone.

**Engine friction:** none: no Rust code beyond comments.
