---
id: "1180"
title: "The playtest bots on the engine's weights targets threats and candidate order"
type: feature
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: high
status: todo
blocked_by: ["0506", "engine#2129", "engine#2130", "engine#2131", "engine#2132", "engine#2532", "engine#2533", "engine#2021"]
nick_input: none
completed:
---

# 1180 — The bots on the engine's playtest pieces

## Context

The 0506 sweep (`docs/engine/friction.md`, ADR-0095) found seven generic
pieces in the game's bots/tooling. Nick asked that other games get them
too. All seven named engine tickets are done and the game pin has passed
them (the 2026-10-25 move to `b4591a81`, and subsequent pins). Keep their
ids as minimum-revision requirements; there is no remaining engine wait.

The 2026-11-01 audit found this adoption still outstanding. The original
combined ticket planned roughly 300 production lines added and 1,000
removed but omitted much of the tests and wrapper deletion from its PR
size. Split it now: this ticket moves bot/rules helpers; 1207, needed
**after this one**, owns the CLI, history and reusable workflow and the
removal of the old xtask wrapper. Every original requirement lives in one
of these two tickets; no feature or policy is dropped.

## Nick input

None. Persona choices and reports remain unchanged, verified by tests and
seeded comparisons. No gameplay sign-off after merge. Any questions about
how useful the personas feel remain the Act 1 playtest's concern (1064),
not a prerequisite for this mechanical adoption.

## Scope

**In:**
- `crates/bots/src/persona.rs`: engine `Weights`, `Weighted` and
  `Persona<PersonaGame>` from 2130; `Threats`/`GoalDistance` from 2131;
  `TacticsOrder`/`OrderHooks` from 2132.
- `crates/bots/src/playtest.rs`: `Targets<Difficulty>` and its tier bands,
  floors and median comparisons from 2129. Keep the game's file reader.
- Enemy AI's per-faction/movement-type distance field on `GoalDistance`,
  and `trpg_core::threat_area` on `strike_area`, retaining their tests.
- Game-owned semantic hooks: feature counts in `Tally::between`, forecast,
  lord/HP caps, command classification and AI pick, talk handling,
  `PersonaBot`, `BattleMeasures`, `BattleEntrant`, and the personas/targets'
  values. Keep the 16 named features and every integer weight exactly.
- Keep `BattleMeasure` and each measure's name as well as
  `BattleMeasures`; the CLI and its consumers use those names. Map the
  old `faster` target to the engine's `medians` entry with the same
  values, and keep `Targets::from_ron` as the game's file reader.
- Update the friction log, bots boundary row and ADR-0095 ownership notes.

**Out (do not do):**
- CLI/history/workflow adoption, summary lines and xtask deletion: 1207
  depends on this one and owns those original 1180 requirements.
- Tuning settings, weights, floors, success rules or measures. Human
  comparison is 0508's; campaign runs are 0512's; modkit is 1021's.
- New game design or a new engine bot API.

**Either order with 0450:** if it is done, use its shared stationary
threat helper in the `Threats` hook; if not, keep today's stationary
rule in the game hook and 0450 changes that hook when it lands. 0516's
feature-count fix is already done: preserve its corrected `approach`
and `enemy_fell` counts in game-owned `Tally::between`.

**Either order with 0507, 0508, 0512 and 1021:** if one is done, adapt
its shared bot types or persona-file reader to the engine-backed types
here, without changing its report or command; if not, its ticket uses
those types when it lands. CLI migration is 1207's, in either order
with those consumers. 0509 is done: preserve its autobalance behavior
and report while changing the common bot types.

## Implementation steps

1. Save seeded persona/baseline measures and command traces before editing
   on the current pin; use Chapter 1 and the existing escape/retreat test
   fixtures, 20 seeds each. Record inputs and ignore wall-clock timing.
2. Replace scoring with engine weighted arrays, preserving all feature
   names/counts and persona values. Compare the decoded settings/weights
   against an explicit pre-migration table in tests.
3. Replace danger maps with `Threats<UnitId>` built from stop tiles and
   attack ranges; stationary enemies use `strike_area_from`. Forecasts,
   exposed/counter damage caps and the lord-alone rule remain game hooks.
   Replace goals with movement-type-keyed `GoalDistance`.
4. Implement `OrderHooks`: preserve the existing strike ordering and
   forecast key, stationary-before-moving heal order, step/end/drop/other
   categories and `next_command` tie behavior. Use `TacticsOrder` rather
   than moving the old ranking algorithm to another game file.
   In particular, a strike keeps `(expected dealt, -expected taken)`
   and its action-without-target/tile `way`; heals are `Extra` group 0
   ranked by missing HP with stationary heals before moving ones;
   `Wait` is `Step`, `EndPhase` is `End`, `Equip` and `Talk` are `Drop`,
   and the remaining commands are `Other`.
5. Use `Targets<Difficulty>` (derive `Ord` only as necessary), keeping
   tier values and entrant names. Move AI distances and `threat_area` to
   the same engine helpers without broadening the legal-command set.
6. Re-run step 1 and require equal command traces, per-try measures and
   verdicts. Keep a small representative trace fixture as a regression
   test. Update ownership docs and measure removals across all spellings.
7. Leave the existing xtask playtest working for 1207's baseline. Record
   completed engine reuse and the precise remaining 1207 pieces.

Expected size: about 180 added production lines and 450 removed, plus
focused test changes; planned under 1,800 changed lines including docs
and this ticket's move. The former command/workflow work is now 1207,
not an optional split based only on added production lines.

## Acceptance criteria

- [ ] Pin is past the seven named engine tickets; `ticket-lint
      --check-engine` passes. No pin move is required on today's main.
- [ ] Seeded command traces, per-try measures and verdicts agree with the
      pre-migration baseline, including retreat/escape and stationary
      enemies. Existing AI and `crates/bots/tests/baseline.rs` tests pass.
- [ ] Decoded personas/targets retain every weight/setting/band/floor;
      `unknown_entrants` is empty for baseline/casual/normal/hardcore.
- [ ] Tests compare the 16 feature names, persona settings, weights and
      decoded target numbers against an explicit pre-migration table;
      the old `faster` comparison is represented by the same median rule.
- [ ] Game-local generic score accumulation, threat storage/danger sum,
      goal-distance traversal, candidate ranking and target evaluation
      are removed, not renamed. Semantic hooks remain game-owned.
- [ ] The existing xtask playtest still works; 1207 has the remaining
      CLI/history/workflow handoff and the friction log records this step.
- [ ] Completion notes name the seven engine PRs/revisions and record
      production-line counts before this ticket, so 1207 can verify the
      original combined reduction of at least 700 production lines.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: existing feature counts, danger/HP/lord caps, stationary threats,
  candidate order and target tests through engine-backed implementations.
- Integration: seeded traces/measures, unchanged AI baseline, decoded
  persona/target-value equality. Behavior tests read state, not pixels.

## Completion notes

*(Filled in by the session that completes the ticket: baseline comparison,
engine APIs used, removals, deviations and the remaining 1207 handoff.)*
