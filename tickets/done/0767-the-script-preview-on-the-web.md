---
id: "0767"
title: "The script preview on the web: a writer drops .dlg files on the page"
type: feature
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0764"]
nick_input: none
completed: 2026-11-14
---

# 0767 — The script preview on the web

## Context

Filed by the engine pin move of 2026-11-10 (`docs/engine/pins.md`,
engine #293, #352 and #355; engine tickets 3069, 3092 and 2588).

0764 builds `story-preview`, the program a writer downloads to see their
`.dlg` scenes play in the game's own text box (on engine#2586's
`DialoguePreview`). It leaves the web out: "it needs files dropped on
the page, engine#3092". That is now in the engine:

- **#352 (3092):** files dropped on the window come to the game as a
  bundle layer, on the desktop and the web (`SystemEvent::FilesDropped`).
- **#355 (2588):** `shuyi_ui::dialogue::preview::drop_scenes(game,
  files)` puts the dropped `.dlg` files in the preview's folder layer and
  tells the modules, as a saved file does; on the web the preview starts
  over an empty layer, and its Scenes tab says "No scenes: drop a .dlg
  file here" (`preview.note.no_scenes_drop`).
- **#293 (3069):** a web build can lay files its page holds over its own
  (`shuyi_app::page::page_layer`, `crates/cli/web/page-files.js`) and
  send its problems back to the page (`shuyi_app::page::report`).

A web preview means a writer trying out for the job (0766's trial
packet) opens a link instead of downloading a Windows program, and works
on macOS and Linux too, which 0764 doesn't build.

## Nick input

None. Verified by Harness tests and `cargo shuyi frame-png` pictures in
the PR. How a writer uses it is judged in 0766's trial.

## Scope

**In:**
- A web build of 0764's preview: the same screens, with no scene in it,
  playing the `.dlg` files dropped on the page (`drop_scenes`), with the
  game's faces and music as 0764's Windows build has them.
- If 0764 built the preview without embedded scripts (an edition with a
  content root, or its `no-embedded-scripts` feature), the web build
  does the same: the Wasm holds no scene.
- Where it is served: a page only the trial's writers get (the
  restricted itch.io project of 0915's kind, or the private mirror's
  pages, 0152), never the public GitHub Pages site (0158). Say which in
  the completion notes.
- Whether a page that hands the files in (#293's page files) is simpler
  for a writer than a drop: decide, and say why.

**Out (do not do):**
- Anything 0764 leaves out (playing a scene inside a running battle,
  voice clips, a script's words).
- A public build: the bought art is in it (ADR-0040 allows a build that
  holds it, not a public page handing it to anyone).

## Implementation steps

1. Read 0764's completion notes (the binary, its start, its build
   without scripts) and engine #355's (`drop_scenes`, the hook
   `BasicGame::preview_start` gives the preview).
2. Build the preview for `wasm32-unknown-unknown` (`cargo shuyi web`
   with the preview's package or bin; a second `web` entry if
   `shuyi.ron` needs one).
3. Hand each drop to `drop_scenes` from the game's system hook.
4. A Harness test drops a scene (`SystemEvent::FilesDropped` in the
   input script) and reads it in the preview's view.
5. Expected size: about 120 production lines (the web start and the drop
   hook), judged against 0764's binary start.

## Acceptance criteria

- [x] A Harness test drops a `.dlg` file on the web preview and the
      scene is listed and plays; dropped again changed, it plays again.
- [x] The web build embeds no `.dlg` file (`strings` on the Wasm finds
      none of `assets/dialogue/`'s scene ids).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the Harness drop test.
- Pictures: `cargo shuyi frame-png` of the empty preview and a dropped
  scene playing.

## Completion notes

**What was done**

- **The drop:** the preview's start (`trpg_ui::story_preview::start`) has
  a system hook, `story_preview::hear`, that hands each
  `SystemEvent::FilesDropped` to the engine's `drop_scenes`: the dropped
  `.dlg` files join the preview's folder in place of a file of the same
  name, as a saved file does, and play on the next frame from the same
  text box. Files of another kind are left out. It works on the desktop
  too (the desktop's facts hear drops as well).
- **The web build** is 0764's program, `cargo shuyi export web --release
  --edition story-preview`: the same edition, so the same content root
  with no `dialogue/`, `default_features: false`, the bought art. The
  binary needed only the web link flag (`--allow-undefined` in
  `crates/story-preview/build.rs`, as `crates/app/build.rs` has); its
  start already gave an empty folder on the web. No second `web` entry in
  `shuyi.ron`: the edition names the package and the program.
- **The empty preview's note:** the game's text gains
  `preview.note.no_scenes_drop`, "No scenes. Drop your .dlg files here",
  which the engine shows wherever drops are heard (the web and the
  desktop). The old note told the writer to drag a folder onto a program,
  which is wrong on the web.
- **`cargo shuyi frame-png --story-preview <folder> --drop <file>`** now
  drops into the preview (the game Harness's `hearing`, the preview's
  facts hear drops), for the pictures below and the MCP session.
- **Where it is served: a Restricted itch.io project of its own, behind a
  password** (the web demo's kind, 0915), `ITCH_PREVIEW_PROJECT`, channel
  `web`. `story-kit.yml`'s preview matrix gains a `web` leg: export, the
  same three-script-line check on the Wasm and its pack, then `butler
  push`; without `BUTLER_API_KEY` or the variable it builds, checks, says
  so and stays green. **Not the private mirror's pages:** GitHub Pages of
  a private repository is a public site, which ADR-0040 rules out.
  `docs/releasing.md` has the steps; the mirror's README tells the writer
  the web preview is for a Mac or Linux.
- **Drop, not the page's files (#293):** the drop is simpler for a
  writer. Page files are written by a parent page into `sessionStorage`
  and read once at the start, so the game must be run again for each
  change, which loses the writer's place, and we would have to build and
  host an editor page around the game. A drop needs nothing but the page,
  keeps the scene playing at its text box, and is what the desktop
  program already does.

**Verification**

- Harness test `a_dropped_file_joins_an_empty_folder_and_plays_and_dropped_again_changed_plays_again`:
  an empty folder shows the drop note; a `.txt` dropped is left out;
  `trial.dlg` dropped lists both its scenes with no problem and plays;
  dropped again changed, the change plays from the same text box. The
  xtask test of `--story-preview` drops a file too.
- The edition's web build: none of the 308 scene ids of the game's
  scripts is in the Wasm (`strings`, whole words). Two are in the pack
  beside it, `ch01_intro` and `ch01_tbc`, in comments of the Chapter 1
  cinematic files that name the scene they open: names, no script text.
  None of 50 script lines (the first five long speech lines of each of the
  ten scripts) is in the Wasm or the pack.
- **Not done here:** `wasm-opt` from MSYS2's Binaryen stalled on this
  machine (80 minutes, 3.5 minutes of CPU), so the local check ran on the
  Wasm before optimizing, which holds more text, not less. The release
  export's `wasm-opt` runs in CI on Linux, as for the demo. The page
  wasn't opened in a browser and nothing was pushed to itch.io: the
  project doesn't exist yet.
- All `run-gates` commands passed locally.

**For Nick, once:** make the itch.io project (Restricted, with a
password), set the repository variable `ITCH_PREVIEW_PROJECT` to
`<user>/<project>`, run **Actions → Story kit**, then tick *played in the
browser* on its `web` upload (`docs/releasing.md`, "The script preview's
itch.io page"). Its link and password go in 0766's trial packet.

**Size:** estimate 40 production lines (a system hook like the engine's
12-line `dropped_scenes`, the web start), measured 29: under 2×. The
ticket's 120 assumed a web start of its own, which 0764's binary already
had. PR size: estimate 400, measured about 200.

**Starting rules:** none; nothing here is a game rule.

**Follow-ups:** none new.
