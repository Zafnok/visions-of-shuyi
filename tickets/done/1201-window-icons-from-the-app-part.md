---
id: "1201"
title: "The window's icons from shuyi.ron's app part, drawn per size"
type: infra
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: ["1199", "engine#3113"]
nick_input: none
completed: 2026-11-01
---

# 1201 — The window's icons from shuyi.ron's app part

## Context

1199 put the game's name, id, version and storage name in `shuyi.ron`'s
`app`, but not its icon. The engine makes every window icon by box-filtering
one 512 px picture (engine ADR-0061), and that blurs the game's `@` at 16 and
32 px: 64 of 256 and 133 of 1024 pixels differ, and the 16 px glyph can't be
read. So `crates/app/src/main.rs` still includes `assets/icon/icon-{16,32,64}.png`,
marked `// engine-friction: engine#3113`. Engine PR #210 has now shipped
that ticket and lets a game give pictures drawn per size. The pin move of
2026-11-01 reached it; `engine#3113` stays as the minimum revision. Game
1199 is already done.

## Nick input

None. The icon's picture doesn't change.

## Scope

**In:**
- ~~Move the pin past engine#3113 as the first step (`engine-change` §7), or
  wait for an open `engine-pin` PR.~~ (done by the pin move of 2026-11-01)
- `icon`, `icon_sizes` and `window_icons` in `shuyi.ron`'s `app`, with
  `window_icons: "crates/app/icon"`, outside `assets/`, which is embedded
  whole.
- `WINDOW` takes `APP.window_icon`; the `engine-friction` marker and the
  comment in `shuyi.ron` go.
- `cargo xtask icon build` writes the 512 px source (`assets-src/icon/icon.png`)
  and the drawn pictures where `icon_sizes` names them.

**Out (do not do):**
- Changing the icon's picture.
- The exe's `icon.ico`, which `cargo xtask icon` keeps writing: a package's
  icon is the engine's 3049.

## Implementation steps

1. ~~Move the pin.~~ (done by the pin move of 2026-11-01)
2. Have `cargo xtask icon build 1` write the source and the drawn pictures,
   and set the `app` part's three fields.
3. Run `cargo shuyi app-info`, and use `APP.window_icon` in `main.rs`.
4. Delete `assets/icon/icon-{16,32,64}.png` if nothing else reads them.

Expected size: about 20 production lines changed plus override/staleness
checks; under 700 changed lines including docs and the ticket move.

## Acceptance criteria

- [x] `crates/app/icon/{16,32,64}.png` have the same pixels as the
      `assets/icon/` pictures they replace.
- [x] `the_window_has_an_icon` passes, with `WINDOW.icon` equal to
      `APP.window_icon`.
- [x] `cargo shuyi app-info --check` and `cargo shuyi check --target all` pass.
- [x] Old direct window-icon includes and the engine-friction marker are
      gone. Missing/incorrect-sized override files fail generation; stale
      generated icons fail `app-info --check`. The exe `.ico` generator
      and output remain intact.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `WINDOW.icon` equals `APP.window_icon` in `crates/app/src/tests.rs`.
- Integration: override pixel equality, missing/invalid-size input refusal
  and stale generated output refusal through the existing engine commands.

## Completion notes

Pin: unchanged, 76e05909, already past engine#3113.

Pixel equality: `crates/app/icon/{16,32,64}.png` were byte-identical to the
deleted `assets/icon/icon-{16,32,64}.png` (compared with `cmp` before the
delete). `the_windows_icons_are_drawn_at_each_size` keeps them equal to the
hero option as drawn at each size. `assets/icon/icon.ico` came out unchanged.

Refusals: `app_info_refuses_bad_or_stale_icons` runs the engine's `app-info`
on `shuyi.ron`'s real `app` part in a temporary folder: a 17 px and a missing
drawn picture are refused, and `--check` names a changed window icon.

Deviation: `icon_sizes` names only 16 and 32, not 64. 64 is a whole fraction
of the 512 px source, so the engine's downscale already gives those exact
pixels and a third drawn file would add nothing. The test above would fail if
that stopped being true.

No gameplay rules decided and no follow-ups. The picture is unchanged, so
nothing is left to judge by playing.

