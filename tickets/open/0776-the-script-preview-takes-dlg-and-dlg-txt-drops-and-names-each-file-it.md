---
id: "0776"
title: "The script preview takes .DLG and .dlg.txt drops and names each file it leaves out"
type: bug
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0776 — The script preview takes .DLG and .dlg.txt drops and names each file it leaves out

## Context

The writer trial (0766) sends about five applicants a web script preview
(0767, a Restricted itch.io page). They drag `dialogue/ch01.dlg` onto it,
edit, save and drag it again. A test audit of the demo before the packets
go out found that the preview **quietly ignores some files and doubles
others**:

- Shuyi Engine's `drop_scenes` (`crates/ui/src/dialogue/preview/folder.rs`
  at the pin, line 80) keeps a dropped file only if
  `name.ends_with(".dlg")`. That check is case-sensitive and matches the
  exact suffix. So `CH01.DLG`, and `ch01.dlg.txt` (from an editor or a
  Windows setting that adds `.txt`), are left out.
- The game's hook `trpg_ui::story_preview::hear`
  (`crates/ui/src/story_preview.rs`, about line 516) throws away
  `drop_scenes`' return value (the names it put in the folder). Nothing
  on screen says a file was left out: the writer sees their old scene
  play and thinks the change didn't work.
- `drop_scenes` replaces a file of the **same name**
  (`layer.replace(&name, …)`), so a browser re-download named
  `ch01 (1).dlg` joins the folder beside `ch01.dlg`. Its scenes are
  listed twice, and the Problems tab reports duplicate scene ids.

