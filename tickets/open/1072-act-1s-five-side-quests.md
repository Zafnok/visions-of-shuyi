---
id: "1072"
title: "Act 1's side quests, part 1: Tamsin and Aske"
type: content
milestone: Post–Chapter 1
model: fable-5.1
effort: high
status: todo
blocked_by: ["1070", "0018", "0069", "0746", "0747", "0509", "0013", "0323", "0608", "0329"]
nick_input: none
completed:
---

# 1072 — Act 1's side quests, part 1: Tamsin and Aske

## Context

**Split by ticket 0135** (five quests are about 4,000 changed lines, past
the limit of about 3,000 for one PR; ADR-0101). This ticket builds Tamsin's
and Aske's quests; 1083 builds Hollis's and Rue's, 1084 Maud's. The table
keeps all five for the whole picture, and the rest of this ticket is what
1083 and 1084 read for how a quest is built.

Side quests (paralogues) are "optional, handcrafted battles tied to one
character's story, with story scenes and a unique reward", unlocked by
story progress and conditions (`world-structure.md`). Act 1 has five
(`docs/story/outline.md`):

| Quest | Whose | Unlocked | What it is |
| --- | --- | --- | --- |
| *The Company Ledger* | Tamsin | after the Aldwater Toll Bridge (Ch 4), Tamsin recruited and alive | she finds the crown paymaster who pocketed the Red Company's wages, and has to decide what the money is worth |
| *Letters Home* | Aske | after Chapter 5's story battle, Aske alive | a refugee boat carrying her letters and silver home has been seized by League privateers |
| *The Captain's Oath* | Hollis | after Chapter 4's story battle, Hollis alive | his wife's grave near Veyne Hall; his goodbye to the life before exile (his confession stays in the main story) |
| *The Other Cart* | Rue | after Chapter 6's story battle, Rue alive | a Candle who escaped with her runs a hideout for runaways |
| *Unwritten* | Maud | once Ossary is reached, Maud alive | she brings the register's names back to their families |

Their rewards are of the kinds 0069 decided, and special weapons come
from 0018 (`combat-arts.md`, *Special weapons*: a personal weapon is a
special weapon locked to one character, built by 0329). The chapters' scripts (0746, 0747, 0749) come first, so the
quests fit what the ledger says has happened. 1070 places their nodes,
locked; this ticket writes and builds them and fills in the nodes.

## Nick input

None; no check after merge (Nick, 2026-10-21). Which reward each quest gives
is picked here, inside 0069's kinds, and listed in the Completion notes for
1064's checklist. The question about the runaway Candle of *The Other Cart*
moved to 1083 with Rue's quest.

## Scope

**In:**
- For Tamsin's and Aske's quests: a short beat sheet (in `docs/story/chapters/`, e.g.
  `side-tamsin.md`), its scenes (`assets/dialogue/side_*.dlg`, by the
  `story-writing` skill with 0736's craft rules and the critique pass),
  its map and battle (as 0848–0854 do: map, units, triggers, place
  maps, `music: Cue("side_quest")`), its unique reward, and its node in
  1070's map with the unlock condition.
- New names in `docs/story/names.md` and `assets/data/names.ron`; minor
  speakers who need a face use the shared portraits (`soldier` and the
  like) unless 0070 picked one.
- The ledger updated for each.
- `cargo shuyi autobalance` and `playtest --bot all` per battle.

**Out (do not do):**
- Hollis's and Rue's quests (1083); Maud's (1084).
- Supports and camp events (1073); fixed skirmishes (1071).
- Changing what the main story says happened.

## Implementation steps

1. Read the `story-writing` skill, the outline, the ledger, Tamsin's and
   Aske's sheets, 0746 and 0747's scripts, 0069's and 0018's answers.
2. Per quest: beat sheet, script, critique pass, map, battle, reward,
   node.
3. Bots; `frame-png` pictures; ledger.

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
