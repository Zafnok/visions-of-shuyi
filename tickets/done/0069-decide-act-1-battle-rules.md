---
id: "0069"
title: "Decide: Act 1's battle rules (escape, a fleeing boss, a boss who can't die, Hollis's death, side-quest rewards, what each chapter introduces)"
type: design-decision
milestone: Design decisions
model: opus-5.5
effort: medium
status: done
blocked_by: []
nick_input: decision
completed: 2026-10-22
---

# 0069 — Decide: Act 1's battle rules

## Context

Act 1 (`docs/story/outline.md`, Chapters 1–8) needs battle rules that
Chapter 1 never did. `core` today has the Rout, DefeatUnit, Seize and
Survive objectives, reinforcements, talk-recruit, battle notes,
elementals, chests and shops. The outline asks for five more things, and
the design docs don't settle any of them:

1. **Chapter 6, "The Ashfields":** "Objective: reach the Brennish lines
   (escape-style; the chapter ticket decides)". No escape objective
   exists.
2. **Chapter 4, "Veyne Hall":** "Dace is the boss and escapes at low HP."
   No rule makes a boss leave the map.
3. **Chapter 8, "Saltmere Docks":** "Dace is the boss and can't be
   killed; he retreats." No rule makes a unit unkillable.
4. **Chapter 8:** "**Hollis dies.** He holds the gangway so the party can
   board." In Classic, "if Hollis already fell, the scene is rewritten so
   that the lead tells Dace instead (the Chapter 8 script ticket writes
   both versions)". How the death happens in play is not decided.
5. **Side quests** give "a unique reward" (`world-structure.md`), and
   random skirmishes "never unique items". What kinds of unique reward
   exist is not decided (0018 decides the special weapons themselves).
6. **What each early chapter introduces.** The outline's table (*New
   things each early chapter introduces*) is "*Claude's proposal; each
   chapter ticket confirms with Nick.*" Confirming it once here, for all
   of Act 1, keeps the chapter tickets free of design questions.

The answers feed 0319 (the rules in `core`), the chapter tickets
(0848–0854; directly 0848 and 0849, the others through 0319), the side
quests (1072), and the Chapter 8 script (0750).

## Nick input

**Decision**, with the `ask-nick` skill (the question pop-up, at most
three questions per message, several rounds expected). Each option comes
with what real games do (check every reference before citing it, and
say honestly when unsure), plus "describe your own". Word the rules as
player situations with an example (Nick didn't follow abstract ones).

## Questions to ask

1. **The escape objective (Chapter 6).** What counts as escaping:
   A) the lead reaches the exit area (Fire Emblem's "Seize" moved to an
   edge; e.g. FE Fates' "escape" maps, where every unit must reach the
   exit tiles), B) **every** deployed unit reaches the exit area and
   leaves the map one by one (Fates), C) any N units reach it,
   D) describe your own. Sub-questions: does a unit that reaches the
   exit leave the map (and is it safe from then on); what happens to a
   unit that never gets there (left behind = retreat in Casual, death in
   Classic?); is there a turn limit.
2. **A boss who flees at low HP (Dace, Chapter 4).** When he flees:
   A) the first time his HP falls below a threshold (e.g. half), he
   says a line and leaves on his next turn (FE Three Houses' bosses who
   "retreat" when beaten), B) the hit that would kill him leaves him at
   1 HP and he leaves at once, C) describe your own. Does the battle end
   when he flees (the objective is "defeat Dace", so fleeing counts as
   the win), or does it go on as a Rout? Does he give EXP as if
   defeated?
3. **A boss who can't die (Dace, Chapter 8).** A) attacks can't take him
   below 1 HP, and he leaves when the objective (hold for N turns) is
   met; B) he can't be targeted at all (he stays back and commands);
   C) he leaves at low HP as in question 2; D) describe your own. What
   does the forecast show (FE: an unkillable boss shows a normal
   forecast; the hit leaves 1 HP)?
4. **Hollis's death (Chapter 8).** A) Hollis is deployed and the battle
   plays out; when it is won, the closing scene kills him, whatever
   happened in the battle (FE Blazing Blade's Leila, killed in a scene);
   B) he must be deployed, and at a set turn he "holds the gangway": he
   leaves the map in a scene, and the battle goes on without him;
   C) he isn't deployable in Chapter 8 at all (he is in the scenes
   only); D) describe your own. And in Casual, where a fallen Hollis
   retreats instead of dying, is the scripted death the same (yes in
   every option above, unless Nick says otherwise)? The Classic case
   where he already died earlier is decided (the lead tells Dace).
5. **The kinds of unique side-quest reward.** Each of the five Act 1
   side quests (Tamsin, Aske, Hollis, Rue, Maud) gives one. Options to
   offer (more than one may be picked): A) a personal weapon for that
   character (FE: Lyn's Mani Katti; 0018's special weapons); B) a skill
   or Combat Art only they learn; C) a stat booster (FE's Energy Ring,
   Seraph Robe); D) a class-change item that unlocks a class otherwise
   locked; E) gold or a rare shop stock in a town; F) describe your own.
   Also: does losing the quest (or skipping it) lose the reward for
   good?
