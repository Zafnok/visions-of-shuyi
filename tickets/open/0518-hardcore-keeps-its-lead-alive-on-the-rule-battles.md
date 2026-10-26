---
id: "0518"
title: "Find why Hardcore loses its lead early on test_escape and test_retreat"
type: bug
milestone: M4 Enemy AI
model: opus-5.5
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0518 — Find why Hardcore loses its lead early on test_escape and test_retreat

## Context

Found by 0516's persona smoke test over `assets-test/battles/`. The
Hardcore persona, the best player of the three (ADR-0095), loses its lead
(a Defeat) on turn 4 or 5 in every one of 3 tries of `test_escape`, and
on turn 1 in two of 3 tries of `test_retreat`. Normal wins most of the
same tries. Hardcore weighs a fallen lord at -10000, the loss at another
-10000 and a lord left where it could fall (`lord_at_risk`) at -5000
(`crates/bots/personas.ron`), so it should never walk its lead into
reach; something in what it sees or is offered is wrong.

Related, seen in 0516's Chapter 1 runs (the same before and after
0516): 5 of Hardcore's 20 tries of `ch01` (seeds 9, 10, 12, 15, 17) stop
unfinished at the 60-turn cap, with nobody fallen and no item used. If
the cause turns out to be the same, fix both; if not, file the Chapter 1
stall as a ticket of its own.

Reproduce: `cargo xtask playtest test_retreat --bot hardcore --runs 3`
and `cargo xtask playtest test_escape --bot hardcore --runs 3`.

Suspects to check, none confirmed: the danger count (`danger`,
`threats` in `crates/bots/src/persona.rs`) missing a threat these maps
have (reinforcements arriving next to the lead on turn 3 in
`test_escape`; a boss who retreats in `test_retreat`); Hardcore's
`turns: -200` pushing it to hurry; a test character that is simply too
weak for the map.

## Nick input

None, unless the cause is a setting or a weight: then stop, say so in
the ticket, and it waits for 0508 (ADR-0095 §5). If the cause is the
test battle itself (it is a test file, not a chapter), changing the test
file is fine.

## Scope

**In:**
- Find the cause by following the tries' commands, and fix it in the
  bots' scoring or order, or in the test file.
- A test that shows it.

**Out (do not do):**
- Any weight, band, floor or persona setting (ADR-0095 §5).
- Casual stalling on `test_escape` (0517).

## Implementation steps

1. Play the tries above and find the command that puts the lead where it
   falls, and what Hardcore scored it against the alternatives.
2. Fix the cause; write the test first.
3. Chapter 1's playtest (`cargo xtask playtest ch01 --bot all --runs
   20`) before and after, in the PR; any change explained.

## Acceptance criteria

- [ ] The cause is named in the Completion notes, with the fix.
- [ ] Hardcore wins at least 2 of 3 tries of each of `test_escape` and
      `test_retreat` (seeds 1 to 3).
- [ ] `personas.ron` is unchanged.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the fixed count or order (`crates/bots/src/persona/tests.rs`).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
