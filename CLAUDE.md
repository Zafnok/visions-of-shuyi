# CLAUDE.md

ASCII-art tactical RPG (Fire Emblem-like, drawn with coloured glyphs like Dwarf
Fortress / Rogue), keyboard-driven with a virtual cursor and vim-style keys.
Rust. Windows exe first, plus web (WASM); itch.io then Steam.

## Who decides what

- **Nick (owner)** decides *game design*: stats, combat, magic, progression,
  story beats, difficulty, look & feel. He does **not** review code and does
  **not** want technical questions. Ask him design questions only via the
  `ask-nick` skill (options drawn from real games + "describe your own").
- **Claude** decides everything technical and records significant choices as
  ADRs in `docs/adr/`.

## Hard rules

1. **Work one ticket at a time** from `tickets/open/`, using the `work-ticket`
   skill. One ticket = one branch = one PR. The PR moves the ticket to
   `tickets/done/`. One exception: a GitHub issue labelled `sonar` is its
   own ticket, filed weekly from SonarCloud and fixed with the `sonar-fix`
   skill (ADR-0071).
2. **No unticketed work.** Don't write game code "to get a head start". Found
   something else to do? Create a ticket (`write-ticket` skill).
3. **Never invent game-design answers.** They come from `docs/design/` (decided
   by `00xx` tickets) and `docs/story/beats.md`. If one is missing, the ticket
   is blocked; say so.
4. **Respect crate boundaries** (ADR-0004): `core` is pure and deterministic
   (no I/O, no clock, no macroquad); state changes only via `Command` → `Event`s;
   only `app` touches macroquad, files, clock, keyboard.
5. **Tests are the review** (ADR-0007). Run the `run-gates` skill before
   pushing. Never weaken a gate to pass it.
6. **Licensing** (ADR-0013): the game is proprietary (see `LICENSE`) and will
   be sold. Only ship dependencies and assets under the permissive licenses
   allowed by ADR-0013: no GPL/LGPL/MPL/copyleft, no non-commercial, nothing that
   costs money. Every non-crate asset goes in `THIRD_PARTY_ASSETS.md`. Never
   change `LICENSE`; that is Nick's call.
7. **Never hard-code a key.** Game code reacts to `Action`s and text names
   keys through the player's keymap; players rebind everything. Follow the
   `keyboard-input` skill whenever input or key names are involved.
8. **Graphics are a skin** (ADR-0038): glyphs today, sprites from image
   files later, and swapping them must stay a small job. `core`, the bots
   and play records know nothing about the look. A picture is a sprite
   item from an asset file, never per-pixel rectangles or cells. Anything
   shown on the battle map goes in the map scene and is painted by the map
   skins, not drawn straight into the buffer. Tests of what happened read
   the scene or the state; only tests of a look read cells and colours.
   Screens follow the same split (ADR-0054): a screen's logic builds a
   view (plain data) and a skin paints it, as the engine's
   `screens/options.rs` does;
   a new screen never decides its look in `draw`.
9. **Code far bigger than the job is a signal** (ADR-0092): write how
   many lines a change should take before coding, measure it with
   `cargo xtask code-lines`, and at 3× or more run the `engine-friction`
   skill (engine feature, engine ergonomics, game-side structure, or
   accept); the engine never names a game concept, and the game never
   waits on the engine.

## Map of the repo

