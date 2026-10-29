---
id: "0745"
title: "Chapter 3 script: Greywater, beat sheet and scenes"
type: content
milestone: M6 Story & dialogue
model: fable-5.1
effort: high
status: done
blocked_by: ["0736", "0743", "0744"]
nick_input: none
completed: 2026-10-28
---

# 0745 — Chapter 3 script: Greywater

## Context

Act 1, Chapter 3 (linear), from `docs/story/outline.md`:

- **Story battle:** Greywater Abbey: a causeway to an island abbey, wardens and Hounds on the walls, the scriptorium as the Seize point, chests in the reliquary.
- **Turn:** The register: Rue's name twice, Wren's name ("Ashfields: lost"), Vosse's pay-book ("D. Marr, informer: eleven children"); Rue tells the lead about the mill, Maud confesses to Rue, Hollis says nothing.
- **Recruits:** none.
- **Named boss (0743):** Prior Edwin Sallow (`greywater_prior`), who dies or surrenders when beaten (Nick, 2026-10-28: "can say dies or surrenders"; this ticket picks). Sergeant Ned Tolley (`hound_sergeant`), an old Veyne guardsman, commands the Hounds guarding him: a named enemy, not the boss. Sheets in `docs/story/characters/`.
- **Arcs advanced:** Maud (breaks with the Vigil), Rue (tells), Hollis (his silence starts to show), the lead and Dace.
- **New in this chapter:** a Seize objective and chests; after this chapter the Act 1 world map opens.

