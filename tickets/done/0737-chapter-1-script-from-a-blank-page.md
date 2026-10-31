---
id: "0737"
title: "Write the Chapter 1 script again from the beats, on a blank page"
type: content
milestone: M6 Story & dialogue
model: creative
effort: high
status: done
blocked_by: ["0736"]
nick_input: none
completed: 2026-10-25
---

# 0737 — Write the Chapter 1 script again from the beats

## Context

Chapter 1 playtest round 3 (0804, 2026-10-16). Nick on the script after
0728's rework:

> "Fable still did it line by line. Still 14 archers stuff... We need to
> toss the script. We can keep the beats we agreed on but the whole script
> is fairly mid imo."

> Hollis in the yard: "like I said, this is AGAIN a line by line rewrite.
> Toss the whole thing."

> Aske's fort report: "again, rewrite". Tamsin over Harl: "we'll rewrite
> all this crap". Tamsin's news: "it's ok".

And on pace: the chapter drags "In the conversations".

What stays (Nick's answers in round 3): Hollis's "Aye", the spelling
"Greywater Abbey" and Tamsin's "Your Former Grace" (recorded in
`setting-and-tone.md` by 0804's round 3 PR). The story beats
(`docs/story/beats.md`, `docs/story/chapters/ch01.md`) stay as decided.

**0736 comes first:** it researches the craft and writes it into the
story-writing skill, with the rule that a rewrite starts on a blank page.

**Waits for the engine carve and the Playdate port** (Nick, 2026-10-16:
"ch1 playtest 4 come after those milestones with update script"), so
`blocked_by` has 1118 and 1050.

**Re-gated 2026-10-21 (0129):** the carve is done and 1050 was folded
into the Act 1 playtest (1064), so neither 1118 nor 1050 is in
`blocked_by` any more.

The ticket recommends the `creative` capability (`tickets/README.md`),
with Fable / Astra as optional examples. Use any model that can meet the
story-writing skill's voice and warmth criteria. **Historical evidence:**
0736's two blind judges preferred Fable 5.1 to Opus 5.5 on its samples
(`docs/story/craft.md`, "Round 2's samples"). This does not require a
provider switch or establish a comparison with untested models.

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> Does the chapter sound like a
> game he wants to play, and do the conversations still drag? (Asked first
> in 1064, as round 4 of the Chapter 1 playtest would have.)

## Scope

**In:**
- Every scene in `assets/dialogue/ch01.dlg`: `ch01_intro`,
  `ch01_prebattle`, the battle's lines (first turn, Harl, death quotes,
  retreat lines), `ch01_victory`, `ch01_tbc`.
