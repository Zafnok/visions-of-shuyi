---
id: "0766"
title: "Find a writer: the ad, the trial scene and the pick"
type: content
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0152", "0153", "0764", "0765"]
nick_input: setup
completed:
---

# 0766 — Find a writer: the ad, the trial scene and the pick

## Context

Nick, 2026-11-08, after playing the Act 1 build:

> the writing is absolute dogshit and needs immediate repair at a dialogue level. The arcs and relationships I think are overall fine but the manner of speaking is robotic and awful. This is probably the 3rd or 4th iteraiton of trying to repair this and at this point I am leaning towards just introducing a human writer. They may look at our arcs and beats and decide they like them or hate them and throw them out entirely if they can't work with them for making compelling dialogue. Expanded in bullet below:
>    * So at this time, we should pause all writing since you can't write for shit
>    * And instead I will make an ad on Reddit probably INAT subreddit
>    * In order to make the ad, the game needs several components first:
>       * We need to have an easy to use scriptwriting kit (script like the dialogue script) that a nontechnical person can use
>          * Expose tone (for AI), var names, flags, and emotion (for the portrait)
>          * We should have an easy playtesting process for them to see how their script renders and add skip ahead / trigger event etc so that certain changes can be seen quickly.
>       * They need limited access to github repo
>          * Only able to make PR or issue and only for the scripts/ folder or whatever
>       * Likely we need to make github repo private
>       * Likely we need to move ALL assets to assets repo like including the script & sound files
>       * Basically, I want to show off the game in an extremely limited manner, without risking outright cloning, and to trial run some writers by asking for i.e. the pre-ch1 battle script to be rewritten as a trial run. This should be able to asked to many writers and pick best one. Without risking us getting copied

This ticket is the ad, the trial and the pick. It waits for the parts
Nick listed: the writer's mirror repository (0152), the game repository
private (0153), the preview program (0764), the guide and cheat sheet
(0765).

### The writing pause

