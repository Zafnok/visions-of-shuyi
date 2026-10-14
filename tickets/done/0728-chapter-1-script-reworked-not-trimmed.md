---
id: "0728"
title: Rework the Chapter 1 script as scenes, not line by line
type: content
milestone: M6 Story & dialogue
model: fable-5.1
effort: high
status: done
blocked_by: []
nick_input: sign-off
completed: 2026-10-14
---

# 0728 — Rework the Chapter 1 script

## Context

Nick, Chapter 1 playtest round 2 (0804, 2026-10-14), about 0724's rewrite:

> "the rewrite was too literal. It took the lines line by line and sucked
> out the quips rather than reworking the script. Aske says 'fourteen men
> in the fort...' the line is so literal and boring.... I don't like the
> original quips but this is just matter of fact and weird."

0724 cut the jokes from `assets/dialogue/ch01.dlg` one line at a time
(its completion notes list every changed line). What's left reads as
reports of facts. Nick wants neither the old quips nor flat lines: scenes
rewritten so people talk like people under threat, with character in how
they say things rather than jokes on top.

Read first: the `story-writing` skill and ADR-0011; `docs/story/` (bible,
`chapters/ch01.md` beat sheet, `beats.md`, the characters' sheets, which
0724 gave sample lines); `docs/design/setting-and-tone.md` (*Where the jokes
go*, *Language*: American English); 0724's and 0707's completion notes. Claude writes all the lines; Nick
decides only story questions.

## Nick input

**Sign-off:** read or play the Chapter 1 scenes. Do the people sound like
people? Is Aske's report of the fort something she'd say?

If the rework needs a change to a story beat (`beats.md`), ask Nick with the
`ask-nick` skill before writing it.

## Scope

**In:**
- Rewrite `ch01.dlg` scene by scene: each scene's purpose (from the beat
  sheet) first, then the lines. Lines may be merged, reordered, cut or
  added; who speaks when may change.
- Keep: every scene id and trigger, every `@choice` and the story facts it
  sets, the `@if` parts (0715/0716), the music and caption directives
  unless a scene's purpose changes, the facts the map depends on (Aske's
  count must still match the fourteen of the fort, `chapter-1.md`).
- The tone rule stays: the opening and the battle are serious; no quips
  under threat. Serious is not flat: voice comes from word choice,
  rhythm, what each person notices and what they don't say.
- Update the beat sheet, the ledger and the sheets' sample lines if they
  change.
- The title cinematic (`assets/cinematics/title.ron`) shows some of these
  lines by box number (`Talk` shots): check them after the rewrite and fix
  `first` / `count` if the boxes moved (0833 reworks the cinematic; say in
  the Completion notes which boxes it shows now).
