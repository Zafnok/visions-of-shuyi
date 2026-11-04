---
id: "0524"
title: Chapters 1 and 2 back inside their bands with the new order
type: tuning
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0523"]
nick_input: none
completed:
---

# 0524 — Chapters 1 and 2 back inside their bands with the new order

## Context

Ticket 0523 changed the order in which the personas weigh their moves
(ADR-0111: each unit's best attack first, its other targets after every
unit's move). It fixed Hardcore's stall at Kell's Ford, and it left two
bands out, measured with 100 tries per seed range:

**Chapter 2** (`assets/battles/ch02.ron`, Kell's Ford, as 0520 tuned it):
Casual now plays it much better.

| Persona | Band | `main`, seeds 1–100 | `main`, 101–200 | 0523, 1–100 | 0523, 101–200 |
| ------- | ---- | ------------------- | --------------- | ----------- | ------------- |
| Casual | 30–55% | 47 ✓ | 50 ✓ | **75 ⚠** | **64 ⚠** |
| Normal | 50–75% | 52 ✓ | 61 ✓ | 64 ✓ | 67 ✓ |
| Hardcore | 50–75% | 55 ✓ | 57 ✓ | 62 ✓ | 60 ✓ |
| Hardcore faster | | ✓ | ✗ | ✓ (5 vs 6) | ✓ (5 vs 6.5) |

Casual's success is a win; Normal's and Hardcore's a win with nobody
dying. Casual now wins about as often as Normal wins cleanly, and Casual's
misses are defeats (its lord falls, or the village falls below its
minimum) where Normal's are deaths. 0523 ran the autobalance twice
(`cargo shuyi autobalance --stage ch02 --seed 1 --turns 60`, 100
evaluations and then 60 more from the best setting, 4.6 h and 2.7 h):
its best setting (raider at 1,12 +2, raider at 22,10 −2, guard at 8,6 −2,
raider at 23,5 +1) gave Casual 58 ⚠, Normal 65, Hardcore 69, and Hardcore
no faster than Normal (5 against 5). It was not applied. Every knob moves
the three alike; something must hurt Casual's wins more than Normal's
clean ones.

**Chapter 1** (`assets/battles/ch01.ron`): Hardcore sits at the top of its
band before and after 0523, so one range of 100 reads ⚠ and the other ✓.

| Persona | Band | `main`, seeds 1–100 | `main`, 101–200 | 0523, 1–100 | 0523, 101–200 |
| ------- | ---- | ------------------- | --------------- | ----------- | ------------- |
| Casual | 30–55% | 47 ✓ | 50 ✓ | 48 ✓ | 49 ✓ |
| Normal | 50–75% | 65 ✓ | 53 ✓ | 53 ✓ | 61 ✓ |
| Hardcore | 50–75% | 68 ✓ | **77 ⚠** | **76 ⚠** | 70 ✓ |

Over 200 tries Hardcore succeeds in 145 (`main`) and 146 (0523): about
73%. Hardcore stays faster than Normal throughout (8 turns against 9).
Chapter 2's army is what Chapter 1's recorded playthrough leaves
(ADR-0110), so Chapter 1 goes first.

## Nick input

None, unless step 4 finds that no setting of the battle can meet
Casual's band on Chapter 2: then the band against this Casual bot is a
design question for Nick (`ask-nick`; `docs/design/playtest-bots.md`:
"Casual's band is the Casual bot's, not a casual player's"), or a calibration for 0508; ask before changing
anything else. What is left to judge by playing goes on 1064's
checklist.

## Scope

**In:**
- Chapter 1: re-autobalance (`cargo shuyi autobalance --stage ch01 --seed
  1 --turns 60 --apply`, built optimised; `docs/playtesting.md`,
  *Autobalancing*) so that every persona is in its band on seeds 1–100
  **and** 101–200. If its enemies change, re-record `ch01_winning.ron`
  (`crates/ui/tests/it/ch01_winnable.rs` says how).
- Chapter 2: find why Casual wins it as often as Normal wins it cleanly
  (trace Casual and Normal tries, as 0523 traced Hardcore's: a throwaway
  test calling `shuyi_core::bots::play_observed` on the arrival army),
  then tune it with the autobalance's knobs or, if they can't, with what
  the trace points at among the battle's numbers. Every band on both seed
  ranges, Hardcore faster. Re-record `ch02_winning.ron` if its Hounds
  change.
- Record the numbers in the Completion notes and in `docs/playtesting.md`.
- Either order works with 0849 (Chapter 3), whose army is what Chapter 2's
  recording leaves: if 0849 is done first and `ch02_winning.ron` changes
  here, play `cargo xtask playtest ch03 --bot all` on both ranges again and
  re-tune Chapter 3 if a band moves.

**Out (do not do):**
- The personas (`personas.ron`, `trpg_bots::order`), the bands, the check.
- Chapters 3–8.
- Hendry, the player's slots, the map, the objective (the knobs never
  touch them).

## Implementation steps

1. Run `cargo xtask playtest ch01 --bot all` and `ch02` on seeds 1–100 and
   101–200 with the current `main` to confirm the tables above.
2. Autobalance Chapter 1 (Scope); play both ranges again.
3. Play Chapter 2 again with the army Chapter 1 now leaves; if a band is
   still out, trace (Scope).
4. Tune Chapter 2 (Scope), or stop and ask (Nick input).
5. Write the numbers down.

## Acceptance criteria

- [ ] `cargo xtask playtest ch01 --bot all` on seeds 1–100 and 101–200:
      every band ✓ and `hardcore · turns median … lower ✓` on both.
- [ ] The same for `ch02`.
- [ ] `ch01_winnable` and `ch02_winnable` pass.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the playtest reports above; `ch01_winnable` and
  `ch02_winnable`.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
