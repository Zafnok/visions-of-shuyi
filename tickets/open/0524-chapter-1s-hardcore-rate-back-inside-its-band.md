---
id: "0524"
title: Chapter 1's Hardcore rate back inside its band
type: tuning
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0523"]
nick_input: none
completed:
---

# 0524 — Chapter 1's Hardcore rate back inside its band

## Context

Chapter 1 (`assets/battles/ch01.ron`) was tuned by 0506 with the
personas of the day: Casual 44, Normal 59, Hardcore 51 out of 100
(`docs/playtesting.md`, *Reading the report*). Tickets since have changed
the rules and the bots, and Hardcore now wins it cleanly much more often. Ticket 0523 measured
`main` (commit `bba9876c`) and its own change, 100 tries per seed range:

| Persona | Band | `main`, seeds 1–100 | `main`, 101–200 | 0523, 1–100 | 0523, 101–200 |
| ------- | ---- | ------------------- | --------------- | ----------- | ------------- |
| Casual | 30–55% | 47 ✓ | 50 ✓ | 48 ✓ | 49 ✓ |
| Normal | 50–75% | 65 ✓ | 53 ✓ | 53 ✓ | 61 ✓ |
| Hardcore | 50–75% | 68 ✓ | **77 ⚠** | **76 ⚠** | 70 ✓ |

Over 200 tries Hardcore succeeds in 145 (`main`) and 146 (0523): about
73%, at the top of its band, so one range of 100 reads ⚠ and the other ✓.
Every later ticket whose acceptance says "Chapter 1's bands still ✓"
(0523 did) then depends on which seeds it ran. Hardcore stays faster than
Normal throughout (8 turns against 9).

## Nick input

None. The bands are Nick's (`docs/design/playtest-bots.md`); this only
moves Chapter 1 back inside them, with the autobalance's knobs, as 0506
and 0520 did for their chapters. What is left to judge by playing goes on
1064's checklist.

## Scope

**In:**
- Re-autobalance Chapter 1 (`cargo shuyi autobalance --stage ch01 --seed
  1 --turns 60 --apply`, built optimised; `docs/playtesting.md`,
  *Autobalancing*) so that every persona is in its band on seeds 1–100
  **and** 101–200.
- If its enemies change: re-record `ch01_winning.ron` (its test,
  `crates/ui/tests/it/ch01_winnable.rs`, says how), then check that
  Chapter 2's arrival still plays (`ch02_winnable`, `cargo xtask playtest
  ch02 --bot all` on both ranges: every band ✓ and Hardcore faster), and
  re-autobalance Chapter 2 if one moves (re-recording `ch02_winning.ron`
  if its Hounds change).
- Record the numbers in the Completion notes and in `docs/playtesting.md`
  (replace the 0506 block's numbers, or add a dated line under it).

**Out (do not do):**
- The personas (`personas.ron`, `trpg_bots::order`), the bands, the check.
- Chapters 3–8.

## Implementation steps

1. Run `cargo xtask playtest ch01 --bot all` on seeds 1–100 and 101–200
   with the current `main` to confirm the table above still holds.
2. Autobalance Chapter 1 (Scope), then play both seed ranges again.
3. Follow the chain into Chapter 2 (Scope) if Chapter 1's enemies changed.
4. Write the numbers down (Scope).

## Acceptance criteria

- [ ] `cargo xtask playtest ch01 --bot all` on seeds 1–100 and 101–200:
      every band ✓ and `hardcore · turns median … lower ✓` on both.
- [ ] If Chapter 1 changed: `ch01_winnable` and `ch02_winnable` pass, and
      Chapter 2's playtest on both ranges has every band ✓ and Hardcore
      faster.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the playtest reports above; `ch01_winnable` and
  `ch02_winnable`.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
