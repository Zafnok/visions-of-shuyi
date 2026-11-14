---
id: "1270"
title: Read the game's folders through a folder reader that leaves files out
type: feature
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1270 — Read the game's folders through a folder reader that leaves files out

## Context

Engine-friction follow-up of 1209 (ADR-0124, `docs/engine/friction.md`:
estimate 160, measured 180, 1.1×, with a workaround for an engine
assumption). Under `cargo shuyi run --watch` the debug build reads
`assets/`, `assets-test/` and the content from disk with
`shuyi_app::read_folder`, which keeps every file of a folder. No build
embeds a README (ADR-0105) or the content's `tests/` (ADR-0118), and
`lang/README.md` in the bundle stops the content loading. So the game
keeps, per folder, the watched layer and a second layer without those
files, and copies each change across: `bundle::Folder`, `Folder::take`,
`bundle::take_changes` and the call in `trpg_ui::hot::HotContent::reloaded`
(about 45 lines, marked `// engine-friction: 1270`).

The generic shape: **a folder read into a layer, and watched, leaving out
the files a filter refuses**. A mod loader or a level folder wants the
same (skip `README.md`, `.git/`, an editor's backups). It names nothing of
this game (leak test passed).

## Nick input

None. (Making the engine ticket and PR needs Nick's OK first, as every
engine step does: the `engine-change` skill.)

## Scope

**In:**
- An engine `read_folder` variant (or option) that takes a filter of the
  paths to keep, applied when the folder is read and when the platform
  polls it, through the `engine-change` skill: an engine ticket and PR,
  then this PR moves the pin.
- The game reads its folders through it with `bundle::embeds`'s rule and
  deletes `bundle::Folder`, `take_changes` and their call.

**Out (do not do):**
- Anything else 1209 reloads or leaves out (maps, battles, pictures).

## Implementation steps

1. Ask Nick for the go-ahead on the engine change (`engine-change` skill),
   then write the engine ticket and PR: `shuyi_app::read_folder_keeping(path,
   name, max_file_bytes, keep: fn(&str) -> bool)` or an equivalent, whose
   poller (`crates/app/src/watch.rs`) skips the paths `keep` refuses.
   About 30 engine lines with its tests.
2. Move the game's pin to it (`engine-change` §7, the `engine-pin`
   skill's review).
3. In `crates/app/src/main.rs`'s `folders_on_disk`, read with the filter
   (`bundle::embeds` made public, or a `bundle::keeps(folder) -> fn`).
4. Delete `bundle::Folder`, `Folder::take`, `bundle::take_changes`, the
   call in `HotContent::reloaded`, and the second layer from
   `read_from_disk`; update ADR-0124's decision 2 by a new ADR that
   supersedes that part.
5. Keep `crates/app/src/tests.rs`'s
   `the_folders_on_disk_load_the_content_the_build_embeds` passing.

## Acceptance criteria

- [ ] No `// engine-friction: 1270` is left in the game.
- [ ] The content read from the folders on disk equals the embedded
      content (the app's test).
- [ ] `crates/ui/tests/it/hot_reload.rs` passes unchanged.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Engine: a folder read with a filter leaves the refused files out, at
  the read and at a poll.
- Game: the existing tests above.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
