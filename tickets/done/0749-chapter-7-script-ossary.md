---
id: "0749"
title: "Chapter 7 script: Ossary, beat sheet and scenes"
type: content
milestone: M6 Story & dialogue
model: creative
effort: high
status: done
blocked_by: ["0736", "0743", "0748"]
nick_input: none
completed: 2026-10-30
---

# 0749 — Chapter 7 script: Ossary

## Context

Act 1, Chapter 7 (world map), from `docs/story/outline.md`:

- **Story battle:** "The Undercroft": wardens in catacombs and narrow stairs; the boss is Crane's champion; Crane himself leaves.
- **Turn:** The Candles' dormitory is empty; Crane is waiting, calls death "a door", tells Maud her report was "admirably thorough"; the Door was opened once before, in the East, and the Jade Reach sealed it; Crane escapes.
- **Recruits:** none (the runaway Candle of *The Other Cart* is 1072's to decide).
- **Named boss (0743):** Crane's champion is Warden Constance Hobb (`undercroft_warden`), who kept the Candles' barracks; sheet `docs/story/characters/undercroft_warden.md`.
- **Arcs advanced:** Maud (faces Crane), Rue (the children), Dace (seen briefly, shaken by Crane).
- **New in this chapter:** nothing new.

The story pipeline (ADR-0011, the `story-writing` skill) goes beat sheet
→ script, and `docs/story/ledger.md` records what each chapter changed,
so the chapters are written in order: this ticket waits for Chapter
6's script (0748). It also waits for 0736 (the craft research,
which governs every Act 1 script: a rewrite starts on a blank page, the
length budget) and 0743 (the new cast's character sheets).

## Nick input

None. The agent writes all scripts; only story decisions go to Nick (a
change to a beat in `beats.md` or the outline, a new named character the
cast doesn't have), asked with `ask-nick` before writing the scene that
needs it. Nick reads the chapter in the Act 1 playtest (1064).

## Scope

**In:**
- **The beat sheet** `docs/story/chapters/ch07.md`, in
  `ch01.md`'s shape: cast on screen (ids from `docs/story/names.md`),
  each scene with its place, who speaks, what each speaker wants in it,
  what changes; the battle's lines (the boss, the new units' Classic
  death and Casual retreat lines, talk conversations).
- **The script** `assets/dialogue/ch07.dlg`: the chapter's intro
  and pre-battle scenes, the battle's scenes, the victory scenes, by the
  rules in `assets/dialogue/README.md` (role ids, name tokens, `{lead}`,
  no line that depends on the lead's gender).
- **Fixed skirmishes:** *Pilgrims' Road* (guards the path) and *The Ossuary Gate* (optional). Write their short scenes here (1071 builds the battles and wires the scenes).
- **Town:** Ossary (no script needed unless the beat sheet puts a scene there).
- **Side quests:** *The Other Cart* (Rue's) and *Unwritten* (Maud's; 1072 writes both).
- New names in `docs/story/names.md` first, then `assets/data/names.ron`.
- The critique pass (`story-writing` skill), then `docs/story/ledger.md`.
- **Either order with the battle ticket (0853):** the beat sheet
  names each scene's place and mood; the `place` directives, their maps
  and the `@music` lines are 0853's, which comes after this ticket
  (the validator checks that a place exists). If 0071 has picked this
  chapter's scene music, add the `@music` lines here and tell 0853.

**Out (do not do):**
- The battle file, map, units and triggers (0853).
- Support conversations and camp events (1073); side quests' scenes
  (1072).
- Changing a story beat without Nick.

## Implementation steps

1. Read the `story-writing` skill (with 0736's craft section), ADR-0011,
   `beats.md`, `bible.md`, the outline, the ledger, every character
   sheet on screen, Chapter 6's beat sheet and script.
2. Write the beat sheet; check it against the canon order.
3. Write the script on a blank page (0736's rule), within the length
   budget.
4. Critique pass; fix; update the ledger.
5. `cargo xtask check-script` and the content gates.

## Acceptance criteria

- [x] `docs/story/chapters/ch07.md` and `assets/dialogue/ch07.dlg` exist and
      cover every beat above.
- [x] The critique pass checklist is ticked in the PR body.
- [x] The ledger records what changed for each character.
- [x] `cargo xtask check-script` passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The dialogue validator (content gates); no Rust code.

## Completion notes

Done 2026-10-30.

- **Written:** the beat sheet `docs/story/chapters/ch07.md` (in
  `ch06.md`'s shape) and the script `assets/dialogue/ch07.dlg`, on a
  blank page from scene cards: the pilgrim road; the two fixed
  skirmishes' scenes (*Pilgrims' Road*, *The Ossuary Gate*); the
  streets, the empty dormitory, Crane, the stair; the battle's lines
  (turn 1, the bone walls, the warden to anyone, to Rue, to Oriel, at
  half strength, dying; the lead's fall; nine death quotes and nine
  retreat lines); the chapter-house yard, the pilgrims' barn and the
  cut to Crane and Dace. Two reply choices (to Crane, to Maud). All
  talk before the first move is 28 boxes on the longest path, the cap.
- **The critique pass** ran twice: a fresh reader's cold read on the
  player view (names shown, then names hidden) and the editor's list.
  Round 1 found one house rhythm in every mouth (repeat the phrase,
  "..." plus "--" in the same box, "somebody" in every dying wish),
  templated retreat lines (everyone going "up"), Crane's door slogans
  and his "I've read your paper" device three times, Rue's attack
  happening off screen, the yard's "out, now!" contradicted by ten
  boxes of standing still, a second roll-call scene in the streets, and
  a dozen British words. All fixed in the second draft; round 2's
  findings are in the PR.
- **Where the Candles went** (the warden's sheet left it to this
  ticket): carted to the king's army two weeks before the party
  arrives, the nineteen whose fire line lit at the Ashfields (Chapter
  6's ledger). Recorded in the sheet. Wren is not among them: Crane
  says "catalogued, not lost" and no more; Dace is told nothing
  (Chapter 8's line stays Hollis's).
- **Names:** no new names. `undercroft_warden` added to
  `characters.ron`'s `speakers` (she speaks before 0853 makes a unit of
  her, as Oriel and the knight did for 0852). Her sheet's "love" and
  "now then" were British (`setting-and-tone.md`, *Language*): the
  script and the sheet now say "dear" and "there, now".
- **Music:** 0071's picks are `holy_city` (the road and the streets),
  `scene_dread` (the dormitory), `talk_antagonist` (Crane's scenes) and
  `battle_church_3` (the battle file's). Only `talk_antagonist` is
  imported, so the script carries it and marks the other two with a
  comment where the `@music` line goes; **0853 imports `holy_city` and
  `scene_dread` and adds those two lines** (its step 3 already says so).
  The scenes 0071 left open use the existing mood cues as *the agent's
  picks* (`dungeon_tense`, `talk_calm`, `scene_sad`).
- **For 0853:** the scene ids and triggers are listed at the top of the
  script; the warden speaks twice before the fight (`ch07_crane`,
  `ch07_prebattle`), so she needs a dialogue face (her sheet allows it:
  "a face only if she speaks outside the fight"); the `@if` blocks
  around companion lines in battle scenes go as each trigger is wired.
  No `@place` directives yet (0853's and 1071's maps).
- **Deviations:** none from the outline. The chapter's "streets" scene
  is three boxes (narration, Rue, Hollis): the cold read found a longer
  one a second roll call, and the budget before the first move is at
  its cap.
- **For the Act 1 playtest (1064):** nothing to judge by playing beyond
  the words. No starting rules were decided (no Rust, no numbers).
- **Follow-ups:** none filed. Hollis hears Oriel say "Marr" in the
  barracks and nothing is made of it here; 1073's Hollis–Oriel support
  can spend it (Chapter 6's note stands).
