---
id: "0525"
title: Hardcore faster than Normal at Veyne Hall
type: bug
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0850"]
nick_input: none
completed:
---

# 0525 — Hardcore faster than Normal at Veyne Hall

## Context

The report's check "Hardcore is faster than Normal"
(`docs/design/playtest-bots.md`, *Hardcore is faster than Normal*) says a
miss is "a problem with the bots, not the map". On Chapter 4 (Veyne
Hall, balanced by 0850) every band is met on seeds 1–100 and 101–200,
but the check fails on both: Hardcore's median is 7 turns, the same as
Normal's (`docs/playtesting.md`, Chapter 4). Chapters 1 to 3 pass it.

What 0850 saw, without tracing it: both bots win in 5–9 turns, most in
7; Dace, the boss, holds the lord's end of the hall (`Guard`) and comes
at whoever reaches it, and a boss who retreats at 0 HP wins the battle
(`DefeatUnit`). Equal medians suggest Hardcore's bigger thinking budget
buys it nothing on a defeat-the-boss map: it may weigh the boss no
higher than Normal does, or wait for him to come out as Normal does.
0523 found Chapter 2's cause the same way (a long tail of stalled tries).

## Nick input

None (a bots fault; `playtest-bots.md` says how the check reads).

## Scope

**In:**
- Trace Hardcore and Normal on Veyne Hall (`cargo xtask playtest ch04`,
  a few seeds command by command, as 0523 did) and find why Hardcore is
  no faster.
- Fix it in the bots (`crates/bots`), with a test that fails without the
  fix, if the cause is a bot's; or say why the check can't be met here.
- Chapters 1 to 4 still in their bands after it, both seed ranges.

**Out (do not do):**
- Changing Chapter 4's battle to make the check pass (it's the bots').

## Acceptance criteria

- [ ] `cargo xtask playtest ch04 --bot all` shows `lower ✓` on seeds
      1–100 and 101–200, every band still met; Chapters 1 to 3 likewise.
- [ ] A test of the fix fails without it.
- [ ] All gates in the `run-gates` skill pass.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
