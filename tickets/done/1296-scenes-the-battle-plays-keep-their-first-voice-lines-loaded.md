---
id: "1296"
title: "Scenes the battle plays, and a world-map battle's intro, keep their first voice lines loaded"
type: bug
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["1293"]
blocks: ["1064"]
nick_input: none
completed: 2026-11-18
---

# 1296 — Scenes the battle plays, and a world-map battle's intro, keep their first voice lines loaded

## Context

Found by 1293 (Nick, 2026-11-17, on the itch.io build: the first voice
line of a scene doesn't load in time). The dialogue screen asks for a
box's clip the frame the box comes up, and the engine skips a clip not
loaded 0.3 s after it was asked for; on the web a clip is fetched, so a
scene's first line is silent unless it was loaded before
(`docs/voice.md`, *How the game plays them*).

1293 keeps a scene's first 3 lines (`DialogueScreen::first_lines`)
loaded a screen ahead wherever the game flow knows the scene coming
(`crates/ui/src/flow/first_lines.rs`, `FlowScreen::coming_lines`): New
Game, each save, the next intro scene, the victory scenes while the
results show, the next chapter at the save prompt and in camp. Two kinds
of scene still start with nothing loaded:

1. **Scenes the battle plays over itself**, from its file's `triggers`
   (`BattleDef::triggers`, `Event::SceneTriggered`): Chapter 1's
   `ch01_first_turn`, `ch01_boss_engage*`, `ch01_boss_half`, the death
   and retreat scenes. All are voiced. `BattleScreen::next_scene`
   (`crates/ui/src/screens/battle/mod.rs`) builds the overlay and pushes
   it the frame it plays.
2. **A world-map battle's intro scenes**: `FlowScreen::arrived` plays
   them the frame the world map pops, and the map can't know which place
   the army will go to.

## Nick input

None. Verified by tests and the delayed-clip web check; whether the
lines sound right goes on the Act 1 playtest's checklist (1064).

## Scope

**In:**
- The battle keeps the first lines of every scene its triggers may
  still play, beside the guide's lines, in the one set it sends
  (`GuideVoice`, `coming_guide_lines`; after 1291, `Guide` in
  `screens/battle/guide.rs`). A trigger already fired (or a scene
  already played) drops out at the next change of the set.
- The world map keeps the intro scenes' first lines of the places the
  army can reach in one move from where it stands (the map's own
  neighbours), while it is on top.
- `docs/voice.md`: the "Not covered" paragraph shrinks to what is left.

**Out (do not do):**
- Changing `LATE_SECS` or `AHEAD`, or holding a box until its clip loads.
- Any line's text or clip (dialogue writing is paused, 0766).
- 1291's move of the guide out of `BattleScreen`: either order. If 1291
  is done, add the scenes' lines in `Guide`; if not, in
  `coming_guide_lines`, and 1291 moves it.

## Implementation steps

1. A function on `BattleScreen` listing the scenes its triggers may
   still play (read the trigger list and what has fired from
   `BattleState`; the death and retreat scenes for units still on the
   map). For each, `DialogueScreen::told_overlay(..).first_lines()`, as
   `next_scene` builds it (for `Present`, those on the map now).
   Build the list only when the set is due (the battle's first frame,
   her box changing, back on top), never per frame.
2. Add those lines to the set `GuideVoice::show` keeps. The engine
   keeps one set: the battle's replaces the flow's on its first frame,
   so this is where they go.
3. `WorldMapScreen`: a `KeptVoices` (`crate::flow::KeptVoices`) sent
   while on top with the first lines of each reachable place's chapter
   intro (`first_lines::chapter_opening`, made `pub(crate)`), and again
   after the army moves.
4. Tests, the web check, `docs/voice.md`.

Expected size: about 60 production lines (one list of scenes, one
`KeptVoices` on the map), judged by 1293's `coming_lines`.

## Acceptance criteria

- [x] A test shows the battle's first `KeepVoices` holds the first lines
      of `test_battle`'s trigger scenes (or a test battle given one) and
      the guide's lines.
- [x] A test shows a scene the battle plays has its first line in the
      last set kept before its `PlayVoice`.
- [x] A test shows the world map keeps a reachable battle's intro lines,
      and the scene's first line is kept when its box asks for it.
- [x] In the non-release web build (`cargo shuyi web`) served with every
      clip held 0.5 s, Chapter 1's `ch01_first_turn` first line is
      fetched before its box shows (the server's log).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the battle's and the map's kept sets.
- Integration: a trigger scene over a battle through the harness,
  reading the voice requests.
- Manual: the delayed-clip web check above.

## Completion notes

**The battle** (`screens/battle/scene_voices.rs`). 1291 isn't done, so
the scenes' lines join the guide's in the set `GuideVoice::show` sends
(`BattleScreen::coming_voice_lines`: guide + scenes). The scenes kept:

- those already fired and waiting their turn (a combat playback's
  untaken scenes, `Playback::waiting_scenes`, then the queue's), told
  for those on the map when they fired. A turn-1 scene fires as the
  battle is set up, before its first frame, so without this
  `ch01_first_turn` would never have been kept;
- those the triggers may still fire, told for those on the map now: not
  a `once` trigger that fired, a `TurnStart` whose turn has passed, a
  `UnitFell` for the other mode, or one about a character neither on
  the map nor among the reinforcements to come.

The set is built only when it was due already (first frame, the
guide's box changing, back on top). One addition: on the frame the
battle pushes a scene, the guide's set used to be rebuilt if her box
went away (the push hides it), which would have dropped the scene's own
lines (its trigger is spent and it has left the queue). Now a frame that
covers the battle doesn't rebuild the set, except for the scene it
pushes, whose first lines (`pushed_lines`) stay in it.

**The world map** keeps the intro's first lines of every battle the
army's next move may start (`flow::world_battle_lines`): its own place
(Enter) and wherever one Travel would end as the rules route it
(`WorldMap::route`, which stops at a guarding battle). Deviation: the
ticket said "the map's own neighbours"; a Travel goes any number of
roads, so the honest set is every place one move can end at. It is
resent when the army moves (the rules move it as it sets off) and
after a screen over the map. `chapter_opening` didn't need to be
`pub(crate)`: the battle's chapter comes from the flow's `world_fight`,
made reachable from `first_lines` (a child of `flow`).

**docs/voice.md:** the two cases added to the list; "Not covered" is
down to a victory scene with no results screen before it.

**Tests.** Unit (`battle/scene_voice_tests.rs`): the Quick Battle's
first set holds every trigger scene's lines but the Casual retreat
line, and the guide's; scenes that can't play are left out (a character
not on the map or coming, a passed turn); a fired, queued scene stays
kept up to the frame it is pushed. Unit (`flow/first_lines/tests.rs`):
`world_battle_lines` from home, from the battle's place, and off the
map; through the flow, the map keeps the intro's line and it is in the
last set before its `PlayVoice`. Integration (`tests/it/battle.rs`): a
turn-1 scene over the test chapter's battle, its line in the last set
kept before its `PlayVoice`. Each fails with the change taken out.

**Web check.** `cargo shuyi web` (non-release), served by a Python
server holding every `.ogg` 0.5 s with `Cache-Control: no-store`,
logging each request; played in the browser pane: New Game, Classic,
familiar with this game, voices yes, the default lead, Chapter 1's two
scenes skipped. Log, seconds from the server's start:

- 62.2–63.2: the battle's first frame, under its objective banner:
  `ch01_first_turn_01eff056`, the boss's engage and half-HP scenes and
  the Classic death scenes (no retreat lines), then the guide's lines.
- About 75: the two banners closed, the scene's box on screen
  ("Kid. Boots holding?..."). No request: the clip had loaded 12 s
  before.

**Engine friction:** estimate 60 (one list of scenes, one `KeptVoices`
on the map, judged by 1293's `coming_lines`), measured 167 (2.8×), accepted: the
lines are the game's own rules of which trigger may still fire (one
arm per trigger kind), the scenes fired and waiting, and where one
Travel ends; nothing the engine should hold. A row in
`docs/engine/friction.md`. Signals: workaround no · third copy no ·
budget no.

**Follow-up tickets:** none.

No gameplay rule was decided.
