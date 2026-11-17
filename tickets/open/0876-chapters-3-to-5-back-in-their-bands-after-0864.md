---
id: "0876"
title: "Chapters 3 to 5 back in their bands after 0864's new recordings"
type: tuning
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0864"]
nick_input: none
completed:
---

# 0876 — Chapters 3 to 5 back in their bands after 0864's new recordings

## Context

0864 made Tamsin Spd 6 and Def 6 (Nick's pick) and retuned Chapter 2
(no house burns on the Hounds' first turn). Both broke the recorded
winning playthroughs, so every one from Chapter 1 to Chapter 5 was
recorded again (`crates/ui/tests/it/fixtures/*_winning.ron`). The bots
start each later chapter with the army those recordings leave
(`docs/playtesting.md`, *The army*), so Chapters 3 to 5 and their guards
are now fought by different armies, and some left their bands.

The bots (`cargo xtask playtest <battle> --bot all`, seeds 1–100;
Casual / Normal / Hardcore success), `main` before 0864 (CI run
44126999009) and after:

| Battle | Before 0864 | After 0864 | Bands |
| --- | --- | --- | --- |
| ch03 | 60 ⚠ / 81 ⚠ / 55 | 68 ⚠ / 70 / 42 ✗ | 30–55 / 50–75 / 50–75 |
| aldwater | 100 / 89 / 90 | 100 / 50 ✗ / 80 | 70–100 / 70–95 / 70–95 |
| ch04 | 52 / 50 / 51 | 50 / 32 ✗ / 31 ✗ | 30–55 / 50–75 / 50–75 |
| salt_road | 98 / 94 / 93 | 100 / 94 / 96 ⚠ | 70–100 / 70–95 / 70–95 |
| ch05 | 62 / 52 / 71 | 51 / 15 ✗ / 11 ✗ | 40–70 / 50–75 / 50–75 |

None is blocked (Casual over 20%), so CI stays green. The armies after
0864: Chapter 3 Lead 5, Hollis 4, Tamsin 7, Aske 3, Maud 2, Rue 4, Joss 3;
Chapter 4 Lead 12, Hollis 5, Tamsin 9, Aske 5, Maud 4, Rue 9, Joss 3;
Chapter 5 Lead 15, Hedda 8, Hollis 11, Tamsin 10, Aske 8, Maud 6, Rue 13,
Joss 6. The new recordings win Chapter 2 on turn 2 (5 of 5 houses) and
Chapter 4 on turn 12 (was 5).

Why the bands move this much: each chapter's army comes from one recorded
win, and a new recording spreads the EXP differently. 0531 is about that;
this ticket only brings the chapters back.

## Nick input

None. Verified by the bots' reports in the PR (as 0529 and 0855); what is
left to judge by playing goes on the Act 1 playtest's checklist (1064).

## Scope

**In:**
- Find why Normal and Hardcore fall in Chapters 4 and 5 (the `try` lines:
  who falls, to what, on which turn; the items), as 0529 did. A bot fault
  is fixed in the bots, never by changing the bands.
- Bring ch03, aldwater, ch04, salt_road and ch05 inside their bands, or as
  near as the levers allow, by the battles' own numbers (`cargo shuyi
  autobalance --stage <battle>`, or by hand).
- Record again whatever playthroughs the change breaks, in order
  (`ch03_winnable::record`, `ch04_winnable::record_aldwater`,
  `ch04_winnable::record`, `ch05_winnable::record_salt_road`,
  `ch05_winnable::record`), and play the later battles again after each.
- The new numbers in `docs/playtesting.md`.

**Out (do not do):**
- Chapters 1 and 2, and Tamsin (0864, Nick's decisions).
- The bands or the success rules (`targets.ron`).
- How the bots build the army (0531). Either order: if 0531 is done
  first, tune against its armies; if not, against the recordings'.

## Implementation steps

1. Read `docs/playtesting.md` (*The army*, Chapters 3–5), 0864's, 0529's
   and 0524's Completion notes.
2. `cargo run --release -q -p xtask -- playtest ch05 --bot hardcore` and
   `… ch04 …`; read the tries that fail.
3. Fix a bot fault if there is one (with a test in `crates/bots`), then
   tune the battles, earliest first (a change to Chapter 3 moves the
   armies of the later ones).
4. Check seeds 101–200 too (`--seed 101`).

## Acceptance criteria

- [ ] ch03, aldwater, ch04, salt_road and ch05 inside their bands on seeds
      1–100 and 101–200, or the Completion notes say which lever was
      tried and why it stops short.
- [ ] `ch03_winnable` to `ch05_winnable` pass.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: `ch03_winnable` to `ch05_winnable`.
- The bots' playtest runs (reported in the PR).

## Completion notes

*(Filled in by the session that completes the ticket.)*
