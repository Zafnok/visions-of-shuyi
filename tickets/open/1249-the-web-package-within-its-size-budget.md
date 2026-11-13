---
id: "1249"
title: The web package within its size budget
type: infra
milestone: Later
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 1249 — The web package within its size budget

## Context

`cargo shuyi bench` (ticket 1244, `docs/targets.md`) gates each target's
package. PR #558's first run measured the web package,
`visions-of-shuyi-0.3.0-web.zip` (built without the bought art), at
**76.3 MB against the web's budget of 25 MB** (the engine's web profile;
itch.io and browsers download it before the game starts). The game's
`music/` folder, which `shuyi.ron`'s `web.music` ships beside the web
build, is 75 MB of it: 46 Ogg Vorbis tracks and jingles.

**From 0914 (the Chapter 1 demo, `--edition demo`):** the demo edition
ships every track too. The web build copies `web.music` whole and an
edition can't choose its tracks (engine ADR-0100; the engine's book says
the same of the Playdate's), so the demo's package is about as big as the
full game's, though Chapter 1 plays a handful of tracks. Whatever this
ticket does should cover the demo's package as well (an edition's own
music list in the engine is one way).

## Nick input

**Decision**, asked before building, only if the fix touches what the
player hears: lower-bitrate tracks (with before/after clips to compare),
or the same files loaded as each track is first needed rather than
before the game starts. The second changes nothing heard but can delay a
track's first play on a slow connection.

## Scope

**In:**
- Find what the 25 MB budget counts for the web (the engine's
  `docs/targets/web.md`): the package downloaded before the game starts,
  or every file it ever loads. If music that streams in later doesn't
  count, ship the music as files the page fetches when first played (the
  engine's web build may support this; else an engine ticket, the
  `engine-change` skill).
- Otherwise, re-encode the tracks at a lower bitrate, as Nick chooses.
- The web package under 25 MB in the `bench` job.

**Out (do not do):**
- Removing tracks (a design decision).
- Raising the budget.

## Implementation steps

1. Read the engine's web target page and `shuyi web`'s music handling.
2. Pick the approach; ask Nick if it changes the sound.
3. Re-run the bench and record the package size in `docs/targets.md`.

## Acceptance criteria

- [ ] The web package is under 25 MB in the `bench` job's table.
- [ ] Every track still plays in the web build (checked in the browser
      pane).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- CI: the `bench` job's package row.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
