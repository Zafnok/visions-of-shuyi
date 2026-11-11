---
id: "0914"
title: A Chapter 1 web demo, built without the later chapters
type: feature
milestone: M8 Release
model: opus-5.5 / gpt-6.1-sol
effort: high
status: blocked
blocked_by: ["engine#3082", "0872"]
nick_input: none
completed:
---

# 0914 — A Chapter 1 web demo, built without the later chapters

## Context

Nick, 2026-11-10, on hiring a writer (Lane J in `docs/ROADMAP.md`;
0152, 0153, 0764–0766):

> specifically I want a web demo build that doesn't leak anything [...]
> I want to ship just ch1 to them (can also include the title screen,
> options, and opening cutscene)

Every web build today holds the **whole game**. `crates/content/build.rs`
embeds every file under `assets/` (`bundle_files.rs`), so all eight
chapters' scripts (`assets/dialogue/ch01.dlg`–`ch08.dlg`, the side
quests), their maps, battles and the world map are inside the Wasm as
plain text, readable with `strings`. The GitHub Pages build adds the
debug tools (`pages.yml`: `cargo shuyi web --release --debug-tools`). A
menu that hides Chapter 2 doesn't help: hidden is still shipped
(engine#3082's context).

What exists to build on:

- **engine#3082** (demo editions): a named edition in `shuyi.ron` picks
  the package, its Cargo features and a **content root**, and the export
  records the edition in `exposure-report.json`. It isn't done; this
  ticket waits for it.
- **The disclosure gate** (engine 3081; the game's part from 1208,
  `shuyi.ron`'s `disclosure`: `allowed_files`, `forbidden_paths`,
  `canaries`): an export whose package holds a canary is refused with
  `SH3012` and removed.
- **Release exports refuse debug features and strip symbols** (engine
  3099).
- **A bundle that leaves files out on purpose** already exists: the
  Playdate's (`crates/content/playdate-bundle.txt`, ticket 1036,
  ADR-0082). `build.rs` leaves the listed files out and tells the bundle
  which (`bundle::left_out`), so a loader tells "left out" from
  "missing", and a test plays Chapter 1 against it
  (`crates/playdate/src/chapter_1.rs`).
- What Nick listed: the title screen (`crates/ui/src/screens/title.rs`),
  the opening cinematic (`assets/cinematics/title.ron`, ticket 0820),
  the options (`crates/ui/src/screens/options.rs`) and Chapter 1
  (`assets/chapters/ch01.ron`: `ch01_intro`, `ch01_prebattle`, the
  battle `ch01`, then `ch01_victory` and `ch01_tbc`, then `next:
  Some("ch02")`).

**This holds the demo, not the repository.** 0153 picked moving the
story and the game's content into the private assets repository (0156,
0159, 0160); until 0160 lands, anyone who finds the repository reads every
chapter, and the history before the move stays public after it. The demo makes sure a *player* of the demo, and anyone who
pulls its files apart, gets Chapter 1 and nothing later.

**Either order with 1249** (the web package within 25 MB): if 1249 is
done, use what it built; if not, the demo ships only the music its files
name, which should bring it under the budget by itself (the 46 tracks
are 75 MB of the 76 MB).

**Either order with 0160:** once 0160 moves the content into the private
assets repository, the demo's content comes from there (the
`private-content` feature, 0156) like the bought art, and a demo export
without the key has the test game, not Chapter 1, so it must fail
(`required: true`). The edition's list of what is left out stays the
same.

