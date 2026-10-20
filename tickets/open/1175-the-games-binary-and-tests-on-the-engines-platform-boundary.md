---
id: "1175"
title: "The game's binary and tests on the engine's platform boundary: main builds a Start, the Harness comes from shuyi-platform"
type: feature
milestone: "Engine: carve"
model: sonnet-5
effort: medium
status: todo
blocked_by: ["engine#3003"]
nick_input: none
completed:
---

# 1175 — The game's binary and tests on the engine's platform boundary

## Context

Engine ticket 3003
([`3003-the-platform-boundary-shuyi-platform-and-one-frame-driver.md`](https://github.com/Zafnok/shuyi-engine/blob/main/tickets/done/3003-the-platform-boundary-shuyi-platform-and-one-frame-driver.md),
engine ADR-0024) moved the run loop out of `shuyi-app` into a new crate,
`shuyi-platform`: its `App` runs one frame of the game for each
backend's own loop (macroquad's now, the Playdate SDK's update callback
with engine ticket 3010), with a loading card first, fixed steps, and
suspend and resume. `shuyi-app` stays the game's desktop and web
backend, but how a game starts on it changed, and the test Harness moved
with the loop. The game's code builds on none of this until its pin
moves past 3003, and **that pin move breaks the build without this
ticket's changes**.

What changed for the game (engine ADR-0024, *Consequences*):

- `shuyi_app::run(config, make)` is `shuyi_app::run(config, start)`,
  where `start` is a `shuyi_platform::Start<Shuyi>`: the loading card
  (`Card::text("…")` or a picture from the bundle), the load (a closure
  from `Setup` to `Result<Game, String>`), and the look
  (`fn(&Shuyi) -> Look<'_>`, the font atlas and image table the frame is
  painted with).
- `AppConfig` has no type parameter, no `font` and no `images` (they are
  the look now), and a new `text_input: Option<Box<dyn TextInput>>`
  (the system keyboard; Steam's is 0903's, so `None` here).
- `Platform::setup(ctx)` is `Setup::ctx(ctx)`: the same storage, voice
  manifest, idle motion, key prompt and music seed.
- The Harness is `shuyi_platform::harness::Harness` (the `harness`
  feature of `shuyi-platform`); `shuyi_ui::harness` and `shuyi-ui`'s
  `harness` feature are gone. Its script API is the same.
- Closing the window now asks the game first (the engine sends
  `QuitRequested`), then quits: one frame later than before.

No engine-friction review: a change of the engine's API.

## Nick input

None.

## Scope

**In:**
- Move the pin past engine#3003 (`engine-change` skill, step 7). Add
  `shuyi-platform` to the workspace's dependencies at the same revision
  as the other engine crates, and to `crates/xtask/src/engine.rs`'s
  `CRATES` (`("shuyi-platform", "crates/platform")`), so `cargo xtask
  engine`'s patch covers it.
- `crates/app/src/main.rs`: `config()` returns `AppConfig` without
  `font` and `images` and with `text_input: None`; `font_atlas` and
  `images` become one `look(game: &Shuyi) -> Look<'_>`; `start(platform:
  &Platform)` becomes `start(setup: Setup) -> Result<Game, String>`, with
  `setup.ctx(ctx)`; `main` builds `Start::new(Card::text("Visions of
  Shuyi"), look, start)` and calls `shuyi_app::run(config(), start)`.
  The card is the game's name in white on black, as the Playdate's
  loading card (ticket 1036).
- `crates/ui`: the `harness` feature turns on `shuyi-platform/harness`
  instead of `shuyi-ui/harness` (`shuyi-platform` an optional
  dependency the feature enables); `src/harness.rs` uses
  `shuyi_platform::harness` (its `Core` alias too); `README.md` and the
  doc comment say so.

**Out (do not do):**
- The Playdate front end (`crates/playdate`): it keeps driving `Game`
  itself until 1170 moves it onto the engine's backend, which runs this
  `App`. `Game::frame` is unchanged.
- A hook that saves on suspend or on close (`Start::on_system`): the
  game saves where it saves today. If Nick wants a save on close, that
  is a ticket of its own.
- The Steam keyboard (`AppConfig::text_input`): 0903.

**Either order:** whichever game PR first moves the pin past engine#3003
does this ticket's work, in this ticket's own PR merged first or inside
that PR (which then closes this ticket in its completion notes).

## Implementation steps

1. Write the estimate: about 40 lines changed, none added net.
2. `cargo xtask engine-pin <commit>` at a commit with 3003 in it; add
   `shuyi-platform` to `Cargo.toml` and to `engine.rs`'s `CRATES`.
3. The `main.rs` change above; `cargo run -p trpg-app` shows "Visions of
   Shuyi" for a moment, then the title; closing the window quits.
4. The Harness's imports and the feature.
5. `cargo xtask engine-pin --check`, then the gates.

## Acceptance criteria

- [ ] `grep -rn "shuyi_ui::harness" crates` finds nothing.
- [ ] `grep -n "shuyi-platform" Cargo.toml crates/xtask/src/engine.rs`
      finds the dependency and the `CRATES` entry.
- [ ] `cargo xtask engine-pin --check` passes.
- [ ] The game starts on desktop and in the web build, shows its card,
      then its title, and quits from the title and from the window's close
      button (manual).
- [ ] All gates in the `run-gates` skill pass, with no test changed but
      its imports.

## Tests required

- The existing Harness, snapshot and replay tests, unchanged: they run
  on the engine's headless backend now.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
