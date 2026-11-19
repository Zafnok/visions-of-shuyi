---
id: "1284"
title: "Read the game's pack file by file, with the loaders off &'static"
type: infra
milestone: M8 Release
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-17
---

# 1284 — Read the game's pack file by file

## Context

The engine pin move of 2026-11-17 passed engine PRs #478 (engine ticket
2262) and #479 (2263, ADR-0128 §2 of the engine). The bundle now hands
out a file's bytes as a `Cow`, and a pack can be read file by file:
`pack::Index::read` reads its header and index alone, and
`Index::into_layer` over a `pack::Source` (`InMemory` today) gives a
layer that reads, checks and unpacks one file when it is asked for and
keeps nothing (`Cow::Owned`). A pack read whole (`Pack::open`, what the
game's packed build does in `crates/app/src/main.rs`'s `startup`) keeps
every file unpacked for the whole run.

The game can't take it yet: its loaders hold `&'static str` and
`&'static [u8]` (`crates/content/src/bundle.rs`'s `file`, `bytes` and
`test_part`), and the pin move keeps that by leaking any file a layer
doesn't lend (`lent`). With a layer read on demand every read would
leak a copy.

## Nick input

None.

## Scope

**In:**
- Measure first: the packed build's resident memory with the pack read
  whole, by the engine's `asset_memory` program or `cargo shuyi perf`'s
  heap, on the desktop and the web.
- The loaders that keep a file's text past their parse copy what they
  keep, so `bundle::file` / `bytes` / `test_part` can hand out the
  bundle's `Cow` and `lent` goes.
- The packed build's `startup` reads its pack with `Index::read` and
  `Index::into_layer` over `InMemory`; a refused file is the start's
  error, as a broken pack is now.
- The numbers before and after in the PR. If the gain is under a few
  MB on every target, say so and keep `Pack::open` (the loaders' change
  still goes in).

**Out (do not do):**
- A source read from disk or the network as it arrives (engine 2265,
  `Bundle::arrived`), and the Playdate's bundle, which isn't a pack.

Estimate: about 150 production lines changed, mostly `.as_deref()` and
copies at the loaders; 15 lines removed with `lent`.

## Implementation steps

1. Measure the packed build as it is.
2. Move the loaders off `&'static` (one crate at a time, `trpg-content`
   first); drop `lent`.
3. Switch `startup` to the index; measure again.
4. `run-gates`, the packed build's tests (`--features packed`).

## Acceptance criteria

- `crates/content` leaks no file the bundle hands out.
- The packed build reads every file equal to the embedded build (the
  existing pack test) through the file-by-file layer, or the PR says
  why it kept `Pack::open`, with numbers.

## Tests required

The existing pack test on the new layer; a test that a pack with one
file changed refuses that file and reads the rest.

## Completion notes

**Measured, and `Pack::open` kept.** The engine's `asset_memory`
program (`examples/hello-grid`, at the pin `e738ffc2`, `--release`,
`--folder` of the files a release bakes: `assets/` with the content over
it, 166 files) on the game's pack: 776,966 bytes (6,890 of header and
index), 1.14 MB unpacked.

| Way | Peak heap | Resident heap |
| --- | --------- | ------------- |
| Read whole and kept (what `pack_beside_program` and the packed build's `startup` do today) | 4.11 MB | 3.86 MB |
| File by file (`Index::read`, `Index::into_layer` over `InMemory`), measured | 2.23 MB | 1.97 MB |
| File by file, with the pack's 0.78 MB read into the heap for `InMemory` (as the packed build would hold it) | about 3.0 MB | about 2.75 MB |

The gain the packed build could take is about 1.1 MB, on the desktop and
on the web alike: both read the same pack whole into the heap (the web's
fetched by the page), and on `wasm32` only the table's pointers shrink
(its 8.9 KB). That is under the few MB the ticket set, so `startup`
keeps `Pack::open` (`shuyi_app::pack_beside_program`). The rest of the
gain (the pack's own 0.78 MB) needs a source that reads the file from
disk or as it arrives, engine 2265, out of scope here. Switching would
also have needed an engine change: the engine has no public way to get
the bytes beside the program without opening them (`read_beside` is
private to `pack_file.rs`).

**The loaders are off `&'static`.** `bundle::file`, `bytes` and
`test_part` hand out the bundle's `Cow`; `lent` is gone, and nothing in
`crates/content` copies or leaks a file it is handed: every loader parses
from a borrow of the `Cow` and keeps what it parsed. The script loaders
(`load_with_scripts`, `load_with_some_scripts`, `dialogue::from_sources`,
`camp::from_sources`) take any text (`AsRef<str>`), so a caller can pass
the bundle's `Cow`s. `AppConfig::new` takes `bundle::bytes` as is.

The Playdate's one-bit font and pictures keep their bytes borrowed for
the whole run (ADR-0128 §3: the one-bit frame reads a picture where the
bundle lends it), so they read through a new `bundle::lent_bytes`: the
bytes where the layer lends them, `None` where a layer would read the
file on demand. It copies and leaks nothing; every Playdate layer lends.

**Tests.** `crates/xtask/src/game_tools/tests.rs`: the baked pack read
file by file reads every file the embedded bundle holds with the same
bytes, and the content loads from it as from the tables (the scenes,
battles and chapters equal: the loaders work over a layer that hands
files out owned); and a pack with one file changed after the bake refuses
that file (it reads as missing, `Layer::problems` lists it once) and
reads every other.

No gameplay rule was decided. No follow-up ticket: reading the pack from
disk is engine 2265, already filed.
