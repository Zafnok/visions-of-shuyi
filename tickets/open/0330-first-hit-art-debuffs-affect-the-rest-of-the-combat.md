---
id: "0330"
title: "First-hit art debuffs affect the rest of the combat and its preview"
type: bug
milestone: M2 Core rules
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0325"]
nick_input: none
completed:
---

# 0330 — First-hit art debuffs affect the rest of the combat

## Context

The audit of the last 60 game and engine PRs (2026-11-01) found a timing
miss in [game PR 454](https://github.com/Zafnok/visions-of-shuyi/pull/454)
(0325). `BattleState::fight` in `crates/core/src/battle.rs` calls `clash`
to resolve the whole combat, then checks whether any strike hit and calls
`debuff`. Rending Arrow, Off Balance and Vital Point therefore change the
target only after its counters and the attacker's follow-ups have finished.

[`combat-arts.md`](../../docs/design/combat-arts.md), *Exact rules*, says
these effects begin on the first hit. Rending Arrow's Def reduction must
affect subsequent damage, Off Balance's avoid reduction subsequent hit
chances, and Vital Point's stat reductions the remaining relevant strikes.
The tests in `crates/core/src/battle/tests/art.rs` check the target after
combat or the next attacker, missing the remainder of the same combat.
`preview_attack` in `battle/arts.rs` also plays the unchanged forecast out,
so its agreement with the current resolver does not prove the decided rule.

This is the game's combat rule, not an engine feature. Preserve
[ADR-0022](../../docs/adr/0022-combat-arts-and-attack-preview.md)'s shared
validated plan for the preview, UI and AI. If its representation needs a
new architectural decision, record it with `write-adr` when implementing.

## Nick input

None. Restore the documented first-hit behavior, verified by failing
regressions and deterministic battle/preview tests. Do not invent a new
rule for adding or reordering strikes when Spd changes during combat:
follow an existing explicit rule, or use `ask-nick` before implementing
an unresolved gameplay decision. Anything left to judge by playing goes
on 1064's checklist; this ticket asks for no check after merge.

## Scope

**In:**
- Apply the shipped arts' first-hit modifiers at the successful strike,
  before resolving subsequent affected damage, hit and counter inputs.
- Keep the no-crit preview's strike plan, actual resolution and the UI's
  displayed per-strike numbers consistent with that timing.
- Preserve first-hit activation after misses, refresh-on-reapply, floors
  at zero, expiry, payment and save/replay. Preserve the existing roll
  protocol: two hit rolls per attempted strike and a crit roll only on
  a hit, without speculative preview rolls or reseeding. Corrected hit
  chances can change whether a crit roll is needed; an earlier kill can
  remove later strikes and their rolls. Explain/review those intentional
  changes rather than require old replay hashes or snapshots unchanged.
- Check Pinning Shot and Pressure Point on the same existing first-hit
  path, without changing their decided rules.

**Out (do not do):**
- New arts, reactions or game rules. 0326's reactions are a separate
  ticket; either order works, and whichever lands second preserves the
  other's strike-sequence tests. 0328 builds its first-hit states after
  this ticket and is not implemented here.
- AI range/Cascade scoring (0521, which follows this ticket), utility
  weights (0511), tuning art costs or retuning chapter balance.
- A new generic engine combat system, new art data, or a new UI look.

## Implementation steps

**Size:** about 250–400 changed production lines and at most 1,800 changed
lines total, including regressions and this ticket's archive move. Compare
0312/0325's `fight`, `clash`, `preview_attack` and existing strike-plan
tests. If a larger independent representation change is necessary, split
it into a prerequisite ticket before coding rather than exceed the budget.

1. Read the design's first-hit, strike-order and expiry rules, ADR-0022,
   `battle.rs`, `battle/arts.rs`, `battle/skills.rs` and `combat.rs`.
2. Add failing regressions first: a surviving target hit by multi-strike
   Rending Arrow; a later Off Balance strike; Vital Point followed by a
   counter. Use state/events and controlled deterministic rolls, not
   glyph snapshots. Include a miss before the first successful hit.
3. Make the smallest shared strike-plan/resolution change that applies
   the effect at its activation point. Do not fix this independently in
   a UI painter or leave the old fixed forecast in a second consumer.
4. Update the preview and any affected playback/AI inputs together.
   Keep existing replay tests and document any unavoidable save/replay
   compatibility consequence; no silent reordering or reseeding.
5. Keep expiry/refresh and no-hit/zero-damage-hit cases covered. Update
   the combat documentation in its existing home if the representation
   changes; 0521 can then use the final strike inputs.

## Acceptance criteria

- [ ] Regressions fail before the fix and pass afterward: subsequent
      Rending Arrow damage uses reduced Def, Off Balance uses reduced
      avoid, and a Vital Point target's later counter uses reduced stats.
- [ ] Misses do not activate a debuff; the first successful hit activates
      it once, including a successful zero-damage hit.
- [ ] The preview's no-crit, all-hit strike outcomes match actual
      guaranteed-hit/no-crit resolution with these modifiers active.
- [ ] Existing expiry, refresh, costs, save/replay and deterministic RNG
      tests pass; no unresolved mid-combat Spd rule was invented.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit / battle: effect timing, miss then hit, counter, expiry and refresh.
- Property: preview/resolution equality for supported deterministic
  fixtures and existing non-negative stat invariants.
- Integration: existing attack-preview/playback and replay tests; add
  one targeted case only if the changed data path is not covered below UI.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-ups and anything for the Act 1 playtest.)*
