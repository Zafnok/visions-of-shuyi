---
id: "1195"
title: Stamp the bots' play records with the build that made them
type: infra
milestone: "Engine: carve"
model: implementation
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1195 — Stamp the bots' play records with the build that made them

## Context

Engine PR #152 (engine ticket 2111), passed by the pin move of
2026-10-28 (`docs/engine/pins.md`, game PR #465), stamps records and
saves with what made them: a `shuyi_core::Made` holding the engine's
version, the game's build and a fingerprint of its content. A game's
tools give their build's stamp in `GameTools::made`. With the stamp,
`shuyi replay` refuses a record made by another engine, another build
or other files (`SH3007`), so a record that replays differently after
a content change is reported as the cause, not as a desync, and
`playtest` keeps the stamp in its history. `--any-stamp` runs it
anyway. The skill pack's `shuyi-debug-replay` explains this.

The game doesn't use it yet. `Trpg` (`crates/xtask/src/game_tools.rs`)
has no `made`, and its `PlayRecord` (`crates/xtask/src/game_tools/plays.rs`)
carries only a format `version`. A record the bots wrote before a
balance change replays as if the rules broke. The engine's `grid`
template shows the pattern (`made()` in its `game` crate's `lib.rs`).

## Nick input

None.

## Scope

**In:**
- `GameTools::made` for `Trpg`.
- A `made` field on `PlayRecord`, written by the bots and passed on by
  `replay`.

**Out (do not do):**
- 0508's human play records in `trpg-ui`. Either order works: if 0508
  is done first and writes this same `PlayRecord`, its records get the
  stamp through this ticket. If 0508 comes later, it fills `made` from
  the same helper.
- Stamping saves (the engine's save stamps are its own business).
- Baked packs (the game bakes none).

## Implementation steps

1. In `crates/xtask/src/game_tools/plays.rs` (or a small `made.rs`
   beside it), add `pub fn made() -> Made`, built once in a `OnceLock`:
   `Made::new(concat!("visions-of-shuyi ", env!("CARGO_PKG_VERSION")),
   trpg_content::bundle::install().fingerprint())`. Mark the build
   string `// check-text: not player text` like the engine's template.
   Check that `install()` is the bundle the bots' content is read from.
2. Implement `fn made(&self) -> Option<Made> { Some(plays::made()) }`
   on `Trpg`.
3. Add `#[serde(default)] pub made: Option<Made>` to `PlayRecord`. Older
   records then still parse, and an unstamped record runs, as the
   engine allows. Fill it with `Some(made())` where `bots` writes the
   record (`plays.rs`, around line 71). Leave `VERSION` at 1: the field
   is optional.
4. In `replay`, set the returned `ReplayRun`'s `made` to
   `played.made.clone()` (`turns` leaves it `None`).

Expected size: about 25 production lines.

## Acceptance criteria

- [ ] `cargo shuyi bots` writes records with a `made` that names this
      engine, `visions-of-shuyi <version>` and the content's fingerprint.
- [ ] `cargo shuyi replay` on a fresh record runs. On the same record
      with its `content` changed, it fails with `SH3007`. With
      `--any-stamp`, it runs.
- [ ] A record without `made` still replays.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (`game_tools/tests.rs`): a bots record round-trips with its
  `made`, and `replay` passes it on. A record whose text has no `made`
  parses and replays with `made: None`. `Trpg.made()` is `Some` and its
  content equals `bundle::install().fingerprint()`.

## Completion notes

*(Filled in by the session that completes the ticket.)*
