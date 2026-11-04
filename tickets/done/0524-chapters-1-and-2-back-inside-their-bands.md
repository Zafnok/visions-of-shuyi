---
id: "0524"
title: Chapters 1 and 2 back inside their bands with the new order
type: tuning
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["0523"]
nick_input: none
completed: 2026-11-04
---

# 0524 — Chapters 1 and 2 back inside their bands with the new order

## Context

Ticket 0523 changed the order in which the personas weigh their moves
(ADR-0112: each unit's best attack first, its other targets after every
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

- [x] `cargo xtask playtest ch01 --bot all` on seeds 1–100 and 101–200:
      every band ✓ and `hardcore · turns median … lower ✓` on both.
- [x] The same for `ch02`.
- [x] `ch01_winnable` and `ch02_winnable` pass.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the playtest reports above; `ch01_winnable` and
  `ch02_winnable`.

## Completion notes

**Done.** Every band is met on both chapters and both seed ranges, with
Hardcore faster than Normal on all four.

- **Step 1** reproduced the ticket's tables exactly (ch01: 48/53/76 ⚠
  and 49/61/70; ch02: 75 ⚠/64/62 and 64 ⚠/67/60).
- **Chapter 1** (`cargo shuyi autobalance --stage ch01 --seed 1 --turns
  60`, 12 evaluations, 45 min): `unnamed enemies' level -1`. Every Red
  Company template is level 1 already, so the only enemy it moves is the
  raider at the road bridge's north end (11,5), level 2 → 1. Applied by
  hand with a comment (the same text `--apply` writes for that line).
  `ch01_winning.ron` re-recorded: won on turn 10 (was 12), nobody lost.
- **Chapter 2, the trace** (a throwaway test calling `play_observed` on
  the arrival army, 200 tries each, house and fall events): Normal won
  178 tries, Casual 139. Normal's misses are deaths; Casual's are defeats,
  28 by the village and 33 by its lord. Casual loses the village mostly
  when a south-bank house goes unevacuated on the Hounds' first turn (it
  strays: depth 2, temperature 100). So a tighter first-turn race costs
  Casual's wins more than Normal's clean wins, which levels can't do
  (they move every persona alike, as 0523 found).
- **Chapter 2, the tune:** a second raider beside the west house's
  raider, at 1,13 (same template and level). Whoever evacuates the west
  house now faces two axes on the Hounds' first turn: Normal sends
  someone sturdier or kills one; Casual often loses its evacuator there
  and runs short later. `ch02_winning.ron` re-recorded against it and the
  new Chapter 1 army: won on turn 14, nobody lost, 3 of 5 houses saved.

| Battle, seeds | Casual (30–55) | Normal (50–75) | Hardcore (50–75) | Hardcore turns vs Normal |
| ------------- | -------------- | -------------- | ---------------- | ------------------------ |
| ch01, 1–100 (was 48 / 53 / 76 ⚠) | 45 ✓ | 61 ✓ | 70 ✓ | 8 vs 9 ✓ |
| ch01, 101–200 (was 49 / 61 / 70) | 50 ✓ | 56 ✓ | 67 ✓ | 8 vs 9 ✓ |
| ch02, 1–100 (was 75 ⚠ / 64 / 62) | 43 ✓ | 54 ✓ | 64 ✓ | 5 vs 6 ✓ |
| ch02, 101–200 (was 64 ⚠ / 67 / 60) | 39 ✓ | 60 ✓ | 72 ✓ | 5 vs 6 ✓ |

  Chapter 2's army is now Lead lv 1, Hollis lv 4, Tamsin lv 8, Aske lv 2,
  Maud lv 1, Rue lv 3 (was Hollis 5, Tamsin 6).

- **Other Chapter 2 variants screened** (Casual / Normal, seeds 1–100 and
  101–200, against the new army): Chapter 2 unchanged 58 ⚠ 58 ⚠ / 61 55;
  the same copy on the count knob's own tile (1,11) 44 34 / 41 ✗ 41 ✗;
  plus a copy by the north-east raider 28 ✗ 33 / 55 49 ✗; plus the
  guarding raider at 23,5 made a raider 33 32 / 55 49 ✗; plus the guard
  at 10,6 one level down 39 36 / 52 61. Against the old army, copies by
  the raiders at 7,9 or 22,10 cost Normal far more than Casual (Normal
  22–44).

- **Two test edits that follow the battle:** `ch02_winnable` counts the
  raid at 15 Hounds (was 14; no scene names the number), and
  `playthrough::on_to_the_save_prompt` gives the end of a battle 60 steps
  of half a second (was 40): the new recording wins on turn 14 with more
  Hounds left to flee one by one, and needs between 41 and 50.

**Deviations:** Chapter 2's change isn't one the autobalance writes: its
count knob puts the copy on the first free tile it tries (1,11), which
is too hard for Normal (41 on both ranges). The ticket allows the
battle's numbers the trace points at when the knobs can't; the enemy is
the knob's copy, one tile further south. Chapter 3 has no battle file
yet (0849 is still open), so there was nothing to replay there.

**Follow-up tickets:** none.

**the agent's starting rules:** none (no gameplay rule; battle numbers
only).

**Left to judge by playing (1064):** whether the second raider at Kell's
Ford's west house feels fair on the first turn (a player has to evacuate
that house with a unit that can take two hits, or kill one raider
first), and whether the road bridge's raider at level 1 makes Chapter 1's
north bank too soft.

Engine friction: none: no code.
