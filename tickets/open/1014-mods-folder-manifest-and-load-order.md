---
id: "1014"
title: "The mods folder: find mods, read their manifests and load order, hand them to content"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: medium
status: todo
blocked_by: ["1012"]
nick_input: none
completed:
---

# 1014 — The mods folder, manifests and load order

## Context

1012 lets `trpg-content` take mod layers as bytes. Only `app` may touch
files (ADR-0004), so `app` finds the mods, reads them and hands them over
before content loads. Where mods live, the manifest's fields and the load
order file are 1011's ADR; the defaults below are its defaults.

## Nick input

None.

## Scope

**In:**
- A `mod.ron` manifest type in `trpg-content` (`ModManifest`: id, name,
  version, author, `game_version`, `requires: [id]`, `conflicts: [id]`),
  parsed and validated there (it's data; `content` gets the text from
  `app`).
- In `app` (native only): find mod folders in `mods/` beside the executable
  and in the player's data folder (the same base folder
  `crates/app/src/storage/native.rs` uses), read each manifest, read the
  enabled list and order from storage key `mods` (a RON list of ids; none
  enabled when it is missing), read every file of each enabled mod, call
  `bundle::install_mod_layers` before content loads.
- Problems (bad manifest, missing requirement, conflict, unreadable file,
  a mod for another game version) disable that mod for this launch and are
  kept in a list the game can show (1015 shows it; until then, log it).

**Out (do not do):** the mods screen (1015); zips (the ADR may add them
later; folders only here); the web build (`#[cfg(not(target_arch =
"wasm32"))]`; the web build starts unmodded); Workshop (1024).

## Implementation steps

1. Read 1011's ADR, 1012's `install_mod_layers`, `crates/app/src/main.rs`
   and `crates/app/src/storage/native.rs`. Find where content first loads
   (`grep -rn load_embedded crates/app crates/ui/src`).
2. `crates/content/src/modding.rs`: `ModManifest`, `parse_manifest(text) ->
   Result<ModManifest, ContentError>`, and `resolve_order(manifests,
   enabled_ids) -> (Vec<id>, Vec<ModProblem>)` (pure: requirements before
   the mods that need them, conflicts refused, unknown ids reported).
3. `crates/app/src/mods.rs`: scanning the folders, reading files (sizes
   capped at a value the ADR sets), building `ModLayer`s in the resolved
   order, installing them. Keep this the only file in `app` that knows the
   folder layout.
4. Make the list of problems reachable from `ui` (a field the flow screen
   or the title can read later), per the existing way `app` passes data to
   `ui`.
5. Test mods under `crates/app/tests/mods/` (a working one, one with a bad
   manifest, one that needs a missing mod).

## Acceptance criteria

- [ ] `resolve_order` orders by requirements, refuses conflicts and
      reports unknown ids (unit tests).
- [ ] A bad manifest disables only that mod, with a problem naming it
      (test).
- [ ] With a test mod enabled that changes an item's name, the game's
      loaded content shows the new name (integration test in `app`, or in
      `content` with the scan stubbed).
- [ ] With no `mods` key in storage, the game loads exactly as before.
- [ ] The web build compiles and runs unmodded.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: manifest parsing, `resolve_order`.
- Integration: the test mods folder end to end.

## Completion notes
