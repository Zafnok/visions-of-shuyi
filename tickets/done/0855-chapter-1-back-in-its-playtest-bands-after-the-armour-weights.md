---
id: "0855"
title: "Chapter 1 back in its playtest bands after the four armour weights"
type: tuning
milestone: M7 Chapter 1 & game flow
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["0321"]
nick_input: none
completed: 2026-10-28
---

# 0855 — Chapter 1 back in its playtest bands after the four armour weights

## Context

Ticket 0321 put 0073's four armour weights in the data: the Leather Vest
went from Def +1 / weight 0 to Def +2 Res +2 / weight 1, Iron Plate to
Def +8 Res −3 / weight 6, Maud and Rue swapped their vests for Linen
Robes (Res +4, weight 0). The bots then played Chapter 1 (`cargo xtask
playtest ch01 --bot all`, seeds 1–100, 2026-10-22) and two personas left
their bands (`docs/playtesting.md`):

| Persona | Before 0321 | After 0321 | Band |
| ------- | ----------- | ---------- | ---- |
| Casual | 44/100 succeed | 51/100 ✓ | 30–55% |
| Normal | 59/100 succeed | 47/100 ✗ too hard | 50–75% |
| Hardcore | 51/100 succeed | 25/100 ✗ too hard | 50–75% |

Hardcore's wins fell from 83 to 46 of 100 and its most-fallen unit is
now the **lead (47 tries)**, where it was Hollis Marr. The likely cause:
the lead (Exile, Str 7) wears a Leather Vest, whose new weight 1 is now
burden (`burden = weapon wt + armour wt − Str / 5`), so the lead loses
1 attack speed and with it the double on Brigands (the forecast
snapshots of 0321 show "16 ×2" become "8 ×1"). Everyone else in vests
loses the same point.

The armour numbers are Nick's decision (`weapons-and-items.md`,
*Armour*, 0073) and stay as they are; this ticket moves Chapter 1's own
numbers so the chapter is back in its bands.

## Nick input

None. Verified by the bots (`cargo xtask playtest ch01 --bot all` in
its bands). What is left to judge by playing (whether the chapter feels
right) goes on the Act 1 playtest's checklist (1064).

## Scope

**In:**
- Find which units and fights changed (the `try` lines: who falls, on
  which turn; the forecast of the lead against a Brigand).
- Retune **Chapter 1's** data so Casual, Normal and Hardcore succeed
  inside their bands: the cast's starting loadouts and levels in
  `assets/data/characters.ron` (Chapter 1's characters only), Chapter
  1's enemies (their levels, loadouts, positions, AI) in
  `assets/battles/ch01.ron`. Prefer the smallest change; say why each
  change was picked.
- Re-record Chapter 1's playthrough if the change breaks it
  (`cargo test -p trpg-ui --test it ch01_winnable::record -- --ignored`,
  `crates/ui/tests/it/ch01_winnable.rs`), and review the snapshots that
  move.
- If 0509 (autobalance) is done, use `cargo shuyi autobalance --stage ch01` to
  find the change; if not, tune by hand with `playtest`.

**Out (do not do):**
- Any armour number (weights, bonuses, prices) or which class wears
  what: decided by 0073; a change is a design question for Nick.
- The burden formula, class bases or growths (`progression.md`, 0013).
- The bands themselves (`docs/playtesting.md`, Nick's decision).
- Other chapters.

## Implementation steps

1. `cargo xtask playtest ch01 --bot all` on `main`; read the report and
   the `try` lines of the Hardcore tries the lead fell in.
2. Pick the smallest Chapter 1 change that explains it (for example: a
   Brigand near the lead's opening path a level lower, or one fewer
   Brigand in the first wave — whichever the numbers point to). Write
   the reason in the data file's comment.
3. Re-run until all three personas are in their bands (✓) on seeds
   1–100; then check seeds 101–200 (`--seed 101`) to make sure it isn't
   one lucky set of seeds.
4. Fix `ch01_winnable` (re-record) and the snapshots that move.

## Acceptance criteria

- [x] `cargo xtask playtest ch01 --bot all`: Casual, Normal and Hardcore
      all ✓ on seeds 1–100 and on seeds 101–200; numbers in the
      Completion notes.
- [x] The lead is no longer Hardcore's most-fallen unit, or the notes
      say why it should be.
- [x] No armour number, class number or band changed.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: `ch01_winnable` (the recorded playthrough still wins with
  nobody lost).
- Snapshot: the ones the change moves, each read before accepting.

## Completion notes

Worked on Nick's request (2026-10-28, after 0519): "make ch1 slightly
harder then. we want hardcore in band". Stacked on 0519, whose fix to
the bots' heal order is what moved the numbers.

- **Where it stood.** By the time this ticket was picked up, the bots had
  changed (0516–0519) and the chapter was no longer too hard: after 0519,
  seeds 1–100, Casual 41 ✓, Normal 62 ✓, Hardcore **76 ⚠ too easy**
  (the lead its most-fallen, 14×, all on turns 6–8 at the toll-fort).
- **The change, one line:** the fort's Archer (21, 3) carries a Steel Bow
  (Mt 9, Hit 70) instead of its class's Iron Bow (Mt 6, Hit 85). Hardcore's
  lead fell storming the fort, so the fort is where a small push belongs.
- **Tried and dropped** (Hardcore, seeds 1–100): the fort's Raider at
  level 2 (77, no change); the road bridge's Raider at level 3 (79, and
  Casual 58 ⚠); every unnamed enemy a level up (76, Casual 58 ⚠). A higher
  enemy level makes Chapter 1 *easier*: it pays the cast more experience,
  and they level up in the battle.
- **Results** (`cargo xtask playtest ch01 --bot all --runs 100`):

  | Persona | Band | Seeds 1–100 | Seeds 101–200 |
  | ------- | ---- | ----------- | ------------- |
  | Casual | 30–55% | 44 ✓ | 52 ✓ |
  | Normal | 50–75% | 66 ✓ | 58 ✓ |
  | Hardcore | 50–75% | 72 ✓ | 73 ✓ |
  | Hardcore turns (vs Normal) | lower | 8 vs 9 ✓ | 8 vs 9 ✓ |

  Hardcore wins 93 and 94 of 100; its most-fallen is now Tamsin Rook (12×
  and 10×), not the lead.
- No armour, class or band number changed; Chapter 1's map and the
  cast are untouched.
- `ch01_winnable` still wins with nobody lost (the recorded playthrough
  needed no re-record); no snapshot moved.
- Left for the Act 1 playtest (1064): whether a Steel Bow on a fort
  archer feels right in Chapter 1.
- No gameplay rule decided. Engine friction: none, no Rust code.
