---
id: "1265"
title: Release builds read the game's files from a baked pack beside the program
type: infra
milestone: Later
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1265 — Release builds read the game's files from a baked pack beside the program

## Context

Filed by the engine pin move of 2026-11-13 (`docs/engine/pins.md`,
engine PR #378, engine ticket 3300, engine ADR-0110). A package with a
`packed` feature now has its `--release` exports of `windows`, `linux`,
`macos` and `web` bake its files into `<package>.pack`, check the pack
against its sources (`bake --check`), search it for build-machine paths,
ship it beside the program and build with `packed`.
`shuyi_app::pack_beside_program(name, stamp)` reads it beside the exe
(or from a `.app`'s `Contents/Resources`); on the web the page fetches
it before the game starts (`pack-file.js`). A missing or foreign pack is
shown in place of the game. On the engine's example the program lost
the pack's size and native loading was 2.3× faster.

The game embeds every file of `assets/` (and the private content laid
over it, ADR-0117) into the program as text: `trpg-content`'s bundle
(`crates/content/src/bundle.rs`, `install`). The game has no `packed`
feature, so its exports build as before.

## Nick input

None: what the player sees and hears doesn't change.

## Scope

**In:**
- Read engine ADR-0110, its `docs/formats.md` measurements and the
  `shuyi-ship` skill's pack steps; decide whether the game's bundle
  (with its layers: private content, the demo edition's tables, the test
  files in debug builds) can be read from a pack, and record that in an
  ADR (`write-adr`).
- If it can: a `packed` feature on `trpg-app` whose build reads the
  bundle from `pack_beside_program`, the embedded bundle kept for debug
  builds, the tests and the Playdate. `cargo shuyi bake` works on the
  game.
- `scripts/check-release-package.sh` expects the pack in each desktop
  and web package, and the release workflow's packages still pass it.
- The release program's size before and after, and its start time, in
  the completion notes.
- If the engine's pack can't hold the game's layers, the engine ticket
  for what is missing (`engine-change`, "ticket and follow-up") and this
  ticket blocked on it.

**Out (do not do):**
- The Playdate's build (its own files, ADR-0070).
- Changing any file of `assets/` or the private content.

## Implementation steps

1. Read engine ADR-0110 and `shuyi_app::pack_beside_program`; read
   `crates/content/src/bundle.rs`'s layers.
2. Write the ADR; add the feature and the read; make `cargo shuyi bake`
   and `bake --check` pass on the game.
3. Export each target with `--release` and run the package check; play
   the Windows and web exports in a build.

## Acceptance criteria

- [ ] A release export of `windows` and `web` holds `visions-of-shuyi.pack`
      and its program holds no RON text of `assets/`.
- [ ] The exported game plays the same (the title, a new game's first
      scene and a battle in the Windows and web exports).
- [ ] `scripts/check-release-package.sh` passes on every target's package.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- A test that the baked pack holds every file the embedded bundle holds,
  with the same bytes.
- CI: the release workflow's package check with the pack.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
