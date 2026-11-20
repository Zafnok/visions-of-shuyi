---
id: "0777"
title: "The script preview says when a folder's scene is one the game never plays"
type: feature
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0777 — The script preview says when a folder's scene is one the game never plays

## Context

The writer trial (0766) asks applicants to rewrite Chapter 1's scenes in
`dialogue/ch01.dlg`, keeping each scene's `@scene` id
(`docs/writer-trial/brief.md` on the 0766 branch: "Keep each scene's
`@scene` id and its `@end`"). A test audit before the packets go out
found that a **renamed** `@scene` id gets no warning in the script
preview (0764, 0767):

- The preview checks the folder with
  `trpg_content::load_with_some_scripts` (`crates/content/src/lib.rs`,
  about line 297). In that load, a scene that a chapter, battle trigger,
  cinematic, camp event, support or world-map place names, but that the
  folder doesn't have, is **not** a problem. `dialogue::named::missing`
  → `noted` (`crates/content/src/dialogue/named.rs`, lines 26-40) puts it
  in `SomeScripts::not_here` instead.
- The preview shows `not_here` only as the generic
  `preview.note.only_here` ("Only the scenes in this folder are
  checked"). That note is always on in the trial, because the packet
  holds Chapter 1's scenes and the game names about 300.
- So when `ch01_intro` is renamed to something else, the old id silently
  joins `not_here`. The new id is named by nothing, so no problem is
  shown. The scene also loses its battle map, because
  `ctx.game.content.battle_of_scene(&play.scene.id)` in
  `story_preview::dialogue_screen` looks it up by id. The writer's entry
  then can't be imported as written (`story-import`, ADR-0122).

This also helps the hired writer later. A new scene that nothing plays
yet is legitimate in the mirror, so this is a note, not a problem: it
never fails `story-check`.

This is not a blocker for the writer trial (0766): the brief already
tells applicants to keep each scene's id, so the note only catches a
slip.

## Nick input

None. This is the writer's tool, not the game: what it notes is
technical. Verified by unit and Harness tests and a `cargo shuyi
frame-png` picture in the PR. No check after merge.

## Scope

**In:**
- `SomeScripts` gains the folder's scenes that no file of the game
  names: `unnamed: BTreeSet<String>` (the scene ids in the scripts loaded
  minus every scene a file names).
- `named.rs` records every scene a file names during a load of only some
  scripts, whether present or not, beside `not_here`.
- The preview's view gains a note naming those scenes, for example "No
  chapter or battle plays ch01_intrro: was its `@scene` id changed?" (final
  wording is the implementer's). It is shown in place of `only_here` when
  any exist.
- Text keys in `assets/lang/en/ui.ron` and
  `assets-test/lang/test/ui.ron`.

**Out (do not do):**
- Making it a problem in `story-check` or the Problems tab (a new scene
  is legitimate in the mirror).
- Guessing which old id a renamed scene came from (e.g. by edit
  distance).
- Any change to Shuyi Engine.
- Files the preview leaves out or doubles on a drop (0776, "The script
  preview takes .DLG and .dlg.txt drops and names each file it leaves
  out"). Either order works. If 0776 is done, its `dropped`
  note takes the tabs' second row first, then this note, then
  `only_here`. If it isn't, this note simply replaces `only_here`, and
  0776 puts its note first.

## Implementation steps

1. In `crates/content/src/dialogue/named.rs`, beside the thread-local
   `NOT_HERE`, record every scene passed to `missing` while a load of
   only some scripts runs (a second set, or one struct holding both).
   `with_some_scenes` returns both sets, and `Restore` puts both back.
   Keep the existing tests in that file passing.
2. Find every way the game plays a scene, and make sure each goes
   through `scene_missing` or is recorded. The current callers are
   `camp.rs`, `chapter.rs`, `cinematic.rs` (which calls
   `!scene_missing`), `support.rs`, `trigger.rs` and `worldmap.rs`. Also
   look for scenes played from code or by an id built at run time
   (`grep -rn "scene" crates/ui/src crates/content/src` for literal ids
   and `format!` ids). Record any such path too, so that the full game
   has no unnamed scene (the all-scripts test under Acceptance
   criteria).
3. In `crates/content/src/lib.rs`, `SomeScripts` gains `unnamed`: the ids
   of the scenes in `scripts` (from the loaded `Content`'s dialogue
   table, or from the parsed scripts when the load failed) that no file
   named.
4. In `crates/ui/src/story_preview.rs`: `ShuyiPreview::problems` already
   runs `load_with_some_scripts`, so keep `unnamed` in a `Cell` /
   `RefCell` as `not_here` is kept, expose it as `scenes_not_played()`,
   and fill a new `StoryPreviewView::unplayed: Option<String>` from it
   with the new text key. Paint it on the tabs' second row in place of
   `only_here` (ADR-0054).
5. Expected size: about 60 production lines (a second set in `named.rs`,
   one field in `SomeScripts`, one view field and its painted line).
   Judge it against 0764's `not_here` plumbing. Measure with
   `cargo xtask code-lines`. PR estimate: about 350 changed lines with
   tests.

## Acceptance criteria

- [ ] Unit test in `named.rs`: in a load of some scenes, a scene named
      and present is recorded as named and not in `not_here`; a scene
      named and absent is in both. A strict load records nothing.
- [ ] Test in `crates/content`: `load_with_some_scripts` on **all** of
      the game's own scripts (as `load_embedded` reads them) gives an
      empty `unnamed` and an empty `not_here`. This guards against false
      notes. The test runs on whatever content is embedded: the public
      stand-ins locally without the private checkout, the real game in
      CI (ADR-0117). If a real scene is legitimately named by nothing,
      the session lists each such scene in the Completion notes, then
      either records how the game plays it (step 2) or files a content
      ticket for it (`write-ticket`); never a silent allow-list in the
      test.
- [ ] Test in `crates/content`: Chapter 1's script with one `@scene` id
      renamed (e.g. `ch01_intro` → `ch01_intro_x`) gives
      `unnamed == {"ch01_intro_x"}` and `ch01_intro` in `not_here`.
- [ ] Harness test in `crates/ui/tests/it/story_preview.rs`: a folder
      whose scene id no file names shows `StoryPreviewView::unplayed`
      naming it, and the Problems tab stays empty. With the ids put back,
      the note is gone.
- [ ] A `cargo shuyi frame-png --story-preview` picture of the note in
      the PR.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `named.rs`'s recording of named scenes; `SomeScripts::unnamed`.
- Property: for any subset of the game's embedded scripts, `unnamed` is
  empty (every real scene is named by some file; the same rule as the
  all-scripts test for a scene that legitimately isn't).
- Snapshot / integration: the Harness test reading `StoryPreviewView`,
  not cells. A `frame-png` picture for the look.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
