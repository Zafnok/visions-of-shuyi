---
name: story-writing
description: Write or revise story material — story bible, character sheets, outline, chapter beat sheets, and .dlg dialogue scripts — following the pipeline in ADR-0011. Use for any 07xx story ticket or whenever dialogue/cutscene text is written. Not for story decisions, which are Nick's (ask-nick).
metadata:
  version: "1.0"
---

# Story writing

Read `docs/adr/0011-story-authoring-pipeline.md` first. The quality comes from
the process; do not skip steps.

## Canon order (higher wins on conflict)

1. `docs/story/beats.md` — Nick's beats. Never contradict. If a beat is
   ambiguous, pick an interpretation and flag it in the PR.
2. `docs/design/*.md` — game systems (e.g. if magic exists, how it works).
3. `docs/story/bible.md`, `characters/*.md`, `outline.md`.
4. `docs/story/ledger.md` — what has happened so far.

## Names are variables

Nick may rename anything (`docs/story/names.md`). Every proper noun has a
stable id there; add new names there first, then to the game's names table,
`assets/data/names.ron`. In `.dlg` scripts, speakers are role ids and names
in text are name tokens: `{n:king}` for "Emeric", `{N:place.thornmarch}` for
"The Thornmarch" at the start of a sentence (`assets/dialogue/README.md`,
"Names"). Never write a registered display name literally; the validator
rejects it. First names are `{n:<id>.first}` (`{n:retainer.first}` for
"Hollis"); the other short forms (family names, a god's name or title) are
in the registry's "Short forms" table. The lead is `{lead}`, never a name token. The story docs use
display names; a rename is a find-and-replace across `docs/story/` in the
same commit as the registry change.

## Craft

Nick has rejected four kinds of writing, and each rejection marks an
edge of what he wants:

- **Quips everywhere** (0707): "cheesy and cringe asf", "too many quips".
- **The jokes cut out** (0724, 0728): "reads like reports", "matter of
  fact", "fairly mid", "your writing is so bland", every voice in the same
  rhythm, "it drags in the conversations".
- **Tight, clever subtext** (0736's first samples): "These people are so
  clearly robots (i.e. Claude's specific vision of an optimized utopian
  world). Have more warmth without cringey jokes every 2 lines. Just talk
  more human."
- **Exclamation marks everywhere** (0736's third samples, after he asked
  for fewer flat periods): "now she's annoying... There just needs to be
  some more expression is what I meant... You are taking my instructions
  at face value, when I need nuance."

Warm, human samples between those edges he called "better... in the
right direction" (0736's second samples, which became the example below).

**Each of Nick's notes is a nudge, not a swing.** Every rejection above
came from swinging to the opposite extreme of the note before it: quips,
then no jokes, then guarded subtext; warm but flat, then exclamations
everywhere. When he says "more X", move one notch toward X, keep what he
liked last time, and check that the last draft's strengths survived.

What he wants, in his words: characters who are "first and foremost
engaging and emotional", so that their deaths matter. **The first question
for every scene is whether a player will like these people more after it,
and would miss them.** Craft comes second. The research behind this
section, with its sources, is `docs/story/craft.md`;
read "What went wrong" and "Why AI characters read as robots" once before
your first scene.

### Rewrites start on a blank page

- **A rewrite of a scene is written from the beats, not from its old
  lines.** Open `docs/story/beats.md`, the chapter's beat sheet
  (`docs/story/chapters/chNN.md`), the character sheets and the ledger.
  Keep the old `.dlg` closed until the new draft has passed the critique
  pass. Then open it only to carry over structure: scene ids, `@`
  directives, the `@choice` replies the story gates read, the `@if`
  blocks.
- **Quoted old lines count as old lines.** Sample lines in the character
  sheets and lines quoted in a beat sheet may come from a rejected draft
  (Chapter 1's do: 0707, 0724, 0728). Read the sheets for want, need,
  flaw, voice notes and *Won't say*; skip the quoted lines. A line the
  beat sheet quotes tells you what that moment must get across, not the
  words, unless Nick kept the words (`setting-and-tone.md`, *Words
  kept*).
- **Never fix a rejected scene line by line, and never splice lines from
  two drafts.** If a scene needs more than a couple of lines changed, it
  needs a new draft from its card. With two drafts, keep the better one
  whole.

### Before the first line: the scene card

Write a card for each scene in a scratch file (never in the script):

1. **How each speaker feels about the other** (likes, worries about,
   resents, owes, is a little afraid of) and **how each feels in their
   body right now** (cold, hungry, sore, exhausted, sick with nerves).
2. **What each one wants here**, and **one thing on their mind that isn't
   the mission** (a person they keep looking at, their feet, a debt, home).
3. **Who is not okay** in this scene: scared, out of their depth, wrong,
   grieving, embarrassed. At least one person.
4. **The turn:** the box where something changes (what someone knows, a
   relationship, the plan, the stakes). One turn per scene.
5. **Where it starts and where it cuts.** Box 1 is a person in a situation:
   trouble, or someone doing something human with trouble close. The scene
   ends on the turn or one box after.
6. **What the player must leave with.** Numbers the map or the objective
   shows stay there.
7. **The obvious staging, written down, then a different one.**
8. **The box budget** (table below), and a **voice card** for anyone who
   doesn't have one yet.

Write the card in the scene's own mood, not as a list of facts: a beat
written as a briefing gets transcribed as a briefing.

### How a scene works

- **Every scene has people being people.** One to three boxes that do
  nothing for the plot and everything for the relationship: a check-in
  ("did you eat?", "you're soaked"), fussing, a tease about a habit, a
  shared complaint about the cold. They are what players come to love
  companions through. Under threat they stay, as plain care; the jokes
  don't.
- **Care lands on the body.** People in danger look after each other's
  feet, hands, food, sleep and wounds. Each companion has a care
  signature: the thing they notice about other people. The carer insists,
  the other waves it off, nobody says thank you, and the scene moves on.
  It isn't funny, so it belongs in the straight opening and in serious
  battles.
- **Small feelings are said out loud; the big one once.** People say they
  are scared, cold, sick of this, glad you came, sorry. The big feeling
  (love, grief, pride) is said once per relationship per act, plainly, at a
  threshold (the night before, right after a loss), and followed by a
  deflection or something practical, never a speech or a matching
  confession. Not allowed: therapy words, narration naming a feeling, the
  next box explaining a hint, a line whose only content is the feeling.
- **Somebody isn't okay, and somebody is wrong.** Nobody has full
  information. People guess wrong, are bad at things, want their pay, want
  to go home. A small selfish or scared motive can stand without the scene
  punishing, explaining or forgiving it. A cast that is always fair,
  competent and calm is the robot Nick saw.
- **People talk about the small thing while the big thing happens.**
  Boots, porridge, whether the ground is too hard to dig. Under strain
  people get ordinary, not eloquent, and the feeling leaks around the
  edges.
- **They reach for the feeling and miss.** A restart, the wrong word, an
  apology for the apology, the listener cutting it short. One false start a
  box at most, landing on a plain word the player can't misread.
- **Disagree like people who like each other.** Half agree first, hedge,
  ask; the one in charge overrules and maybe regrets it. When someone runs
  themselves down, the others contradict them, fast. Conflict is one engine
  of a scene, not the only one: worry, teasing and shared fear are others,
  and sometimes two people are simply on the same side.
- **Shared history sounds like habit.** An old argument worn smooth (salt
  or honey in the porridge), never settled; correcting each other's
  stories; repeating each other's words; a ritual from before the story
  (skipping stones, a song). It is boring on purpose: the warmth is the
  familiarity.
- **Facts are said once, plainly, by someone who cares about them.** A
  fact the map shows stays on the map. Exposition is said once; people may
  repeat themselves when upset or scared, which is a different thing.
- **The plainest line is the one people remember.** A scene has at most
  one box that ends on a short, punchy sentence. No similes or crafted
  images in danger, no slogans, no "not X, but Y", no lists of three, no
  moral to end on. Dry wit belongs to one character's voice, not to every
  serious scene. If a line sounds like the writer, it is the writer.
- **Plant something small for everyone who can die.** A plan for after the
  war (small and a little odd: bees, a sister's wedding, fixing a cart
  wheel), an object seen in their hands in life, or a story they put off
  ("ask me after"). Said once, early, in a quiet scene. Their death lands
  on it, without explaining it.
- **Narration shows people.** A `>` line shows a face, hands, someone
  sitting down, one object in use. It never explains a silence or a
  feeling, never speaks for the dead, and a scene never ends on a symbol
  for the player to decode.
- **Start late, cut on the turn.** Drag is repetition, recaps and talk
  after the turn, not warmth. Cut a report box for every human box you
  add.

### Grief and death

- **Last words are about the living.** Errands, worry about someone else,
  a nickname, a habit: "the gray mule has a stone in her hoof", "make the
  boy eat". No speech, no "avenge me", no summing up.
- **The one hurting most says the least.** Their boxes are the shortest; a
  box of only "..." is allowed. The others fumble: apologize, offer to come
  back later, trail off. The bereaved deflects with something gruff or a
  tiny memory before the real thing comes out.
- **The dead are remembered with a small, true, unflattering story.** He
  couldn't swim; he snored; he owed four pennies and brought it up every
  year. Never a list of virtues. Someone may be angry at the wrong person.
  The work goes on (dig, feed the horses), and the dead turn up later in
  someone else's habit or saying.
- **A death needs warmth spent before it.** Kill nobody the player never
  saw in an ordinary, light moment.

### Voices

Each speaker gets a **voice card** in their character sheet (or the
scene card, for a minor character):

- **Age and experience:** how a person this age and with this life talks.
  A sixteen-year-old runs on, asks too much, overstates, wants to be taken
  seriously and is embarrassed when she says too much. Skills the life
  didn't teach don't appear.
- **Temperature:** hot (blurts, swears, shouts), cold, anxious or warm. A
  scene needs more than one.
- **Openness:** says it straight, says it angrily or clumsily, or hides it.
  The hiders only read as hiding because others say it.
- **When scared:** babbles, goes quiet, gets angry, prays, fusses, makes a
  joke that falls flat.
- **Care signature** and **one fault the others put up with** (vain,
  slow, eats too much, snores, sulks).
- **Line length** and **one habit** (answers with a question, trails off,
  counts things, never uses contractions).
- **Words from their life:** the people they love, their memories, their
  body; a trade's vocabulary at most once a chapter.
- **One thing they never say** (the sheet's *Won't say*).

- **Punctuation** (below): how often they exclaim, trail off, cut
  themselves off or end flat.

Under threat each voice does its own thing: a frightened teenager talks
more and faster, not less.

### Punctuation is the voice acting

The game has no voice acting on most lines and no stage directions, so
punctuation tells the player how a line is said. Nick, on 0736's second
samples: "A period is a fact, an ellipses or an emdash... at least implies
some trailing thought. And what about exclamation marks?" Those samples
ended 86% of their sentences on a period and had no exclamation mark at
all, even from a frightened sixteen-year-old. The next draft put an
exclamation mark in nearly every line she spoke, and Nick: "now she's
annoying. There just needs to be some more expression is what I meant...
now she's like Tiny Tina." **The target is in between: expression, not
exclamation.**

- **`!`** raises the voice: urgency, surprise, delight, alarm, a name
  called across a field, warmth that spills over ("You came back!").
- **`...`** trails off: a thought not finished, hesitation, a feeling the
  speaker can't get to the end of, a pause before the hard word.
- **`--`** cuts off: interrupted, self-corrected, a word swallowed. It is
  the script's dash (plain ASCII; the screen shows it as a dash).
- **`?`** asks, including the half-question of someone unsure ("You're
  coming back?").
- **`.`** is flat and final. It is a voice too: the gruff one's periods are
  his gruffness, and a period after a run of `!` lands hard. It is not the
  default for everyone.

Most of the expression comes from endings that fit the moment: a
question, a thought that trails off, a word cut short, and plain periods
for plain statements. **`!` is for peaks**: the laugh, the shout across a
field, alarm, a plea. In 0736's final sample the excitable sixteen-year-old
exclaims in 2 of her 15 boxes (the laugh at a privy door, pleading to
wait for dark) and the sergeant once, dying; the rest of her voice is
questions, trailing thoughts and cut-offs, and about 70% of sentences
still end on a period. Rough calibration: even the most excitable
character exclaims in one box in five or fewer, a calm one once a chapter
or at a death, and a scene with more than two or three `!` in all is
shouting.

A voice card's punctuation habit is a tendency, not a mark on every line:
the young or hot one exclaims and asks more than the others, the anxious
one trails off and cuts in, the cold one ends flat, and even he trails off
once when it matters. The faults are at both ends: a scene where every box
ends on a period (a report), and one speaker who ends nearly every box the
same way, whether `!`, `...` or `--` (a tic). Don't stack marks (`!!`,
`?!`), don't put `...` on every line (The Blazing Blade's Ninian is the
warning), and keep the plot facts in plain sentences. Put different temperatures and line lengths in
every two-portrait scene, so the voices are told apart by more than
rhythm. Run the cover test (critique pass) on every scene.

### Battle lines

- **A trigger line is one box** (two at most) in the person's normal
  voice, under about 120 characters. It reacts to what is happening on
  the map; it is not a tip. Fear, care and a name said twice are allowed;
  jokes are not.
- **A boss** gets one engage box for anyone (what he wants, and a jab or an
  offer), one defeat box, and a 2–4 box exchange only with someone who
  shares his history. His grievance is one a player could share. Dying, he
  can be scared, bitter or pleading, not tidy.
- **A death quote is one box** (two at most), about the living (above).
  Vary the shape across the cast.

### The silent lead

The lead's rules are below (*Writing the lead*). For the craft:

- **The cast carries the scene, and the lead's reply moves someone.** A
  reaction starts with what the reply did to that person (stung, relieved,
  embarrassed, grateful, amused), shown plainly; information may follow.
  Two replies that get the same answer are one reply.
- **Replies differ in stance and warmth, not information.** Each choice
  point offers at least one warm reply, and replies can be acts ("Eat
  something.", "I'll take the watch."). Saying nothing can be a reply, and
  the companion fills the gap.
- **The cast notices the lead.** In quiet scenes someone notices the lead's
  body or habits (the cut they hid, the meal they skipped) and cares for
  them. The cast may thank or be touched by the lead. What stays out is
  anyone telling the player the choice was correct, or praising the lead
  for something they didn't do.

### Getting human lines out of a model

The research found that the model's house character is a calm,
competent consultant, and that it hides feelings far more than people do.
Rules alone don't fix that (craft.md). What helps:

- **Write scripts on Claude Fable 5.1.** In 0736's blind test (four
  writers, the same three scenes, two judges who didn't know which was
  which), both judges ranked Fable 5.1's draft first and above Opus 5.5's,
  with or without the extra method below. Fable also scores a little
  higher on the public fiction benchmarks. No outside model was clearly
  better (craft.md, "Which model").
- **Optional tools, no proven gain in our test:** for the boxes that
  matter most (a fear line, a death line), draft five versions and keep a
  plain, less likely one; for a key scene, play the characters (one
  subagent per character, each given only its own sheet, one private want
  and one thing it hides, with a director who stops anyone agreeing before
  the turn; never let two agents chat freely, as they drift into
  politeness). Judge the result blind against a straight draft.
- **The cold read** (critique pass) asks who the reader would miss, not
  only what each person wanted.

### Scene-length budget

Claude's starting numbers (tunable; set by 0736 because Nick said Chapter
1 drags in the conversations). A box is one speech or narration line of
up to 200 characters; a reply's reaction counts. Each scene's budget
includes about two boxes for the people (a check-in, a tease, fussing).

| Scene | Target | Cap |
| ----- | ------ | --- |
| A chapter's first scene, opening captions included | 8–14 | 16 |
| Pre-battle scene | 5–10 | 12, plus 1 for each character the player meets for the first time |
| All talk before a chapter's first move | | 28 (Chapter 1 had 38–41, much of it repeats and reports) |
| Battle trigger (first turn, an area, half HP) | 1 | 2 |
| Boss engage, anyone | 1 | 2 |
| Boss and someone who shares his history | 2–4 | 5 |
| Death quote, retreat line | 1 | 2 |
| Victory scene (longest `@if` path) | 6–12 | 16 |
| Quiet scene: camp, travel, a support rank | 6–12 | 16 |
| Cut to the enemy (a tease) | 3–6 | 6 |
| A reply's reaction | 1–2 | 4 (the format's limit) |

