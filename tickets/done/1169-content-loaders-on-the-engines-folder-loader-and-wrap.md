---
id: "1169"
title: "Content loaders on the engine's folder loader, and the cinematic page check on the engine's word wrap"
type: feature
milestone: "Engine: carve"
model: implementation
effort: medium
status: done
blocked_by: ["engine#2208", "engine#2209"]
nick_input: none
completed: 2026-10-20
---

# 1169 — Content loaders on the engine's folder loader and wrap

## Context

The first engine-friction sweep (1153, `docs/engine/friction.md`)
measured the cinematics (`crates/ui/src/cinematic*`,
`crates/content/src/cinematic*`) at 1614 production lines against an
estimate of 1050 (1.5×), and found two signals in `crates/content`:

- **The same folder loader eight times** (about 25 lines each; the
  line `"file is not valid UTF-8"`): `battle.rs`, `battler.rs`,
  `chapter.rs`, `cinematic.rs`, `cinematic/opening.rs`, `map.rs`,
  `portrait.rs`, `tileset.rs`. Engine ticket 2208 adds
  `Bundle::load_each`.
- **Engine code copied:** `page_lines` in `cinematic.rs` re-writes the
  engine's `word_wrap` line count and its longest-token expansion,
  because `word_wrap` is in `shuyi-ui`, which `trpg-content` doesn't
  depend on. When the engine's break rules change (its ticket 2302),
  the copy would pass text the box breaks differently. Engine ticket
  2209 moves `word_wrap` to `shuyi-assets` and adds
  `lines_at_longest`.

## Nick input

None.

## Scope

**In:**
- Move the pin past both (`engine-change` skill, step 7).
- The eight loaders on `load_each`, each keeping its own parse and
  checks and its error messages.
- `page_lines` replaced by the engine's `lines_at_longest`, with the
  game's longest-name rule passed in.

**Out (do not do):**
- Changing what any loader accepts or reports: the content
  validation's output (`cargo xtask` content checks) stays the same.
- The engine's format registry (1173): either order works. If 1173
  landed first, a loader it moved to `Formats::load_under` stays there;
  only the game's own folder loops move to `load_each`.

## Implementation steps

1. Write the estimate: about 190 lines removed, 40 added.
2. One loader at a time with its tests.
3. `page_lines`; its tests move to cover the call.

## Acceptance criteria

- [x] `grep -rn "file is not valid UTF-8" crates/content/src` finds
      nothing outside tests.
- [x] `grep -n "fn page_lines" crates/content/src/cinematic.rs` finds
      nothing.
- [x] The content checks report the same on the real and test files.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The existing content tests, unchanged in what they check.

## Completion notes

- **Pin:** moved to engine `1d3da5d` (the merge of engine 2209, after
  2208), not the engine's `main`: `main` (`2567085`) has later breaking
  changes (`shuyi-ui` no longer has the `harness` feature the game asks
  for), which belong to their own game tickets. The only fallout of this
  pin: `shuyi_ui::widgets::wrap` is gone, so the game's `widgets`
  re-exports `word_wrap` from where the engine now has it (3 imports).
- **Loaders:** 1173 had already moved `cinematic.rs` and `map.rs` onto
  the format registry, so six loops were left (battle, battler, chapter,
  portrait, tileset, opening). All six go through `bundle::load_each`
  (a one-line wrapper over `Bundle::load_each`, like the module's other
  functions), keeping their parse, checks and messages. Four are a
  single call. Chapter and battler check the files that parsed
  *together* (`check_next`, the default battler) even when another file
  is broken, which the engine's all-or-nothing `load_each` doesn't give:
  their parse reports each file's errors itself and returns `Option`, and
  `bundle::parsed` keeps the `Some`s. One small difference in an error
  case only: when one of those folders has a file that isn't UTF-8, its
  error comes after the other files' errors (not in file order), and
  the together-checks then see no files. Real and test content are all
  valid, so the content checks report the same (the content tests pass
  unchanged).
- **`page_lines`:** gone; the talk shot's check calls the engine's
  `lines_at_longest` with `LeadTokens` and the names table's longest
  name. Its test now runs the same assertions through that call, and
  they all pass unchanged, so the engine's wrap counts lines the way the
  copy did.
- **Engine friction:** estimate 30 added, measured 71 added and 182
  removed (2.4×), accepted: a changed line counts as added, and the
  `parsed` helper is the price of the together-checks (row in
  `docs/engine/friction.md`; a third such loader would make it an engine
  variant).
- No gameplay rules decided. Nothing for Nick to check by playing.
