---
id: "0520"
title: Bots play a chapter with the army the chapters before it leave
type: feature
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0848", "0330", "0521"]
nick_input: none
completed:
---

# 0520 — Bots play a chapter with the army the chapters before it leave

## Context

The playtest and the autobalance (`docs/playtesting.md`; the engine's
`shuyi playtest`, `shuyi autobalance`) play a battle with "the
characters of the battle's player slots, fresh from the character data"
(`crates/xtask/src/playtest.rs`, `stage`; `trpg_content::battle_campaign`)
and the battle's default pack. That is right for Chapter 1, but not for
a later chapter:

- **Levels.** A player reaches Chapter 2 with Chapter 1's EXP (about two
  levels a battle early on, `progression.md`, *Pace*, 0608) and its
  unused-rewind bonus; the bots play it at the character data's levels.
- **The pack.** A battle with Preparations has no default pack (the
  player fills it from the stock), so the bots bring **no** consumables,
  where a player brings Chapter 1's three leftover Potions and whatever
  its gold buys.

So a later chapter tuned by the bots (0848 tuned Chapter 2 this way) is
tuned for a weaker army than a player brings, and plays easier than its
bands say. 0848 found it; its Completion notes list the numbers.

The 2026-11-01 PR audit confirmed this remains an acceptance deferral of
[PR 481](https://github.com/Zafnok/visions-of-shuyi/pull/481), not proof
that Chapter 2 is balanced for a campaign army. First-hit effects (0330)
and AI technique scoring (0521) must be corrected before the final
calibration here, so those fixes cannot invalidate its certified results.

**Either order with 1207:** if the engine playtest-command adoption is
done, put the shared campaign-army construction in the active
`game_tools` plan/stage adapter and use `cargo shuyi playtest`. If not,
use the existing wrapper with the same shared provider; 1207 preserves
that provider and the resulting campaign-army reports when it migrates
the command. This balance correction does not wait for optional cleanup.

## Nick input

None. The bands and personas are Nick's (`docs/design/playtest-bots.md`);
this changes only which army the bots bring, so they play a chapter as a
player arrives at it. Verified by tests and the bots' reports;
anything left to judge by playing goes on 1064's checklist.

## Scope

**In:**
- The army the bots field for a story battle: the campaign as the
  chapters before it leave it. One way (the session decides, and writes
  an ADR if it is a new pattern): play the earlier chapters' recorded
  playthroughs (`crates/ui/tests/it/fixtures/ch0N_winning.ron`, 0803,
  0848) through the rules and apply each result, as
  `crates/ui/tests/it/ch02_winnable.rs`'s `chapter_2` does; or a
  persona's own run of the earlier chapters; or a level per chapter in
  data. Whatever is chosen must be deterministic.
- For a battle with Preparations: the pack filled from that army's
  stock the way a player would (consumables up to the pack cap), and
  what its gold buys at the chapter's basic shop, by a rule written in
  `docs/playtesting.md` (*the agent's starting rule*).
- Re-run `cargo shuyi autobalance --stage ch02 --apply` and the playtest
  with the new army; update `assets/battles/ch02.ron` and 0848's numbers
  where they move.
- Record the starting army, carried resources, seed set and configured
  band verdicts alongside the report. A fresh-army run remains a diagnostic
  comparison and cannot satisfy the campaign-army acceptance check.
- Find out why Hardcore is not faster than Normal on `ch02` (0848: median
  6 turns against 4–5, `lower ✗`), where it was on `ch01`. A village
  race may make saving houses cost turns, which the check doesn't know:
  fix the bot if it is the bot, or say in `playtest-bots.md`'s report
  section why the check reads that way on a village map.

**Out (do not do):**
- Changing the personas, bands or what a persona knows.
- Chapters 3–8's balance (their own tickets, which then use this).
- Preparations choices beyond the pack (loadouts stay as they are).

## Implementation steps

1. Read `docs/playtesting.md`, `crates/xtask/src/playtest.rs` (or the
   active `game_tools` playtest plan after 1207),
   `crates/xtask/src/game_tools/balance.rs`, `crates/bots/src/playtest.rs`
   and `trpg_content::chapter::battle_campaign`.
2. Build the army for a story battle (Scope) in one function used by the
   playtest and the autobalance alike; keep the battle played on its
   own (the debug Quick Battle) as it is.
3. Fill a Preparations battle's pack (Scope).
4. Tests: the army for `ch02` has Chapter 1's levels and the pack holds
   the Potions; Chapter 1's army is unchanged.
5. Re-run the autobalance and the playtest for `ch02`; record the
   numbers in the Completion notes and in `docs/playtesting.md`.

## Acceptance criteria

- [ ] The active command plays with the army Chapter 1 leaves and a
      pack of its Potions (a test shows the setup):
      `cargo xtask playtest ch02 --bot all` before 1207, or
      `cargo shuyi playtest --stage ch02 --entrant all` afterward.
- [ ] Chapter 1's playtest numbers don't move.
- [ ] Chapter 2 meets its configured bands with the campaign army; the
      report includes every persona and the Hardcore/Normal comparison.
      A written explanation of a failed band does not count as a pass;
      unresolved failures remain tracked and block certification.
- [ ] The playtest and autobalance use the same deterministic starting
      army, stock, gold and pack; a regression rejects a fresh level-data
      army or an empty default pack for this campaign case.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the army and pack built for `ch01` and `ch02`.
- Integration: the playtest's setup for `ch02` (xtask tests).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
