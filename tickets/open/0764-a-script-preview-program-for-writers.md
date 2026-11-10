---
id: "0764"
title: A script preview program for writers
type: feature
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0152", "engine#2586"]
nick_input: none
completed:
---

# 0764 — A script preview program for writers

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

This ticket is "an easy playtesting process for them to see how their
script renders". The kit's other parts: 0152 (the mirror repository a
writer works in, the checker `story-check`, the `kit` release), 0765
(the guide and the cheat sheet), 0766 (the ad and the trial packet).

**What a writer has today** (ticket 0723):

- `cargo run -p trpg-app -- --scene <id> [--lead f]`
  (`crates/app/src/args.rs:10`, `crates/ui/src/debug/scene_preview.rs`,
  `ScenePreviewScreen` `~:38`): plays one scene in the real dialogue
  screen. It needs Rust and the source, which a writer won't have; it
  rebuilds after every edit, because the scripts are embedded; it plays
  with everyone there (`Present::Everyone`, `~:57`), so an `@else` is
  never seen; it starts at the first line; it shows the real faces only
  with `--features private-assets`.
- `cargo xtask check-script`, which 0152 turns into the program
  `story-check` that a writer can run.
- Nothing else: the debug menu (F2, `crates/ui/src/debug.rs`) can't jump
  to a chapter, a battle or a scene.

**What Nick's words map to in the `.dlg` format** (reference:
`assets/dialogue/README.md`):

| Nick asked for | What exists |
| -------------- | ----------- |
| tone (for AI) | Directions, `name (out of breath): text`, and `@delivery`. The player never sees them; the voice tool reads them |
| var names | Name tokens `{n:<id>}` / `{N:<id>}` (`assets/data/names.ron`), the lead's `{lead\|the exile}` and pronoun tokens `{they}` … |
| flags | **None.** The only condition is `@if <character>` / `@else` / `@endif`: whether that character is still in the army or on the map. No `@set`, no variables; reply choices set nothing and all rejoin |
| emotion (for the portrait) | Expressions, `name[angry]: text`, and `@left <who> <expression>` |
| skip ahead | Nothing yet |
| trigger event | A battle's triggers play scenes (`assets/battles/*.ron`); a scene a battle plays is drawn over that battle's map, which `--scene` already does (`with_battle`) |

So the program shows those, says plainly that there are no flags, and
makes the `@if` conditions the switches a writer flips.

**The engine half** is engine#2585 and engine#2586: a dialogue preview
any game on Shuyi Engine builds (a list of the scenes read from a folder
on disk, playing one in the game's own text box and faces, switches for
the conditions a scene tests, a jump to any line, a reload when the file
is saved, and the problems shown in the window). 2585 is done and split
(its completion notes): it gave the pieces (`Scene::conditions`,
`ParsedScene::text_boxes` and `text_box_at`,
`DialoguePlayer::from_text_box` and `direction`, the watched folder,
`shuyi_app::read_watched_folder`); **engine#2586** builds the preview
screen itself, so this ticket waits for it. This ticket is the game's half: its
dialogue screen, its checks, its names and lead, and a program a writer
downloads.

**Either order with 1209:** if 1209 is done, read the scripts folder
through the disk path it gives the debug build, if it fits a program
that is not a debug build; if not, nothing to do here: the preview reads
its folder through engine#2585's watched folder, and 1209 may reuse the
preview's code when it lands.

**Either order with engine#3082** (demo editions: a program built with
only some content): if it is done and pinned, build the preview as an
edition whose content root leaves out `dialogue/`; if not, leave the
scripts out with a `trpg-content` feature (step 3).

## Nick input

None. Verification is the Harness tests and `cargo shuyi frame-png`
pictures of the program's screens in the PR. Nothing here is a game
rule; how a writer uses it is judged by the writer in 0766's trial.

## Scope

