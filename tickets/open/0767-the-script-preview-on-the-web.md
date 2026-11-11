---
id: "0767"
title: "The script preview on the web: a writer drops .dlg files on the page"
type: feature
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0764"]
nick_input: none
completed:
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

- [ ] A Harness test drops a `.dlg` file on the web preview and the
      scene is listed and plays; dropped again changed, it plays again.
- [ ] The web build embeds no `.dlg` file (`strings` on the Wasm finds
      none of `assets/dialogue/`'s scene ids).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the Harness drop test.
- Pictures: `cargo shuyi frame-png` of the empty preview and a dropped
  scene playing.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
