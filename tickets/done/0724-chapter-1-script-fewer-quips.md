---
id: "0724"
title: "Chapter 1 script: play the opening and the battle straight (fewer quips)"
type: content
milestone: M6 Story & dialogue
model: fable-5.1 / gpt-6-astra
effort: high
status: done
blocked_by: ["0707"]
nick_input: sign-off
completed: 2026-10-12
---

# 0724 — Chapter 1 script: play the opening and the battle straight

## Context

Nick read lines from the Chapter 1 script (`assets/dialogue/ch01.dlg`,
ticket 0707) in the title cinematic mockups (ticket 0036) and rejected the
tone:

> dawg these dialogue lines are cheesy and cringe asf we'll need to rewrite
> them.
>
> there's just too many quips in general as a starting tone... quips should
> be during downtime, not on ch1 opening not on a serious battle... I'm not
> against quips as support ranks rise as the game reaches more high notes
> in terms of closeness but this just ain't it chief.

The lines he saw were Hollis's "That eyebrow's had one scar off me already,
and I'm not proud of it." / "A bit proud.", Aske's "Twelve men. No.
Fourteen. Bad archers.", and Rue's "I'm not a cook, Keeper. I'm a girl who
can light the oven without a match. Different thing." They are typical of
the script: nearly every character answers danger with a joke.

The rule is now in `docs/design/setting-and-tone.md` (*Where the jokes
go*): the game's opening and serious battles are played straight; banter
belongs to downtime and grows as characters get closer (support ranks).
The tone is still "dark with warmth and humour"; this changes **where**
the humour sits, not whether the game has any.

The title cinematic plays the opening lines of `ch01_intro` and
`ch01_prebattle` straight from this file (`title-screen.md`, *Intro
cinematic*), so what this ticket writes is also what the title shows.

**From Nick's playtest (ticket 0804, 2026-10-12).** He met the same lines
in the game and they still don't work; his words say why, so use them:

> "Hollis and Veyne first dialogue is too weird... idk what this means
> 'that eyebrow's had one scar off me already' wtf? sounds weird as hell"
>
> "'A bit proud' also sounds weird unless you enhance it like 'mumbles' or
> \*mumbles\* or something"
>
> "'pedlar' is that supposed to be 'peddler'? wtf is this olde english or
> something?"
>
> "Aske saying 'twelve men. no. fourteen. bad archers' also doesn't sit
> right with me. bad archers...? so why would they not be counted?"

What to take from it:

