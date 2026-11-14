---
id: "1229"
title: "The exe's icon and version resource from the engine's Windows export"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#3049"]
nick_input: none
completed:
---

# 1229 — The exe's icon and version resource from the engine

## Context

Ticket 0902 gave the exe its icon and version info on the game's side:
`cargo xtask icon build` writes `assets/icon/icon.ico` with its own
PNG-in-ICO writer (`fn ico` in `crates/xtask/src/icon.rs`, lines
114-130, at `ICO_SIZES` 16/32/48/256), and `crates/app/build.rs`
(`embed_windows_resources`, lines 22-44) compiles it with the
`winresource` build dependency, with `FileDescription`, `ProductName`,
`InternalName` and `OriginalFilename` typed out again beside
`shuyi.ron`'s `app.name`. None of it is the game's own.

The game/engine boundary audit of 2026-11-03 found that
[engine#3049](https://github.com/Zafnok/shuyi-engine/blob/main/tickets/open/3049-the-games-identity-in-every-package.md)
writes the same into every `shuyi export windows` package: an `app.ico`
from `shuyi.ron`'s `app.icon` (16 to 256 px), a `VERSIONINFO` from the
`app` part, compiled by `windres` and linked into the game's binary
only. The game's release already exports with `cargo shuyi export`
(`.github/workflows/release.yml`), and `scripts/check-release-package.sh`
already refuses an exe with no `.rsrc`. Two resource sections would
clash at link time, so the game's own goes in the same PR as the pin
move.

The rest of `icon.rs` stays: the glyph designs, `preview`, and the
window's PNGs (`assets-src/icon/`, ticket 1201) that `app.icon` and
`app.icon_sizes` read.

## Nick input

None. The icon and the name in Explorer stay the same; verified by the
release package check and the engine's resource test.

## Scope

**In:**
- Move the pin past engine#3049 (`engine-change` skill §7, the
  `engine-pin` skill), unless an open `engine-pin` PR has done so.
- `crates/xtask/src/icon.rs`: `fn ico`, `ICO_SIZES`, the `icon.ico`
  write in `build` and the `ico_header_points_at_the_pictures` test go;
  `every_option_draws_at_every_size` keeps the sizes the PNGs need.
- `assets/icon/icon.ico` goes, with its lines in
  `crates/content/playdate-bundle.txt` and the bundle test that names it
  (`crates/content/src/bundle.rs`, around line 398).
- `crates/app/build.rs`: `embed_windows_resources` (both `cfg`s) goes;
  its wasm `--allow-undefined` line stays. The `winresource` build
  dependency goes from `crates/app/Cargo.toml` and `Cargo.lock`.
- The module docs of `icon.rs` and `build.rs`, and any doc that says
  where the exe's icon comes from, name the engine's export.

**Out (do not do):**
- A new icon design, or dropping `preview` and the unused options.
- A `publisher` or `build` in `shuyi.ron`'s `app` part (the version
  stays the package's, as today).
- macOS, Linux and web identity files: 3049 adds them to every package;
  nothing to do here.

**Either order with 1233:** if 1233 is done, its xtask `icon` row
(`docs/engine/boundary.md:269`) says "moves with 1229": write it as
moved; if not, change that row here, and 1233 leaves it.

## Implementation steps

1. Move the pin (as above). Read 3049's completion notes: the `.ico`
   sizes, whether its 16 and 32 px pictures come from `app.icon_sizes`,
   and the `windres` it needs (MSYS2's binutils, already in
   `release.yml`'s Windows setup).
2. Delete the pieces listed under **In**.
3. Run `cargo shuyi export windows --release` and
   `bash scripts/check-release-package.sh windows dist/export/windows`;
   look at the exe's Details tab in Explorer (a screenshot in the PR).
4. Measure with `cargo xtask code-lines`: about 45 production lines and
   15 test lines removed, none added.

## Acceptance criteria

- [ ] `shuyi-engine.rev` is at or past the commit that closed
      engine#3049; `cargo xtask ticket-lint --check-engine` passes.
- [ ] `grep -rn "winresource\|icon.ico\|fn ico" crates assets` finds
      nothing.
- [ ] The exported exe has an `.rsrc` section, shows the icon and
      "Visions of Shuyi" in Explorer, and the release package check
      passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `icon.rs`'s remaining drawing tests; the bundle's Playdate list.
- Integration: the release package check on an exported Windows package.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
