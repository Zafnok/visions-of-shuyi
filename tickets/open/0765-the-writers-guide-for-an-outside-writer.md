---
id: "0765"
title: The writer's guide for an outside writer
type: infra
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0152", "0764"]
nick_input: none
completed:
---

# 0765 — The writer's guide for an outside writer

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

This ticket is the words of "an easy to use scriptwriting kit ... that a
nontechnical person can use": the guide, a one-page cheat sheet, and the
start page of the writer's repository. The rest of the kit: 0152 (the
mirror repository `Zafnok/visions-of-shuyi-story`, its checker
`story-check`, its `kit` release), 0764 (the preview program
`story-preview`), 0766 (the ad and the trial packet, which reuses these
pages).

`docs/story/writers-guide.md` (578 lines, ticket 0723) was written for
someone working **inside this repository**, and it shows:

- §10 *Setting up* has the writer install Rust and
  `git clone https://github.com/Zafnok/visions-of-shuyi`, which a writer
  won't use: they work in the mirror (0152), and the scripts may leave
  this repository (0153's research); §8 and §9 check and watch scenes with `cargo xtask
  check-script` and `cargo run -p trpg-app -- --scene`.
- "Claude maintains" the wiring, and "ask Claude" for a new character
  or name (§2, §4, §5): an outside writer can't ask Claude.
- §6 *What is fixed* calls Nick's beats canon that only Nick changes.
  Nick, 2026-11-08: a writer "may look at our arcs and beats and decide
  they like them or hate them and throw them out entirely if they can't
  work with them for making compelling dialogue."
- §7 says "Dialogue isn't translated or voiced yet". Since then Chapter
  1 and Vix are voiced (`docs/voice.md`; ticket 0762's count,
  2026-11-07: 1,059 voiced lines in the scripts).
- Nothing tells the writer that the scripts there now are AI drafts
  Nick finds robotic, or that they may keep or discard any line.

`docs/story/README.md` *For a human writer* lists what to read, and
0723's completion notes list six checker rules a human may find wrong
(typewriter punctuation, the length limits, three replies and four
reaction lines, capitalized words that are also names, `{They} knows`,
a new name needing data first).

Nick's words name what the writer must find quickly: "tone (for AI), var
names, flags, and emotion (for the portrait)". In the format those are
directions and `@delivery`; name tokens and the lead's tokens; **no
flags** (only `@if <character>`); expressions. That is the cheat sheet.

## Nick input

None. The guide states what Nick already said (the beats and arcs are
open to the writer's proposal; Nick decides) and the rules that exist
today; it decides nothing new. Questions a writer raises go to Nick
through the mirror's issues once one is hired (0766).

## Scope

**In:**
- **`docs/story/writers-guide.md` rewritten** for a writer who has the
  mirror and the kit, and no Rust, no source, no Claude:
  - *Your copy*: the mirror's files (`dialogue/`, `story/`, `design/`,
    0152), what each is; the kit from the mirror's **Releases** page.
  - *Working*: two ways for someone new to Git. **GitHub Desktop**
    (clone, a branch per piece of work, commit, push, "Create pull
    request"), with a plain text editor (VS Code, Notepad++, BBEdit, or
    TextEdit in plain-text mode, as today's §1 says). Or, for a few
    lines, **github.com's own editor** (the pencil on a file, "Propose
    changes", which makes the branch and the pull request). Steps for
    both, Windows and macOS, with what each button is called.
  - *Watching your scene*: `story-preview` (0764): start it, pick a
    scene, flip who is there, switch the lead, jump to a line, show the
    directions, save and see it change, read the problems panel.
  - *Checking*: the preview's problems panel, and the check on the pull
    request (a red mark with the line). "Finished" is no problem in
    either.
  - *How a change gets into the game*: a pull request in the mirror;
    Nick approves it; it is taken into the game and the pull request
    closes with a note (0152's import). Never push to `main`.
  - *Asking*: an issue in the mirror for a new character or speaker, a
    new name, a scene to be played at a new moment, a rule in the way.
    No "ask Claude" anywhere; "the developer" where someone acts.
  - *What is open and what is fixed*, replacing §6:
    - **Open to your proposal, Nick decides:** the beats
      (`story/beats.md`), the cast and their arcs (`story/characters/`),
      the world (`story/bible.md`), the outline and its twists
      (`story/outline.md`), names. Say it with Nick's words above. A
      proposal is a pull request or an issue; nothing is thrown out
      until Nick says so.
    - **Yours:** every line, the scenes inside a chapter
      (`story/chapters/`), expressions, directions, captions, music
      moods, where reply choices fall and what they say.
    - **The scripts there now are drafts written by AI.** Nick finds
      their manner of speaking robotic. Keep a line, or don't; don't
      learn the game's voice from them.
    - **Set by the game's design** (`design/`): the lead's rules
      (`setting-and-tone.md`, *Rules for writing the lead*), who can die
      and the `@if` blocks that follows from (`death-and-difficulty.md`),
      each chapter's battles. Proposals welcome; Nick decides.
  - *What a rewrite costs* (today's §7, brought up to date): line ids;
    Chapter 1 and Vix have voice clips, remade by the developer for
    each changed line; no translations yet, but each changed line will
    cost one when there are; a direction change keeps the id and costs
    only the clip; renaming a scene changes every line's id; ask before
    renaming or removing a scene, because the game names some.
  - Keep: §1–§4's teaching of the format with its tested ` ```dlg `
    examples (reworded where they say "Claude"), the American English
    rule, the jokes rule.
  - Links that work in both places: `docs/story/` and `docs/design/`
    sit side by side as the mirror's `story/` and `design/` do, so
    `../design/x.md` works in both; the format reference doesn't
    (`../../assets/dialogue/README.md` here, `../dialogue/README.md`
    there), so name it in words ("`dialogue/README.md` in your copy"),
    and do the same for any other path outside the two folders.
- **`docs/story/cheat-sheet.md`**, one printed page, mirrored as
  `story/cheat-sheet.md`, each item one line and one example:
  - Tone for the voice: `name (direction): text`, `> (direction): text`,
    `@delivery … / @delivery clear`; the directions the voice tool was
    tested on (from today's guide).
  - Emotion for the portrait: `name[angry]: text`, `@left who
    expression`; the five every character has; where to see a
    character's others (the preview's list).
  - Names ("var names"): `{n:id}`, `{N:id}`, never a typed name; the
    lead: `{lead|the exile}`, `{they}` `{them}` `{their}` `{theirs}`
    `{themself}`, and `{They} knows`.
  - Conditions ("flags"): **there are no flags or variables.** Only
    `@if <character>` / `@else` / `@endif` (still in the army, or on the
    map in a battle). Choices set nothing and every reply rejoins.
  - Choices: `@choice`, `* tone: reply`, two-space reactions,
    `@endchoice`.
  - Stage: `@caption`, `@place`, `@music` (the mood cues), `@left/@right
    clear`.
  - Limits: 200 characters a text box, 60 a reply, 40 a `lead:` line,
    3 replies, 4 lines a reaction, two people on screen, plain ASCII
    (`'` `"` `...` `--`); a name token counts as the longest name.
  - The six rules from 0723's notes that may get in the way, and "open
    an issue if one does".
  - What a change costs, in two lines.
- **`tools/story-mirror/README.md`** (0152's stub, the mirror's start
  page): what this is, in three sentences; read in this order (the
  cheat sheet, the guide, then `story/README.md`'s list); get the kit;
  propose a change; ask in an issue; the files are confidential (0766's
  terms, linked when they exist).
- **`docs/story/README.md`** *For a human writer*: the new order, the
  cheat sheet first; `craft.md` described as notes on why earlier AI
  drafts failed, read last if at all.
- **`assets/dialogue/README.md`**'s opening lines: the preview program
  and `story-check` beside the `cargo` commands.
- The test that checks the guide's ` ```dlg ` examples
  (`crates/xtask/src/check_script.rs:341` today; wherever 0152 moved it)
  checks the cheat sheet's too; `.github/workflows/changes.yml`'s
  *read by* list (`:62`) names the cheat sheet beside the guide.

**Out (do not do):**
- Any change to a script, a beat, a character sheet or the outline.
- New format features (flags, variables): none exist, and adding one is
  a design question for Nick, not this ticket.
- The trial packet's own pages and the ad (0766).
- Screenshots of the programs in the guide (they'd go stale); the
  preview's own help line names its keys.
- Translating the guide.

## Implementation steps

1. Size first (rule 9): about 1,000 changed lines (the guide's rewrite
   about 400, the cheat sheet 150, the mirror README 80, the two
   READMEs 40, the test 30, this ticket 300).
2. Read 0152's and 0764's completion notes for the exact names: the
   repository, the release, the programs' file names, the buttons the
   preview shows.
3. Rewrite the guide, then the cheat sheet; keep every ` ```dlg `
   example a complete scene that passes the checker.
4. The READMEs and the test.
5. Read the guide and cheat sheet as someone who has never used Git:
   every step names the button or menu as the program shows it. Use a
   subagent that knows nothing of the repository to follow them and say
   where it got lost; fix those places.

## Acceptance criteria

- [ ] `docs/story/writers-guide.md`, `docs/story/cheat-sheet.md` and
      `tools/story-mirror/README.md` don't contain `cargo`, `rustup`,
      `Claude` or a link to the game's repository
      (`github.com/Zafnok/visions-of-shuyi` not followed by `-story`)
      (a check in the test).
- [ ] Every ` ```dlg ` example in the guide and the cheat sheet passes
      the checker (the test).
- [ ] The cheat sheet says in words that there are no flags or
      variables, and shows `@if`/`@else`/`@endif`.
- [ ] The guide says the beats, arcs and outline are open to the
      writer's proposal and that Nick decides.
- [ ] After the next sync, the mirror shows the new README and
      `story/cheat-sheet.md` (checked on GitHub, noted).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the examples test over both files; the grep of the
  forbidden words.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