**This ticket is the writing pause.** From 2026-11-08, while it is in
`tickets/open/`, no session writes or rewrites the words of dialogue:
no new scene, no reworded line, no support or camp event, no reply
choice (Nick: "we should pause all writing since you can't write for
shit"). What may still go ahead:

- edits to scripts that change no words: `@place`, `@music`, wiring a
  scene id into a chapter or battle, a format fix a tool needs (0741,
  0742, 0759; the chapter battles 0851–0854's places and music);
- tools that read the scripts (voice and translation tooling, line ids);
- short Vix tips and lessons in plain words (0483, 1081): an instruction
  to the player, not a character's scene.

What waits for the hired writer (each says so in its text, by an edit in
this ticket's PR or sooner):

- **1073** (Act 1's supports and camp events): its scenes.
- **1083** (side quests of Hollis and Rue) and **1084** (Maud's): their
  scenes and beat sheets. Their maps, battles and rewards don't need the
  words; whether they go ahead before the scenes is that ticket's split.

Not writing, but redone by any rewrite (a changed line gets a new line
id, and its voice clip and translations are lost: `docs/story/writers-guide.md`
§7): 0757 (Chapters 2–8's voices), 0719 and 0726 (Chapter 1 in Japanese
and Chinese), 0755 and 0756 (their voices). This ticket's completion
notes say whether each should wait for the hired writer's script.

When this ticket closes the pause does **not** end for AI sessions: the
words become the hired writer's. If Nick picks nobody, the completion
notes say so and the pause stays until Nick ends it.

### What the trial is

Nick: "asking for i.e. the pre-ch1 battle script to be rewritten as a
trial run". There is no battle before Chapter 1's. Chapter 1 plays two
scenes before its battle (`assets/chapters/ch01.ron:7`, `intro_scenes`):

- `ch01_intro` (`assets/dialogue/ch01.dlg:69`): the opening captions,
  then the letter;
- `ch01_prebattle` (`:100`): the Red Company; the party gathers.

About 85 script lines together. On screen: the lead, Hollis
(`retainer`), Tamsin (`sergeant`), Aske (`poacher`), Maud (`keeper`), Rue
(`heretic`); named but not on screen: the Red Company's captain
(`red_captain`) and `rival`. Their beats are `docs/story/chapters/ch01.md`
§0–§2 (`:71`–`:140`). Each scene has one reply choice for the lead.

What an applicant gets is a **packet**, never access to a repository:
the preview program (0764, which holds no script of its own), the two
scenes as they are now (marked as the AI draft to rewrite or discard),
the cheat sheet (0765), a one-page brief, a one-page world note with no
spoiler past Chapter 1, the beats of the two scenes, the sheets of the
six on screen (a line each for the two off screen), and the names they
use. Each packet carries the applicant's mark (a line at the top of its
`.dlg` file and in the zip's name), so a leaked packet can be traced.

### Terms

`LICENSE` §7 (*Contributions*) gives Nick a license to anything
submitted as a contribution to the Software. A trial entry sent by
message is not clearly that, and an applicant has agreed to nothing.
And the mirror (0152) copies no `LICENSE`. So:

- the packet carries a short plain-language note: the packet is
  confidential; what Nick may do with an entry (read and compare it;
  use it only if he hires its writer, or as the note says); what the
  applicant keeps. **The terms are Nick's.** The session drafts options
  in plain words and says it is not legal advice; a standard template
  or a lawyer's look is Nick's choice;
- the mirror gets `LICENSE` (a copy, not an edit: added to 0152's files
  list), so a hired writer's pull requests fall under §7.

Paid or unpaid, and what the hired writer is offered, are Nick's.

Related: engine 3083 (parked, marks and capture limits in a closed
preview program) is Out; 0901 (itch.io) is not needed for this.

## Nick input

**Decisions first** (the session asks with the question pop-up, one
question at a time, options plus "describe your own"):

1. The trial scenes: `ch01_intro` and `ch01_prebattle` as above, or
   another scene, or these plus `ch01_first_turn` (`:154`).
2. Compensation, as the ad will state it: paid trial, unpaid trial,
   paid work after the pick, a revenue share, or describe your own.
3. The terms for entries and the packet (the session's plain-language
   options, above).
4. The time box: the session proposes about a week for about three
   hours of work; Nick sets it.
5. Where packets are shared from (a link from Nick's own cloud drive,
   or sent as a file in a message) and how entries come back.

**Setup** (Nick's, in this order):

1. Read the ad draft and post it on r/INAT (and anywhere else he picks)
   from his account.
2. For each applicant, run the packet command the session gives him (or
   ask a session to), and send the packet.
3. Put each entry that comes back in `dist/trial/entries/<handle>/`
   (never committed), and ask a session to make the comparison page.
4. Pick, from the page. The session may suggest a second round for two
   or three finalists (another scene); Nick decides.
5. For the writer picked: the mirror → **Settings** → **Collaborators**
   → **Add people** → their GitHub username → they accept the email
   invitation. On a personal account a collaborator can write to the
   repository; the mirror holds no secret and the game takes in only
   pull requests Nick approved (0152). Removing them later: the same
   page, **Remove**.

## Scope

**In:**
- **The ad**, `docs/writer-trial/ad.md`: after reading r/INAT's current
  rules and posting template (for example whether the title must carry
  a tag for paid, revenue share or hobby, and the required sections),
  a draft in that shape: the game in two sentences (a tactics RPG in the
  manner of Fire Emblem, drawn in colored glyphs or pictures; Act 1
  playable); what is wanted (a writer for the dialogue; the arcs and
  beats exist and are open to their proposal); the trial; the
  compensation as Nick decided it; how to apply. American English, no
  link to any repository. Three or four pictures of the game made with
  `cargo shuyi frame-png` with the bought art (a screenshot of the
  running game may show it: `AGENTS.md`, *Bought art*), saved outside
  the repository for Nick to attach.
- **The packet's pages**, in `docs/writer-trial/`: `brief.md` (what to
  do, the time box, how to send, what is judged), `world.md` (one page
  from `docs/story/bible.md`, Chapter 1's knowledge only), `terms.md`
  (Nick's terms). The beats and sheets are copied from `docs/story/` at
  build time, not duplicated.
- **`cargo xtask trial-packet --for <handle>`**: builds
  `dist/trial/<handle>.zip` from the kit release's preview program (0764)
  and those files; the scenes extracted from `assets/dialogue/ch01.dlg`
  (by id, cut from the file at the `@scene` and `@end` lines so the
  comments and layout stay as written; not by line numbers), with the mark
  at the top; checks the result with `story-check` (no problem) before
  zipping.
- **`cargo xtask trial-compare`**: reads `dist/trial/entries/*/` and
  writes one page with the entries side by side, scene by scene, named
  Entry A, B, C … in a random order (Nick judges without names), the
  current draft as the first column, each entry's checker problem count
  under it. The key from letters to handles is a separate file, shown
  only after the pick. The page goes outside the repository.
- **A short reading guide** at the top of the comparison page, from
  Nick's own words: does it sound like people talking, not robotic;
  does each character sound like their sheet's voice notes; is there
  warmth without jokes every few lines (Nick's words quoted in
  `docs/story/craft.md` `~:28`); does the lead stay within the lead's
  rules. Nick picks
  by his own reading; the guide only says what to look at.
- The mirror's files list (0152) gains `LICENSE` as a copied file.
- The waiting tickets' text (1073, 1083, 1084): a line that their words
  are the hired writer's, with this ticket's number.
- When Nick has picked: `docs/design/voices-languages-and-script.md`
  gains a dated line (who was picked is recorded as "a writer" unless
  Nick wants the name in the repository), and the completion notes say
  what happens to the pause and to 0757, 0719, 0726, 0755 and 0756.

**Out (do not do):**
- Any writing of scenes (the pause; the trial is the applicants').
- Committing applicants' entries, names or contact details.
- Posting the ad or messaging anyone: Nick does (it is posted from his
  account and in his name).
- Marks inside the program or limits on capture (engine 3083, parked).
- A public demo build (engine#3082 and a game ticket, if Nick wants one).
- Giving applicants any repository access.

## Implementation steps

1. Size first (rule 9): about 1,100 changed lines (the pages 350,
   `trial-packet` 150 and its tests 80, `trial-compare` 120 and its
   tests 60, the other tickets' lines 30, this ticket 350).
2. Ask the five questions. Record the answers in the completion notes
   (Nick's words quoted).
3. The pages, the two commands, the files-list line, the waiting
   tickets' lines. Build one packet for a made-up handle and open it on
   Windows: the preview starts on its folder and plays both scenes with
   no problem.
4. Open the PR as a draft and keep the ticket `in-progress`: Nick posts
   the ad and sends packets (this can take weeks). A later session
   resumes on the same branch for the comparison.
5. When entries are in: `trial-compare`, the page to Nick, his pick (and
   a second round if he wants one); then the design note's line, the
   completion notes, and the PR made ready.

## Acceptance criteria

- [ ] `cargo xtask trial-packet --for test` makes a zip whose `.dlg`
      holds exactly the chosen scenes and the mark, and which
      `story-check` passes (test).
- [ ] `cargo xtask trial-compare` on two fixture entries writes a page
      with Entry A and Entry B side by side and no handle in it (test).
- [ ] `docs/writer-trial/ad.md` names no repository and states the
      compensation Nick chose.
- [ ] No applicant's entry, name or address is in the PR's diff.
- [ ] Nick's pick (or "nobody yet") is in the completion notes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: scene extraction by id; the mark; the random lettering keeps no
  handle on the page.
- Integration: a packet built from a fake kit; the comparison page over
  fixture entries.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
