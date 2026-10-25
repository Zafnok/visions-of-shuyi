---
id: "0737"
title: "Write the Chapter 1 script again from the beats, on a blank page"
type: content
milestone: M6 Story & dialogue
model: fable-5.1
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

The ticket's `model` follows ADR-0010 (creative writing → fable-5.1).
Nick named Fable as the writer of 0728's rewrite; if he runs this one on
another model, that's his call. **0736 tested it:** in blind tests with
two judges, Fable 5.1 wrote warmer, more human scenes than Opus 5.5 every
time (`docs/story/craft.md`, "Round 2's samples"), so the story-writing
skill says to write scripts on Fable 5.1. If this ticket is started on
Opus, say so to Nick before writing.

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

Done 2026-10-25 on Fable 5.1 (the model the skill asks for), by the
skill's blank-page process: voice cards and scene cards first (a scratch
file, not committed), a draft from the cards with the old `ch01.dlg`
closed, three rounds of critique (each round two fresh subagents: a cold
read of the boxes as the player sees them, and a names-removed speaker
test), fixes from each round, then the old file opened only to carry over
its header comments, scene ids and directives.

**Box counts, longest path (old counts from `docs/story/craft.md`):**

| Scene | Before | After | Budget |
| ----- | ------ | ----- | ------ |
| `ch01_intro` (3 captions included) | 19 | 13 | 8–14, cap 16 |
| `ch01_prebattle` | 21 | 14 | 5–10, cap 12 + 4 first-met |
| All talk before the first move | 38–41 | 27 | cap 28 |
| Battle lines | 1–4 each | 1 each; Harl and Tamsin 4; Harl's death 2 | 1–2; 2–4; 1–2 |
| `ch01_victory` | 20 | 13 (everyone alive, and with only Hollis alive) | 6–12, cap 16 |
| `ch01_tbc` | 6 | 6 | 3–6 |

**The voice, two or three lines a scene** (the token-free reading):

- The letter: "That's good. No-- don't help me up, I'm getting up. That
  was good, kid." / "Peddler brought it last night, after you'd gone to
  bed. I should have woken you. ...I didn't." / "Eat something. Then
  boots, the good ones. And nobody is burning anything in this house."
- By the chapel: Rue to Aske, "You. Eat this. You're gray in the face, and
  if you fall over before they've even shot at us I'm not carrying you."
  / Aske, "I am not cold. I am angry. ... Half. You eat half, you are
  thinner than me." / Maud, "I have buried four children from this village
  and I am not burying you. Go down. Please."
- The battle: Hollis dying, "The porridge pot's still on the fire at the
  house. Somebody take it off." / Harl to Tamsin, "I brought forty men
  home from that war. Forty! And the crown gave us the road to starve
  on." / Tamsin, "...I know what I held."
- The fort: Tamsin over Harl, "He put me on my first horse. I fell off it
  in front of the whole yard and he laughed till he had to sit down. ...It
  was a job. That's what I'm writing. A job." / Rue, "Was it you? ...Was
  it your father?" / Hollis, "Find me a shovel. I'm not leaving Harl to
  the crows."
- Veyne Hall: "Tell him it's done. Tell him I said so." / "One thing I've
  asked of you in four years, and you-- Of course you didn't."

**What the critique rounds caught and the draft fixed:** one "..."
rhythm in every mouth (45 of 57 boxes in round 1, 8 in the shipped
script); Tamsin and Rue telling apart only by their nicknames for the
lead (Tamsin now runs on and deflects into the horse; Rue hammers short
imperatives, overstates and corrects herself, "I can burn all of them--
I can burn some of them"); five death lines on one template (now an
errand, a question, a will, a plea and a cold-fire question); the crest
identified four times (now once by Harl dying and once over the body);
Maud as a stammer (she now finishes a sentence and takes the lead's
hidden cut); a joiners' roll call in the victory scene (Aske asks Tamsin
why she's smiling; Rue's joining line cut); a dozen British words
("wet through", "asked did I want", "mind" as a tag, "gone up", "I'll
not", "I've men").

**Kept words:** Hollis's "Aye" (three boxes), Tamsin's "Your Former
Grace" (two boxes in the chapter plus one reply reaction). "Greywater
Abbey" doesn't come up in Chapter 1's new script; its spelling stands
for whenever it does.

**Facts said differently (the ledger records each):** the letter reads
"Get out of the march. Tonight. They are coming for everyone who was
there that night. Burn this."; no count of the fort's men is said (the
map shows them: Nick's "14 archers stuff"); Aske wants nine arrows back,
not eleven; Harl's order is never read aloud, so three hundred crowns a
head is said only by Harl to the lead if the lead fights him; Coldwell is
"a farm", with nothing said about anyone in it (the old line's "people"
was mine, not the sheets'); Hollis says "My girl's dead" once, to the
lead (the one allowed mention of Wren); Tamsin dying asks who else is
down and for her mare, and no longer asks for her ledger, so who has it
after a Classic death is a later chapter's call; Rue no longer says she
walks behind the lead (she still suspects the lead's family to their
face); Hollis goes out to pick up spears, not to the horses, so the
horses stay Tamsin's.

**Not a line-by-line rewrite:** no scene keeps the old order of boxes or
speakers; the yard opens after the disarm instead of before it, the
chapel scene starts on Tamsin alone from the saddle and runs Rue–Aske and
Maud–Rue exchanges instead of one line each to the lead, the fort scene
starts on Tamsin writing in the ledger and ends on the shovel.

**The title cinematic:** unchanged. The three captions are still
`ch01_intro`'s boxes 0–2 and Tamsin's news is still `ch01_prebattle`'s
box 0, so `title.ron`'s `Talk` shots play the intended boxes; its two
snapshots changed with the words.

**Deviations:** step 3's voice profile (0751) isn't built, so the
names-removed test covered it, as the ticket allows. 0752 isn't done, so
no delivery cues were written; 0752 files the ticket that adds them. The
plain build has no portrait for anyone but the lead, so the script uses
only the five expressions every character has (`weary`, `flustered`,
`sneer`, `smug`, `cold` would fail the validator here); the sheets still
list them for the bought art. Pictures: `cargo shuyi frame-png` of the
yard and the chapel scenes were checked and are in the PR's description
as text (the picture files aren't committed).

**Claude's starting rules:** none that touch play. For the Act 1
playtest (1064): do the conversations still drag, and does the chapter
sound like a game Nick wants to play (the question this ticket inherited
from round 3).

**Follow-ups:** none filed. 0722 regenerates the voices from the new line
ids; 0751 and 0752 stand as written.

**Engine friction:** none, no Rust code beyond the test updates.
