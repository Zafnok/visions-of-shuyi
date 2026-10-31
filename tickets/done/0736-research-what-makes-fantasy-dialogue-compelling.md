---
id: "0736"
title: "Research: what makes fantasy game dialogue compelling, written into the story-writing skill"
type: research
milestone: M6 Story & dialogue
model: reasoning
effort: high
status: done
blocked_by: []
nick_input: decision
completed: 2026-10-23
---

# 0736 — Research: what makes fantasy game dialogue compelling

## Context

Chapter 1 playtest round 3 (0804, 2026-10-16). The script has now been
rewritten twice (0724, 0728) and Nick rejects both:

> "Fable still did it line by line. Still 14 archers stuff... We need to
> toss the script. We can keep the beats we agreed on but the whole script
> is fairly mid imo. We need to have you get a crash course on fantasy
> writing that's compelling. Idk a skill repo for writing? Your writing is
> so bland."

On single lines: Hollis in the yard, "like I said, this is AGAIN a line by
line rewrite. Toss the whole thing."; Aske's fort report, "again,
rewrite"; Tamsin over Harl, "we'll rewrite all this crap". He also said
the chapter drags in the conversations (`ch1-pace`).

So the problem isn't one more pass of edits; it's the writing craft
itself. This ticket builds that craft into the tools before **0737**
rewrites the script from the beats.

**Waits for the engine carve and the Playdate port.** Nick, same day: "I
don't want to delay engine carve or playdate port any more. So we can have
ch1 playtest 4 come after those milestones with update script." So
`blocked_by` has 1118 (the carve's last ticket) and 1050 (Nick plays
Chapter 1 on his Playdate).

**Re-gated 2026-10-21 (0129):** the carve is done and 1050 was folded
into the Act 1 playtest (1064), so neither 1118 nor 1050 is in
`blocked_by` any more.

## Nick input

**Decision** (a story-direction question, `ask-nick`): Nick reads the
new craft section's before/after examples
(step 4) and says whether they read like a game he'd want to play. If he
wants a different direction, he says which of the studied games is
closest to what he wants.

## Scope

**In:**
- Look for existing writing-craft resources made for the agent or other AI
  writers: published skills (for example Anthropic's skills repositories
  and community skill collections), writing-craft prompts and guides.
  For each, note what it teaches and its licence. Copy nothing whose
  licence ADR-0013 doesn't allow; borrow ideas only, in our own words.
- Study what makes dialogue compelling in story-heavy fantasy and tactics
  games Nick knows or that fit the tone (`docs/story/bible.md`,
  `setting-and-tone.md`): for example Fire Emblem (The Blazing Blade, Three
  Houses), Tactics Ogre, Final Fantasy Tactics, Triangle Strategy,
  Unicorn Overlord, Octopath Traveler, plus a few fantasy novels known for
  dialogue. Pull out techniques, not quotes: subtext, what's left unsaid,
  each voice's rhythm, conflict inside a scene, entering late and leaving
  early, how exposition hides in argument, and specific detail over
  generic. Any quote kept for study stays under 15 words, credited, and
  only in the research notes.
- Name what went wrong in 0724 and 0728 from their completion notes and
  the current `assets/dialogue/ch01.dlg`: line-by-line edits, characters
  reporting facts ("fourteen in the fort"), the same rhythm in every
  voice, and scenes that run long.
