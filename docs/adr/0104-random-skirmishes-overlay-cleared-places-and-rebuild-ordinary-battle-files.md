# ADR-0104: Random skirmishes overlay cleared places and rebuild ordinary battle files

- **Status:** Accepted
- **Date:** 2026-10-31
- **Related tickets:** 1008
- **Extends:** ADR-0005 (data), ADR-0035 (battle flow), ADR-0039 (saves), ADR-0099 (world maps)

## Context

World structure requires fixed authored skirmishes and repeatable random
groups after story victories. Encounters must remain reproducible through
saving and retrying. The future generator (0510) needs to produce ordinary
battle files rather than another map format.

## Decision

- Fixed skirmishes are a world-node kind with a battle/scenes record and
  `guards_path`. Only guards block through travel. Arrival starts either.
- Random groups overlay cleared battle places in `WorldState::skirmishes`,
  keyed by place id. They leave the underlying cleared state intact, so
  unlocking conditions never close again. Travel may pass a group.
- `skirmishes/<map-id>.ron` holds each act's spawn range, capacity, level
  offset, group size, battle pool and weighted reward list. Content loads
  it with the matching world map; the format lives in
  `assets/worldmaps/README.md`.
- `Campaign::clear_node` rolls groups only for a first story victory and
  returns `WorldEvent::SkirmishSpawned`. Winning a group removes only it.
  Existing groups also respect a later lower cap (`SkirmishesCapped`);
  when it admits no valid enemy level, they leave.
- `Campaign::rng` is an optional saved PCG stream, seeded at 0 on first use
  and retained across acts. Groups store battle id, level, count, seed and
  reward. New fields have deliberate serde defaults and are omitted when
  empty, preserving older version-2 saves without a version bump.
- Pool files are restricted ordinary Easy rout battles. Content rebuilds
  their generic enemy entries at the saved level through the same loader
  and template default loadouts as authored battles. It substitutes count,
  seed and clear loot, then validates the result. Flow resolves that same
  definition both when arriving and when continuing a suspend, so retry
  and rewards need no alternative battle rules.
- Marker, hover and Find logic read the overlay before the underlying node.
  Both skins paint the same marks from the plain world-map view.

## Consequences

A group's win cannot advance a story chapter whose cleared place it uses.
Death, retreat, support points, rewind charges and unused-charge EXP use
the existing battle machinery. Pool authors must order enemy placements
so every permitted prefix is playable. Rebuilding parses one small file
when starting or continuing a random encounter.

## Alternatives considered

- **Replace the node kind in the map** — mutates shared content and loses
  the underlying cleared node and its unlocking conditions.
- **Store full generated battle definitions in saves** — duplicates maps
  and rules tables rather than the rolls needed to reconstruct them.
- **A second enemy builder** — risks drifting from generic-template stats,
  learned spells and loadout validation in the ordinary battle loader.
