---
id: "0251"
title: Reserve the world map cues until they are imported
type: bug
milestone: M1 Engine
model: sonnet-5
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0251 — Reserve the world map cues until they are imported

## Context

`main` has been red since PR #448 (ticket 0025, the world map music):
0025 added three cues to `docs/design/audio.md`'s *Music cues*
(`world_map`, `world_map_war`, `world_map_somber`), and importing the
files is another ticket's job (1007 for `world_map`; the others when a
story stretch first plays them). But
`audio::tests::every_design_cue_is_in_the_manifest`
(`crates/content/src/audio.rs`) wants every cue `audio.md` lists in
`assets/audio/audio.ron`, but for its `RESERVED` list:

```
left: ["world_map", "world_map_war", "world_map_somber"]
right: []
```

`RESERVED` is meant for exactly this: "cues `audio.md` reserves for a
moment the story doesn't have yet (0022, 0023): each is imported by the
ticket that first plays it." Found while working 0517.

## Nick input

None.

## Scope

**In:**
- Add the three cues to `RESERVED` in `crates/content/src/audio.rs`'s
  tests, and widen its doc comment to name 0025.
- In `tickets/open/1007-world-map-nodes-travel-towns.md`, add a step:
  remove `world_map` from `RESERVED` when it is imported (the test's
  "drop it from RESERVED" assertion fails otherwise).

**Out (do not do):**
- Importing any of the tracks, or credits for them (1007 and later).
- Changing `audio.md`.

## Implementation steps

1. `const RESERVED: [&str; 6] = ["victory_grand", "ceremony_intro",
   "scene_loss", "world_map", "world_map_war", "world_map_somber"];`
   with the doc comment naming 0025 beside 0022 and 0023.
2. The step in 1007 above.
3. `cargo test -p trpg-content --lib audio::`.

## Acceptance criteria

- [ ] `every_design_cue_is_in_the_manifest` passes.
- [ ] 1007 says to drop `world_map` from `RESERVED` when it imports it.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the existing `every_design_cue_is_in_the_manifest`.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
