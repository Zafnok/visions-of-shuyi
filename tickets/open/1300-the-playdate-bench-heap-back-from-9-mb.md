---
id: "1300"
title: "The Playdate bench's peak heap back from 9.3 MB, or shown to be the bench's alone"
type: infra
milestone: M1 Engine
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1300 — The Playdate bench's peak heap back from 9.3 MB

## Context

Found by 1285 while reading the `bench` numbers since the pin move of
2026-11-17 (PR #732). The Playdate's peak heap in every `cargo shuyi
bench` scenario went from 5.7 MB to 9.3 MB (`battle-*`; 5.4 to 9.0 for
`dialogue-portrait` and `options-menu`) between PR #742 (0445, still
5.7 MB) and PR #746 (0812, the chapter title card, 9.3 MB). The
Playdate's budget is 10 MB (`docs/targets.md`), so 0.7 MB is left. The
colour targets didn't move (5.0 and 4.7 MB).

#746 added the Cinzel lettering, baked by `cargo xtask lettering` into
glyph sheets drawn as sprites (ADR-0128); its notes say "the Playdate's
card is in the cell font" and that the Playdate leaves sheets out. The
likely cause is that the bench's Playdate session (`session_on` in
`crates/xtask/src/game_tools.rs`) still decodes or holds those sheets,
or that the bundle's sheets are loaded at start on every target.

## Nick input

None.

## Scope

**In:**
- Find what holds the 3.6 MB: the engine's memory tags and heap snapshot
  (`shuyi-perf` skill) on the Playdate's `battle-idle` session, before
  and after #746.
- If the Playdate build (`crates/playdate`) carries it too: keep the
  lettering sheets out of the Playdate's loads (or load them only when
  the card shows on a target that draws it).
- If only the bench's session carries it (the Playdate build already
  leaves the sheets out): make the session load as the Playdate build
  does, so the bench measures what the device holds.
- Update `docs/targets.md`'s table from the PR's `bench` summary.

**Out (do not do):**
- The Playdate's frame instructions (1248, 1299).
- Changing the chapter card's look (0812, Nick's pick).

## Implementation steps

1. Read `shuyi-perf`; take a heap snapshot by tag of the Playdate's
   `battle-idle` (`cargo shuyi bench --scenario battle-idle
   --no-instructions` prints the peak; the MCP `perf` tool or a test
   through the Harness gives the tags).
2. Compare with the tree at `a377d97f` (#742's merge) to name the
   allocation #746 added.
3. Fix where it is loaded (steps above), with a test that the Playdate
   session doesn't load the lettering sheets.
4. Read the PR's `bench` summary; update `docs/targets.md`.

## Acceptance criteria

- [ ] The PR's `bench` job shows the Playdate's peak heap at or under
      6 MB in every scenario, or the Completion notes show, with the
      snapshot, that the Playdate build doesn't hold it and the bench now
      measures the same.
- [ ] A test fails if the Playdate session loads the lettering sheets.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration (xtask or app): the Playdate session's loaded sprite
  sheets leave out the lettering.

## Completion notes

*(Filled in by the session that completes the ticket.)*
