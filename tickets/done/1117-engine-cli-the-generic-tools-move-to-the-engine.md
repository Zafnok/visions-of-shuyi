---
id: "1117"
title: "shuyi-cli: frame-png, font-atlas, check-keys, check-text, lang-status, web and private-assets move to the engine, configured by a file"
type: infra
milestone: "Engine: carve"
model: sonnet-5
effort: high
status: done
blocked_by: ["1111", "1115"]
nick_input: none
completed: 2026-10-18
---

# 1117 — shuyi-cli: the generic tools move

## Context

About half of `xtask` is generic tooling (`docs/engine/boundary.md`,
*xtask*):

- `frame-png`: headless screenshots of any scripted frame. Agents need
  this most (ADR-0038 §4).
- `font-atlas`, `check-keys`, `check-text`, `lang-status`, `web`,
  `private-assets` (ADR-0040's pinned private repository, useful to
  anyone with bought art).
- `lines` and `check-script` (with the dialogue format).

Their paths are written into the code (for example `check_keys.rs:32-60`
and `check_text.rs:66-73`).

Since 1110 the web shell's files in `web/` serve the engine's
`shuyi-app` (`gamepad.js` is the `shuyi_gamepad` plugin of
`engine/crates/app/src/pads/web.rs`), but they still live in the game's
`web/`, and `--allow-undefined` is in the game's `build.rs` (a link
argument only reaches a binary from its own build script). `web` moving
here is the time to give the engine its copy of the shell and say how a
game's binary gets that link argument.

## Nick input

None.

## Scope

**In:**
- New crate `engine/crates/cli`: package `shuyi-cli`, binary `shuyi`.
  It holds the listed commands as library functions plus a thin `main`.
- A project file `shuyi.ron` at the game's root gives:
  - the paths each check scans;
  - the Harness entry point for `frame-png`;
  - the private repository settings;
  - the web build's package name.
- `cargo xtask` keeps the repository-only commands: `ticket-lint`,
  `clean-merged-targets`, the bought-art imports, `title-art`,
  `effect-marks`, `playtest` and `test-units`. Its old subcommand names
  for the moved tools forward to `shuyi-cli`, so skills and CI keep
  working. Update `run-gates`, `work-ticket`, `CLAUDE.md` and the CI
  workflows to the new commands in the same PR.
- `frame-png` runs any game that exposes a Harness constructor named in
  `shuyi.ron`. The game's `--light` option becomes a game argument
  passed through.

**Playdate:** the Playdate port (1025's tickets, 1027–1053) builds
`core`, `content` and `ui` without `std` and adds a second front end,
`crates/playdate`. If those tickets have landed, code this ticket moves
keeps its default `std` feature and stays in the `no-std` CI build, and
`cargo xtask playdate` keeps building (ADR-0070 §6).

**Out (do not do):** the JSON contract, `new`, `doctor` (1129); MCP (1133).

## Implementation steps

1. Create the crate. Move one command at a time with its tests.
2. Add `shuyi.ron` for this repository. Run each command before and
   after and compare outputs.
3. Update skills, CI and `CLAUDE.md`.

## Acceptance criteria

- [x] Each moved command gives the same output on this repository as
      before (listed in the PR).
- [x] `frame-png` renders the toy game (1111) and the game.
- [ ] CI is green with the new commands.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the moved tests.
- Integration: `shuyi-cli` on the toy game and on this repository.

## Completion notes

Done. New engine crate `engine/crates/cli` (`shuyi-cli`, binary `shuyi`,
run as `cargo shuyi <command>`): `frame-png`, `font-atlas`, `check-keys`,
`check-text`, `lang-status`, `web` and `private-assets` as library
functions with thin wrappers. This repository's `shuyi.ron` gives the
paths each check scans, the required glyphs, the web build and the private
repository. ADR-0089 records the design.

- **The game's tool program.** `frame-png` and `lang-status` run the game,
  so a game gives `GameTools` (its Harness started with its own options,
  its languages) in a program that calls `shuyi_cli::main_with`;
  `shuyi.ron`'s `game` names it and `shuyi` runs it with `cargo run`. Here
  it is xtask (`crates/xtask/src/game_tools.rs`). The game's `--light`,
  `--layout`, `--playdate`, `--map`, `--prompt` and `--tips` are game
  options passed through.
- **`cargo xtask`** keeps the repository's own commands; the moved
  commands' old names forward to `shuyi-cli` with the game's tools.
  Skills, CI, `CLAUDE.md` and the docs now say `cargo shuyi …`.
- **Same output on this repository** (each command run before and after):
  `check-keys`, `check-text` and `lang-status` (test pack, `--only`,
  `--file`, an unknown pack) print byte-identical output; every
  `frame-png` picture tried (title, Quick Battle via keys, `--playdate`,
  `--light dusk --map`, `--tips --prompt --layout left`, the Playdate map)
  is byte-identical; `font-atlas` writes the same `atlas.png` pixels and
  `atlas_1bit.bin` bytes. The only differences: usage lines and the
  generated `atlas.ron` header say `cargo shuyi` (the project's `command`),
  and a bad `--layout` value is now an exit-1 error from the game rather
  than exit 2 from the parser.
- **Web.** The web shell moved to the engine (`engine/crates/cli/web/`,
  built into `shuyi`); `index.html` is a template filled with the game's
  title and binary, so the game's `web/` folder is gone. The shell's
  README says how a game's binary gets `--allow-undefined` (its own
  `build.rs`; the game's keeps it). `gamepad.js` only changed its header
  comment.
- **Frame-png on the toy.** `engine/crates/cli/toy/shuyi.ron` names the
  crate's `toy` example as the toy's tool program; `cargo shuyi frame-png
  toy.png --lobby` from `engine/crates/cli/toy/` renders it (the toy has
  no font: letters are boxes). Integration tests run the `shuyi` binary on
  scratch projects and the toy's tools; xtask's tests run the game's.
- **Deviations.** `lines` and `check-script` stay in xtask (the title's
  list; they read the game's dialogue with its directives, which
  `GameTools` doesn't give). The test card's pixels moved to
  `shuyi_cli::test_card` because the moved painter tests draw it; `cargo
  xtask test-card` still writes the file. The game's required glyphs are in
  both `shuyi.ron` and `trpg_content::font::REQUIRED_GLYPHS`, kept equal by
  a test.
- **Playdate:** nothing moved is in the `no_std` build; `cargo xtask
  playdate` is unchanged.

No gameplay rules were decided. Nothing for Nick to play: this is tooling.