| Path | What |
| ---- | ---- |
| `tickets/` | Backlog. `README.md` explains numbering & lifecycle. `open/` → `done/` |
| `docs/ROADMAP.md` | Milestones and the Chapter 1 critical path |
| `docs/adr/` | Technical decisions (read the index) |
| `LICENSE`, `THIRD_PARTY_ASSETS.md` | Proprietary source-available license; registry of shipped third-party assets |
| `docs/design/` | Nick's game-design decisions (filled by `00xx` tickets) |
| `docs/story/` | Story beats, bible, characters, outline, ledger (ADR-0011) |
| `.claude/skills/` | `work-ticket`, `write-ticket`, `write-adr`, `ask-nick`, `story-writing`, `ascii-art`, `run-gates`, `keyboard-input`, `sonar-triage`, `sonar-fix`, `engine-friction` |
| `crates/` | `core` (`trpg-core`), `content` (`trpg-content`), `ui` (`trpg-ui`), `app` (`trpg-app`, binary `visions-of-shuyi`), `bots` (`trpg-bots`, playtest bots: dev tooling, never in the game), `xtask` (the repository's own tooling, and the game's tool program for `cargo shuyi`) |
| `shuyi.ron` | The project file of Shuyi's tools (`cargo shuyi <command>`: `frame-png`, `font-atlas`, `check-keys`, `check-text`, `lang-status`, `web`, `private-assets`; ADR-0089): the paths each check scans, the web build, the private repository |
| `crates/playdate` | `trpg-playdate`, the Playdate front end (ADR-0070): a second platform crate beside `app`, a `staticlib` linked with `shim/shim.c` by `cargo xtask playdate` into `target/playdate/VisionsOfShuyi.pdx`. Needs the Playdate SDK and MSYS2's ARM compiler (its `README.md`) |
| `assets/` | Everything embedded in the game: `data/`, `fonts/` (later: maps, dialogue, portraits) |
| `assets-test/` | The **test files**: test characters, the test chapter, the debug Quick Battle, test tilesets, the `test` language pack. Laid out like `assets/`, embedded only in debug builds and builds with the debug tools, never in a build that ships (ADR-0057) |
| `tools/sonar-triage/` | Files SonarCloud's backlog as batched `sonar` GitHub issues, weekly (ADR-0071); `policy.jsonc` is where a finding is accepted |
| `assets-src/` | Inputs to asset tools (e.g. the font BDF for `cargo shuyi font-atlas`); not embedded |
| `assets-private/` | **Git-ignored**: the checkout of the private repository with the bought art (ADR-0040). `game/` is embedded over `assets/` by the `private-assets` feature; `library/tiny-tales/` is the bought bundle, sorted. `assets-private.rev` (tracked) names the commit to build with |
| `engine/` | Shuyi Engine's code (its tools, `shuyi-cli`, in `engine/crates/cli`) while it is carved out of the game (ADR-0063, tickets 1102–1118 and 1154); moves to the engine's own repository, [`Zafnok/shuyi-engine`](https://github.com/Zafnok/shuyi-engine) (public, MIT OR Apache-2.0), with 1131. That repository holds the engine's ADRs, tickets and plans |
| `voice/` | **Git-ignored**: voice clips by dialogue line id, shipped beside the game, never embedded (ADR-0046, `docs/voice.md`). Copied from the private repository's `voice/` by `cargo shuyi private-assets`. Test clips: `crates/app/tests/voice/` |

## Environment

- Nick's machine: Windows 11, Rust with the **GNU** host toolchain; **no MSVC**.
  Never require MSVC locally. CI builds with MSVC on GitHub runners.
- Local builds need a full MSYS2 mingw-w64 toolchain (`winget install -e --id
  MSYS2.MSYS2` then `pacman -S mingw-w64-x86_64-gcc`): rustup's self-contained
  GNU linker ships without some import libraries (e.g. `imm32`, needed by
  macroquad). `.cargo/config.toml` points the `x86_64-pc-windows-gnu` linker
  at the MSYS2 install.
- Use `cargo install --locked <tool>` (cargo-binstall fails to build here).
- `cargo xtask clean-merged-targets [--dry-run]` deletes the `target/` build
  folder of every worktree whose PR has merged (10+ GB each).
- **Bought art** (ADR-0040): the files are never in this repository, and
  neither is a picture that hands the art out (a sheet, a cut-out sprite, a
  mockup pasted together from the files). A screenshot of the running game
  that shows the art may be committed (Nick, 2026-10-05: the README and the
  store pages need them; "we just don't distribute the raw files").
  `cargo shuyi private-assets` checks the private repository out into
  `assets-private/` (only `game/`; add `--library` for the bought packs in
  `assets-private/library/`). Gates never read it. To see it in a build:
  `cargo run -p trpg-app --features private-assets`. After pushing a change
  to it, run `cargo shuyi private-assets --pin` and commit
  `assets-private.rev` in the same PR. On Nick's machine the full copy
  (with the character generators and the original zips) is
  `D:\tactical-rpg\assets-private\`.
- Git remote: `https://github.com/Zafnok/visions-of-shuyi` (public).
- **Never write a bare `@word` on GitHub** (PR title or body, commit message,
  comment, release notes): GitHub turns it into a mention of that user, and
  the generated release notes then list them as a contributor (it happened
  in v0.1.0 with `@music`, a dialogue directive). Put it in backticks or
  drop the `@`. Only mention a real person on purpose.
