---
id: "0251"
title: Reserve the world map's music cues until a ticket imports them
type: bug
milestone: M1 Engine
model: implementation
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-10-27
---

# 0251 — Reserve the world map's music cues until a ticket imports them

## Context

`main` has been red since PR #448 (ticket 0025, the world map music).
`audio::tests::every_design_cue_is_in_the_manifest` in
`crates/content/src/audio.rs` (ticket 0214) checks that every cue
[`docs/design/audio.md`](../../docs/design/audio.md) lists is in
`assets/audio/audio.ron`, and now fails with

```
left: ["world_map", "world_map_war", "world_map_somber"]
right: []
```

0025 added those three cues to `audio.md`'s *Music cues*, but none of the
files is imported yet: `world_map` is imported by ticket 1007 (the world
map), the other two by the story ticket that first plays them (Act 3's map
and the somber stretch). The test already has a `RESERVED` list for exactly
this case (cues 0022 and 0023 reserved for moments the story doesn't have
yet), so the fix is to reserve these three too.

## Nick input

None.

## Scope

**In:**
- Add `world_map`, `world_map_war` and `world_map_somber` to `RESERVED` in
  `crates/content/src/audio.rs`, and name 0025 in its doc comment.
- Add a step to 1007: remove `world_map` from `RESERVED` when it imports
  the track (the test's "drop it from RESERVED" assertion fails otherwise).

**Out (do not do):**
- Importing any track, its credits or `THIRD_PARTY_ASSETS.md` lines (1007
  and the story tickets).
- Any change to `docs/design/audio.md`.

## Implementation steps

1. In `crates/content/src/audio.rs`, `mod tests`: change
   `const RESERVED: [&str; 3]` to `[&str; 6]`, appending
   `"world_map"`, `"world_map_war"`, `"world_map_somber"`, and widen the
   doc comment's ticket list to `(0022, 0023, 0025)`.
2. In `tickets/open/1007-world-map-nodes-travel-towns.md`, add an
   implementation step: when importing `world_map`, remove it from
   `RESERVED` in `crates/content/src/audio.rs`'s tests.

## Acceptance criteria

- [x] `cargo test -p trpg-content --lib audio::` passes.
- [x] 1007's implementation steps say to drop `world_map` from `RESERVED`.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the existing `every_design_cue_is_in_the_manifest` passes; no new
  test (the fix is to the test's own data).

## Completion notes

Done as planned: `RESERVED` in `crates/content/src/audio.rs` now holds
`world_map`, `world_map_war` and `world_map_somber` too, its doc comment
names 0025, and 1007 has a step 9 to drop `world_map` from it when it
imports the track. `world_map_war` and `world_map_somber` stay reserved
until the story ticket that first plays them (none is open yet); that
ticket drops them the same way, as the test's assertion says. No
gameplay rules decided, no follow-up tickets.
