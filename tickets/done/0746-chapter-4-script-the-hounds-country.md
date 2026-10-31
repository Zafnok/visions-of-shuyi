---
id: "0746"
title: "Chapter 4 script: The Hound's Country, beat sheet and scenes"
type: content
milestone: M6 Story & dialogue
model: fable-5.1 / gpt-6-astra
effort: high
status: done
blocked_by: ["0736", "0743", "0745"]
nick_input: none
completed: 2026-10-28
---

# 0746 — Chapter 4 script: The Hound's Country

## Context

Act 1, Chapter 4 (world map), from `docs/story/outline.md`:

- **Story battle:** "Veyne Hall": storming the lead's childhood home; Hollis and the lead know the old servants' passages; Dace is the boss and escapes at low HP.
- **Turn:** Dace doesn't deny it ("I did what fed us. You had a house; I had a sister."), then to Hollis: "You knew. You always knew." Dace escapes.
- **Recruits:** Gil Parrow (`prizefighter`, Brawler), recruited in the *Millhaven Road* skirmish (1071 builds the battle; this script writes his scenes).
- **Arcs advanced:** the lead and Dace, Hollis (his secret is out), Tamsin.
- **New in this chapter:** the world map, a town (Millhaven), fixed skirmishes (*Aldwater Toll Bridge*, guarding the path; *Millhaven Road*, optional), the first side quest (Tamsin's, 1072).

The story pipeline (ADR-0011, the `story-writing` skill) goes beat sheet
→ script, and `docs/story/ledger.md` records what each chapter changed,
so the chapters are written in order: this ticket waits for Chapter
3's script (0745). It also waits for 0736 (the craft research,
which governs every Act 1 script: a rewrite starts on a blank page, the
length budget) and 0743 (the new cast's character sheets).

## Nick input

None. The agent writes all scripts; only story decisions go to Nick (a
change to a beat in `beats.md` or the outline, a new named character the
cast doesn't have), asked with `ask-nick` before writing the scene that
needs it. Nick reads the chapter in the Act 1 playtest (1064).

## Scope

**In:**
- **The beat sheet** `docs/story/chapters/ch04.md`, in
  `ch01.md`'s shape: cast on screen (ids from `docs/story/names.md`),
  each scene with its place, who speaks, what each speaker wants in it,
  what changes; the battle's lines (the boss, the new units' Classic
  death and Casual retreat lines, talk conversations).
- **The script** `assets/dialogue/ch04.dlg`: the chapter's intro
  and pre-battle scenes, the battle's scenes, the victory scenes, by the
  rules in `assets/dialogue/README.md` (role ids, name tokens, `{lead}`,
  no line that depends on the lead's gender).
- **Fixed skirmishes:** *Aldwater Toll Bridge* (guards the path) and *Millhaven Road* (optional; Gil's recruitment). Write their short scenes here (1071 builds the battles and wires the scenes).
- **Town:** Millhaven (no script needed unless the beat sheet puts a scene there).
- **Side quests:** *The Company Ledger* (Tamsin's; 1072 writes it).
- New names in `docs/story/names.md` first, then `assets/data/names.ron`.
- The critique pass (`story-writing` skill), then `docs/story/ledger.md`.
- **Either order with the battle ticket (0850):** the beat sheet
  names each scene's place and mood; the `place` directives, their maps
  and the `@music` lines are 0850's, which comes after this ticket
  (the validator checks that a place exists). If 0071 has picked this
  chapter's scene music, add the `@music` lines here and tell 0850.

**Out (do not do):**
- The battle file, map, units and triggers (0850).
- Support conversations and camp events (1073); side quests' scenes
  (1072).
- Changing a story beat without Nick.

## Implementation steps

1. Read the `story-writing` skill (with 0736's craft section), ADR-0011,
   `beats.md`, `bible.md`, the outline, the ledger, every character
   sheet on screen, Chapter 3's beat sheet and script.
2. Write the beat sheet; check it against the canon order.
3. Write the script on a blank page (0736's rule), within the length
   budget.
4. Critique pass; fix; update the ledger.
5. `cargo xtask check-script` and the content gates.

## Acceptance criteria

- [x] `docs/story/chapters/ch04.md` and `assets/dialogue/ch04.dlg` exist and
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
beat sheet `docs/story/chapters/ch04.md` from the outline and the 0743
sheets, then `assets/dialogue/ch04.dlg` from the cards with the earlier
scripts closed. Three rounds of critique by fresh subagents given only
the scenes as the player sees them (a cold read and a names-removed
speaker test each round, plus an American-English line editor in round
1 and a verification read after round 3). Each round's hits were fixed
before the next read.

**What the readers caught, round by round:** the death lines on one
template again (an object, where it is, an errand for "somebody") and
the retreat lines on another (a one-word wound, a self-correction, where
I'll be), both now a different shape each (an order, a verb, a count, a
request, a question, a question asking for a lie, a compliment to the
killer); Dace's "..." in 13 of 24 boxes and Gil's in 7 of 13 (now 8 and
4); terse lines nobody could place ("Not so hard", "Lie down, you're
shivering", "Eat while you talk") until each got its speaker's own tag
or habit; Rue drifting into Maud's lane by feeding people without her
rudeness; Gil's numbered rules as a gimmick (rule eleven cut; two rules
stay, and the pie carries him); the same observation twice in a row
(Tamsin wrong about the posts, then Hollis right: Tamsin's cut); rule
two explained in the fell line and again in the join; "the Captain" for
Hollis in Dace's mouth colliding with "the captain" for Dace (now "my
father"); "the two books" (now the register and the pay-book); a dozen
British words ("I've not got", "mind the", "go and sit", "needn't",
"properly hit", "smells of", "docket", "pie stall", "you'll not", "Cook"
as a title, "post" for mail); the writer's lines ("The dirt did
nothing", "Every road is his, and every road is open", "The gate shuts.
It takes four men", "See what it gets you this time", "Down is not
beaten", "it's what sleeves are for").

**Box counts, everyone alive, Gil beaten** (budget: the `story-writing`
skill; the beat sheet's for the skirmishes):

| Scene | Boxes | Budget |
| ----- | ----- | ------ |
| `ch04_intro` | 13 | 8–14, cap 16 |
| `ch04_aldwater_intro` / `_victory` | 6 / 5 | 3–6 / 2–5 |
| `ch04_millhaven_road_intro` | 6 | 3–6 |
| `ch04_millhaven_road_victory` (Gil joins; longest reply) | 14 | 6–12, cap 16 |
| `ch04_eve` (longest reply) | 14 | 8–12, cap 16 |
| `ch04_gate` (longest reply) | 13 | 5–10, cap 12 + 1 for Dace |
| The rise and the gate together | 27 | cap 28 |
| Dace and the lead / Hollis / Rue / Joss | 3 / 4 / 4 / 3 | 2–4, cap 5 |
| Other battle lines | 1 each | 1–2 |
| `ch04_victory` (longest path) | 15 | 6–12, cap 16 |
| `ch04_tbc` | 5 | 3–6 |

Reply choices: three in the chapter (the rise, the gate, the stable),
plus one in Gil's join scene, which plays only if he was beaten on the
optional road. The chapter has three `!` in all.

**Choices made inside the outline** (story detail, no beat changed; the
Act 1 playtest, 1064, is where Nick vetoes any):

- **The confrontation is at the gate, before the battle**, not in a boss
  exchange only some players fight: Dace opens his own gate, notices the
  hole in the lead's coat and that nobody has slept, offers breakfast,
  and when the lead puts the pay-book to him doesn't deny it ("I did
  what fed us. You had a house. I had a sister.", the outline's words),
  then says "You knew. You always knew" past the lead to his father.
  Hollis answers "...Aye." and nothing else until the stable.
- **Hollis says only that he knew**, "not all of it, but enough", and
  that he will tell the lead all of it, "not today, and not in his
  stable". His sheet gives the whole confession (the purse, the list,
  the pass, why he burned them) to Chapter 5's main story, the night
  before Saltmere; this chapter spends none of it.
- Dace refuses to kill the lead: he asks for the register and the
  pay-book and offers to write to Ossary that the party burned with the
  prior. Refused, "Then take it, Veyne. I'll be in the hall."
- **Dace retreats by the laundry door**, the servants' passage the lead
  and Hollis know ("I know the laundry door too"); the Hounds who stayed
  are in the cellar. He is wounded in the arm.
- **The stable and the bay mare.** Ned Tolley's Chapter 3 death line
  (Hollis's bay, kept unridden four years) pays off: Hollis is found
  brushing her, and the one true thing he says sideways is that the boy
  kept her fed and shod himself.
- **Dace lies to Ossary a second time**, for his father: the letter
  asked after "the old man" by name; "Write: The old man was not
  there." (In Chapter 1 he lied that the job was done.) The party
  doesn't know Ossary asked.
- **Tamsin's side quest's hook**: the toll-chest's tally carries the
  crown paymaster's mark, which she knows from three years of refused
  pay-notes; his office is in Millhaven. 1072 names him and writes the
  quest; the chapter says nothing more about him.
- **Gil's join**: beaten, he says "Well struck" and rule two; after the
  fight he bows the whole way down and calls the lead "Champion"; the
  party reacts as people (the mare he bowed to first, his ear, three
  pies, a bow Aske wants stopped, the six ribbons and the bare seventh
  place). His plan for after (a fair of his own, no betting over the
  gate) is planted here, in Joss's block: if Joss is dead it waits for a
  support or camp scene (1073). If he wasn't beaten, one narration box:
  he walks toward the town bell and is never seen again.
- Rue's beat with Dace (the belt, the crying, "you don't even know me")
  is the battle's exchange, not the gate's; he doesn't remember her.
  Joss's is Hendry's exact words from Kell's Ford in his captain's
  mouth ("Stand by me. You were in the smoke. That's what we'll say"),
  and Dace not knowing which of them said them first.
- Nobody says the price on the lead's head, the count on the walls or
  the toll: Tamsin reads the price and won't say it.
- Joss's mother's farm is "two valleys over, by a ford with a willow";
  he can't go, and says so. Nothing is made of it beyond his death
  line.
- A new short form, `place.aldwater.river` ("the Aldwater"), for the
  river the toll bridge crosses, so a rename of the bridge reaches it.

**Script data outside the script:** `prizefighter` added to
`characters.ron`'s `speakers` so the script validates before 1071 makes
him a unit (1071 moves him to `characters`); `place.aldwater.river` in
`names.md` and `names.ron`. Every companion line in a battle-trigger
scene sits in an `@if` block for them, as Chapters 2 and 3's do, because
the validator can't know who a scene is about until the battle file
names its triggers; 0850 removes those blocks as it wires each trigger
(a note in the script's header). Gil is guarded with `@if prizefighter`
everywhere outside his own battle lines: the validator can't yet know
he can be lost (no 1071 battle file), and the blocks are right once it
lands. No `@place` lines (0850's and 1071's maps); `@music` lines are
this ticket's picks from the mood cues (`talk_calm`, `scene_sad`,
`talk_antagonist`, `scene_tragic`); the story battle's `battle_rival` is
Nick's and goes in the battle file; the skirmishes use the `skirmish`
pool (1008).

**Wiring for 0850 and 1071** (the script's header lists every id):
`ch04_intro` plays when the chapter opens (the world map appears);
`ch04_eve` and `ch04_gate` before the Veyne Hall battle;
`ch04_victory` and `ch04_tbc` after it. The two skirmishes each have a
scene before and after; Millhaven Road also has Gil's engage line and
his fell line (`UnitFell(unit: "prizefighter", recruit: true)`). Dace's
`ch04_boss_retreat` is his `UnitFell` with the map's "retreats instead
of falling" mark (0319, `turn-structure.md`). The passage trigger is an
area anyone can enter.

**Deviations:** none in the pipeline. The chapter has four reply
choices, not three: the fourth is in Gil's optional join scene, so a
player who skips Millhaven Road sees three.

**the agent's starting rules:** none that touch play. For the Act 1
playtest (1064): whether the gate confrontation (13 boxes before the
battle, after a 14-box night scene) drags; whether Hollis's "Aye" and
the stable are enough of a crack before Chapter 5's confession, or too
little; whether Gil is funny or a bit (two cold readers called him "a
bit" and wouldn't miss him yet; his join is his only quiet scene until
1073); whether Dace saying "You had a house. I had a sister" in the
outline's words lands or reads as a slogan.

**Follow-ups:** none filed. 0850 wires the story battle's scenes,
places and `battle_rival`, and removes the `@if` guards; 1071 builds the
two skirmishes and Gil's unit and wires their scenes; 1072 takes the
paymaster from the toll-chest's tally; 1073 may pick Chapter 4's
supports (Gil has first scenes with Tamsin, Maud, Aske and Joss; Hollis
and the lead have their crack).

**Engine friction:** none: no Rust code.
