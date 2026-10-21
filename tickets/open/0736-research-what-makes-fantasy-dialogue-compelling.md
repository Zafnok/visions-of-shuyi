---
id: "0736"
title: "Research: what makes fantasy game dialogue compelling, written into the story-writing skill"
type: research
milestone: M6 Story & dialogue
model: opus-5.5
effort: high
status: todo
blocked_by: []
nick_input: decision
completed:
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
- Look for existing writing-craft resources made for Claude or other AI
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

- [ ] `docs/story/craft.md` lists the sources looked at, with licences,
      and the techniques taken from each studied game.
- [ ] The story-writing skill has the new Craft section, its scene
      checklist, the blank-page rule and a scene-length budget.
- [ ] Nick has seen the before/after examples and his answer is recorded.
- [ ] `typos` passes on the changed files.

## Tests required

- None (documentation). `cargo xtask ticket-lint` passes.

## Completion notes

*(Filled in by the session that completes the ticket.)*