**Waits for 0872** (save export; Nick, 2026-11-11, below): the demo
ends on "Export save?", which hands the player a file through 0872's
export, so 0872 must be done first. 0872 itself waits for engine#3309
(Zafnok/shuyi-engine PR #359 was open on 2026-11-11) and for Nick's
answers on where Export lives in the full game.

## Nick input

**Decided 2026-11-11** (the session that first picked this ticket up),
recorded in [`docs/design/demo.md`](../../docs/design/demo.md) with
Nick's words:

1. **After winning Chapter 1:** all of Chapter 1's scenes, then **no**
   "Save your progress?" ("given saving the game does nothing to a
   chapter specified demo, I would prefer only having Export option on
   demos, NOT the save option on demos"), then an end card **"Thanks for
   playing, {the lead's name}"**, then **"Export save?" Yes / No**, which
   works through 0872's export and makes a file the full game imports
   (0873) and continues into Chapter 2 from. Then the title.
2. **`ch01_tbc` stays** ("if it's a ch1 ending scene and not a ch2 intro
   scene we can keep it"): it is in `ch01.ron`'s `victory_scenes` and
   `ch01.dlg`, so it is Chapter 1's ending.

`docs/design/demo.md` also lists the agent's starting rules for the end
(the question's cursor on *Yes*, the card's look borrowed from "To be
continued").

Nothing else here is Nick's: which files leave and how is technical.
Verification is the tests and the export's own check, and a
`cargo shuyi frame-png` picture of the end in the PR. What is left to
judge by playing goes on the Act 1 playtest's checklist (1064) only if
Nick asks for it.

## Scope

**In:**
- **The edition**: a `demo` edition in `shuyi.ron` (engine#3082's
  format) for the web target, and for Windows if the edition costs
  nothing more there: the title, the opening cinematic, the options, the
  profile and credits screens, and Chapter 1 with everything it reads
  (its scenes, battle, map, tips and lessons, voices, music, the bought
  art it shows).
- **What it leaves out**, as a list in the repository the way
  `playdate-bundle.txt` is (or the edition's content root, whichever
  engine#3082 gives): Chapters 2–8's scripts, chapters, battles and maps,
  the side quests, the skirmishes, the world map, and the test files
  (`assets-test/`, already left out of every shipping build). Files that
  every chapter shares (`assets/data/characters.ron`, `names.ron`,
  `supports.ron`, `camp.ron`, the English text in `assets/lang/en/`)
  are read for what they name past Chapter 1: what a player of the demo
  never meets is cut from the demo's copy at build time or the file is
  split, the session's call, recorded in an ADR. The demo must not need
  a second hand-kept copy of any data file.
- **The flow** (`docs/design/demo.md`), in the demo only: after
  Chapter 1's last scene (`ch01_tbc`), no "Save your progress?" (0869's
  prompt stays in the full game); the end card "Thanks for playing,
  {lead}" (a new text key; the "To be continued" card's look, in
  silence); then "Export save?" Yes / No. *Yes* writes the campaign as
  the full game's chapter save after Chapter 1 would hold it into a
  bundle through 0872's export (the demo has no save of its own to
  export) and offers the file; then the title. The title shows no entry
  that reaches a later chapter, the skirmishes or the debug tools
  (`Item::QuickBattle` is already debug-only). The battle's suspend save
  the demo makes loads in the demo.
- **Importable into the full game:** the exported file holds a save the
  full game's import (0873) takes, and loading it there continues into
  Chapter 2. The demo's build must not change the save format, and the
  file names no Chapter 2 content (the full game's own `ch01.ron` names
  `next`).
- **No debug tools**: the demo is a release export (engine 3099 refuses
  debug features). Since the pin move of 2026-11-10 the refusal reads the
  build Cargo resolves (engine #322 and #344): a debug feature, or the
  live link, reached through a default or a dependency is refused too,
  so the demo edition's features need no list of what to keep off.
- **Its notices**: `NOTICES.txt` lists the crates of the build as Cargo
  resolves it (engine #348), so a demo built with
  `default_features: false` names no crate only the full game's default
  features bring, and a canary can't trip on them.
- **The disclosure part for the demo**: canaries that are found in the
  full game and must not be in the demo. Use ids and file names (a
  Chapter 2 scene id, `ch02.map`, a character id first met in Chapter
  3), not script lines: `shuyi.ron` is public, and once 0160 moves the
  scripts private their lines must not sit in a public file (0157's rule).
- **CI**: a job that exports the demo edition for the web without the
  bought art on PRs that change `assets/`, `crates/content/` or
  `shuyi.ron`, so a later change that pulls Chapter 2 into a shared file
  fails `SH3012` on that PR rather than on the day of a demo. A
  `workflow_dispatch` input on `release.yml` (or a small `demo.yml`)
  builds it with the bought art and uploads the package as an artifact;
  0915 publishes it.
- An ADR: what the demo holds, how the shared files are cut, and why the
  edition and not a menu lock.

**Out (do not do):**
- Hosting or publishing the demo (0915), or changing GitHub Pages (0158
  decides what Pages shows and moves Nick's full web build to a
  restricted itch.io page).
- Any change to a script's words (the writing pause, 0766). Changing
  `ch01.ron`'s `next` for the demo only is wiring, not writing.
- Moving the scripts or content private (0156, 0159, 0160).
- Marks, watermarks or capture limits (engine 3083, parked).
- A pack of baked files instead of embedded text (engine 3300): it hides
  nothing, by its own ticket's word.
- A Playdate demo.

## Implementation steps

1. ~~Move the engine pin past engine#3082 (`engine-pin` skill, its review
   and ledger section).~~ (done by the pin move of 2026-11-10): engine #346, with the bake's
   content root (#350) and an edition's notices (#348). Read 3082's completion notes, ADR-0082 and
   `crates/content/src/bundle.rs`'s `left_out`.
2. Size first (rule 9): **about 1,200 changed lines**: the edition and
   left-out list 80, the shared files' cut at build time 250 (judge by
   `build.rs`'s Playdate table, about that size), the demo's flow end
   (the card, the export question, no save prompt) and title entries
   250 (judge by `title.rs`'s debug-only `QuickBattle` and
   `screens/save_prompt.rs`),
   the disclosure part 60, the CI job 60, tests 250, the ADR 120, this
   ticket 300. At 3× the production estimate, run `engine-friction`.
3. ~~Ask Nick the two questions.~~ (done 2026-11-11:
   `docs/design/demo.md`.) Read 0872's completion notes: its export
   function, the bundle's keys and file name.
4. Export the full web build and read its `exposure-report.json` and
   `strings` for what names a later chapter; write the left-out list and
   the canaries from that.
5. The edition, the cut of the shared files, the flow end, the title.
6. The CI job; run the demo export by hand with the bought art and play
   it in the browser pane from New Game to the end (agents' memory:
   *web build testing in browser pane*).
7. The ADR and its index line.

## Acceptance criteria

- [ ] `cargo shuyi export web --release` for the demo edition passes the
      disclosure gate, and its `exposure-report.json` names the edition.
- [ ] A canary planted in a Chapter 2 file fails the demo's export with
      `SH3012` and passes the full game's (test or recorded run).
- [ ] `strings` on the demo's Wasm finds no Chapter 2–8 scene id, map
      name or the ids of characters first met after Chapter 1 (a CI step).
- [ ] A Harness test plays the demo from New Game through Chapter 1's
      win: both victory scenes, no "Save your progress?", "Thanks for
      playing, {lead}", "Export save?"; *Yes* offers one file and returns
      to the title, *No* returns to the title with none. The title shows
      no entry past Chapter 1.
- [ ] A test: the demo's exported file, imported into the full game's
      content (0873's import, or its checker when the screen isn't
      needed), gives a save that loads and starts Chapter 2.
- [ ] The full game still asks "Save your progress?" after Chapter 1.
- [ ] The full game's build and tests are unchanged.
- [ ] The CI job runs on a PR that touches `assets/`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the left-out list's reading; the shared files' cut drops what
  names a later chapter and keeps what Chapter 1 reads.
- Integration: the Harness play-through of the demo; the canary test;
  the demo export in CI.
- Pictures: `cargo shuyi frame-png` of the title, the end card and the
  "Export save?" question in the PR.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
