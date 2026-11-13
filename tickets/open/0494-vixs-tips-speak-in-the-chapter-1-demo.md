---
id: "0494"
title: "Vix speaks every tip and lesson in the Chapter 1 demo"
type: bug
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0494 — Vix speaks every tip and lesson in the Chapter 1 demo

## Context

Nick, 2026-11-12, after playing the Chapter 1 demo on its itch.io page
(0915, PR #657): "Vix's voicelines are not all included in the demo --
several of her ch1 voicelines don't play." The text of the box shows;
her voice doesn't.

What 0915's session found while filing this (not yet a diagnosis):

- **All 23 of her clips ship in the demo.** `voice/en/vix/` has 23
  files, and the demo edition's `disclosure` in `shuyi.ron` allows all
  23. So the clips aren't cut by the demo bundle
  (`crates/content/demo/bundle.txt`).
- **3 of her 26 texts have no clip at all, in every build** (the full
  game too). Matching each lesson's and tip's `text`
  (`assets/data/lessons.ron`, `assets/data/tips.ron`) against `spoken` in
  `voice/en/voice.ron`, the tips with no clip are `weapon_kinds` (added by
  0489, PR #619), `export_save` and `export_profile` (added by 0874,
  PR #644). Neither ticket generated a clip. `weapon_kinds` comes up in
  Chapter 1 (its Red Company riders and the spears).
- **Her lines are never preloaded.** `docs/voice.md` ("How the game plays
  them"): a screen preloads a scene's next lines with
  `ctx.preload_voices`, and "a clip that isn't loaded within 0.3 s of
  being asked for is skipped rather than played late". The battle screen
  asks for her clip when a box comes up
  (`crates/ui/src/screens/battle/guide_voice.rs`, `GuideVoice::show`),
  with no preload. On the desktop the file is on disk; on the web it is
  fetched from itch.io's servers. A fetch over 0.3 s means silence. This
  would hit the web demo and the full web build alike.

Either one, or both, may be what Nick heard. The fix must cover both and
start from a reproduction.

Related: 0476 (the guide's lines voiced), 0475 (`text` voiced, items and
`how` silent), 0915 (the demo's pages), 0766 (the demo goes to the
writers applying).

## Nick input

None. Clip generation runs on this machine (`cargo xtask voice generate`,
`docs/voice.md`); no listening check after merge. Whether her lines
sound right goes on the Act 1 playtest's checklist (1064).

## Scope

**In:**
- Reproduce first: in the web demo (`cargo shuyi export web --release
  --edition demo` served locally, or the itch.io page), with voices on, play
  turn 1 of Chapter 1 at Full and note which of her boxes stay silent.
- Clips for the three tips with none (`weapon_kinds`, `export_save`,
  `export_profile`), made by `cargo xtask voice generate --scene vix` and
  committed through the private repository (`docs/voice.md`, "Committing
  the clips"), then `cargo shuyi private-assets --pin`. Their rows in both
  `allowed_files` lists in `shuyi.ron` (the full game's and the demo's).
- Her lines preloaded on the battle screen, so a slow fetch doesn't skip
  them. For example, preload the lines of every lesson and tip the
  battle could still show (the ones not yet seen) when the battle starts,
  or the next one as soon as it is queued. The session picks the approach
  and says why in the completion notes.
- A check that fails when a lesson's or tip's `text` has no clip: an
  xtask test over `voice/en/voice.ron` when the private copy is checked
  out, like the existing check of `voice/` against `shuyi.ron`. A new tip
  without a clip then fails CI, not a playtest.

**Out (do not do):**
- Rewording any of her lines (dialogue writing is paused, AGENTS.md rule
  3; 0766).
- Her tips taking Confirm and Cancel from other screens (0495).
- The 0.3 s rule itself for scene dialogue (the dialogue screen already
  preloads).

## Implementation steps

1. Reproduce as above. Note each silent box and whether its line has a
   clip. If a line with a clip stays silent, check the browser's network
   log for its fetch time.
2. Generate the three missing clips (`cargo shuyi private-assets`, then
   `cargo xtask voice status --scene vix`, `cargo xtask voice generate
   --scene vix`). Commit `voice/` in `assets-private/`, push, `cargo shuyi
   private-assets --pin`. Add the files to both `allowed_files` lists in
   `shuyi.ron`.
3. Preload her lines: in `crates/ui/src/screens/battle/` (`guide_voice.rs`
   or where the battle screen starts), call `ctx.preload_voices` with the
   guide lines (`trpg_content::voice::guide_line`) of the lessons and tips
   still to come.
4. The check: a test in `crates/xtask` (beside the one that checks `voice/`
   against `shuyi.ron`) that every line of the guide's scene
   (`trpg_content::voice::guide_scene`) has an entry in `voice.ron`. It is
   skipped without the private copy, like that one.
5. Export the web demo again and play turn 1 in the browser pane: every
   box speaks (agents' memory: *web build testing in browser pane*).

## Acceptance criteria

- [ ] `voice/en/voice.ron` has a clip for `weapon_kinds`, `export_save` and
      `export_profile`, and `assets-private.rev` points at that commit.
- [ ] The new xtask test fails with a tip whose `text` has no clip (tried
      by a throwaway edit, recorded in the notes) and passes on the real
      data.
- [ ] A test shows the battle screen preloads the guide's lines (the
      lines it hands to `preload_voices`, read from the test `Ctx`).
- [ ] In the web demo with voices on, every lesson and tip box shown on
      Chapter 1's turn 1 at Full speaks (noted in the completion notes,
      with the build served the way it was tested).
- [ ] The demo's export still passes its disclosure gate.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the preload (which lines, when).
- Integration: the xtask test of the guide's clips.
- Manual: the web demo in the browser pane, as above.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
