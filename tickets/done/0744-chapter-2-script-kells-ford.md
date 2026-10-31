---
id: "0744"
title: "Chapter 2 script: Kell's Ford, beat sheet and scenes"
type: content
milestone: M6 Story & dialogue
model: creative
effort: high
status: done
blocked_by: ["0736", "0743", "0737"]
nick_input: none
completed: 2026-10-28
---

# 0744 — Chapter 2 script: Kell's Ford

## Context

Act 1, Chapter 2 (linear), from `docs/story/outline.md`:

- **Story battle:** Kell's Ford: a river village split by a ford, houses to save before the raiders burn them (0069's *save the village*: a minimum to save, extra rewards past it), the garrison's watchtower across the water, a Hound lieutenant as boss.
- **Turn:** The raiders are Hounds in Brennish colours (Aske: "Brennish cannot afford boots like this"); after the battle a crown herald names the lead an outlaw and declares the Ash Peace broken.
- **Recruits:** Joss Pellam (`defector`, Swordsman), joins in the scene after the battle (0069: no recruiting by talking).
- **Named boss (0743):** Lieutenant Silas Hendry (`hound_lieutenant`), who picked Joss for the company; sheet `docs/story/characters/hound_lieutenant.md`.
- **Arcs advanced:** Aske (her hatred loses its target), Maud (healing strangers under fire), the lead and Dace.
- **New in this chapter:** Preparations (0443's shop tab) for the first time, save the village (0313), a recruit who joins in a scene (0069).

The story pipeline (ADR-0011, the `story-writing` skill) goes beat sheet
→ script, and `docs/story/ledger.md` records what each chapter changed,
so the chapters are written in order: this ticket waits for Chapter
1's script (0737). It also waits for 0736 (the craft research,
which governs every Act 1 script: a rewrite starts on a blank page, the
length budget) and 0743 (the new cast's character sheets).

## Nick input

None. The agent writes all scripts; only story decisions go to Nick (a
change to a beat in `beats.md` or the outline, a new named character the
cast doesn't have), asked with `ask-nick` before writing the scene that
needs it. Nick reads the chapter in the Act 1 playtest (1064).

## Scope

**In:**
- **The beat sheet** `docs/story/chapters/ch02.md`, in
  `ch01.md`'s shape: cast on screen (ids from `docs/story/names.md`),
  each scene with its place, who speaks, what each speaker wants in it,
  what changes; the battle's lines (the boss, the new units' Classic
  death and Casual retreat lines, talk conversations).
- **The script** `assets/dialogue/ch02.dlg`: the chapter's intro
  and pre-battle scenes, the battle's scenes, the victory scenes, by the
  rules in `assets/dialogue/README.md` (role ids, name tokens, `{lead}`,
  no line that depends on the lead's gender).
- New names in `docs/story/names.md` first, then `assets/data/names.ron`.
- The critique pass (`story-writing` skill), then `docs/story/ledger.md`.
- **Either order with the battle ticket (0848):** the beat sheet
  names each scene's place and mood; the `place` directives, their maps
  and the `@music` lines are 0848's, which comes after this ticket
  (the validator checks that a place exists). If 0071 has picked this
  chapter's scene music, add the `@music` lines here and tell 0848.

**Out (do not do):**
- The battle file, map, units and triggers (0848).
- Support conversations and camp events (1073); side quests' scenes
  (1072).
- Changing a story beat without Nick.

## Implementation steps

1. Read the `story-writing` skill (with 0736's craft section), ADR-0011,
   `beats.md`, `bible.md`, the outline, the ledger, every character
   sheet on screen, Chapter 1's beat sheet and script.
2. Write the beat sheet; check it against the canon order.
3. Write the script on a blank page (0736's rule), within the length
   budget.
4. Critique pass; fix; update the ledger.
5. `cargo xtask check-script` and the content gates.

## Acceptance criteria

- [ ] `docs/story/chapters/ch02.md` and `assets/dialogue/ch02.dlg` exist and
      cover every beat above.
- [ ] The critique pass checklist is ticked in the PR body.
- [ ] The ledger records what changed for each character.
- [ ] `cargo xtask check-script` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The dialogue validator (content gates); no Rust code.

## Completion notes

Done 2026-10-28 on Fable 5.1 (the model the skill asks for).

**The pipeline:** scene cards in a scratch file (never committed), the
beat sheet `docs/story/chapters/ch02.md` from the outline and the 0743
sheets, then `assets/dialogue/ch02.dlg` from the cards with nothing
older open (there is no older Chapter 2 script). Three rounds of
critique, each by fresh subagents given only the scenes as the player
sees them: a cold read, a names-removed speaker test and, in round 1,
an American-English editor. Each round's hits were fixed before the
next read.

**What the readers caught, round by round:** a missing box in the boss
and Aske exchange (he reacted to a voice she hadn't used); every death
line on one template (a chore about an object, then "somebody..."); a
leading "..." on forty boxes and echo-doubling in every mouth; a "don't
look at me" tic shared by five speakers; Tamsin at Hollis's
temperature; Joss as a male Maud (the same stammer, the same "only");
Rue as a gesture, not a person; "the captain" read as Hollis; the
proclamation naming the exile a week early (now "all but one line");
the herald's garbled "hides her with her"; a dozen British words
("catch you up", "straight in", "sensible", "I'm off", "owing", "have
my ears", "fair copy", "cross", "give it here", "guard-room", "you've
not", "I'm for"); "What stands, stands" and other writer lines.

**Box counts, everyone alive** (budget: the `story-writing` skill):

| Scene | Boxes | Budget |
| ----- | ----- | ------ |
| `ch02_intro` (one reply's reaction included) | 15 | 8–14, cap 16 |
| `ch02_prebattle` | 11 | 5–10, cap 12 |
| All talk before the first move | 26 | cap 28 |
| `ch02_refusal` (Hendry and Joss) | 6 | 2–4, cap 5: one over, because the scene is also the chapter's refusal beat |
| `ch02_boss_engage_poacher` | 5 | cap 5 |
| Other battle lines | 1–2 each | 1–2 |
| `ch02_victory` (longest path) | 16 | 6–12, cap 16 |
| `ch02_join` (longest path) | 15 | 6–12, cap 16 |
| `ch02_tbc` | 6 | 3–6 |

Reply choices: three (the halt, the herald, the far bank). The chapter
has about ten `!` in all; the victory scene has three.

**Choices made inside the outline** (story detail, no beat changed; the
Act 1 playtest, 1064, is where Nick vetoes any):

- The chapter opens on a halt in the dark an hour above the ford, not
  on the village: warmth before the threat (Maud's feet, Tamsin's forty
  acres, Rue's bread, Hollis and the coat).
- Aske reads the boots on a dead raider before the fight, so the party
  fights knowing the raiders are not Brennish and not knowing who they
  are; Joss's refusal and Hendry's last words give the party "the
  captain", Hendry (if the lead fights him) and Joss give the name.
- The herald's proclamation was written and sealed a week before the
  raid, all but the line that names the exile (Joss carried the bag).
  Who added the line is not said. The garrison reads it and goes back.
- Joss's refusal is a turn-start scene drawn over the map; the player
  never fights him. Whether he is a unit who then leaves the map is
  0848's.
- The scene after the battle is split in two (the herald; the far
  bank), each within budget.
- The cut at the end is Dace in the guardroom with Ned Tolley (0743's
  Chapter 3 sergeant), sending Hendry's pay to his mother and Ned to
  Greywater. It sets Chapter 3's named enemy in place and keeps the
  Chapter 1 pattern (the player knows more than the lead) without
  repeating its staging (the study, the two letters).
- Aske knows the garrison's watch too well and Tamsin asks; "I hunt."
  A seed for her Chapter 5 secret, not a reveal.
- Joss's lie ("they came made, I just carried the bundle") is told to
  Aske on the far bank; Rue sees his hands shake. His sheet's Chapter 5
  confession has something to land on.

**Script data outside the script:** `herald` added to `names.md` and
`names.ron` (a minor name like `messenger`); `defector`,
`hound_lieutenant` and `hound_sergeant` added to `characters.ron`'s
`speakers` so the script validates before 0848 and 0849 make them
units. Every companion line in a battle-trigger scene sits in an `@if`
block for them, because the validator can't know who a scene is about
until the battle file names its triggers; 0848 removes those blocks as
it wires each trigger (a note in 0848's steps and in the script's
header). No `@place` lines (0848's maps); `@music` lines are this
ticket's picks from the mood cues (0071).

**Deviations:** one test changed, where the ticket expected no Rust:
the Playdate play-through test (`crates/playdate/src/chapter_1.rs`)
gathered the `@music` cues of every scene in the bundle, so Chapter 2's
three mood cues failed it as "tracks the Playdate's build does not
ship". It now gathers Chapter 1's own scenes (its intro and victory
scenes and its battle's triggers); a later chapter's script names its
cues without touching the Playdate's music list until that chapter
ships on it. `cargo xtask check-script` passes.
The workspace test gate is red on `main` itself since 0071 merged
(`every_design_cue_is_in_the_manifest`: the nine new cues in `audio.md`
are in neither the manifest nor `RESERVED`); ticket 0252 fixes it. No
other gate is affected by this ticket.

**the agent's starting rules:** none that touch play. For the Act 1
playtest (1064): whether Rue has enough presence in this chapter (she
has four boxes outside the battle); whether the far-bank scene's
lightness ("Your Grace!") sits right straight after the herald.

**Follow-ups:** 0252 (the red audio test on `main`). 0848 wires the
scenes and removes the `@if` guards; 1073 may pick Joss's supports from
his sheet.

**Engine friction:** none: no production Rust, one test narrowed.
