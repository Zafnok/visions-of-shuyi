---
id: "0748"
title: "Chapter 6 script: The Ashfields, beat sheet and scenes"
type: content
milestone: M6 Story & dialogue
model: fable-5.1 / gpt-6-astra
effort: high
status: done
blocked_by: ["0736", "0743", "0747"]
nick_input: none
completed: 2026-10-29
---

# 0748 — Chapter 6 script: The Ashfields

## Context

Act 1, Chapter 6 (world map), from `docs/story/outline.md`:

- **Story battle:** "The Ashfields": crossing a burnt plain between two armies, with echoes (fire elementals) and battle notes ("Echoes of Vael: weak to frost, drink fire").
- **Turn:** Oriel Mast deserts to the party and confesses the prince burned on the king's own order and was never found in the ash; Rue tells Hollis that Wren walked out of the fire alive; the Brennish commander believes the proof, but the king's army attacks anyway.
- **Recruits:** Oriel Mast (`battlemage`, Sorcerer, tier 2), who deserts to the party.
- **Arcs advanced:** Rue and Aske, Hollis (Wren lives), the king's secret (for the player).
- **New in this chapter:** elemental enemies (echoes) and battle notes (`magic.md`).

The story pipeline (ADR-0011, the `story-writing` skill) goes beat sheet
→ script, and `docs/story/ledger.md` records what each chapter changed,
so the chapters are written in order: this ticket waits for Chapter
5's script (0747). It also waits for 0736 (the craft research,
which governs every Act 1 script: a rewrite starts on a blank page, the
length budget) and 0743 (the new cast's character sheets).

## Nick input

None. The agent writes all scripts; only story decisions go to Nick (a
change to a beat in `beats.md` or the outline, a new named character the
cast doesn't have), asked with `ask-nick` before writing the scene that
needs it. Nick reads the chapter in the Act 1 playtest (1064).

## Scope

**In:**
- **The beat sheet** `docs/story/chapters/ch06.md`, in
  `ch01.md`'s shape: cast on screen (ids from `docs/story/names.md`),
  each scene with its place, who speaks, what each speaker wants in it,
  what changes; the battle's lines (the boss, the new units' Classic
  death and Casual retreat lines, talk conversations).
- **The script** `assets/dialogue/ch06.dlg`: the chapter's intro
  and pre-battle scenes, the battle's scenes, the victory scenes, by the
  rules in `assets/dialogue/README.md` (role ids, name tokens, `{lead}`,
  no line that depends on the lead's gender).
- **Fixed skirmishes:** *The Burnt Mill* (guards the path; the first echoes) and *Brennish Outriders* (optional). Write their short scenes here (1071 builds the battles and wires the scenes).
- **Town:** Cairnford (no script needed unless the beat sheet puts a scene there).
- **Side quests:** none new.
- New names in `docs/story/names.md` first, then `assets/data/names.ron`.
- The critique pass (`story-writing` skill), then `docs/story/ledger.md`.
- **Either order with the battle ticket (0852):** the beat sheet
  names each scene's place and mood; the `place` directives, their maps
  and the `@music` lines are 0852's, which comes after this ticket
  (the validator checks that a place exists). If 0071 has picked this
  chapter's scene music, add the `@music` lines here and tell 0852.

**Out (do not do):**
- The battle file, map, units and triggers (0852).
- Support conversations and camp events (1073); side quests' scenes
  (1072).
- Changing a story beat without Nick.

## Implementation steps

1. Read the `story-writing` skill (with 0736's craft section), ADR-0011,
   `beats.md`, `bible.md`, the outline, the ledger, every character
   sheet on screen, Chapter 5's beat sheet and script.
2. Write the beat sheet; check it against the canon order.
3. Write the script on a blank page (0736's rule), within the length
   budget.
4. Critique pass; fix; update the ledger.
5. `cargo xtask check-script` and the content gates.

## Acceptance criteria

- [x] `docs/story/chapters/ch06.md` and `assets/dialogue/ch06.dlg` exist and
      cover every beat above.
- [x] The critique pass checklist is ticked in the PR body.
- [x] The ledger records what changed for each character.
- [x] `cargo xtask check-script` passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The dialogue validator (content gates); no Rust code.

## Completion notes

Done 2026-10-29 (Fable 5.1).

**What was done**

- `docs/story/chapters/ch06.md`: the beat sheet, in `ch05.md`'s shape:
  cast on screen, where each part of the outline's three-part turn goes,
  ten scenes with place, who, what changes, the battle's lines, and
  continuity notes.
- `assets/dialogue/ch06.dlg` (783 lines): the north road; the Burnt
  Mill and Brennish Outriders skirmish scenes (before and after each);
  three scenes before the first move (the plain's edge, the night Oriel
  walks in, first light), 28 boxes on the longest path; the battle's
  triggers (turn 1, the black ground, the Hounds' wave, the knight to
  anyone and to Oriel, half HP, fell, the lead's fall, a death quote and
  a retreat line for all nine companions); the Brennish lines, the dead
  ground and the cut to Dace. Two reply choices (to Oriel, to Hollis).
- Names: `brennish_commander` ("Commander") and `crown_knight`
  ("Knight"), nameless speakers like the magistrate, in `names.md` and
  `names.ron`; `battlemage`, `brennish_commander` and `crown_knight`
  added to `characters.ron`'s speakers until 0852 makes units of them.
- `docs/story/ledger.md`: *After Chapter 6* (what happened, who knows
  what, relationships, status, every absence). `docs/story/README.md`'s
  status table brought up to date through Chapter 6.

**The critique pass**: three cold reads by a fresh reader (the player
view, then the cover test with names and `@if` tags stripped), each
followed by fixes, then the editor's list. What the reads caught and
what changed: a roll-call opening (Joss's paper worry and Rue's bread
cut from the intro); "it drinks fire" said three times and Oriel
retelling the king's order at the lines (cut to the body alone); a
"Nobody..." tic in a dozen boxes and doubled closers; death and retreat
lines on two templates (reshaped: Gil's death is now his coat folded
like church clothes, Oriel's the dog he was never allowed, which the
earnest reply plants); Hedda's "Easy" seven times (three now); British
words (round, lot, hayrick, standing about, the pair of you); narration
that editorialized ("She is not going home"); and four lines a reader
had to guess at. The cover test's last round told Aske, Oriel and Gil
apart on voice alone, Rue and Hedda mostly, and still blurred Hollis,
Hedda and the two officers in their short orders, and Tamsin, Joss and
Maud when panicking: noted for 1073 and the next chapter rather than
pushed further here (each round's fixes were nudges, not swings).

**Deviations and decisions** (story staging, not game rules; Nick
decides beats, and no beat changed):

- The outline names no boss for the Ashfields and leaves the three-part
  turn unordered. The boss is a nameless knight of the king's vanguard
  (the king's army, not the Hounds, so Dace stays offstage until the
  cut); the Candles are never on the map or on screen, only a row of
  fires lighting along the king's front, seen from the Brennish lines.
  The turn is spread: the king's order at the night fire (why Oriel
  deserted), the body never found at the lines (asked by the
  commander), Wren alive on the dead ground after the crossing. If Oriel
  fell in the crossing (Classic) the body clue goes unsaid on that path.
- Oriel joins in the night scene (way 2, like Hedda) and crosses as a
  player unit (0852's to deploy); his lines after the battle are in
  `@if battlemage` blocks.
- Maud's question to Oriel (were the children fed) was cut for the
  night's cap of 13 boxes; it is a support's (1073). Joss's "it was for
  nothing" is said and unanswered here, as his sheet puts the answer in
  1073.
- New small facts the sheets didn't have, recorded in the ledger: Oriel
  once wrote to Dace about Wren ("the strongest"), and Dace kept the
  letter; the knight's brother died in the war's ninth year.

**For 0852** (the battle): wire the scene ids in the script's header;
remove the `@if` blocks the triggers make one-way; deploy Oriel as a
player unit for the crossing; the battle notes' text (the outline's
"Echoes of Vael: weak to frost, drink fire"); the `@place` lines and
their maps; `@music` is this ticket's picks from the mood cues
(`graveyard_desert` for the plain's edge and the dead ground), since
0071 picked only the battle's `battle_war` for this chapter. **For
1071**: the four skirmish scene ids. **For 0750** (Chapter 8): Hollis's
last words assume Rue told him on the dead ground; without Rue (or
without Hollis) the ledger says what he was and wasn't told.

**Follow-ups created:** none. **Nick input:** none (no beat changed; the
chapter is read in the Act 1 playtest, 1064).
