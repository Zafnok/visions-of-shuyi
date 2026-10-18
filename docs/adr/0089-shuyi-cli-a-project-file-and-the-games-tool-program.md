# ADR-0089: Shuyi's tools: `shuyi-cli`, a project file (`shuyi.ron`) and the game's tool program

- **Status:** Accepted
- **Date:** 2026-10-18
- **Related tickets:** 1117
- **Extends:** ADR-0063 (an engine and a game), ADR-0038 §4 (`frame-png`),
  ADR-0081 (the platform shell is the engine's). Amends where ADR-0009
  and ADR-0040 name `cargo xtask web` and `cargo xtask private-assets`:
  both are now `cargo shuyi …` (the old names still work).

## Context

About half of `xtask` was generic tooling (`docs/engine/boundary.md`,
*xtask*): `frame-png`, `font-atlas`, `check-keys`, `check-text`,
`lang-status`, `web` and `private-assets`. Their paths were written into
the code (the folders each check scans, the private repository, the web
build's package), and two of them run the game itself: `frame-png` plays a
script in the game's test Harness, and `lang-status` needs the game's
languages, whose source text (the data's names, the dialogue) only the
game can gather. The engine's tools must know no game (ADR-0063 §4), and
Rust has no way for a tool to load a game's code at run time.

Since ticket 1110 the web shell (`gamepad.js` and the page that loads it)
served the engine's `shuyi-app`, but lived in the game's `web/`.

## Decision

1. **A new engine crate, `shuyi-cli`** (`engine/crates/cli`), with the
   binary `shuyi`. Each command is a module of library functions; `lib.rs`
   holds the thin wrappers and the dispatch (`run`), `main.rs` only calls
   `main_with`. `cargo shuyi <command>` (an alias in `.cargo/config.toml`)
   runs it.
2. **A project file, `shuyi.ron`, at a game's root.** `shuyi` runs on the
   project of the first folder from the working folder up that holds one.
   It gives, per command: the paths each check scans and lets through
   (`check_keys`, `check_text`, with their hints), the glyphs an atlas
   must have (`font_atlas`), the web build's package, binary, title,
   features and folders (`web`), the private repository's settings
   (`private_assets`), the game's tool program (`game`) and what the
   project calls the tool in what it prints (`command`). RON with
   `implicit_some` on, so a part is written as it is. A command whose part
   is missing says so and exits 2.
3. **The game's tool program.** The commands that run the game take a
   `GameTools`: `frame(options, script, scale)` starts the game's Harness
   with the game's own options, plays the engine's `Script` of keys,
   buttons and waits, and paints the frame (with the engine's `Painter`,
   or the one-bit renderer and `one_bit_picture`); `lang()` gives the
   game's `Lang` and its line variants. A game gives them in a program of
   its own that calls `shuyi_cli::main_with(args, Some(&tools))`;
   `shuyi.ron`'s `game` names it (package, binary or example, features),
   and `shuyi` runs it with `cargo run` for `frame-png` and `lang-status`.
   `frame-png`'s generic options are the output, the steps and `--scale`;
   a game adds its own (`GameOption`, here `--layout`, `--playdate`,
   `--map`, `--prompt`, `--tips`, `--light`), which `shuyi` parses and
   passes through in order.
4. **This game's tool program is xtask** (`crates/xtask/src/game_tools.rs`,
   `Trpg`). `cargo xtask` keeps the repository's own commands
   (`ticket-lint`, `clean-merged-targets`, `check-engine-boundary`, the
   bought-art imports, `title-art`, `effect-marks`, `playtest`,
   `test-tileset`, `sfx`, `icon`, `playdate`, `lines`, `check-script`, …)
   and forwards the moved commands' old names to `shuyi_cli::run` with
   `Trpg`, so old habits and scripts keep working.
5. **The web shell is the engine's**, built into `shuyi`
   (`engine/crates/cli/web/`, `include_str!`): the page is a template
   (`%TITLE%`, `%WASM%`) filled from `shuyi.ron`. The game's `web/` folder
   is gone. The `--allow-undefined` link argument stays in the game's
   binary's `build.rs`: a link argument reaches a binary only from its own
   package's build script; the shell's README shows the four lines.
6. **The toy game is a project too.** `engine/crates/cli/toy/shuyi.ron`
   names the crate's `toy` example (`shuyi_cli::toy::ToyTools`, the `toy`
   feature) as its tool program; the toy has no font, so its pictures draw
   letters as boxes.

## Consequences

- Each moved command gives the same output on this repository, except
  that the usage lines and `atlas.ron`'s header name `cargo shuyi` (the
  project's `command`). Pictures are byte for byte the same.
- The game's required glyphs are written in `shuyi.ron` and in
  `trpg_content::font::REQUIRED_GLYPHS`; an xtask test keeps them equal,
  as one does the credits file's paths.
- The test card's pixels moved with `frame-png`'s tests
  (`shuyi_cli::test_card`); `cargo xtask test-card` still writes the file.
- `lines` and `check-script` stay in xtask: they read the game's dialogue
  with its directives, which `GameTools` doesn't give yet.
- `new`, `doctor` and the JSON contract (1129) and MCP (1133) build on
  this crate in the engine's repository.