- Write it into the tools:
  - a new **Craft** section in `.claude/skills/story-writing/SKILL.md`
    (replacing or extending today's *Craft rules*), with a short
    checklist a writer runs on every scene;
  - the rule that a rewrite starts from the beats (`docs/story/beats.md`,
    `docs/story/chapters/ch01.md`) on a blank page, **without reading the
    old lines**;
  - a scene-length budget, because Nick said the chapter drags in the
    conversations;
  - a stronger *Critique pass* that catches "reads like a report" and "a
    line-by-line rewrite".
- Research notes in `docs/story/craft.md` (sources, techniques, what each
  studied game does), linked from the skill.

**Out (do not do):**
- Rewriting any of `ch01.dlg` (that's **0737**).
- Changing the beats, the bible or any character's arc: those are Nick's
  (`docs/story/beats.md`).

## Implementation steps

1. Search for writing skills and guides (web search). List them in
   `docs/story/craft.md` with URL, licence, and what each is worth.
2. Study the games and books above. For each: 3–5 techniques, with a
   one-line description of a scene where it shows (no long quotes).
3. Read 0724's and 0728's completion notes and `ch01.dlg`, and write a
   short "what went wrong" section.
4. Write the skill's Craft section and checklist. Show the technique with
   2–3 short **before/after** examples written fresh for this ticket
   (made-up characters, not Chapter 1's), so Nick can judge the voice
   without the old script.
5. Update the Critique pass and add the blank-page rule and the length
   budget.
6. Show Nick the before/after examples (an Artifact page is fine) and
   record his answer in the Completion notes.

## Acceptance criteria

- [x] `docs/story/craft.md` lists the sources looked at, with licences,
      and the techniques taken from each studied game.
- [x] The story-writing skill has the new Craft section, its scene
      checklist, the blank-page rule and a scene-length budget.
- [x] Nick has seen the before/after examples and his answer is recorded.
- [x] `typos` passes on the changed files.

## Tests required

- None (documentation). `cargo xtask ticket-lint` passes.

## Completion notes

Done 2026-10-23 on Opus 5.5 (the ticket's model), with research and
writing subagents: Nick asked for "maybe 20" at the start, then for more
rounds; about 40 ran in all (round 1: 16 research and critic agents, 3
writers, 2 judges; round 2: 8 research and critic agents, 4 writers, 2
judges; round 3: 3 writers, 2 judges). Sample writers ran on Opus 5.5 and
Fable 5.1 as the tests below say.

**What was done**

- `docs/story/craft.md`: the research notes. Nick's words; what went wrong
  in 0724 and 0728 (process and the script's patterns, with box counts);
  what went wrong in 0736's own first samples; warmth (what makes players
  care before a death); why AI characters read as robots; which model; the
  three sample rounds and their blind judgings; the AI-writer resources
  with licences (nothing copied; ideas only); craft teaching; what each
  studied game and book does (Fire Emblem 7 and Path of Radiance, Three
  Houses, Tactics Ogre, FFT, Triangle Strategy, Unicorn Overlord, Octopath
  I and II, Abercrombie/Cook/Martin, Pratchett/Lynch/Le Guin/Hobb,
  Expedition 33/Witcher 3/Persona/Yakuza); measured scene lengths; rule by
  rule, where each skill rule comes from; quotes under 15 words.
- The `story-writing` skill's *Craft rules* became **Craft**: Nick's four
  rejections and the rule that his notes are nudges; the blank-page rule
  (quoted old lines in sheets and beat sheets count as old lines); the
  scene card; how a scene works (people being people, care on the body,
  small feelings said, somebody not okay, small talk, disagreeing like
  friends, shared history, facts once, plain lines, plants for anyone who
  can die, narration shows people); grief and death; voice cards (age,
  temperature, openness, fear, care signature, fault, punctuation);
  punctuation as the voice acting, in proportion; battle lines; the silent
  lead; getting human lines out of a model (write on Fable 5.1); the
  scene-length budget; the before/after examples; a seven-question scene
  checklist. The **critique pass** now starts with a cold read by a fresh
  subagent (who would you miss? who says each box with names hidden?) and
  checks for robots, the writer talking, syrup, reports, a line-by-line
  rewrite, drag and stock lines.
- `docs/story/README.md` lists `craft.md`.
- Ticket 0737 updated: skip the quoted old lines; write voice cards into
  the sheets (replacing the "under threat" notes and sample lines 0724 and
  0728 copied from the rejected script); the objective is Defeat Harl, so
  no line says all of the company must die (the critic found the old
  "it has to be all of them" now contradicts 0845); the cold read; write
  on Fable 5.1; voice directions if 0752 is done. 0721 updated for 0752.
- `docs/design/voices-languages-and-script.md`: Nick's words on shipping
  this AI script until a human rewrite, and on voice cues.
- `_typos.toml`: Shepard, CoSER and Hask (names in the research and the
  examples).

**Nick's answers (verbatim), round by round**

1. Round 1 samples (written to craft rules: subtext, facts as ammunition,
   nobody names a feeling): "for the characters to have any emotional
   ties, for the deaths to matter, they need to be first and foremost
   engaging and emotional. These people are so clearly robots (i.e.
   The agent's specific vision of an optimized utopian world). Have more
   warmth without cringey jokes every 2 lines. Just talk more human, I
   don't know. Run more research subagents with this in mind. Or if there
   is a better model more suited to writing that isn't so 'tell, don't
   show' the way you're writing now then find that model. If it's a fine
   tune or something idk."
2. Round 2 (warmth research; Fable 5.1 won the blind test): "it's better,
   still a bit off. This is in the right direction though. One thing I
   don't like is the overuse of a period to end a sentence. A period is a
   fact, an ellipses or an emdash (as AI as it looks) at least implies some
   trailing thought. And what about exclamation marks? anyway, this is
   definitely better. Maybe one more round?"
3. Round 3 (punctuation as delivery; two versions): "now this Wenna
   character is too annoying, exclamations in every single line they
   give. Idk, there needs to be a balance. You are taking my instructions
   at face value, when I need nuance. I ask for more exclamations and now
   every line from this character has them, now she's annoying. There
   just needs to be some more expression is what I meant the first time,
   but now she's like Tiny Tina and that character didn't age well."
4. Round 4 (round 2's scenes, one notch more expression): "this is
   serviceable... still reads incredibly AI written but at least there's
   something here. It has the facade of a soul behind it. Eventually,
   we'll strip all the script for a real human made script but we'll ship
   with this AI script until I make enough money for a rewrite." Plus the
   voice cues (ticket 0752).

So the recorded answer: the round 4 voice is the accepted direction for
0737 ("serviceable"), with a human rewrite planned when money allows. The
round 4 scenes are the skill's example.

**Deviations from the plan**

- One round with Nick was planned; four ran, because his answers asked
  for more research (round 2, including the model search) and another
  round (round 3), and round 3 overshot.
- Step 4's examples were written by subagent panels and judged blind
  rather than written by the session alone, and were replaced twice.
- The skill's first Craft draft (round 1) was rewritten after Nick called
  its samples robots; craft.md keeps round 1's findings and says what was
  overcorrected.

**the agent's starting rules** (story pacing and style Nick hasn't fixed;
tunable, for the Act 1 playtest, 1064):

- The scene-length budget: a chapter's first scene 8–14 boxes (cap 16), a
  pre-battle scene 5–10 (cap 12, plus 1 per new character), all talk
  before the first move at most 28 (Chapter 1 had 38–41), a battle line 1
  box, a victory scene 6–12 (cap 16), a quiet scene 6–12 (cap 16). Nick
  saw it on the samples page.
- Punctuation in proportion: even the most excitable character exclaims
  in one box in five or fewer; a scene with more than two or three `!` is
  shouting.
- The big feeling (love, grief, pride) said once per relationship per act.

**Follow-up tickets:** 0751 (`cargo xtask voices`: a voice profile per
speaker, so "every voice in the same rhythm" can be measured), 0752
(voice-direction cues in `.dlg`, Nick's request).

**Engine friction:** none: no Rust code.
