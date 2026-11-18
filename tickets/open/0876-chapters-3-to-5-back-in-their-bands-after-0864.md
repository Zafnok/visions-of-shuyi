---
id: "0876"
title: "Chapters 3 to 5 back in their bands after 0864's new recordings"
type: tuning
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0864", "0531", "0527", "0528"]
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

**Why it waits** (retrospective of PRs #616–#735, T2:
`docs/research/retro-2026-11-17-prs-616-735.md`). #623 (0529) tuned
ch04 and ch05 into band, and six days later #731 (0864) re-recorded and
threw that away. This is the fourth back-in-bands ticket (0524, 0855,
0529). So it waits for what would move the armies or the enemies again
and needs no Nick decision: 0531 (a stable army), 0527 (per-unit numbers
to see who falls short, as Lane I asks of every balance ticket) and 0528
(the enemies' AI, which re-records and retunes every battle itself).
After those, start by measuring: Chapters 3–5 may already be close.

It does **not** wait for 0333, 0610, 0870, 0867 or 0868, though they
also change the balance and re-record: each waits on a decision of
Nick's (0080, 0081, 0870's own, 0082, 0084), and waiting on them would
hold this ticket for as long as those take. Each of them re-runs the
bands itself when it lands.

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
- How the bots build the army (0531, done first: tune against its
  armies).
- The report's intervals (0532). Either order: if 0532 is done, a band
  is met when no ✗ or ⚠ shows; if not, check both seed ranges as below.

## Implementation steps

1. Read `docs/playtesting.md` (*The army*, Chapters 3–5), 0864's, 0529's,
   0524's, 0531's and 0528's Completion notes. Run the five battles
   first: the table above is from before 0531 and 0528.
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