- **A line must be understood on first reading.** The scar line hides what
  happened (Hollis once cut Veyne's eyebrow) behind a turn of phrase; the
  count line leaves out its own logic (Aske missed two because they were
  hidden, not because they shoot badly). If such a line survives the
  rewrite at all, it says plainly what it means.
- **An aside needs to read as one.** If a line is muttered, the script
  says so in the way `.dlg` allows (check the format for a stage
  direction or an expression; if it has none, don't write lines that need
  one, and don't invent `*mumbles*` markup without a ticket for it).
- **Spelling: "peddler", not "pedlar".** Also in
  `docs/story/chapters/ch01.md` and `docs/story/ledger.md`. Asked whether
  that means American spelling everywhere, Nick (2026-10-12): "yea no
  bri'ish english only american... idk about modern but not bri'ish". So:
  **American spelling and American words throughout the game's text**
  (color, armor, gray, peddler; no "whilst", "bloody", "mate"). He did
  **not** rule out old-fashioned words as such; don't strip them on his
  account. Go through the whole of `ch01.dlg` and the language files for
  British spellings, and add the rule to the `story-writing` skill and
  `docs/design/setting-and-tone.md` so later scripts follow it.

## Nick input

**Sign-off:** the PR lists, scene by scene, the lines that changed (old
and new, as text). Nick reads them and comments. The agent writes every line
(Nick writes no dialogue); only tone and story decisions go to him.

It waits until after his Chapter 1 playtest (0804). Asked whether it
should land before (2026-10-03): "nah it can wait til after". So this
ticket is blocked by 0804, and his playtest notes on the script's tone, if
any, are folded into the rewrite.

**Changed 2026-10-12:** round 1 of the playtest is played and its notes
are above. Nick parked 0804 until the feedback is built, this ticket
included, so the order turned round: **0804 now waits for this ticket**,
and this one no longer waits for 0804.

## Scope

**In:**
- Rewrite `assets/dialogue/ch01.dlg` with the `story-writing` skill so
  that:
  - `ch01_intro` and `ch01_prebattle` are played straight: people who have
    just learned that armed men are coming for them talk like it. Warmth
    (Hollis's care for the lead, Maud's kindness) stays; one-liners,
    running gags and comic bits go, or move to the wry reply choice.
  - Battle scenes (`ch01_first_turn`, the boss scenes, death and retreat
    lines) carry no jokes. A dry or bitter line that is in character
    (Harl's "Still owed.") is fine; a punchline is not.
  - `ch01_victory` and `ch01_tbc` stay serious; at most one light moment
    after the tension breaks.
- Characters keep their voices (`docs/story/characters/*.md`): a voice is
  how someone talks, not how often they joke. Where a sheet's voice notes
  or sample lines push a character toward constant quipping, update the
  sheet too.
- **The lead's `wry` reply choices stay** (`setting-and-tone.md` rule 2:
  replies come in distinct tones and the player picks). Humour the player
  chose is not the script quipping at them. The other characters' answers
  to a wry reply may be light, briefly.
- Every story fact and every scene id stays: what the letter says, who is
  on Harl's paper, the seal, the count of fourteen, who joins and why. The
  ledger (`docs/story/ledger.md`) changes only where a rewritten line
  stated a fact differently.
- A short note in the `story-writing` skill (its tone section) pointing at
  *Where the jokes go*, so later chapters start from this rule.

**Out (do not do):**
- New scenes, new plot, new characters, changed outcomes.
- The scenes for a dead companion (0716). If 0716 is done, rewrite its
  lines to the same rule in this ticket; if it isn't, add a line to 0716
  pointing at *Where the jokes go* so it is written straight from the
  start.
- Support conversations and camp banter (1002, 1003): that is where the
  banter goes later; nothing to write now.
- The title cinematic (0820): it reads this file and needs no change.

## Implementation steps

1. Read `setting-and-tone.md` (all of Q2 and *Where the jokes go*), the
   `story-writing` skill, `docs/story/chapters/ch01.md`, the character
   sheets and the ledger.
2. Go through `ch01.dlg` scene by scene. For each joke, decide: cut, turn
   into a plain line that carries the same information, or keep (a wry
   reply's answer, or the single light moment in the victory scene). Keep
   a list of old and new lines as you go: it is the PR's sign-off text.
3. Keep every line inside the dialogue screen's limits (three rows per
   box; the content validator checks). Keep `@choice` blocks, name tokens
   and scene ids as they are.
4. Update character sheets, the beat sheet and the ledger where they quote
   or depend on a changed line.
5. Run content validation and the dialogue tests; update snapshots that
   show changed text.

## Acceptance criteria

- [x] `ch01_intro`, `ch01_prebattle` and every battle scene contain no
      joke by a character other than in answer to the lead's `wry` reply;
      the PR's list shows each removed or changed line.
- [x] Every scene id, `@choice` and story fact of 0707 is still there
      (the ledger's Chapter 1 entries still hold).
- [x] Content validation passes for `ch01.dlg`.
- [x] The `story-writing` skill points at *Where the jokes go*.
- [x] If 0716 was done, its scenes follow the rule too; if not, 0716 says
      to.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot / integration: existing dialogue and flow tests that show
  Chapter 1 text, updated; content validation of the script.

## Completion notes

Done 2026-10-12 (run on Opus 5.5, Nick's choice over the ticket's
creative).

**What was done**

- `assets/dialogue/ch01.dlg` rewritten to *Where the jokes go*. The opening,
  the scene on the green and every battle scene are played straight; the
  full list of changed lines is below. Every scene id, `@choice`, reply
  text and story fact is as 0707 left it. The script's header comment now
  states the tone rule.
- Nick's four lines: the scar line and "A bit proud." are cut (the format
  has no stage direction, so no line that needs to be muttered was
  written); "pedlar" is "peddler"; Aske now says plainly what she counted
  ("Fourteen men in the fort. I counted twice. Some have bows, and they
  shoot badly."); Rue's oven line is a plain refusal to hide.
- 0716's scenes (the `@if` parts of `ch01_victory`) were read against the
  rule: they were already serious; one word changed ("got" to "gotten").
- Character sheets: Hollis, Tamsin, Aske, Maud and Rue now say that their
  jokes belong to quiet scenes and how each talks under threat, with new
  sample lines; the lead's sheet no longer calls the scar a running joke
  of this script. Beat sheet and ledger follow the new lines.
- American English: the rule is in `docs/design/setting-and-tone.md`
  (*Language*), the `story-writing` skill and the writer's guide. In the
  script: peddler, payday, "my men" (was "my lads"), "my woods" (was "my
  wood"), "gotten", "shovel" (was "spade"), and "that lot", "That's me
  done" and "I'll have the kettle on" went with their jokes. In the menus:
  "Armour" is now "Armor" (`assets/lang/en/ui.ron`, 6 texts; the test pack
  and 7 snapshots follow).
- The `story-writing` skill's tone rule points at *Where the jokes go*, and
  has a new rule: a line is understood on first reading.

**Deviations**

- The count "twelve, no, fourteen" is now just fourteen. The fact kept is
  the fourteen; the correction was the part Nick couldn't follow.
- The ledger's line for what Rue said on the green changed with her line:
  she says she is "sworn to fire" (was "can light the oven without a
  match"). Same fact: all six now know she is gifted and swears fire.
- Tamsin's line over Harl's body is no longer a joke that falls flat; she
  marks him "paid in full" without a smile. Her beat (she calls it just a
  job) is unchanged.
- Tamsin's ledger is no longer mentioned before the battle. It first
  appears in her death line or over Harl's body, both of which explain
  themselves.

**For Nick to rule on (not changed)**

- **"Aye"** (Hollis, three times). Kept as an old soldier's word, since
  old-fashioned words are allowed. If it reads British, it becomes "Yes" /
  "Yeah".
- **"Greywater Abbey"** is spelled the British way ("grey"). It is a name,
  so it is Nick's to rename ("Graywater").
- **"Your Former Grace"**, Tamsin's name for the lead, stays in serious
  scenes: it is what she calls the lead, not a joke she makes each time.
- **"autumn"** stays (Americans use it too, and "fall" also means a unit
  falling).

**Not done**

- The code and data files still spell the field `armour`. That is not game
  text; no ticket written, because nothing a player sees depends on it.
- 0820 (title cinematic) is still open, so there were no box counts of
  its to re-check; the scenes open with fewer boxes than before.

**Follow-up tickets:** none.

### Changed lines (old and new)

Names are shown as the player sees them today.

#### The opening: the letter (`ch01_intro`)

| Who | Old | New |
| --- | --- | --- |
| Hollis | Guard up, Veyne. Higher. That eyebrow's had one scar off me already, and I'm not proud of it. | Guard up, Veyne. Higher. Keep it there. |
| Hollis | A bit proud. | *(cut)* |
| Hollis | ...Huh. About time. Pick that up for me. My knees won't. | ...Good. That's the first time you've taken it out of my hand. |
| Hollis | Then sit down, kid. A pedlar left this last night. No seal, no name. I know the hand. So will you. | Leave it. Sit down, kid. A peddler left this last night. No seal, no name. I know the handwriting. So will you. |
| Hollis, to the wry reply ("Four years of silence, and now he's urgent.") | Ha. He always did write like a man owed money. / He's not the one who's owed. | Ha. He never did write unless he needed something. / I'd take four more years of quiet over this. |

#### Before the battle: the green (`ch01_prebattle`)

| Who | Old | New |
| --- | --- | --- |
| Tamsin | Captain! Your Former Grace! Red Harl and his whole sorry company took the toll-fort in the night. I rode up to ask why, and he offered me my old job back. | Captain! Your Former Grace! Red Harl took the toll-fort in the night, with his whole company. I rode up to ask why. He offered me my old job back. |
| Maud | Sergeant! We heard the horse from the chapel. Is someone hurt? I've brought bandages, and Rue, to, ah, to carry the bandages. | Sergeant! We heard the horse from the chapel. Is someone hurt? I've brought bandages, and Rue to help me carry them. |
| Tamsin | Nobody's hurt yet, Keeper. He has a paper, with a lord's seal he kept his thumb on, and three people he's paid to find. "The exile, the old man, and the girl with the burned hands." | Nobody's hurt yet, Keeper. He showed me a paper with a lord's seal on it, and kept his thumb over the seal. It names three people, and a price for each. / "The exile, the old man, and the girl with the burned hands." |
| Tamsin | I'm not on it. I checked twice. That's going in the ledger, under insults. | *(cut)* |
| Aske | Twelve men. No. Fourteen. Bad archers. | Fourteen men in the fort. I counted twice. Some have bows, and they shoot badly. |
| Tamsin | Morning, Brennish. Up a tree all night, counting brigands for us? I'm touched. | That's what I saw too, Brennish. Why bring it to us? |
| Aske | Nobody has touched you. And it is not for you. They killed three deer last night and left two to rot. In my wood. I will help you, noble. Once. For the deer. | Not for you. They killed three deer last night and left two to rot, in my woods. I will help you, noble. Once. For the deer. |
| Maud | Rue, the chapel cellar bars from the inside. I would ask you to go down. I do know, with respect, what you are about to say. | Rue, the chapel cellar bars from the inside. Please go down. It is you they have come for. |
| Rue | I'm not a cook, Keeper. I'm a girl who can light the oven without a match. Different thing. | No. I've hidden in enough cellars. I'm sworn to fire, Keeper, and you know it. I can burn them. I'm fighting. |
| Hollis | Wooden swords. Four years of wooden swords. Try not to cut yourself on the real one. | Four years of wooden swords. This one cuts. So will theirs. |
| Aske, to the earnest reply | The miller's wife buys my rabbits. She pays too much. She thinks I do not notice. I notice. Keep them away from her. | The miller's wife buys my rabbits. She pays too much, on purpose. Keep them away from her. |
| Tamsin, to the wry reply ("Only fourteen? I'm insulted.") | Ha! Finally, somebody said it. I've seen that lot fight, Your Former Grace. Half of them couldn't hit the river from the bridge. | Ha! Somebody had to say it. / Don't be. They were good soldiers once, and Harl still is. |
| Rue, to the blunt reply | Giving orders already, heir? Pity. It's the one I'd have given. | Giving orders already, heir? ...Fine. It's the right one. |
| Tamsin | Fourteen of Harl's finest, against an exile, an old man, a keeper, a cook, a poacher and me. Put it on the crown's tab. | Harl won't leave with that paper unpaid. Any of them who gets away comes back tonight with torches. So it has to be all of them. |

#### In the battle

| Who, when | Old | New |
| --- | --- | --- |
| Harl, fighting the lead | Nothing personal, Your Grace. You're three hundred crowns and a hot meal for my lads. | ...a hot meal for my men. |
| Harl, fighting Tamsin | Rook. You left before pay-day. Bad habit. | Rook. You left before payday. Bad habit. |
| Hollis, dying | Hnh. Guard was low. Mine, this time. | Hnh. I'm down, kid. Don't stop for me. |
| Hollis, retreating | That's me done. Fifty-six, and beaten off a field by my own knees. / Guard up, Veyne. Finish it. I'll have the kettle on. | My leg's gone. I'm no use in the line like this. I'm falling back. / Guard up, Veyne. Finish it. I'll be at the house. |
| Tamsin, retreating | That's a hole in me and a hole in the jacket. I only own the one jacket. / I'm out, Your Former Grace. Try not to win it too cheaply without me. | I'm hit. I can't stay in the saddle like this. / I'm out, Your Former Grace. Don't let any of them walk away. |
| Aske, retreating | I am hit. It is not deep. I will go and sit behind the mill and be angry. / Do not lose, noble. I will know. | I am hit. It is not deep, but I cannot draw the bow. / I go back to the mill. Do not lose, noble. |
| Maud, dying | Oh. Oh, dear. That is... that is supposed to be on the inside. | Oh. Oh... I can't close this one. Not on myself. |
| Maud, retreating | I'm hurt. I think. Yes. No, don't look, I'm the keeper, I'll look. / I must withdraw. Ama keep you. Also, please, keep pressure on anything that leaks. | I'm hurt. I... yes. I can't heal anyone like this. / I must withdraw. I'm sorry. Ama keep you all. |
| Rue, dying | Vael's teeth. Typical. First time in years I ate every day. / Don't stare, heir. The Keeper's stew was all right. It wasn't. I'd have told her it was. | Vael's teeth. Not now. I only just stopped being hungry. / Don't stare, heir. The Keeper's stove was warm. I should have told her so. |
| Rue, retreating | Ow! Vael's teeth! Fine. Fine! I'm going. / Don't you dare lose while I'm not looking, heir. | Vael's teeth! I can't hold the fire steady. I'm going. / Finish them, heir. All of them. |

Unchanged in the battle: Hollis's first-turn line, Harl's other lines
("Still owed. Figures." included), the lead's fall, Tamsin's and Aske's
death lines, the rest of Hollis's and Maud's.

#### After the battle (`ch01_victory`)

| Who | Old | New |
| --- | --- | --- |
| Tamsin, over Harl | Pay-day at last, Harl. I'll mark the ledger: paid in full. | Payday, Harl. I'll mark you in the ledger. Paid in full. |
| Narration | Nobody laughs. She doesn't either. She crouches, closes his eyes with two fingers, and takes a folded paper from inside his vest. | She doesn't smile. She crouches, closes his eyes with two fingers, and takes a folded paper from inside his vest. |
| Tamsin's face on "I'll ride along as far as there's pay. ...Further, probably." | grinning | plain |
| Narration (wry reply, Hollis and Tamsin both dead) | It would have got a laugh, yesterday. | It would have gotten a laugh, yesterday. |
| Hollis | Go and pack. I'll find a spade. ... | Go and pack. I'll find a shovel. ... |

The one light moment kept here: Aske's "You owe me eleven arrows, noble."
`ch01_tbc` (Dace at Veyne Hall) is unchanged.
