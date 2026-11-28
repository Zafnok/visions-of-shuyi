---
id: "1401"
title: "The editions' cut on the engine's cut list: demo_bundle.rs keeps only the language-pack and scene rules"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["1288", "engine#3444"]
nick_input: none
completed:
---

# 1401 — The editions' cut on the engine's cut list

## Context

The third engine-friction sweep (1397, row C2) found that
`crates/content/src/demo_bundle.rs` (about 470 production lines, ticket
0914, ADR-0119) is mostly a generic cut list. It leaves out files and
folders, keeps some back with `+`, keeps named entries of a RON file,
swaps a text and strips RON comments. Only its language-pack and
dialogue-scene rules (ticket 0934) and its `art` lines (0858) are the
game's. Engine ticket 3444 adds that cut list to `shuyi_embed`, with a
hook for a game's own rules. The file is marked
`// engine-friction: engine#3444, moves (game 1401 deletes it)`.

**Needed first: 1288**, which makes `demo_bundle::Plan` the one parser
for every edition (the Playdate's `playdate-bundle.txt` included). That
leaves this ticket one parser to move, not two.

## Nick input

None.

## Scope

**In:**
- Move the pin past engine 3444 if it is behind (`engine-change` §7).
- `crates/content/build.rs` cuts each edition with
  `shuyi_embed::cut::Plan`, through the engine's hook.
- The game keeps, as its hook, only:
  - the language-pack rules (`Plan::hold`, the `data.ron` keys by the
    demo's files);
  - `dialogue/<file>.ron` going with its `.dlg`;
  - the scenes a dialogue file keeps;
  - the `art` lines.
- The list files (`crates/content/demo/bundle.txt`, the preview's, the
  trial's and the Playdate's) keep their syntax. Where the engine's
  syntax differs, rewrite the lists, and note it in the completion
  notes.
- Every embedded table stays byte for byte the same: the edition tests
  (`demo_bundle/tests.rs`, and the `build.rs` table tests) pass
  unchanged.

**Out (do not do):**
- Changing what any edition ships.

## Implementation steps

1. Write the engine-friction estimate: about 80 lines kept (the hook),
   about 390 removed.
2. Move the pin, if needed.
3. Write the hook. Point `build.rs` at the engine's plan. Delete the
   generic parts of `demo_bundle.rs` and their tests (the engine has
   them).
4. Check the tables: dump each edition's table of paths and hashes
   before and after, and compare.

## Acceptance criteria

- [ ] Each edition's embedded files (path and hash) are the same before
      and after (the comparison is in the completion notes).
- [ ] `demo_bundle.rs` has no RON comment stripper, no `keep` or
      `swap` parser, and no folder matcher.
- [ ] All gates in the `run-gates` skill pass, the WASM and Playdate
      builds included.

## Tests required

- Unit: the hook's language-pack and scene rules (moved from today's
  tests).
- The existing edition tests, unchanged.

## Completion notes

*(Filled in by the session that completes the ticket.)*
