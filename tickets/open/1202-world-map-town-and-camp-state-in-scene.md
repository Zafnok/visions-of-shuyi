---
id: "1202"
title: Expose world map town and camp state through the existing scene tool
type: feature
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: high
status: todo
blocked_by: ["1181", "1007", "1008", "1003"]
nick_input: none
completed:
---

# 1202 — World map, town and camp data in scene

## Context

Game PRs #483 and #495 shipped the world map and random training groups;
#477 expanded camp. The 2026-11-01 audit found that the game's CLI/MCP
scene adapter (`crates/xtask/src/game_tools.rs`, `scene`) still requires a
`BattleScreen`, so those screens answer "no battle on screen".
Ticket 1181 intentionally covered battles only; this expands that adapter
as gameplay has expanded, without claiming an unmet 1181 criterion.

The engine's existing `Headless::with_scene` and `Scene::data` already
accept arbitrary plain data. Screen `campaign()`/`view()` methods hold
this game's meaning. This needs no new engine operation or game concepts
in the engine (ADR-0063, ADR-0054).

## Nick input

None. Tests inspect scene/state data. No new UI or design decision, and
no check after merge. This tool integration does not block Act 1 content.

## Scope

**In:**
- World-map scene kind/text/data: stable place ids, army and cursor place,
  shown/open/cleared places, traversable roads and current random groups
  (level/size/seed where already public campaign state), plus active menu.
- Town-shop scene data from the shop's current campaign/view: town/tab,
  focused deal, gold/stock and confirmation state. Camp scene data from
  its campaign/view: current section/list/focus and available entries.
- The world map's internal `Open::Town` chooser is distinct from a
  `TownScreen` shop: while the map is topmost, expose the chooser's town,
  available shops and focus before any shop screen is pushed.
- Dispatch by the top supported screen before falling back to a battle
  below it, so a covered map's stale campaign is not reported as the
  active town/camp state. Include screen/stack identity in the result.
- Deterministic, documented scene shapes and integration tests through
  the same callback that CLI and MCP already use.

**Out (do not do):**
- New tools, MCP protocol, UI, skins, key bindings or changes to the engine.
- Revealing hidden story places or unreached content to a player-facing
  UI. The developer scene reports currently shown places and actual
  campaign state; it does not invent future random rolls.
- Generalizing every screen or adding a debug command API.
- Browser captures (0141), architecture goldens (0142) and bot record
  stamps (1195). Either order works; these tests use a local session and
  existing test content, not those tickets' new infrastructure.

## Implementation steps

1. Read `scene`, the existing session tests, and world-map/town/camp
   `campaign()` and `view()`. Extend the game adapter using its existing
   JSON construction pattern. Add small read-only game screen accessors
   only where ids/focus cannot be obtained from existing plain state.
2. Give each supported scene a stable kind and JSON fields, recording
   them in the existing tools/development documentation. Use ids for
   programmatic identity and optional displayed labels separately; do
   not infer behavior from cells, glyphs, pixel positions or colors.
3. Dispatch using `h.top_screen()` and supported screen types. Handle a
   town or camp above the map before the map itself, and keep the
   existing battle scene shape/behavior intact. For other unsupported
   screens retain an explicit limit naming that screen.
4. Extend `game_tools` session tests using the existing test-world-map
   debug route: read scene, travel, enter a town, make one deal, leave,
   save/reload, and read scene again. Open camp and change focus; assert
   that its returned scene is camp, not the map beneath it.
   Include a town with several shops: read its internal chooser while
   the world map is still topmost, then enter one shop and assert the
   scene kind/data changes to that active shop.
5. Add a random-group fixture from 1008 to check stable ids and saved
   state. Read scenes repeatedly without stepping and prove the calls
   do not mutate state or advance RNG. Exercise CLI/MCP through the
   existing shared session adapter rather than writing another protocol.

Expected size: about 100–180 production lines, judged against 1181's
76-line scene/session adapter, plus integration tests; under 1,200
changed lines including docs and this ticket's move.

## Acceptance criteria

- [ ] `scene` succeeds on world map, town shop and camp with the correct
      active scene kind and screen identity; an overlay does not return
      stale underlying-map data.
- [ ] World-map data identifies army/cursor, open/cleared places and
      current saved random groups; the travel/town/save/reload script's
      scene values agree with campaign state before and after reload.
- [ ] A town deal changes gold/stock in its scene immediately; camp
      focus/entries match its view. Repeated reads leave state/RNG equal.
- [ ] A multi-shop town's chooser reports town/shop ids and focus while
      the world map is still topmost, then reports the actual shop scene
      after selection; the chooser is not mistaken for a `TownScreen`.
- [ ] Existing battle scene tests pass with their shape unchanged;
      unsupported screens still explain the actual limit.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: shared session callback across world map, town and camp;
  travel/deal/save/reload and overlay priority, using existing test assets.
- State assertions: random-group identity, no mutation on scene reads,
  and unchanged battle output. Protocol transport tests stay engine-owned.

## Completion notes

*(Filled in by the session that completes the ticket: scene fields,
commands/tests, deviations and follow-ups.)*
