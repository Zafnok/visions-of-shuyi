---
id: "1059"
title: "Playdate: the battle map at 50 frames a second, the most the screen can show"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: ["1057", "1038"]
nick_input: setup
completed:
---

# 1059 — The battle map at 50 frames a second on the Playdate

## Context

Nick asked (2026-10-17) whether the Playdate version could run at 60
frames a second instead of struggling to hold 30. **60 is impossible:**
the SDK's `setRefreshRate` says "Maximum is 50 fps" (*Inside Playdate
with C*, SDK 3.1.2), so 50 is the screen's limit. The default, and what
the game asks for today, is 30 (`REFRESH_RATE` in
`crates/playdate/src/pd/entry.rs`; ADR-0070, `crates/playdate/README.md`,
*The frame loop*: "The device numbers say whether to change it").

At 50 a frame has **20 ms** for everything. Today, on Nick's Playdate
(Rev B, release build, 2026-10-16, 1055's run):

- the battle's `Game::frame`: 18.2 min / 25.5 avg / 432 max ms, even on
  frames whose picture doesn't change;
- drawing and copying a changed frame (ADR-0077): 17.6 avg / 25.7 max ms.

1057 cuts `Game::frame` in the battle's player phase to 10 ms on average
at most, for a steady 30. This ticket goes on from there to 50, on the
battle map first: moving the cursor, walking a unit, opening a menu. The
game's motions are timed by `dt`, not by frames, so they keep their speed
at any rate; only their smoothness changes. A quick press is never lost
at any rate (`crates/playdate/src/pads.rs`).

Running faster costs battery: the update callback runs 50 times a second
instead of 30 even when nothing moves. The SDK pushes only rows that
changed, so a still screen costs the game's own work, not the screen's.

## Nick input

**Setup** (a measurement, not a check): Nick sideloads the release
build at 50, plays a minute of Chapter 1's first battle (moving the
cursor, walking units, opening menus) and pastes the Simulator
console. Whether 50 feels better than 30 and whether the battery
drain is acceptable go on the Act 1 playtest's checklist (1064; Nick,
2026-10-21: no spot checks of gameplay tickets).

## Scope

**In:**
- **Measure first, on the device:** with 1057's bench marks, a battle
  minute at a refresh rate of 50 (frames a second, `Game::frame` and
  render-and-copy min / avg / max, frames drawn). Record it in the
  completion notes.
- **Cut** what stands between those numbers and 20 ms a frame, where the
  numbers point: e.g. work done every frame that only needs doing when
  something changed, the render of a frame where only the cursor or one
  unit moved (ADR-0077's changed tiles), the copy to the SDK's frame.
  Without changing what any screen shows or what the game decides
  (snapshots, play records and bot results unchanged).
- **The rate:** if the battle map holds 50, set it (the whole game, or
  50 on the battle map and 30 elsewhere if other screens can't hold it:
  the front end asks the SDK for the rate the screen on top wants,
  through a value `trpg-ui` exposes, never a clock or the SDK inside
  `ui`, ADR-0004). If it can't, keep 30 and record the best reached.
- Update `crates/playdate/README.md` (*The frame loop*) and ADR-0070 or
  ADR-0077 with the rate chosen and why.
- **Goal:** the battle's player phase averages at least 48 frames a
  second over Nick's minute, with no visible hitch when the cursor moves.

**Out (do not do):**
- The enemy AI's planning pauses (1051).
- Loading time (1036) and the steady cost 1057 already cuts.
- Screens other than the battle map, beyond keeping them at 30 when they
  can't hold 50.
- Changing a skin, an animation's timing, or what a screen shows.
- The map pictures 0067 decides: if it chose bought tiles and sprites and
  their skin is built before this ticket, measure with it (the skin the
  player will see); if not, measure the glyph skin (1037), and the
  sprite skin's build ticket checks its own frame time at 50.

## Implementation steps

1. Read ADR-0004, ADR-0070, ADR-0077, `crates/playdate/README.md`,
   `src/pd/entry.rs`, `src/pd/bench.rs`, and 1055's and 1057's
   completion notes.
2. Build the bench build at 50 (`cargo xtask playdate --release
   --bench` with `REFRESH_RATE` 50), ask Nick to sideload it and paste the
   console after a minute in the first battle (one Nick turn).
3. Time the same frames on the PC first (1057's release-mode probe
   through the Harness on `ConsoleSpec::PLAYDATE`) to find what to cut.
4. Cut the biggest parts; keep every test and snapshot as it is.
5. Choose the rate (one for the game, or per screen) and wire it.
6. Hand Nick the 50 build for the measurement; record the before /
   after numbers in the completion notes and the PR.

## Acceptance criteria

- [ ] The device numbers at 50, before and after, are in the completion
      notes and the PR.
- [ ] The battle's player phase averages at least 48 frames a second on
      Nick's Playdate, or the notes say what was reached, the rate stays
      30, and a follow-up ticket names what is left.
- [ ] `crates/playdate/README.md` names the rate chosen.
- [ ] No snapshot, play record or bot result changes.
- [ ] The 50-or-30 and battery questions are in the Completion notes
      (1064's checklist).
- [ ] All gates in the `run-gates` skill pass, and the CI `playdate` job.

## Tests required

- Unit: any new timing report lines (`crates/playdate/src/report.rs`);
  the rate a screen asks for, if it is per screen.
- Property / integration: whatever replaces work done each frame (a
  cache, work done once) gives the same result as before, over the
  Harness's battle frames.
- Manual: Nick's device runs at 50 and 30.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