Over budget means a scene has two jobs: split it, or move one job after
the battle. Cut a scene's job or its repeats, not its people.

### Examples

Read [`references/examples.md`](references/examples.md) before
drafting: the scenes Nick rejected (a report, robots) and three scenes
written from cards on a blank page, with what makes them work.

### Scene checklist

Run it on every scene, after the draft and before the critique pass:

1. Will a player like at least one of these people more after this scene?
   Who would they miss?
2. Does somebody visibly feel something, and is somebody not okay?
3. Is there a moment of people being people (a check-in, fussing, a tease
   about a habit)?
4. Does anyone talk like the writer: a clever closer, a simile under
   threat, a slogan? More than one box ending on a punchy line?
5. Could two speakers swap lines unnoticed (age, temperature, openness,
   punctuation, not only length)? Does a warm or scared character end
   nearly every sentence on a flat period?
6. Is anything said twice, or explained after it was shown?
7. Is it inside the box budget?

### Tone and language

- **Every named character** has: want (external goal), need (what they must
  learn), flaw, a secret or pressure, and an arc (start state → end state).
  Personal stories are the collision of wants between characters.
- **Every scene changes something** — information, relationship, plan, or
  stakes. If nothing changes, cut it. A relationship getting warmer is a
  change.
- **Tone** follows `bible.md`. Humour is allowed in dark stories; it makes the
  dark parts land. **Where it goes is decided**
  (`docs/design/setting-and-tone.md`, *Where the jokes go*): the game's
  opening and serious battles are played straight. No one-liners, running
  gags or comic bits before, in or over a fight that matters: boss lines,
  death and retreat lines included. People in danger talk like it. Banter
  belongs to downtime (camp, travel, quiet scenes after a fight) and grows
  as characters get closer (support ranks). The lead's `wry` reply is the
  player's own joke: it stays, and the answer to it may be light, briefly.
  **Played straight means no comedy bits, not no warmth:** scared people
  are kind to each other, and that kindness is the heart of a serious
  scene. Humor in downtime comes from a person (a habit, a weakness, a
  delight), never from a remark on the situation.
