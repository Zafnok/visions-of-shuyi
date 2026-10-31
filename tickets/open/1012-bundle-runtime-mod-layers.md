---
id: "1012"
title: "Content reads mod files: runtime layers in the asset bundle"
type: feature
milestone: Post–Chapter 1
model: reasoning
effort: medium
status: todo
blocked_by: ["1011", "engine#2207"]
nick_input: none
completed:
---

# 1012 — Content reads mod files: runtime layers in the asset bundle

## Context

1011's ADR decides how mods are layered over the game's files. This ticket
builds the part inside `trpg-content`: the bundle takes extra layers at
runtime, from bytes `app` hands it, without `content` doing any file I/O
(ADR-0004). It does **not** read a mods folder (1014) or merge data tables
entry by entry (1013): a mod file here simply replaces the file at the same
path, like the private art does today (ADR-0040).

**The layers are the engine's: engine#2207** (in
[Shuyi Engine's tickets](https://github.com/Zafnok/shuyi-engine/tree/main/tickets)).
Since the carve (1103) the bundle's layering is Shuyi Engine's: a
`Bundle` of `Layer`s read in order, installed once, with `&'static`
reads. `crates/content/src/bundle.rs` gives it the game's layers
(`layers()`: the private art over `assets/`, then the test files),
installs them (`install()`) and keeps the functions every loader calls
(`file`, `bytes`, `files_in`, `files_under`, `display_path`). 2207 adds a
layer of files handed in at run time (owned paths and bytes, kept for the
program's life, with path checks and its own shown name) and a way for
the platform shell to read a folder on disk into one. This ticket is the
game's half: putting mod layers in the game's bundle, in order.

The defaults below are 1011's; if the ADR chose otherwise, follow it.

## Nick input

None.

## Scope

**In:**
- `pub fn install_mod_layers(layers: Vec<ModLayer>) -> Result<(), ...>`
  in `trpg_content::bundle`: called at most once, before the first read;
  later calls are an error. Mod layers come first (the last mod in the
  list is read first), then the private art, then `assets/`, then the
  test files.
- Each mod is an engine#2207 layer shown as `mods/<id>`, so a path read
  from a mod shows as `mods/<id>/<path>` in `display_path` and every load
  error already names the mod.
- Every existing function sees the mod layers.

**Out (do not do):** the engine's half (engine#2207); reading files from
disk (1014); merging tables by entry (1013); the mods screen (1015);
scripts (1017).

## Implementation steps

1. Move the pin to an engine `main` that has engine#2207 (`cargo xtask
   engine-pin <commit>`, the `engine-change` skill §7).
2. Read ADR-0004, ADR-0005, ADR-0040, 1011's ADR and `bundle.rs` with its
   tests.
3. `ModLayer { id: String, files: BTreeMap<String, Vec<u8>> }` (paths
   relative to the mod root, `/`-separated) becomes the engine's run-time
   layer named `mods/<id>`. The engine refuses bad paths (absolute, `..`,
   backslashes); the game also refuses empty files under `data/`, with a
   clear error naming the mod and the path.
4. `install_mod_layers` keeps the mod layers for `layers()` and fails
   once the bundle is installed (the first read installs it).
5. Tests can't install a global layer and then un-install it, so test
   the order on bundles built by hand (`Bundle::new` over hand-made
   run-time layers and an embedded one), and the once-only rule in a way
   that doesn't touch the global in other tests.

## Acceptance criteria

- [ ] A mod's file replaces an embedded file at the same path, and a
      mod-only file is added (unit tests on reads and listings).
- [ ] The last of two mods wins for a path both have (unit test).
- [ ] `display_path` shows `mods/<id>/...` for a file read from a mod
      (unit test).
- [ ] A second `install_mod_layers` call, or one after the first read,
      returns an error (unit test in a separate test binary, or a
      documented `#[ignore]`-free approach that doesn't touch the global
      in other tests).
- [ ] Empty files under `data/` are refused with the mod id and path in
      the message (unit tests).
- [ ] Without mods, every existing test passes unchanged.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the game's layer order with mod layers; the game's own path rule.
- Property: for any set of mod layers, a listing is sorted and
  de-duplicated and a read agrees with it.

## Completion notes
