---
id: "1296"
title: "Scenes the battle plays, and a world-map battle's intro, keep their first voice lines loaded"
type: bug
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["1293"]
nick_input: none
completed:
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

- [ ] A test shows the battle's first `KeepVoices` holds the first lines
      of `test_battle`'s trigger scenes (or a test battle given one) and
      the guide's lines.
- [ ] A test shows a scene the battle plays has its first line in the
      last set kept before its `PlayVoice`.
- [ ] A test shows the world map keeps a reachable battle's intro lines,
      and the scene's first line is kept when its box asks for it.
- [ ] In the non-release web build (`cargo shuyi web`) served with every
      clip held 0.5 s, Chapter 1's `ch01_first_turn` first line is
      fetched before its box shows (the server's log).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the battle's and the map's kept sets.
- Integration: a trigger scene over a battle through the harness,
  reading the voice requests.
- Manual: the delayed-clip web check above.

## Completion notes

*(Filled in by the session that completes the ticket.)*