6. **What each early chapter introduces** (the outline's table, Claude's
   proposal at gate 2): Chapter 2 Preparations, talk-to-recruit and
   villages; Chapter 3 a Seize objective and chests; Chapter 4 the world
   map, a town, fixed skirmishes and the first side quest; Chapter 6
   elemental enemies (echoes) and battle notes. Keep it, or move
   something (e.g. FE GBA brings in one new thing per chapter for its
   first ten)?

## Scope

**In:**
- The questions above, asked with mockups where a picture helps (an
  escape area on a map, the forecast against an unkillable boss).
- Record the answers in the design docs: the objectives in
  `docs/design/turn-structure.md` (or `battle-scenes-and-recruitment.md`
  for the scene parts; pick the one that already holds objectives), the
  Hollis rule beside the Classic/Casual rules in
  `death-and-difficulty.md`, the reward kinds in `world-structure.md`
  (*Side quest*), the confirmed table in `docs/story/outline.md` (drop
  "*Claude's proposal; each chapter ticket confirms with Nick*"), Nick's
  words verbatim in each.
- Update 0319's *Scope* if an answer adds or removes a rule.

**Out (do not do):**
- Implementing anything (0319).
- Which reward each side quest gives (1072 picks, inside Nick's answer).
- Chapter-specific numbers (turn counts, HP thresholds): the chapter
  tickets set them as *Claude's starting values* and the bots tune them.

## Implementation steps

1. Read the outline's Chapters 4, 6 and 8, `death-and-difficulty.md`,
   `world-structure.md`, `battle-scenes-and-recruitment.md` and how
   `core` does Seize and Survive today (`crates/core/src/objective*` or
   wherever `Objective` lives).
2. Prepare the options with references and mockups (`ascii-art` skill;
   frames from `cargo shuyi frame-png` if a current map helps).
3. Ask (`ask-nick`), in rounds of at most three.
4. Record the answers; adjust 0319 if needed.

## Acceptance criteria

- [x] All six questions answered by Nick; his words are in the design docs.
- [x] 0319's scope matches the answers.
- [x] `cargo xtask ticket-lint` passes.

## Tests required

- None (decision).

## Completion notes

Asked in five rounds with the question pop-up (2026-10-21/22). Recorded:

- **Escape (Ch 6):** everyone out one by one (`Escape` leaves the map,
  safe); won when every unit still standing is out; no turn limit,
  pressure from reinforcement waves. `turn-structure.md`.
- **Dace, Ch 4:** retreats at the killing blow; counts as defeated (the
  battle is won, kill EXP); normal forecast.
- **Dace, Ch 8** (Nick's own idea): at 0 HP he is down, grayed out, no
  actions, untargetable by anyone, blocking his tile; the hold goes on;
  normal forecast.
- **Hollis:** fights the battle, dies in the closing scene in every mode;
  if he falls in this battle in Classic (or died earlier), the "lead tells
  Dace" version. `death-and-difficulty.md`.
- **Side-quest rewards:** a personal weapon, a signature upgrade, rare shop
  stock or gold; more kinds later; no stat boosters. Can be missed for good,
  with a deadline reminder once discovered. `world-structure.md`.
- **Chapter table:** confirmed with two changes Nick asked for:
  **no talk-to-recruit** (Joss joins in a scene, Gil when beaten; Nick
  listed other ways to recruit, now in `battle-scenes-and-recruitment.md`),
  and **no plain villages**: Chapter 2's houses are a "save the village"
  race (a minimum to save, extra rewards past it; saved by `Evacuate` or by
  keeping raiders off). Nick didn't comment on the rest of the table; it
  is recorded as kept.

**Deviations:** two questions grew out of Nick's answers and were asked
too (recruiting without talk, and how villages work). That changed ticket
**0313** from "Villages and the Visit action" into the save-the-village
rule, and the village parts of 0037, 0409, 1018 and
`weapons-and-items.md`. 0319's scope now holds the exact rules; the
chapter tickets 0744, 0848–0854 and 1071 say how their recruits join.

**Follow-up ticket:** **1074**, a discovered side quest shows when it
closes (Nick's "deadline" reminder); in 1064's `blocked_by` and the
roadmap.

**Claude's starting rules** (Nick may veto):
- The lead may escape before the others; `Escape` gives no EXP; green
  units needn't escape; escaped units count as standing.
- A raider burns a house by ending its turn on its door or one of the
  four tiles beside it; the battle is lost the moment the minimum can't
  be met.
- A downed Dace counts as defeated on the results screen.
- While Hollis is in the army he is always deployed in Chapter 8.
- Raiders walk to the nearest unsaved house and prefer burning to
  attacking (in 0313, tuned by the bots).
