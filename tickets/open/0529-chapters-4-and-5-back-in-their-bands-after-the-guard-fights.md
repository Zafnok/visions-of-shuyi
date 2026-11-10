---
id: "0529"
title: Chapters 4 and 5 back in their bands after the guard fights
type: tuning
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["1071"]
nick_input: none
completed:
---

# 0529 — Chapters 4 and 5 back in their bands after the guard fights

## Context

1071 put two fixed skirmishes that guard the road on the Act 1 map:
*Aldwater Toll Bridge* before Veyne Hall (Chapter 4) and *The Salt Road*
before the Envoy's House (Chapter 5). Every player wins them on the way,
so the bots' arrival army (`crates/bots/src/arrival.rs`) and the chapter
tests (`ch04_winnable`, `ch05_winnable`) now win them first, and Chapters
4 and 5 are fought with a stronger army. Their recorded playthroughs were
recorded again with it.

The bots' playtests after 1071 (100 tries, seeds 1–100; Casual / Normal /
Hardcore success):

| Battle | Before 1071 | After 1071 | Bands |
| --- | --- | --- | --- |
| ch04 | 52 / 61 / 63 | 59 / 58 / 51 | 30–55 / 50–75 / 50–75 |
| ch05 | 58 / 58 / 69 | 77 / 85 / 13 | 40–70 / 50–75 / 50–75 |

Chapter 4's Casual is 4 points over its band. Chapter 5 is out on all
three. It is no harder: every bot wins it 98% of the time. But Hardcore
loses someone in 41 tries of 100, where Normal does in 15, and uses all
three of its items (its success allows one), so it succeeds in 13. The
army: Lead lv 11, Hedda 8, Hollis 11, Tamsin 11, Aske 10, Maud 8, Rue 16,
Joss 5.

## Nick input

None. Verified by the bots' reports in the PR; what is left to judge by
playing goes on the Act 1 playtest's checklist (1064).

## Scope

**In:**
- Find why Hardcore does worse than Normal in Chapter 5 (deaths, and the
  items), as 0526 did for the door: a bot fault is fixed in the bots,
  never by changing the bands.
- Bring Chapters 4 and 5 back inside their bands, or as near as the
  levers allow, by the battles' own numbers (`cargo shuyi autobalance
  --stage ch05`, or by hand, as `docs/playtesting.md` records for 0851
  and 0526), or by the guards' (fewer or lower Hounds give less EXP; keep
  the guards in their own bands).
- Record again whatever playthroughs the change breaks, in order:
  `ch04_winnable::record`, `ch05_winnable::record_salt_road`,
  `ch05_winnable::record`.
- The new numbers in `docs/playtesting.md`.

**Out (do not do):**
- Changing the bands or the success rules (`targets.ron`).
- New rules or new map features.

## Implementation steps

1. Read `docs/playtesting.md` (Chapters 4 and 5, 0526), 1071's Completion
   notes and `crates/bots/src/arrival.rs`.
2. Play `target/release/xtask playtest ch05 --bot hardcore` and read the
   tries that lose someone: who falls, to what, on which turn, and when
   the items go.
3. Fix a bot fault if there is one (with a test in `crates/bots`), then
   tune the numbers.
4. Record the playthroughs again as needed; run the playtests of
   aldwater, ch04, salt_road and ch05.

## Acceptance criteria

- [ ] Chapter 5's three bots inside their bands, and Chapter 4's (or the
      reason a band can't be met, in the Completion notes).
- [ ] The guards still inside theirs.
- [ ] `ch04_winnable`, `ch05_winnable` and the bots' tests pass.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The bots' playtest reports (before and after) in the PR.
- A test in `crates/bots` for any bot fault fixed.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
