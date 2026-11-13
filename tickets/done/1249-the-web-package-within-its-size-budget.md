---
id: "1249"
title: The web package within its size budget
type: infra
milestone: Later
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-13
---

# 1249 — The web package within its size budget

## Context

`cargo shuyi bench` (ticket 1244, `docs/targets.md`) gates each target's
package. Its first runs measured the web package,
`visions-of-shuyi-0.3.0-web.zip` (built without the bought art), at
76–82 MB against the web's budget of 25 MB. The game's `music/` folder,
which `shuyi.ron`'s `web.music` ships beside the web build, is 75–80 MB
of it: 46 Ogg Vorbis tracks and jingles.

The budget is the engine's web profile (engine ADR-0008 §6,
`docs/targets/web.md`): "25 MB download (prov.)", beside a start in
5 s at 25 Mbit/s. It stands for what a player waits for **before the
game starts**, not every file the game may ever load.

**The web build already loads music when it's first played.** Checked
in the browser pane on 2026-11-12, with a release web build
(`cargo shuyi web --release`, no bought art, no wasm-opt) served from
`dist/web`:

| When | What the page downloaded |
| --- | --- |
| Opening the page | `index.html`, the seven scripts (≈ 80 KB), `visions-of-shuyi.wasm` (10.5 MB) |
| The first key on the title | `voice/en/voice.ron` (1 KB) |
| The title song starting | `music/new_sunrise_v1.ogg` (2.5 MB), the only track |
| The opening scene starting (New Game) | `music/squirrel_village_loop.ogg` (2.5 MB) |

The engine fetches each track when its cue first plays, in the
background (`crates/platform/src/audio/player.rs` calls `load_music` when a
music cue is asked for; on the web `crates/app/src/audio.rs`'s `load_music`
runs `load_file` in a coroutine). itch.io serves an uploaded HTML5 zip
as separate files, and so does GitHub Pages, so the 75 MB of music is
never waited for. So the bench's web number is too big:
`crates/cli/src/bench.rs`'s `packages` takes the whole zip's size
(`report.sizes.first()`), the folders shipped beside the build included.

So the package doesn't need to shrink; the measure does. Nothing the
player hears changes.

## Nick input

None: no change to what the player hears or sees.

## Scope

**In:**
- ~~An engine change (the `engine-change` skill: an engine ticket and PR
  first, then this game's PR that moves the pin)~~ (done: engine PR #393,
  engine ticket 3313, in the pin since the pin move of 2026-11-13): for the web, `shuyi
  bench`'s package number counts what the page loads before the game's
  first frame, which is the zip without the folders `web.music` and
  `web.voice` ship beside the build. Its table shows the whole zip's
  size too, so the upload size stays visible. The engine's
  `docs/targets/web.md` says what the web's `package_bytes` counts.
- Remove `web/package` from `KNOWN_OVER` in `.github/workflows/ci.yml`.
- `docs/targets.md`: the new web package row, what it counts and why,
  and the "Over budget" entry for 1249 removed.

**Out (do not do):**
- Re-encoding, cutting or removing tracks.
- Raising the budget.
- The demo's own music list (0914: `--edition demo` ships all 46
  tracks). With music loaded on first play, the demo's player downloads
  only the tracks Chapter 1 plays. The unused ones only make the upload
  bigger. Not worth a ticket unless itch.io's upload limit is hit.
- wasm-opt on CI's web export (0916): the 10.5 MB `.wasm` is already
  under the budget; 0916 makes it smaller.

## Implementation steps

1. ~~Write the engine ticket and change `packages` in `bench.rs` (and its
   report) as in Scope, with a test of a web export whose music folder
   is over the budget and whose loaded-at-start files are under it.~~
   (done by engine PR #393: its package table gains a "Whole package"
   column)
2. ~~Move the game's pin (`cargo xtask engine-pin`)~~ (done by the pin
   move of 2026-11-13), drop `web/package`
   from `KNOWN_OVER`, re-run the `bench` job and record its web package
   row in `docs/targets.md`.
3. Play the web build in the browser pane: the title song and a later
   track (New Game's opening scene) both play, and are fetched only when
   they start (the page's network requests).

## Acceptance criteria

- [x] The `bench` job's web package row is under 25 MB, with the whole
      zip's size shown too.
- [x] `KNOWN_OVER` no longer lists `web/package`.
- [x] In the browser pane, the title song and the opening scene's track
      play, and neither is downloaded before it starts.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Engine: a `shuyi bench` test of the web package measure (music beside
  the build not counted, the whole zip still reported).
- CI: the `bench` job's package row.

## Completion notes

- The engine change (engine PR #393, ticket 3313) and the pin move had
  already landed, so this PR is the game's side only.
- `web/package` is out of `KNOWN_OVER` in `.github/workflows/ci.yml`; the
  comments there and `scripts/bench-known-over.sh`'s warning now name
  ticket 1248, the one left.
- `docs/targets.md`: the package row from the `bench` job of PR #674
  (2026-11-13): **10.7 MB** gated against 25 MB, 81.9 MB the whole zip; a
  paragraph on what the web's package budget counts and why; the 1249
  entry gone from "Over budget".
- Browser pane, a release web build without the bought art
  (`cargo shuyi web --release`, no wasm-opt), on 2026-11-13: opening the
  page fetched the scripts and `visions-of-shuyi.wasm` (10.5 MB) and no
  music; `music/new_sunrise_v1.ogg` (2.5 MB) was fetched when the title
  came up after the layout picker, and `music/squirrel_village_loop.ogg`
  (2.5 MB) with the opening scene's first voice clips when New Game
  reached Chapter 1's opening. No other track was fetched.
- Deviation: in the pane I could confirm when each track is fetched but
  not hear it: the sound mixer's state isn't reachable from page scripts,
  and the pane gave no sign of blocks being mixed. Nothing about playback
  changed in this PR (no code changed), so the check covers what the
  ticket changes, the measure.
- Follow-ups: none.
