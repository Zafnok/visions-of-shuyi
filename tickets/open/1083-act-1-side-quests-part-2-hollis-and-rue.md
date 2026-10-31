---
id: "1083"
title: "Act 1's side quests, part 2: Hollis and Rue"
type: content
milestone: Post–Chapter 1
model: creative
effort: high
status: todo
blocked_by: ["1072", "1070", "0018", "0069", "0746", "0748", "0509", "0013", "0323", "0608", "0329"]
nick_input: decision
completed:
---

# 1083 — Act 1's side quests, part 2: Hollis and Rue

## Context

Split from 1072 by ticket 0135 (five quests are about 4,000 changed lines,
past the limit of about 3,000 for one PR; ADR-0101). 1072 holds the context,
the scope per quest and the steps: **read it first**, and build these two the
way it built Tamsin's and Aske's. It comes first so the names and the ledger
stay in order; the stories are independent.

| Quest | Whose | Unlocked | What it is |
| --- | --- | --- | --- |
| *The Captain's Oath* | Hollis | after Chapter 4's story battle, Hollis alive | his wife's grave near Veyne Hall; his goodbye to the life before exile (his confession stays in the main story) |
| *The Other Cart* | Rue | after Chapter 6's story battle, Rue alive | a Candle who escaped with her runs a hideout for runaways |

## Nick input

**Decision**, one story question asked first (`ask-nick`): the outline
says the runaway Candle of *The Other Cart* is a recruit "or" not,
"decided by that side quest's ticket". Offer: she joins (with what she
adds to the army), she stays at the hideout (a camp event later), or
describe your own. If she joins and the join happens outside battle,
that is 1009's question (quest recruitment): answer 1009 first, add it
to `blocked_by`, and stop until it is done.

Nothing else is asked; no check after merge (Nick, 2026-10-21). Which
reward each quest gives is picked here, inside 0069's kinds, and listed
in the Completion notes for 1064's checklist.

## Scope

**In:** for these two quests, everything 1072's scope lists per quest (beat
sheet, scenes, map and battle, unique reward, node in 1070's map with its
unlock condition, names, ledger, autobalance and playtest).

**Out (do not do):** Tamsin's and Aske's quests (1072); Maud's (1084);
supports and camp events (1073); fixed skirmishes (1071); changing what the
main story says happened.

## Implementation steps

1. Read 1072 and what it built, the `story-writing` skill, the outline, the
   ledger, Hollis's and Rue's sheets, the scripts of Chapters 4 and 6, 0069's
   and 0018's answers.
2. Ask the Candle question; record the answer in the outline.
3. Per quest: beat sheet, script, critique pass, map, battle, reward, node.
4. Bots; `frame-png` pictures; ledger.

Expected size: about 1,700 changed lines (0135; `cargo xtask pr-size`).

## Acceptance criteria

- [ ] Each quest unlocks under its condition and not otherwise (tests).
- [ ] Each can be won (winning replays) and gives its reward.
- [ ] Scripts pass `cargo xtask check-script`; the critique pass is
      ticked in the PR body.
- [ ] The bots land in their bands for each; reports in the PR.
- [ ] Starting values and the rewards are in the Completion notes
      (1064's checklist).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: unlock conditions (character dead or not recruited →
  locked); winning replays; the reward in the army's stock.
- Content checks for every new file.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
