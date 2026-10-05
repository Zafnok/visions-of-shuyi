---
id: "1012"
title: "Content reads mod files: runtime layers in the asset bundle"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: ["1011"]
nick_input: none
completed:
---

# 1012 — Content reads mod files: runtime layers in the asset bundle

## Context

1011's ADR decides how mods are layered over the game's files. This ticket
builds the part inside `trpg-content`: the bundle can take extra layers at
runtime, from bytes `app` hands it, without `content` doing any file I/O
(ADR-0004). It does **not** read a mods folder (1014) or merge data tables
entry by entry (1013): a mod file here simply replaces the file at the same
path, like the private art does today (ADR-0040).

Today `crates/content/src/bundle.rs` has a `static LAYERS: &[Layer]`, each
`Layer` holding a `display_root` and an embedded `&'static Dir`. `file`,
`bytes`, `files_in`, `files_under` and `display_path` read the first layer
that has a path.

The defaults below are 1011's; if the ADR chose otherwise, follow it.

## Nick input

None.

## Scope

**In:**
- A `Layer` that is either embedded (`include_dir`) or runtime (a sorted
  map of path → bytes, with its own display root such as
  `mods/my_mod`).
- `pub fn install_mod_layers(layers: Vec<ModLayer>) -> Result<(), ...>`:
  called at most once, before the first read; later calls are an error.
  Mod layers come first (the last mod in the list is read first), then the
  private art, then `assets/`. The bytes are leaked once so every read
  stays `&'static`.
- Every existing function sees the mod layers.

**Out (do not do):** reading files from disk (1014); merging tables by
entry (1013); the mods screen (1015); scripts (1017).

## Implementation steps

1. Read ADR-0004, ADR-0005, ADR-0040, 1011's ADR and `bundle.rs` with its
   tests.
2. Replace `static LAYERS` with a `OnceLock<Vec<Layer>>` built on first use
   from the embedded layers plus whatever `install_mod_layers` set before
   that. Make `Layer` an enum (`Embedded { display_root, dir }`,
   `Runtime { display_root: &'static str, files: BTreeMap<&'static str,
   &'static [u8]> }`) with `get_file`, `files_in_dir` and `contains_dir`
   helpers so `find` and `list` work over both.
3. `ModLayer { id: String, files: BTreeMap<String, Vec<u8>> }` (paths
   relative to the mod root, `/`-separated). Reject at install: absolute
   paths, `..`, backslashes, empty files under `data/` (each with a clear
   error naming the mod and the path).
4. Keep the existing functions' signatures. A path read from a mod shows as
   `mods/<id>/<path>` in `display_path`, so every load error already names
   the mod.
5. Tests can't install a global layer and then un-install it, so make the
   layer logic testable without the global: keep `find`, `list` and
   `display_in` taking `&[Layer]` (as they do today) and test them on
   hand-built runtime layers.

## Acceptance criteria

- [ ] A runtime layer's file replaces an embedded file at the same path,
      and a mod-only file is added (unit tests on `find` and `list`).
- [ ] The last of two mods wins for a path both have (unit test).
- [ ] `display_path` shows `mods/<id>/...` for a file read from a mod
      (unit test).
- [ ] A second `install_mod_layers` call, or one after the first read,
      returns an error (unit test in a separate test binary, or a
      documented `#[ignore]`-free approach that doesn't touch the global
      in other tests).
- [ ] Bad paths are refused with the mod id and path in the message (unit
      tests).
- [ ] Without mods, every existing test passes unchanged.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the layer search and listing over embedded and runtime layers;
  path validation.
- Property: for any set of layers, `list` returns sorted, de-duplicated
  paths and `find` agrees with it.

## Completion notes