**In:**
- **`story-preview`**, a program built from this repository (a second
  binary of `trpg-app`, or of 0152's `trpg-writer` if that is simpler
  for the platform shell: your call, say which in the notes). It:
  - reads the `.dlg` files of a folder: the one given on the command
    line, else a `dialogue/` folder beside the program, else asks for
    nothing and shows how to start it (a writer's clone of the mirror
    has `dialogue/` at its root);
  - lists the scenes by file, in file order;
  - plays the picked scene in the game's own `DialogueScreen`
    (`crates/ui/src/screens/dialogue.rs:179`, `new` `:221`), with the
    bought faces and the game's music, over its place (`@place`, and a
    battle's map for a scene a battle plays, as `ScenePreviewScreen::play`
    does);
  - has a switch for each character the scene's `@if` blocks test
    (engine#2585's list of conditions), all on at first; flipping one
    replays the scene from the same line with that character gone, so
    the `@else` plays;
  - has a switch for the lead: male or female (the pronoun tokens), and
    the default name or a 12-letter name, the longest a player can type,
    to see the longest text box;
  - jumps to any text box: a list of the scene's lines (speaker and the
    start of the words, with the file's line number), and a jump to the
    line the writer types;
  - shows a line's direction (the tone) under the text box when the
    writer turns that on (the player never sees it), and the expression
    each face is showing;
  - reloads a file when it is saved, keeping the scene and the line
    where it was when it still exists;
  - shows every problem in the window (file, line, message, the fix
    where the checks know it) instead of playing a broken scene, and
    keeps the last good version playing until it is fixed;
  - lists the names (every `{n:<id>}` with today's name) and each
    character's id and expressions, so a writer can look one up without
    the repository.
- **Checks:** the same rules as `story-check`. With every scene the game
  names present (a clone of the mirror), every rule. With only some (a
  trial packet: 0766 sends two scenes), only the scenes in the folder:
  a chapter or battle naming a scene that isn't there is not a problem
  here, and the window says "Only the scenes in this folder are
  checked". This needs a content load that accepts missing scenes
  (`trpg_content::load_with_scripts`, `crates/content/src/lib.rs:271`,
  fails on them today), for the preview only.
- **No script inside the program:** the preview's build embeds no
  `.dlg` file, so a program sent to a trial applicant holds no scene but
  the ones in its folder. (See the clause with engine#3082.)
- **Packaging:** 0152's `story-kit.yml` builds it for Windows with the
  bought art (the `private-assets` action and `PRIVATE_ASSETS_KEY`, as
  `release.yml` does; the art is inside the program, which ADR-0040
  allows for a build), zips it with `music/`, `NOTICES.txt` and a short
  `README.txt` (how to start it: double-click, or drag the `dialogue`
  folder onto it), and uploads it to the mirror's `kit` release. macOS
  only when the run asks for it (macOS minutes cost ten times).
- Controls by action, never a key (`keyboard-input` skill): the
  preview's controls are the game's existing actions (the cursor,
  Confirm, Cancel, Options, PrevUnit/NextUnit in
  `crates/content/src/keymap.rs`), mapped onto engine#2586's, with a
  help line naming the player's keys. No new default key: the defaults
  are Nick's (`docs/design/controls.md`). If the screens can't be driven
  with those, say so in the notes rather than add one.

**Out (do not do):**
- Playing a scene inside a running battle at its trigger's moment, or
  starting the game at a chapter, battle or scene from the debug menu.
  The preview draws a trigger's scene over its battle's map, as the game
  does. If a writer asks for more, write a ticket then.
- A web build (it needs files dropped on the page, engine#3092) and a
  Linux build.
- Voice clips (Nick's machine only, `docs/voice.md`).
- Changing `--scene` or `ScenePreviewScreen`; they stay the developers'
  tool.
- Any change to a script's words (the writing pause, 0766).
- The guide's pages about the program (0765).

## Implementation steps

1. Read engine#2585's and engine#2586's completion notes (the pieces,
   the trait, the screen and the folder watch), move the engine pin past
   2586 if it isn't yet (`engine-pin` skill), then read ADR-0040, ADR-0054, ADR-0057, `scene_preview.rs` and
   `game.rs`'s `start_on_scene` (`crates/ui/src/game.rs:90`).
2. Write the size estimate first (rule 9). **Expected size: about 400
   production lines**: the game's side of engine#2586's preview (about
   150: build a `DialogueScreen` for a scene, who is there, the lead and
   a first line), the lenient content load (about 80), the build without
   scripts (about 40 in `crates/content/build.rs`), the binary's start
   and arguments (about 60), the names and expressions list (about 70).
   Judge it by `scene_preview.rs` (204 lines with its tests). At 3× run
   `engine-friction`.
3. The build without scripts: if engine#3082 is done, an edition;
   if not, a `trpg-content` feature `no-embedded-scripts` that leaves
   `dialogue/` out of the embedded table the way
   `crates/content/playdate-bundle.txt` leaves files out, turned on only
   by the kit build and never by a gate (as `private-assets`).
4. The lenient load, then the program, then its screens' views and
   skins (ADR-0054).
5. The kit workflow's new job; run it by hand; download the zip on
   Windows and start it on the mirror's `dialogue/` (completion notes).
6. `cargo shuyi frame-png` pictures for the PR: the scene list, a scene
   playing with a direction shown, a switch off and its `@else` line,
   the problem list.

## Acceptance criteria

- [ ] From a temporary folder holding one `.dlg` file, the scene list
      shows its scenes (Harness test reading the view).
- [ ] With a condition switched off, the scene plays its `@else` line
      (Harness test).
- [ ] A jump to line N shows that text box with the portraits, caption
      and place the lines before it set (Harness test).
- [ ] A saved change shows on the next frame; a broken file shows its
      problem with file and line and keeps the old scene playing
      (Harness tests through the headless backend's `change_file`).
- [ ] With only `ch01_intro` and `ch01_prebattle` in the folder, the
      preview plays both and reports no problem about other scenes
      (test).
- [ ] The preview's build embeds no `dialogue/` path (a test of its
      bundle), and the built program contains none of three chosen
      script lines (a step in the kit job).
- [ ] The kit release holds the Windows zip (link in the completion
      notes).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the lenient load's dropped problems; the argument parsing.
- Integration: the Harness tests above; the bundle test of the build
  without scripts.
- Pictures: the `frame-png` set in the PR.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
