---
id: "1169"
title: "Content loaders on the engine's folder loader, and the cinematic page check on the engine's word wrap"
type: feature
milestone: "Engine: carve"
model: sonnet-5
effort: medium
status: todo
blocked_by: ["engine#2208", "engine#2209"]
nick_input: none
completed:
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

## Implementation steps

1. Write the estimate: about 190 lines removed, 40 added.
2. One loader at a time with its tests.
3. `page_lines`; its tests move to cover the call.

## Acceptance criteria

- [ ] `grep -rn "file is not valid UTF-8" crates/content/src` finds
      nothing outside tests.
- [ ] `grep -n "fn page_lines" crates/content/src/cinematic.rs` finds
      nothing.
- [ ] The content checks report the same on the real and test files.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The existing content tests, unchanged in what they check.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