- List the lines with `cargo xtask lines <scene>` and keep the content checks green (line
  ids change; voice clips don't exist yet for Chapter 1).

**Out (do not do):**
- New story beats or changes to what happens (ask Nick first).
- Other chapters.
- The dialogue screen or engine.

## Implementation steps

1. Read everything in *Context*. For each scene write down its purpose and
   what each speaker wants in it (in the beat sheet, `chapters/ch01.md`).
2. Rewrite the scenes. Read each aloud in your head as the character; if a
   line only states a fact, give it the speaker's angle on that fact.
   Example of the direction (not a line to use): Aske's count could say
   what kind of hunter she is and what she thinks of the Red Company,
   not just the number.
3. Run the gates (the content checks read the script) and the Chapter 1
   tests (the winning replay plays through the scenes).
4. In the Completion notes, quote three before/after pairs, including
   Aske's fort line.

## Acceptance criteria

- [x] Every `ch01_*` scene rewritten; scene ids, triggers, choices and
      facts unchanged (the content tests pass).
- [x] Beat sheet, ledger and sheets updated where they changed.
- [x] The cinematic's `Talk` shots still show the opening lines they meant
      to (or the notes say what 0833 must change).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Existing content and flow tests (script parsing, triggers, the Chapter 1
  replay). Snapshots that show script text are reviewed and accepted.

## Completion notes

Done 2026-10-14 on Fable 5.1.

**What was done**

- Each scene of `assets/dialogue/ch01.dlg` was reworked from its purpose:
  the beat sheet (`docs/story/chapters/ch01.md`) now carries a **Wants**
  line per scene (what each speaker is after in it), and the lines were
  written from those. The scene shapes are the beat sheet's and did not
  move; what changed is what each person brings into the scene and says
  about it. Where a line already carried its speaker's angle (Harl's
  prices, the death quotes, Rue's cart, Hollis over the graves) it was
  kept, so the rework is heaviest where Nick found it flat: the yard, the
  green, the retreat lines, Tamsin over Harl's body. The full list of
  changed lines is below.
- Every scene id, trigger, `@choice`, reply text and `@if` block is as
  0716 left it. Aske still counts fourteen; she now also says three have
  bows, which is the map's three Archers (`chapter-1.md` notes both).
- Box budget: opening + pre-battle 39 to 41 boxes by reply (was 38 or
  39); the victory scene is unchanged in length.
- Docs: beat sheet (Wants per scene, the quotes that changed), ledger
  (Tamsin's "He taught me to ride" is now said aloud; Aske's count and
  the paper's wording), Aske's sheet (voice note and fort sample line; her
  lines use no contractions), `chapter-1.md`, the story README.
- The title cinematic's `Talk` shots: `ch01_intro` boxes 0 to 2 are still
  the three opening captions (unchanged words, so their line ids are the
  same); `ch01_prebattle` box 0 is still Tamsin's news, now "Captain. Your
  Former Grace. Red Harl is in the toll-fort with the whole company. I
  rode up to the gate to ask him what he wanted here. He asked me if I
  wanted my old job back." `title.ron` needs no change; its shot 4
  snapshot was reviewed and accepted. 0833 shortens that shot and should
  keep `first: 0, count: 1` for it.
- `cargo xtask lines` after the rewrite: 24 ids in `ch01_intro`, 26 in
  `ch01_prebattle`, 59 in `ch01_victory`. Every rewritten line has a new
  id; no translation or voice clip exists yet for Chapter 1, so nothing
  went stale.

**Three before/after pairs** (names as the player sees them today).

Aske's fort report:

> *Before:* "Fourteen men in the fort. I counted twice. Some have bows, and
> they shoot badly." / "Not for you. They killed three deer last night and
> left two to rot, in my woods. I will help you, noble. Once. For the deer."
>
> *After:* "Fourteen in the fort. I counted at dusk and again at first
> light. Three have bows. Last night they shot at the deer from the wall.
> Many arrows. Most of them hit nothing." / Tamsin: "Fourteen is what I
> counted from the gate, too. Why bring it to us, Brennish?" / "Not to you.
> They killed three deer and carried off one. The other two are lying in my
> woods for the crows. I will help you, noble. Once. For the deer."

Hollis, the disarm and the letter:

> *Before:* "...Good. That's the first time you've taken it out of my
> hand." / "Leave it. Sit down, kid. A peddler left this last night. No
> seal, no name. I know the handwriting. So will you."
>
> *After:* "...Good. That's good. Four years I've been waiting for you to do
> that." / "Leave it where it is. Sit down, kid. There's something in my
> coat that I've spent all morning not giving you." / "A peddler left it
> last night. No seal, no name. It didn't need one. I'd know that
> handwriting anywhere, and so will you."

Tamsin over Harl's body:

> *Before:* "Payday, Harl. I'll mark you in the ledger. Paid in full." /
> "...Just a job. That's all it was."
>
> *After:* "Payday, Harl. You go in the ledger with the rest of them. Paid
> in full." / "He taught me to ride. ...It was a job. That's all it was."

**Deviations**

- None from the steps. No story beat changed, so Nick was not asked.
- The rework did not merge or reorder boxes between speakers: once the
  purposes were written down, the beat sheet's order was already the order
  the scene needs (Tamsin's news before Maud arrives, Hollis silent until
  the swords). Lines were added (Hollis on the letter in his coat), split
  (the paper) and cut (none wholesale).

**Not done / for 0833**

- `title.ron` shows the same boxes as before; 0833 only re-times them.

**Follow-up tickets:** none.

**Gameplay rules decided where the design was silent:** none. Text only.

### Changed lines (old and new)

Unchanged, on purpose: the three opening captions, every reply text, every
`lead:` line, Harl's lines, every death quote, Rue's and Maud's lines on
the green except as listed, all of `ch01_victory` except Tamsin's two lines
over Harl, and `ch01_tbc`.

#### The opening: the letter (`ch01_intro`)

| Who | Old | New |
| --- | --- | --- |
| Hollis | Guard up, Veyne. Higher. Keep it there. | Guard up, Veyne. Higher. It drops when you're tired. Don't let it. |
| Narration | Three strokes. The fourth takes the old man's sword out of his hand. | Three strokes. On the fourth, the old man's sword is lying in the dirt between them. |
| Hollis | ...Good. That's the first time you've taken it out of my hand. | ...Good. That's good. Four years I've been waiting for you to do that. |
| Hollis | Leave it. Sit down, kid. A peddler left this last night. No seal, no name. I know the handwriting. So will you. | Leave it where it is. Sit down, kid. There's something in my coat that I've spent all morning not giving you. / A peddler left it last night. No seal, no name. It didn't need one. I'd know that handwriting anywhere, and so will you. |
| Hollis | Four years, and not a word. Not to you. Not to his own father. Then that. | Four years. Not a line to you. Not one to his own father. And when he finally writes, it's that. |
| Hollis, to the blunt reply | Us gone, by the look of it. What I'd ask is who's coming, and how he knows. | Us gone, by the look of it. What I'd ask him is who's coming. And how he knows. |
| Hollis | That's the sergeant. She doesn't ride like that for good news. | That's the sergeant. She doesn't ride like that for good news. Get your coat. |

#### Before the battle: the green (`ch01_prebattle`)

| Who | Old | New |
| --- | --- | --- |
| Tamsin | Captain! Your Former Grace! Red Harl took the toll-fort in the night, with his whole company. I rode up to ask why. He offered me my old job back. | Captain. Your Former Grace. Red Harl is in the toll-fort with the whole company. I rode up to the gate to ask him what he wanted here. He asked me if I wanted my old job back. |
| Maud | Sergeant! We heard the horse from the chapel. Is someone hurt? I've brought bandages, and Rue to help me carry them. | Sergeant? We heard the horse from the chapel. Nobody rides onto the green like that unless someone is hurt. I've brought bandages, and Rue to carry the rest. |
| Tamsin | Nobody's hurt yet, Keeper. He showed me a paper with a lord's seal on it, and kept his thumb over the seal. It names three people, and a price for each. / "The exile, the old man, and the girl with the burned hands." | Nobody's hurt yet, Keeper. He showed me a paper with a lord's seal on it. His thumb stayed over the seal the whole time I was reading. / Three people on it, and a price by each that would feed his company till spring. No names. "The exile, the old man, and the girl with the burned hands." |
| Aske | Fourteen men in the fort. I counted twice. Some have bows, and they shoot badly. | Fourteen in the fort. I counted at dusk and again at first light. Three have bows. Last night they shot at the deer from the wall. Many arrows. Most of them hit nothing. |
| Tamsin | That's what I saw too, Brennish. Why bring it to us? | Fourteen is what I counted from the gate, too. Why bring it to us, Brennish? |
| Aske | Not for you. They killed three deer last night and left two to rot, in my woods. I will help you, noble. Once. For the deer. | Not to you. They killed three deer and carried off one. The other two are lying in my woods for the crows. I will help you, noble. Once. For the deer. |
| Maud | Rue, the chapel cellar bars from the inside. Please go down. It is you they have come for. | Rue. The chapel cellar bars from the inside. Please go down. It is you they have come for. |
| Hollis | They've come for three of us. There are forty people between here and that fort who never did a thing to anyone. Your call, Veyne. | That paper has three of us on it. Between here and that fort there are forty people who aren't on any paper. Your call, Veyne. |
| Aske, to the earnest reply | The miller's wife buys my rabbits. She pays too much, on purpose. Keep them away from her. | The miller's wife buys my rabbits. She pays me too much and pretends she cannot count. Keep them away from her. |
| Tamsin, to the wry reply (second box) | Don't be. They were good soldiers once, and Harl still is. | Don't be. I rode with them. They were good soldiers once, and Harl still is. |

#### In the battle

| Who, when | Old | New |
| --- | --- | --- |
| Hollis, first turn | Stay close, kid. The first real one's always ugly. | Stay close, kid. The first real one's always ugly. Guard up, and let them come to us. |
| Hollis, retreating | My leg's gone. I'm no use in the line like this. I'm falling back. / Guard up, Veyne. Finish it. I'll be at the house. | Hnh. That's my leg. I'm no good to you in the line on one leg. / Guard up, Veyne. Finish it, then come and find me at the house. |
| Tamsin, retreating | I'm hit. I can't stay in the saddle like this. / I'm out, Your Former Grace. Don't let any of them walk away. | That's a hole in me. I can't keep the saddle with it. / I'm out, Your Former Grace. Don't let one of them walk away. |
| Aske, retreating | I am hit. It is not deep, but I cannot draw the bow. | I am hit. It is not deep. But I cannot draw the bow with this arm. |
| Maud, retreating | I'm hurt. I... yes. I can't heal anyone like this. / I must withdraw. I'm sorry. Ama keep you all. | I'm hurt. I... oh. I can hardly stand, and I can't mend anyone like this. / I must go back. I'm sorry. Ama keep you all. |
| Rue, retreating | Vael's teeth! I can't hold the fire steady. I'm going. | Vael's teeth! I can't hold the fire. It's shaking in my hands. |

#### After the battle (`ch01_victory`)

| Who | Old | New |
| --- | --- | --- |
| Tamsin, over Harl | Payday, Harl. I'll mark you in the ledger. Paid in full. | Payday, Harl. You go in the ledger with the rest of them. Paid in full. |
| Tamsin | ...Just a job. That's all it was. | He taught me to ride. ...It was a job. That's all it was. |

### Critique pass

Read as an editor after writing, against the `story-writing` checklist:

- [x] A line another character could say unchanged: none left. Aske's
      fort report is a hunter's (counts twice, judges by arrows and waste);
      Tamsin's is a sergeant's (the thumb, the price, what it buys); Maud
      hears one phrase; Rue hears eyes; Hollis says "that's good" before
      anything else.
- [x] Exposition: the longest run is still the three boxes about the seal
      and the title in the victory scene, unchanged.
- [x] Contradictions: none with beats, bible or ledger. The map has three
      Archers, so "three have bows" holds; the ledger records what is now
      said aloud (Harl taught Tamsin to ride).
- [x] Every scene changes something: unchanged from 0707.
- [x] Lengths and ids: `cargo xtask check-script` reports 0 problems.
- [x] Main plot and arcs: unchanged.
