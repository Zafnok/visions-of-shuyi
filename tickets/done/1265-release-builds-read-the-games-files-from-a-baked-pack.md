---
id: "1265"
title: Release builds read the game's files from a baked pack beside the program
type: infra
milestone: Later
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-13
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

- [x] A release export of `windows` and `web` holds `visions-of-shuyi.pack`
      and its program holds no RON text of `assets/`. (The pack is
      `trpg-app.pack`, the engine's name for it; see the deviations.)
- [ ] The exported game plays the same (the title, a new game's first
      scene and a battle in the Windows and web exports). All three on the
      web; on Windows the title and the first scene, not a battle (see
      Played).
- [x] `scripts/check-release-package.sh` passes on every target's package.
      (Windows and web locally; Linux and macOS in `release.yml`'s dry run.)
- [x] All gates in the `run-gates` skill pass.

## Tests required

- A test that the baked pack holds every file the embedded bundle holds,
  with the same bytes.
- CI: the release workflow's package check with the pack.

## Completion notes

**Done.** The engine's pack holds the game's layers, so no engine ticket
was needed; ADR-0120 records how.

- `trpg-app` has a `packed` feature (it turns on `trpg-content/packed`).
  With it, the content crate embeds no table of `assets/`, the content or
  the demo's cut, and reads that layer from the pack the program lays in
  with `bundle::use_pack`. `main.rs` reads it with
  `shuyi_app::pack_beside_program(CARGO_PKG_NAME, bundle::PACK_STAMP)`
  before the start. A missing or foreign pack is the start's error, shown in
  place of the title ("the game's files, trpg-app.pack, aren't beside the
  program … bake it again"). The bought art stays in the program, and debug
  builds, tests, tools, the Pages web build and the Playdate embed as before.
- xtask's `Trpg::bake` bakes `bundle::pack_layers()` (the content over
  `assets/`, or the demo's table in the demo edition) as raw bytes, stamped
  `visions-of-shuyi <version>`. It refuses the Playdate, which bakes its own
  files. `cargo shuyi bake --target desktop` gives 166 files: 1,131,858
  bytes of sources become a 772,522-byte pack (68%). `bake --check` passes.
- `shuyi.ron`: both disclosure parts allow `trpg-app.pack` beside the
  program (and in the `.app`'s Resources) and list every pack entry
  (`allowed_pack_entries`: 166 for the full game, 111 for the demo). A test
  keeps those lists matched to the bundle.
- `scripts/check-release-package.sh` expects the pack in each desktop and web
  package. It fails a program (exe, Linux binary, Mac binary or wasm) that
  still holds `assets/data/keymap.ron`'s first line. That line is in the
  pre-change program.
- `docs/releasing.md` says how it works.
- Tests: `the_baked_pack_holds_every_file_of_the_embedded_bundle_with_the_same_bytes`
  and `the_disclosure_lists_name_every_file_of_the_pack` (xtask).

**Measured (Windows, local `cargo shuyi export windows --release`):**

- Program: 13,731,328 bytes before (the same release build without
  `packed`, debug sections stripped as the export strips them). 12,613,632
  bytes after: 1.12 MB less, the size of `assets/`. The pack beside it is
  772,522 bytes.
- Web export: `visions-of-shuyi.wasm` 10,259,061 bytes plus a 772,518-byte
  pack. The demo edition's web pack is 536,704 bytes.
- Start: the window opens after about 230 ms either way (4 runs each,
  215–291 ms), and the files aren't read until after it. The bake reports
  reading the pack in 0.8 ms in a debug build. Raw bytes don't give the
  engine's 2.3× faster loading; that needs the RON kinds baked into the
  pack's binary forms (ADR-0120, Alternatives), which no measurement here
  calls for.

**Played:** Windows export: the title (art, text, font from the pack), and
the first scene with `--scene ch01_intro`. With the pack moved away, the
"bake it again" message showed. Simulated key presses don't reach
macroquad's window here, so the battle was played on the web export only.
The web export, served locally, played the layout picker, the title menu
with Vix's tip, New Game's questions, the lead picker, the first scene and
Chapter 1's first battle map. Both exports read the same pack bytes, which
the byte-for-byte test proves equal to the embedded files.

**Deviations:**
- The pack is `trpg-app.pack`, not `visions-of-shuyi.pack`. The engine names
  it after the package (`<package>.pack`, engine ADR-0110 §4), and renaming
  `trpg-app` was out of scope.
- Locally, `check-release-package.sh` ran on the Windows and web packages.
  Linux and macOS are checked by `release.yml`'s dry run on this branch
  (see the PR).

**Engine friction:** estimate 80 production lines (Hello, Grid's `packed`
read and its tools' bake, plus the game's three tables); measured 70
added: under 2×. **PR size:** estimate 800, measured 714.

**For Nick:** nothing changes in play. A release now ships one more file
beside the program, `trpg-app.pack`, and the game won't start without it.
