---
id: "1201"
title: "The window's icons from shuyi.ron's app part, drawn per size"
type: infra
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: ["1199", "engine#3113"]
nick_input: none
completed:
---

# 1201 — The window's icons from shuyi.ron's app part

## Context

1199 put the game's name, id, version and storage name in `shuyi.ron`'s
`app`, but not its icon. The engine makes every window icon by box-filtering
one 512 px picture (engine ADR-0061), and that blurs the game's `@` at 16 and
32 px: 64 of 256 and 133 of 1024 pixels differ, and the 16 px glyph can't be
read. So `crates/app/src/main.rs` still includes `assets/icon/icon-{16,32,64}.png`,
marked `// engine-friction: engine#3113`. That engine ticket lets a game give
pictures drawn per size.

## Nick input

None. The icon's picture doesn't change.

## Scope

**In:**
- Move the pin past engine#3113 as the first step (`engine-change` §7), or
  wait for an open `engine-pin` PR.
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

1. Move the pin.
2. Have `cargo xtask icon build 1` write the source and the drawn pictures,
   and set the `app` part's three fields.
3. Run `cargo shuyi app-info`, and use `APP.window_icon` in `main.rs`.
4. Delete `assets/icon/icon-{16,32,64}.png` if nothing else reads them.

Expected size: about 20 production lines changed.

## Acceptance criteria

- [ ] `crates/app/icon/{16,32,64}.png` have the same pixels as the
      `assets/icon/` pictures they replace.
- [ ] `the_window_has_an_icon` passes, with `WINDOW.icon` equal to
      `APP.window_icon`.
- [ ] `cargo shuyi app-info --check` and `cargo shuyi check --target all` pass.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `WINDOW.icon` equals `APP.window_icon` in `crates/app/src/tests.rs`.

## Completion notes