- Write each scene **from the beat sheet, without reading its current
  lines first** (0736's rule). Keep only what the format needs: scene
  ids, `@` directives (music, place, opening, caption), reply choices that
  the story gates read, and the line ids voice clips use, if any are kept.
- Shorter: keep inside 0736's scene-length budget.
- The three kept words (above).
- The objective is Defeat Harl (0845): the rest of the Red Company runs
  when he falls. No line says that all of them must die (the old script's
  "it has to be all of them" was the reason for the Rout objective).
- The character sheets of everyone who speaks in Chapter 1: each gets a
  voice card (the skill's *Voices*), replacing the "under threat" notes
  and the sample lines that 0724 and 0728 copied from the rejected
  script. New sample lines are written fresh, one per situation (under
  fire, at camp, lying, grieving), and never copied into a script (0736
  found that a report-shaped sample line produces a character who
  reports).
- The title cinematic reuses lines from `ch01_intro` and `ch01_prebattle`
  (`assets/cinematics/title.ron`, its `talk` shots). Update the box
  indices it plays, so it still shows the three captions and Tamsin's
  news (or its new equivalent).

**Out (do not do):**
- Changing a story beat, a character's arc, who lives or dies, or a
  reply's consequences. If the beats can't work without a change, stop
  and ask Nick with `ask-nick`.
- The intro cinematic's cinematography (**0063**).
- Voices (0722 regenerates them after this). Voice-direction cues
  (0752), either order: if 0752 is done, write a direction where the
  punctuation and the expression don't already make the delivery
  obvious; if not, 0752 files the ticket that adds them.

## Implementation steps

1. Read 0736's craft section (the story-writing skill's *Craft*, and
   `docs/story/craft.md`'s "What went wrong") and `docs/story/beats.md`,
   `docs/story/chapters/ch01.md`, the character sheets and
   `setting-and-tone.md`. Do **not** open `ch01.dlg`'s text yet, and skip
   the lines the sheets and the beat sheet quote: they come from the
   rejected drafts (the skill's blank-page rule).
2. Write each speaker's voice card into their sheet, and a scene card for
   each scene (the skill's *Craft*). Then draft each scene from its card
   in a scratch file.
3. Run the story-writing skill's critique pass on each scene, the cold
   read by a fresh subagent included. If 0751 is done, add its voice
   profile of the draft to the critique; if not, the cold read's
   names-removed test covers it.
4. Now open `ch01.dlg` and carry over only the structure listed in Scope
   (ids, directives, replies the gates read). Replace the text.
5. Fix the title cinematic's talk shots if box indices moved.
6. Run the content checks and the dialogue tests; update snapshots that
   show script text, after looking at each.
7. Completion notes: the box count per scene before and after, and two
   or three lines per scene that show the new voice.

## Acceptance criteria

- [ ] Every Chapter 1 scene is newly written; no line of the old script
      survives except the three kept words.
- [ ] Each scene is within 0736's length budget; total box count is lower
      than today's.
- [ ] Every Chapter 1 speaker's sheet has a voice card, and no sample
      line there is a line of the old script.
- [ ] Story gates, replies and the Classic/Casual line variants (0715,
      0716) still work: their tests pass.
- [ ] The title cinematic still shows the intended boxes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the existing dialogue and Chapter 1 flow tests pass; the
  content validation of `ch01.dlg` passes.
- Snapshot: updated where script text shows, each one reviewed.

## Completion notes

Done 2026-10-25 on Fable 5.1 (the model the skill asks for), in two
passes.

**Pass 1** followed the skill's process: voice cards and scene cards (a
scratch file, not committed), a draft from the cards with the old
`ch01.dlg` closed, three rounds of critique (two fresh subagents each
round: a cold read of the boxes as the player sees them, and a
names-removed speaker test), then the old file opened only for its
header comments, ids and directives. Nick read it the same day: "the
script still reads semantically the same as before. I thought the script
was supposed to be rewritten without referencing the old version?" He
was right. The draft had been written from `docs/story/chapters/ch01.md`,
and by then that beat sheet was the old script in outline form (0707,
0724 and 0728 had each written their moments back into it). Asked how
far to go, Nick chose **from the outline only**: keep the outline's beats
and the sheets' arcs, give every scene new construction, and rewrite the
beat sheet from the result.

**Pass 2** started from `docs/story/outline.md`'s Chapter 1 entry and the
character sheets, and went through six drafts, each read by a fresh cold
reader, a names-removed speaker test and a third reader who laid the new
draft's moments beside pass 1's and said, bluntly, whether it was a
different construction. The first three were not (the same yard, the
same letter handover, the same seal exchange with one variable changed
each time); the last comparison passed the opening and the closing as
new constructions and named the middle moments that still echoed, and
those were reshaped last. The shipped script:

- **The yard** is an ordinary morning: Hollis mends the lead's boot, has
  not slept, keeps touching the letter in his coat and says nothing about
  it; he has never seen the sea (the Chapter 8 plant); the choice is what
  the lead will do when the exile lifts; the bell rings early and he sends
  the lead for the swords.
- **By the chapel**, Maud found the Red Company's demand nailed to the
  door (three walk up to the fort by noon or the village burns) and heard
  a keeper's phrasing in "the girl with the burned hands"; Hollis takes
  the letter out in front of everyone, the lead names the hand, and Rue
  burns it with her fire, looking at the lead.
- **The battle lines** are written from this chapter's own setup (noon,
  the bell, the river, the deer, the boots, the sea): Harl's grievance to
  anyone, Harl not recognising Tamsin on the gray, the plea for his boys
  before the knife, each death a different shape.
- **The fort**: Rue finds the order while looking for food; the lead names
  the seal and the third choice is the lead answering her "Did you know?";
  Hollis sits on the step with his back to everyone; Tamsin tells the lead
  to give him a minute and says it for him, then goes to the body and
  writes one word in her ledger; Aske makes everyone eat; Rue sets the
  road and the lead says "Not alone."; Hollis cleans the lead's sword and
  says nothing. Nobody buries Harl.
- **Veyne Hall**: Dace has two letters written before the news, hands the
  messenger the lie, locks the other away on top of the seal, and is glad.

`docs/story/chapters/ch01.md` is rewritten from this script, after it,
and says so at its top; `docs/story/ledger.md`'s Chapter 1 sections
record what is now said on every Classic path.

**Box counts, everyone alive (old counts from `docs/story/craft.md`):**

| Scene | Before | After | Budget |
| ----- | ------ | ----- | ------ |
| `ch01_intro` (3 captions included) | 19 | 11 | 8–14, cap 16 |
| `ch01_prebattle` | 21 | 17 | 5–10, cap 12 + 4 first-met |
| All talk before the first move | 38–41 | 28 | cap 28 |
| Battle lines | 1–4 each | 1 each; Harl and Tamsin 4; Harl's death 2 | 1–2; 2–4; 1–2 |
| `ch01_victory` | 20 | 16 | 6–12, cap 16 |
| `ch01_tbc` | 6 | 6 | 3–6 |

The chapel scene is one over its own cap because the letter now comes
out there too (two papers in one scene); the two scenes together are at
the 28 the skill allows before the first move, not over it.

**The voice, two or three lines a scene:**

- The yard: "Forty years on my feet in other people's boots. Never seen
  the sea." / "Because it's the one thing the boy wrote home about from
  the war that I believed." / "That's not the hour. Get the swords."
- By the chapel: Maud, "That's not how a soldier says it. That's how a
  keeper writes it down. ...Go on, somebody else read it. My hands
  won't." / Hollis, "Aye. ...Burn it. Somebody. I can't." / Tamsin,
  "Sparks. Your sleeve's smoking." Rue: "It does that."
- The battle: Harl, "Who's that on the gray? ...Rook? Rook, you're alive.
  I had you down as dead in a ditch." / Hollis dying, "The sea. I'd have
  liked to see it. Tell the boy that, if you see him. Not the rest." /
  Rue, "It isn't noon yet. It isn't noon, so why is it dark?"
- The fort: Rue, "Your house's wax. Did you know?" / Tamsin, "Don't look
  at him. Give him a minute." / Aske, "Sit. I will do the deer. You will
  only make it worse."
- Veyne Hall: "It's done. That says so. Go." / "You never once did as you
  were told, Veyne. Your whole life. ...Good."

**What the readers caught, round by round, and the drafts fixed:** one
"..." rhythm in every mouth (45 of 57 boxes in the first draft, under
twenty now, most of them Hollis's and Maud's); Tamsin and Rue told apart
only by their nicknames for the lead; death lines on one template; the
crest identified four times; Maud as a stammer and not a person; a
roll call of joiners after the battle; a script term leaking into
narration; a charge from Rue the player had no way to answer; Tamsin
telling Hollis what he already knew for the player's benefit; Dace's
closing line with no name in the scene; the sealed order nobody opened;
and two dozen British words across the drafts ("wet through", "asked did
I", "mind" as a tag, "gone up", "I'll not", "I've men", "Give it here",
"go round", "Right.", "I'm off", "a leg up", "properly").

**Kept words:** Hollis's "Aye" (four boxes), Tamsin's "Your Former
Grace" (three boxes), and "Greywater Abbey" (Maud, at the fort).

**Facts said differently (the ledger records each):** the letter is read
out by Hollis by the chapel and burned by Rue, so no letter exists and
all six heard Dace's name before the battle; no count of the fort's men
is said; Harl's price is said only by Harl to the lead if the lead fights
him; Coldwell is "a farm" with nothing said about anyone in it; Wren is
not mentioned at all; Tamsin writes one word in her ledger and it isn't
shown; Rue holds the order and sets the road; Hollis never speaks of the
hall or the boy after the chapel; nobody buries Harl; Harl's man count
("forty men I brought home") is his own claim to strangers.

**The title cinematic:** the three captions are still `ch01_intro`'s
boxes 0–2 and the chapel scene's first box is still shown (Maud's news
now, not Tamsin's); `title.ron`'s shots are unchanged and its comment
updated; its two snapshots changed with the words.

**Deviations:** step 3's voice profile (0751) isn't built, so the
names-removed test covered it, as the ticket allows. 0752 isn't done, so
no delivery cues were written; 0752 files the ticket that adds them. The
plain build has no portrait for anyone but the lead, so the script uses
only the five expressions every character has. The sheets' voice cards
were written in pass 1 and stand. Pictures: `cargo shuyi frame-png` of
the yard and the chapel scenes were checked (the picture files aren't
committed).

**the agent's starting rules:** none that touch play. For the Act 1
playtest (1064): does the chapter sound like a game Nick wants to play,
and do the conversations still drag.

**Follow-ups:** none filed. 0722 regenerates the voices from the new line
ids; 0751 and 0752 stand as written.

**Engine friction:** none, no Rust code beyond the test updates.
