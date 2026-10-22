---
id: "0855"
title: "Chapter 1 back in its playtest bands after the four armour weights"
type: tuning
milestone: M7 Chapter 1 & game flow
model: sonnet-5
effort: medium
status: todo
blocked_by: ["0321"]
nick_input: none
completed:
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
- If 0509 (autobalance) is done, use `cargo xtask autobalance ch01` to
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

- [ ] `cargo xtask playtest ch01 --bot all`: Casual, Normal and Hardcore
      all ✓ on seeds 1–100 and on seeds 101–200; numbers in the
      Completion notes.
- [ ] The lead is no longer Hardcore's most-fallen unit, or the notes
      say why it should be.
- [ ] No armour number, class number or band changed.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: `ch01_winnable` (the recorded playthrough still wins with
  nobody lost).
- Snapshot: the ones the change moves, each read before accepting.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
