---
id: "1112"
title: "Generic screens and widgets move to the engine: menus, help, key bindings, layout picker, options shell, debug menu, dialogue player, text"
type: infra
milestone: "Engine: carve"
model: opus-5.5
effort: high
status: todo
blocked_by: ["1111"]
nick_input: none
completed:
---

# 1112 — Generic screens and widgets move to the engine

## Context

`docs/engine/boundary.md`, *ui*: these are generic with this game's rows
or details mixed in:

- `widgets/` (menu, help, wrap);
- `screens/key_bindings` (only its `ROWS` are the game's; 1106 already
  made them injected);
- `screens/layout_picker` (layouts are data after 1106);
- `screens/options` (a generic shell with this game's rows);
- `debug*` (the menu, glyph sampler, portrait viewer, backdrop camera are
  generic; promote/reclass, skin switch, test cinematic and scene preview
  are the game's);
- `dialogue.rs` and `screens/dialogue` (the player state machine and the
  box are generic; the lead and cast are the game's);
- `words.rs` (text by key; the names of classes and items are the
  game's);
- `tips.rs` (`{Action}` fill and the seen-set are generic; placing the
  tip on the battle map is the game's);
- `audio.rs` (assumes four menu cue ids exist), `storage.rs`.

Each screen already follows ADR-0054 (logic, view, skin) or is moved so
it does.

## Nick input

None. Nothing a player sees changes.

## Scope

**In:**
- Move the generic parts to `shuyi-ui::{widgets, screens, debug,
  dialogue, text, audio, storage}`; the game injects rows, entries,
  cue ids (`AudioConfig { menu_move, menu_select, menu_cancel,
  cursor_move }`) and the lead/cast hooks into dialogue
  (`DialogueHooks`).
- The debug menu takes entries from the game (`Vec<DebugEntry<G, A>>`).
- The options screen takes rows from the game: each row a label key and
  a typed value (`Toggle`, `Choice`, `Volume`, `Action`).

**Out (do not do):** cinematics (1113); the map view (1115).

## Implementation steps

1. Move one module at a time, each with its tests; inject what is the
   game's; keep aliases.
2. Snapshots must not change. If one does, the move changed a look: fix
   the move, never accept the snapshot.

## Acceptance criteria

- [ ] The toy game (1111) uses the engine's menu, options shell, key
      bindings screen and dialogue box in a Harness test.
- [ ] No snapshot changes; every test passes; `check-keys` and
      `check-text` pass.
- [ ] `docs/engine/boundary.md` rows updated.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: toy-game Harness tests for each moved screen.

## Completion notes