- **Understood on first reading.** A line says what it means: no turn of
  phrase that hides what happened, no line that leaves out its own logic.
  Be vague about feelings if you like, never about facts or the plan. The
  format has no stage directions, so don't write a line that only
  works if it is muttered or said aside; use narration (`>`) if how it is
  said matters.
- **American English** (`setting-and-tone.md`, *Language*): American
  spelling and American words in all game text (color, armor, gray,
  peddler; no "whilst", "bloody", "mate", "lads", "that lot").
  Old-fashioned words are not ruled out; British ones are. Plain words,
  no thee and thou, no dialect spelled out.
- **Gameplay tie-in:** pre-battle scenes set up *why this fight*; in-battle
  lines react to what the player is doing (boss taunt, recruit conversation,
  death quote); post-battle scenes pay off consequences.
- **Plain-prose rules are for narration only.** Advice that pushes text
  toward plain, literal, compressed statement (anti-"AI slop" lists, "say
  the literal thing") is right for `>` lines, menus and tips, and wrong for
  characters: people speak in idiom, fragments, repeats and false starts.

## Format constraints

- Dialogue goes in `assets/dialogue/*.dlg`. The format and every rule the
  validator enforces are in `assets/dialogue/README.md`; read it before
  writing a script. Run `cargo test -p trpg-content` to validate.
- Two portraits on screen max: `left` and `right`. Only use expressions listed
  in the character's sheet.
- Text box is 3 lines × ~70 characters. Keep each line ≤ 2 boxes (~200 chars).
- Plain ASCII punctuation in scripts (`'` `"` `...` `--`) — the font may not
  have smart quotes.

## Writing the lead

The lead is Persona-style (`docs/design/setting-and-tone.md`, "Rules for
writing the lead"); the validator enforces the limits below.

- **Few lines.** Outside reply choices the lead (`lead:`) speaks only in
  short, neutral lines of at most 40 characters ("Let's move."). Other
  characters carry scenes.
- **Reply choices** (`@choice` … `@endchoice`) at key moments: 2–3 replies
  with distinct tones (earnest / wry / blunt…), each at most 60 characters.
  Each reply gets a reaction of at most 4 text boxes, then the scene rejoins
  and goes on the same way for every reply. Choices never branch the plot,
  recruit or lose units, or change the ending. Budget: about 1–3 choice
  points per chapter.
- **Never fix the lead's personality** outside what the player picks: no
  cruelty, jokes or strong opinions in `lead:` lines.
- **Gender-neutral text.** The player picks the lead's gender and first name,
  so refer to the lead with `{lead}` and `{they}` `{them}` `{their}`
  `{theirs}` `{themself}` (`{They}` etc. at the start of a sentence). They
  become he/she, so the verb agrees with he/she: `{They} knows`, not
  `{They} know`. No line may depend on the lead's gender.

## Critique pass (mandatory before committing a script)

Follow [`references/critique-pass.md`](references/critique-pass.md):
the cold read by a fresh subagent, then the editor's list. Fix every
hit, then update `docs/story/ledger.md` with what changed for each
character.

## Gates

Stop and ask Nick (via the `ask-nick` style: short summary, options) only at:
bible + cast summary, and act outline. Present a **one-page summary**, not the
full documents.
