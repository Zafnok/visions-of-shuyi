---
id: "1199"
title: "The game's identity in shuyi.ron: name, id, version, storage and icon, read from a generated APP"
type: infra
milestone: M8 Release
model: implementation
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1199 — The game's identity in shuyi.ron

## Context

Engine PRs #186 (ticket 3048) and #188 (3072), reached by the pin move of
2026-10-30, give `shuyi.ron` an `app` part: the game's name, a reverse-DNS
id, version, build, publisher, storage name and one square icon. `cargo
shuyi app-info` writes it as a `const APP: AppInfo` and the 16, 32 and 64 px
window icons, `--check` fails when they are stale, and `check --target`
checks the part. `shuyi export` takes its version from it.

The game has its identity by hand in `crates/app/src/main.rs` (`WINDOW`'s
title and icons, `STORAGE_NAME`), and its own icon tool, `cargo xtask icon`
(ticket 0902). With no `app` part, `check --target` only prints a `limits`
line, so nothing checks any of this. A store build (0901, 0903) needs an id
and a version in one place.

## Nick input

None. The name stays "Visions of Shuyi".

## Scope

**In:**
- An `app` part in `shuyi.ron`, its `code` and `window_icons` in `crates/app`.
  The id is the agent's choice, not under `com.example.`.
- **`storage: "tactical-rpg"`, exactly**: any other name loses every player's
  saves (the comment on `STORAGE_NAME`).
- `crates/app/src/main.rs` reads the title, the icons and the storage name
  from `APP`, and `WINDOW` and `STORAGE_NAME` lose their literals.
- `cargo shuyi app-info --check` joins the `run-gates` list and CI.
- The version kept equal to the package's (`CARGO_PKG_VERSION`, what
  `release.yml` tags): a test or a check says so.

**Out (do not do):**
- Moving to `BasicGame` or `run_game`: the game keeps its own start.
- Changing the icon's picture. If `assets/icon/`'s PNGs aren't what the
  engine's downscale makes from the 512 px source, keep the look: compare the
  frames, and say so in the PR.
- The Playdate's `pdxinfo` (1052).

## Implementation steps

1. Add the `app` part, run `cargo shuyi app-info`, commit what it writes.
2. Use `APP` in `crates/app/src/main.rs`; `crates/app/src/tests.rs` keeps
   checking the title and the storage name, as literals.
3. Delete `cargo xtask icon` and its tests if `app-info` now does its job;
   otherwise say why it stays.
4. Add the gate, in `run-gates` and `.github/workflows/ci.yml`.

Expected size: about 40 production lines changed, and about 30 to 100 removed
with `cargo xtask icon`.

## Acceptance criteria

- [ ] `cargo shuyi check --target desktop` reports the part with no problem
      and no `limits` line about it.
- [ ] The window's title and icon are as before, and a save made by the
      previous build still loads (storage name unchanged, tested).
- [ ] `cargo shuyi app-info --check` passes, and is in CI.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `APP`'s name and storage name equal the literals players' saves depend
  on; its version equals the package's.
