---
id: "0774"
title: "The trial's web demo and preview keep their saved settings apart from the playtest page, as decided"
type: feature
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0090"]
nick_input: answer-first
completed:
---

# 0774 — The trial's web demo and preview keep their saved settings apart from the playtest page, as decided

## Context

The playtest page, the web demo (0914, 0915) and the web preview (0767, 0771) are all served from `html-classic.itch.zone` (`docs/releasing.md`). All three save under the storage name `tactical-rpg`, set in `crates/app/src/app_info.rs` (`.with_storage("tactical-rpg")`) and used by both `crates/app/src/main.rs` and `crates/story-preview/src/main.rs`. On the web that means one set of `localStorage` keys under `tactical-rpg/`, shared by every page in one browser. This has two effects:

1. **Nick's playtest page leaks into the demo.** Its saves (any chapter, debug tools) and settings show up in the demo. This is the agent's call: the web demo gets a storage name of its own.
2. **A writer's demo settings reach the preview.** The preview has no Options screen to undo them: the key layout, Auto-advance (`crates/ui/src/screens/dialogue.rs`, `auto_advance()`), the portrait look and the volumes. Ticket 0090 decided what happens here (`docs/design/demo.md`, *The demo and the script preview's settings*):
   - A: the preview keeps its own settings;
   - B: only the keys follow;
   - C: everything follows and the preview gets the game's Options;
   - D: as it is.
   0090's session edits this ticket down to the chosen option.

The full game's storage name never changes (`shuyi.ron`: "Never change `storage`, or every player's saves are lost"). Exporting a demo save into the full game goes by file (`docs/design/demo.md`, rule 5), not by storage, so a separate demo store breaks nothing.

For the writer trial (0766): a new demo storage name resets each applicant's in-browser demo saves and settings once, the first time they open the demo after this ships. That is harmless before the first packet goes out and worth a line to applicants after.

## Nick input

**Answer first:** ticket 0090. Nothing to check after merge: tests and the PR's `frame-png` pictures verify it. Anything left to judge by playing goes on the Act 1 playtest's checklist (1064).

## Scope

**In:**
- A storage name of its own for the `demo` edition (both its web and Windows packages are that edition).
- The `story-trial` edition's storage, as 0090 decided.
- For B, reading only the demo's key layout.
- For C, the game's Options screen reachable from the preview.
- `docs/releasing.md`: which pages share saved settings.
- An ADR, or an amendment to ADR-0119 (the demo edition), for a storage name chosen per edition.

**Out (do not do):**
- Changing the full game's storage name `tactical-rpg`, or anything about the playtest page's own saves.
- The hired writer's desktop kit (`story-preview` edition), unless 0090 says otherwise.
- Moving or copying existing browser saves between names.
- 0766's packets.
- Pushing the demo and the web preview together (0775). Either order: if 0775 is done, the `story-trial` edition is exported by `demo.yml`; the storage name comes from the edition, not the workflow, so this ticket's change is the same whichever workflow builds it.
- Framed pages with third-party storage blocked (0931). Either order: if 0931 is done, its Completion notes say which browsers partition or block storage; mention them in `docs/releasing.md`'s note on shared settings. If not, say nothing about blocked browsers there.

## Implementation steps

Expected size: about 40 to 80 production lines for A or D (two storage-name choices and their docs); about 120 for B; about 150 to 250 for C (reusing the game's Options screen, not a new one). Compare with `crates/story-preview/src/main.rs`'s `config()` and `crates/ui/src/story_preview.rs`'s `game()`.

1. Read 0090's answer in `docs/design/demo.md`, plus ADR-0050 (settings under the `settings` key), ADR-0031 (`keybindings`), ADR-0119 and ADR-0120.
2. **The demo's own storage (every option).**
   - In `crates/app/src/main.rs`, replace the fixed `STORAGE_NAME` in `config()` with a pure `fn storage_name(demo: bool) -> &'static str`. Keep `APP.storage` for the full game and use a new name such as `tactical-rpg-demo` for the demo.
   - Call it with `trpg_content::bundle::is_demo()`. `main()` evaluates `startup()` (which lays a packed build's pack) before `config()`; confirm that order holds on the web.
   - If the edition cannot be known there on the web, use a compile-time switch set by the `demo` edition's features in `shuyi.ron` instead, and say why in the Completion notes.
   - Keep the comment that the full game's name never changes.
3. **The preview, per 0090:**
   - **A:** in `crates/story-preview/src/main.rs`, the `story-trial` edition uses a name of its own (for example `tactical-rpg-story-trial`), chosen the same way as step 2. Find out how the trial's bundle marks its edition (`crates/content/story-trial/bundle.txt`, `crates/content/build.rs`, `trpg_content::edition`); add an edition name to `edition.rs` if it has none. The desktop kit keeps `APP.storage`.
   - **B:** as A. In addition, when the preview's own store has no saved layout, `crates/ui/src/story_preview.rs` reads only the saved layout (and the `keybindings` key) from the demo's store and uses it. Use the engine's storage API (`shuyi_app::storage`) to open the second name read-only. If the engine has no way to read another name, follow the `engine-change` skill and do not work around it.
   - **C:** the `story-trial` edition uses the demo's name from step 2. Add an Action for opening the game's Options screen from the preview (`keyboard-input` skill: a default key in the keymap, the help bar naming the player's key). Push the existing Options screen; build no new one. Changes take effect on the next line.
   - **D:** the `story-trial` edition uses the demo's name from step 2; nothing else.
4. `docs/releasing.md`: in *The playtest page*, the demo's section and *The script preview's itch.io page*, say which pages share saved settings and that the playtest page's never reach the demo.
5. Record the storage name per edition (`write-adr`, or an amendment to ADR-0119).
6. In the PR, say whether 0766's packets have gone out. A new demo name resets each applicant's in-browser demo saves and settings once.

## Acceptance criteria

- [ ] A unit test shows `storage_name(false)` is `"tactical-rpg"` (equal to `APP.storage`) and `storage_name(true)` is a different, non-empty name.
- [ ] The chosen option holds, shown by its test below:
  - **A:** the trial preview's storage name differs from the demo's and the game's.
  - **B:** a preview started over an empty store of its own uses the layout saved in the demo's store, and Auto-advance stays at the game's default even when the demo's store has it On.
  - **C:** pressing the new Action in the preview opens Options; setting Auto-advance On there makes the next line advance on its own.
  - **D:** the trial preview's storage name equals the demo's.
- [ ] The desktop kit's (`story-preview` edition) storage name is still `APP.storage` (a unit test).
- [ ] `docs/releasing.md` says which web pages share saved settings.
- [ ] For C, `cargo shuyi check-keys` passes and the PR shows a `cargo shuyi frame-png` picture of Options open over the preview.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `storage_name` per edition in `crates/app` and `crates/story-preview` (pure functions, no real bundle needed).
- Integration (ui, the engine's Harness with `MemoryStorage`): for B, the preview's context over a demo store holding a layout and Auto-advance On; for C, opening Options from the preview and changing Auto-advance.
- Snapshot / picture: for C, a `frame-png` picture in the PR (no snapshot of the real content, ADR-0118).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations, follow-up tickets created, notes for Nick.)*
