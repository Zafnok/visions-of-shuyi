---
id: "1016"
title: "vos-modkit: check a mod and start a new one, outside the game"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: medium
status: todo
blocked_by: ["1013", "1014"]
nick_input: none
completed:
---

# 1016 — `vos-modkit`: check a mod, start a new one

## Context

Modders need to know what's wrong with their mod without launching the
game and reading a log. The game already validates all content on load and
reports every error (`trpg_content::load_embedded`, ADR-0005). 1011's ADR
puts the modder's tools in a separate binary, `vos-modkit`, in a new
`crates/modkit` crate, shipped beside the game (never linked into it).

## Nick input

None.

## Scope

**In:**
- `crates/modkit` (binary `vos-modkit`), depending on `trpg-content` and
  `trpg-core`, never on macroquad.
- `vos-modkit check <mod folder> [more mods in load order]`: loads the game
  data with those mods layered on top (1012, 1013), prints every error with
  its file and line, exits non-zero on errors. Also checks the manifest and
  the order (1014's `resolve_order`).
- `vos-modkit new <id>`: makes a mod folder with a filled-in `mod.ron`, an
  empty `data/` and a README that links the modding guide (1023).
- Messages a non-programmer can act on: say which file, which line, what
  was expected, and an example of a correct line where the loader has one.
- The release workflow (0107) puts `vos-modkit` next to the game in the
  Windows zip.

**Out (do not do):** a GUI; `playtest` (1021); Workshop upload (1024);
anything that reads `assets-private/` (the tool runs on the public build;
it never exports art).

## Implementation steps

1. Read 1011's ADR, `crates/content/src/lib.rs`, `error.rs` and 0723's
   script checker (if done) for the message style.
2. Add the crate to the workspace (`publish = false`, `license-file`, the
   workspace lints). No new dependency beyond what the ADR allows; parse
   arguments by hand unless a permissive crate is already in the tree.
3. `check` reads the mod folders from disk (the tool may do file I/O; it
   isn't `content`), builds `ModLayer`s and calls `install_mod_layers` then
   `load_embedded`.
4. `new` writes the template files; refuse to overwrite.
5. Add the binary to `.github/workflows/release.yml`'s Windows package.

## Acceptance criteria

- [ ] `vos-modkit check` on a correct test mod exits 0 and prints "OK"
      (integration test running the binary, `assert_cmd`-style with only
      std, or calling its `run(args)` function).
- [ ] On a broken test mod it exits non-zero and every message names the
      mod file and line (test).
- [ ] `vos-modkit new demo` creates a mod that `check` accepts (test).
- [ ] The release package contains `vos-modkit.exe` (workflow change,
      checked on the next release).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: argument parsing, the template.
- Integration: check and new on test mods under `crates/modkit/tests/`.

## Completion notes