The story pipeline (ADR-0011, the `story-writing` skill) goes beat sheet
→ script, and `docs/story/ledger.md` records what each chapter changed,
so the chapters are written in order: this ticket waits for Chapter
2's script (0744). It also waits for 0736 (the craft research,
which governs every Act 1 script: a rewrite starts on a blank page, the
length budget) and 0743 (the new cast's character sheets).

## Nick input

None. Claude writes all scripts; only story decisions go to Nick (a
change to a beat in `beats.md` or the outline, a new named character the
cast doesn't have), asked with `ask-nick` before writing the scene that
needs it. Nick reads the chapter in the Act 1 playtest (1064).

## Scope

**In:**
- **The beat sheet** `docs/story/chapters/ch03.md`, in
  `ch01.md`'s shape: cast on screen (ids from `docs/story/names.md`),
  each scene with its place, who speaks, what each speaker wants in it,
  what changes; the battle's lines (the boss, the new units' Classic
  death and Casual retreat lines, talk conversations).
- **The script** `assets/dialogue/ch03.dlg`: the chapter's intro
  and pre-battle scenes, the battle's scenes, the victory scenes, by the
  rules in `assets/dialogue/README.md` (role ids, name tokens, `{lead}`,
  no line that depends on the lead's gender).
- New names in `docs/story/names.md` first, then `assets/data/names.ron`.
- The critique pass (`story-writing` skill), then `docs/story/ledger.md`.
- **Either order with the battle ticket (0849):** the beat sheet
  names each scene's place and mood; the `place` directives, their maps
  and the `@music` lines are 0849's, which comes after this ticket
  (the validator checks that a place exists). If 0071 has picked this
  chapter's scene music, add the `@music` lines here and tell 0849.

**Out (do not do):**
- The battle file, map, units and triggers (0849).
- Support conversations and camp events (1073); side quests' scenes
  (1072).
- Changing a story beat without Nick.

## Implementation steps

1. Read the `story-writing` skill (with 0736's craft section), ADR-0011,
   `beats.md`, `bible.md`, the outline, the ledger, every character
   sheet on screen, Chapter 2's beat sheet and script.
2. Write the beat sheet; check it against the canon order.
3. Write the script on a blank page (0736's rule), within the length
   budget.
4. Critique pass; fix; update the ledger.
5. `cargo xtask check-script` and the content gates.

## Acceptance criteria

- [x] `docs/story/chapters/ch03.md` and `assets/dialogue/ch03.dlg` exist and
      cover every beat above.
- [x] The critique pass checklist is ticked in the PR body.
- [x] The ledger records what changed for each character.
- [x] `cargo xtask check-script` passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The dialogue validator (content gates); no Rust code.

## Completion notes

Done 2026-10-28 on Fable 5.1 (the model the skill asks for).

**The pipeline:** scene cards in a scratch file (never committed), the
beat sheet `docs/story/chapters/ch03.md` from the outline and the 0743
sheets, then `assets/dialogue/ch03.dlg` from the cards with the earlier
scripts closed. Three rounds of critique by fresh subagents given only
the scenes as the player sees them (a cold read and a names-removed
speaker test each round, plus an American-English editor in round 1
and a short verification read after round 3). Each round's hits were
fixed before the next read.

**What the readers caught, round by round:** a death-line template
(every dying person handing "somebody" an errand, then "..." and a
short tag) and a retreat-line template (name the wound, "go on"); "..."
in 59% of boxes and "--" in 41% (now 16% and 18%); Maud's letter said
in the boss exchange and again in her confession (the exchange now
turns on the hand he taught her, so the kitchen is the first time the
player hears it); the causeway as a roll call of one observation each
(Rue's line cut; Tamsin's now addresses Maud); Rue's food lines in
Hollis's order-giving register; Tamsin and Hollis sharing "I watched
you not"; Joss sounding like Tamsin over the sleeve and like Maud over
"sorry"; the prior's and Ned's lines too close to their sheets' sample
lines (rewritten); a dozen British words ("give it here", "copied it
fair", "that is" as a tag, "any more", "boat-shed", "Right." as an
opener, "after" alone, "gone fat", "take the pot off her"); "Vosse" on
the pay-book, a name the player has never heard (Hollis now places it:
"the man from the mill"); "Candle" read by Aske when Rue's reading
hadn't shown the word (Rue now skips one word aloud); the cart in Rue's
woods against the killing at the mill (now both); a dying prior putting
Maud "on the wall" she never stood on; "company black" colliding with
the Red Company (Joss now says "Hound black", the first time the word is
said aloud in the party); the writer's images ("like a dog at a gate",
"making the water nervous", "a moment late", the guttering lantern).

**Box counts, everyone alive** (budget: the `story-writing` skill):

| Scene | Boxes | Budget |
| ----- | ----- | ------ |
| `ch03_intro` (longest reply's reaction included) | 15 | 8–14, cap 16 |
| `ch03_prebattle` | 10 | 5–10, cap 12 |
| All talk before the first move | 25 | cap 28 |
| `ch03_sergeant_engage_retainer` (Ned and Hollis) | 5 | 2–4, cap 5 |
| `ch03_boss_engage_keeper` (the prior and Maud) | 4 | 2–4, cap 5 |
| Other battle lines | 1 each | 1–2 |
| `ch03_register` (longest path) | 16 | 6–12, cap 16 |
| `ch03_telling` (longest path) | 11 | 6–12 |
| `ch03_confession` (longest path) | 13 | 6–12, cap 16 |
| `ch03_tbc` | 6 | 3–6 |

Reply choices: three (the boathouse, the causeway at night, the
kitchen). The chapter has six `!` in all, two of them Tamsin's.

**Choices made inside the outline** (story detail, no beat changed; the
Act 1 playtest, 1064, is where Nick vetoes any):

- **The prior dies** (Nick: "can say dies or surrenders"). The outline
  gives Chapter 3 only the Seize objective and the chests, and a boss who
  walks off at the killing blow is Chapter 4's introduction (Dace), so
  Edwin Sallow dies at his desk and Maud takes the register from the
  desk, not his hands. If Nick would rather the old man lived, 0849 can
  make him leave the map at 0 HP and the victory scene needs one box.
- The chapter opens the night before, in a boathouse on the shore, not
  at the abbey: porridge, letters in the ash, a sleeve, before the
  threat. Maud says once what the register is; Joss says once whose
  black is on the wall.
- Maud's letter about Rue is not mentioned in the battle. Her exchange
  with the prior is about the hand he taught her and the fire; the
  kitchen is where the player hears the confession, from her, to Rue.
- Rue reads her own two lines aloud and skips one word; Aske, who has
  had two nights of letters with Maud, reads the word herself. Aske
  leaves the room and has no line to Rue after it.
- Hollis asks Maud to find Wren's line ("you wrote her in, you'll know
  the page"): the first warm thing between them. The line says *lost*,
  and he hears the word; nothing more is made of it until Chapter 6.
- The pay-book is read as written, "D. Marr", by Tamsin, slowly. Hollis
  places the name on the cover (the player had never heard Vosse's
  name). Joss says it's another Marr; nobody says "Dace" over it; Hollis
  goes to the window and Rue watches him, which she tells the lead that
  night.
- Rue tells the lead at the causeway's end, alone, and sees the lead
  didn't know; she says the book is wrong about one thing and won't say
  more (Wren alive stays Chapter 6's). Her "heir" survives once.
- Ned Tolley calls Dace "your boy" to Hollis, says Dace kept the old
  guard on and asks after Hollis every winter, and dies telling him the
  bay mare (Chapter 2's "not the bay") has gotten fat.
- The cut at the end is Dace in the yard with a rider's report, not the
  study or the guardroom: he knows "the rest of his papers" went with
  the exile, closes every road to the hall (Chapter 4's conflict) and
  orders the exile brought to him untouched.
- A new short form, `group.candles.one` ("Candle"), for the register's
  word beside a name, so a rename of the Candles reaches it.

**Script data outside the script:** `greywater_prior` added to
`characters.ron`'s `speakers` so the script validates before 0849 makes
him a unit; `group.candles.one` in `names.md` and `names.ron`. Every
companion line in a battle-trigger scene sits in an `@if` block for
them, as Chapter 2's do, because the validator can't know who a scene
is about until the battle file names its triggers; 0849 removes those
blocks as it wires each trigger (a note in the script's header). Joss
is guarded too, though the validator can't yet know he can be lost (no
Chapter 2 battle file): the blocks are right once 0848 lands. No
`@place` lines (0849's maps); `@music` lines are this ticket's picks
from the mood cues (`talk_calm`, `dungeon_tense`, `scene_tragic`,
`scene_sad`, `talk_antagonist`); the battle's `battle_church_2` is
Nick's and goes in the battle file.

**Deviations:** none in the pipeline. One dialogue line was reworded
because an `xtask` unit test checks a `lang-status` report for the
words "left out" and matched Rue's "He just left out the rest";
ticket 1079 narrows that test. No test was changed.

**Claude's starting rules:** none that touch play. For the Act 1
playtest (1064): whether the prior should die or live; whether Tamsin
has enough in this chapter (her arc is Chapter 4's, and the cold reads
found her the companion they'd miss least here); whether Joss's sleeve
bit in the boathouse reads, or is one bit too many after his Chapter 2
cuff; whether "Hounds" being said aloud for the first time by Joss
lands.

**Follow-ups:** 1079 (the brittle test). 0849 wires the scenes, picks
the Seize tile and the chests, and removes the `@if` guards; 1073 may
pick Chapter 3's supports (Hollis and Maud now have their first warm
line; Aske and Rue their break).

**Engine friction:** none: no Rust code.
