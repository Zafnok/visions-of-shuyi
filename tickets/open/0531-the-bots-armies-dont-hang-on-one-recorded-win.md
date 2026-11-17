---
id: "0531"
title: "The bots' armies don't hang on one recorded win"
type: feature
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0531 — The bots' armies don't hang on one recorded win

## Context

The bots start every story chapter after the first with the army the
recorded winning playthroughs of the chapters before it leave
(`docs/playtesting.md`, *The army*; `crates/bots/src/arrival.rs`). A
recording is one search's best line (`crates/ui/tests/it/playthrough.rs`,
`record`): it knows the battle's luck and feeds the EXP to whoever it
happened to use. Any change that alters one roll of an earlier chapter
breaks the recording; the new one spreads the EXP differently, and every
later chapter's bands move with it, though the chapters didn't change.

0864 showed how far: Chapter 1 recorded again **with no change at all**
moved Chapter 2 from 45 / 63 / 55 (Casual / Normal / Hardcore, all in
band) to 14 / 37 / 46 (blocked). The old recording had fed Tamsin to
level 8 with the lead at 1; the new one ended Tamsin 5 and the lead 5.
After 0864's change, Chapters 4 and 5 dropped out of band the same way
(0876). Every tuning ticket since 0524 has paid for this.

## Nick input

None while the army stays what `docs/playtesting.md` says it is (the
army a player brings). If the fix changes that meaning (for example an
"average player's" army rather than one played line), ask with
`ask-nick` first, since `docs/design/playtest-bots.md` is Nick's.

## Scope

**In:**
- Make the arrival army stable against a new recording: the agent's
  choice, recorded as an ADR. Candidates: build the army from several
  recordings (different seeds) and judge each chapter over all of them;
  or keep the recording for the replay tests only and give the bots a
  reference army per chapter (as the side quests' `bots_army` does),
  levelled by the EXP the chapters before it pay; or a recorder that
  spreads the EXP the way the personas do.
- Show it on 0864's case: Chapter 1 recorded with two different seeds
  gives Chapter 2 reports within the noise (±5) of each other.
- `docs/playtesting.md`, *The army*.

**Out (do not do):**
- Retuning any chapter (0876 and later tuning tickets).
- The bands, the success rules, the personas' play.

## Implementation steps

1. Read `crates/bots/src/arrival.rs`, `crates/xtask/src/playtest.rs`
   (`stage`), `crates/ui/tests/it/playthrough.rs` and 0864's Completion
   notes.
2. Measure the spread first: record Chapter 1 with three seeds (the
   `seed` argument of `playthrough::record`, 803 today) and play Chapter
   2 with each army.
3. Pick the fix, write the ADR, build it, and measure the spread again.
4. Re-run every story battle's playtest; report the numbers before and
   after in the PR.

## Acceptance criteria

- [ ] Chapter 2's three success rates move by no more than about 5
      points between armies built from Chapter 1 recordings with
      different seeds (numbers in the PR).
- [ ] An ADR records the choice; `docs/playtesting.md` says how the army
      is built.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the army builder (the same content gives the same army; two
  recordings give one army, or the reports say which army each try used).
- The bots' playtest runs (reported in the PR).

## Completion notes

*(Filled in by the session that completes the ticket.)*