On the 0766 branch, `docs/writer-trial/brief.md` (step 2 of "See your
script") and `docs/writer-trial/start-here.txt` (step 3) work around this
in prose ("Keep the name `ch01.dlg`", "one that ends in `.txt` is skipped
without a message"). An applicant who misses that line is lost, with no
one to ask in the moment.

This doesn't block the writer trial (0766): the packet already warns
writers ("Keep the name `ch01.dlg`"). If this lands first, step 6
shortens that wording.

This is fixed game-side, in the game's own drop hook, before it calls
`drop_scenes`. The engine is not changed and the game does not wait on it
(AGENTS.md rule 9, ADR-0092). Builds on 0764 (the preview), 0767 (the web
drop) and ADR-0054 (a screen's view is plain data, painted by a skin).

## Nick input

None. This is the writer's tool, not the game: what it says about a
dropped file is technical, as 0767's "No scenes. Drop your .dlg files
here" was. Verified by Harness tests and `cargo shuyi frame-png` pictures
in the PR. No check after merge.

## Scope

**In:**
- In `hear`, before calling `drop_scenes`, rename each dropped file to
  the name the writer meant:
  - a name whose extension is `.dlg` in any case (`CH01.DLG`,
    `ch01.Dlg`) → the same stem with `.dlg`;
  - a name ending in `.dlg.txt` (any case) → the name without `.txt`;
  - a name of the form `<stem> (<digits>).dlg` (a browser's re-download
    copy, after the two rules above) → `<stem>.dlg`, so it **replaces**
    the file it was copied from instead of joining beside it.
- A note in the preview's view, for the last drop, that names each file
  it changed or left out and why, in plain words. For example (final
  wording is the implementer's): "ch01 (1).dlg played as ch01.dlg";
  "notes.txt left out: only .dlg files play". The next drop replaces the
  note. A drop that took every file as named, with no renaming, shows no
  note.
- The new text keys in `assets/lang/en/ui.ron` and the test pack
  `assets-test/lang/test/ui.ron`, following `preview.note.no_scenes_drop`.
- The writer-trial docs, either order (step 6).

**Out (do not do):**
- Any change to Shuyi Engine (`drop_scenes`, `DialogueFiles::read`). The
  desktop program also reads its **folder** through the engine's
  `DialogueFiles::read`, which keeps the same lowercase-`.dlg` rule. That
  is not part of the trial (the applicants use the web page). If it
  matters later, it goes through the `engine-friction` / `engine-change`
  skills, and only with Nick's OK on an engine step.
- A drop on the itch.io page **around** the game's frame. The engine's
  web page catches drops on its own window only, and itch.io's page is
  outside the game. START-HERE and brief.md already say to drop onto the
  game itself.
- Guessing at other names (`ch01 v2.dlg`, `ch01-copy.dlg`): those stay
  separate files, as today. Duplicate scene ids are already reported on
  the Problems tab.
- A scene id that was renamed inside the file (the sibling ticket 0777,
  "The script preview says when a folder's scene is one the game never
  plays").
- A dropped folder or an unreadable file (the engine's
  `shuyi_platform::dropped::accept` handles those).

## Implementation steps

1. Read `crates/ui/src/story_preview.rs` (`hear`, `StoryPreviewView`,
   `paint`, `view`) and the engine's `drop_scenes` at the pin
   (`shuyi-engine.rev`; `~/.cargo/git/checkouts/shuyi-engine-*/<rev>/crates/ui/src/dialogue/preview/folder.rs`).
2. Add a pure function in `story_preview.rs`, for example
   `fn meant_name(name: &str) -> Meant`. It gives either the name to put
   in the folder (with a flag when it differs from `name`) or "left out"
   (not a `.dlg` after the rules in Scope). Unit-test it on its own.
3. In `hear`: map each dropped file through `meant_name`, hand the kept
   ones (renamed) to `drop_scenes`, and record the drop's note, i.e. the
   changed and left-out names. Store it where the screen can read it: a
   field of the preview's game state in `crates/ui/src/story_preview.rs`,
   next to what `hear` already reaches through its `game` (as
   `drop_scenes` reaches the folder): for example beside `ShuyiPreview`'s
   `not_here` count, which the view already reads, or on `StoryPreview`.
   Pick the lightest way the API allows and say which in the completion
   notes. Cross-check against `drop_scenes`' returned
   names, so a file the engine still refuses is reported as left out,
   never silently dropped.
4. Add `dropped: Option<String>` to `StoryPreviewView`, filled from that
   state with the new text keys (`ctx.text_with`). Paint it on the tabs'
   second row, where `only_here` goes, in its place while it is shown
   (ADR-0054: `view` decides, `paint` draws).
5. Add the text keys to `assets/lang/en/ui.ron` and
   `assets-test/lang/test/ui.ron` (e.g. `preview.note.dropped_as`,
   `preview.note.dropped_left_out`), American English, with no key name
   in them (keyboard-input skill: the drop is a mouse action, so no key
   is named).
6. Docs, either order. If `docs/writer-trial/` is on `main` (0766
   merged), update `docs/writer-trial/brief.md` ("Keep the name
   `ch01.dlg`…") and `docs/writer-trial/start-here.txt` step 3 to say
   what the preview now does: keep the advice to save over the same
   file, and drop the "skipped without a message" and "plays beside the
   first" sentences. If it isn't, leave those docs to 0766's branch and
   say in the completion notes that the two sentences can go. Also
   update `assets-src/package/story-preview/README.txt` if it names the
   rule.
7. Expected size: about 70 production lines (the name function, the
   hook's bookkeeping, one view field, one painted line, two text keys).
   Judge it against 0767's 29-line drop hook. Measure with
   `cargo xtask code-lines`. PR estimate: about 400 changed lines with
   tests.

## Acceptance criteria

- [ ] Unit test `meant_name` cases: `ch01.dlg` → `ch01.dlg`, unchanged;
      `CH01.DLG` → `CH01.dlg`; `ch01.dlg.txt` → `ch01.dlg`;
      `ch01 (1).dlg` → `ch01.dlg`; `ch01 (12).DLG.txt` → `ch01.dlg`;
      `ch01 v2.dlg` → unchanged; `notes.txt` and `ch01.dlg.bak` → left
      out.
- [ ] Harness test (in `crates/ui/tests/it/story_preview.rs`, beside
      `a_dropped_file_joins_an_empty_folder_…`): dropping `ch01.dlg.txt`
      on an empty preview lists its scenes and plays them, and
      `view.dropped` names the file and the name it played as.
- [ ] Harness test: drop `trial.dlg`, then a changed copy as
      `trial (1).dlg`. The scene list holds each scene id once, the
      Problems tab is empty, the changed line plays, and the note says
      `trial (1).dlg` played as `trial.dlg`.
- [ ] Harness test: dropping `notes.txt` with `trial.dlg` plays
      `trial.dlg`, and the note names `notes.txt` as left out. The next
      drop of `trial.dlg` alone shows no note.
- [ ] `cargo shuyi frame-png --story-preview <folder> --drop <file>`
      pictures in the PR: the note after a renamed drop and after a
      left-out file. (`--drop` is a step of the engine's `frame-png`,
      shuyi-engine `crates/cli/src/frame_png.rs:254`, which
      `--story-preview` turns on: `crates/tools/src/game_tools.rs:494-497`.)
- [ ] The writer-trial docs are updated as step 6 says, or the
      completion notes say they wait for 0766.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `meant_name` (the cases above).
- Property: for any name, `meant_name` either leaves it out or gives a
  name ending in lowercase `.dlg`. Applying it twice gives the same
  result as once.
- Snapshot / integration: the Harness drop tests above, reading
  `StoryPreviewView` and the engine preview's view (scenes, problems),
  not cells. `frame-png` pictures for the look.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
