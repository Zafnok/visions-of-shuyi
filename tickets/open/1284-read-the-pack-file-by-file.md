---
id: "1284"
title: "Read the game's pack file by file, with the loaders off &'static"
type: infra
milestone: M8 Release
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
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

*(Filled in by the session that completes the ticket.)*
