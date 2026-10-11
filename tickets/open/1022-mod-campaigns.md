---
id: "1022"
title: "Mod campaigns: a mod can add its own campaign, started from the title menu"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: ["1015", "0054"]
nick_input: sign-off
completed:
---

# 1022 — Mod campaigns

## Context

A campaign today is the game's one chain of chapters: `assets/data/new_game.ron`
(the starting roster and the first chapter) and `assets/chapters/*.ron`
(each chapter's scenes and battle), run by the flow screen (ADR-0035,
0801). A mod can already change these files (1012), but that **replaces**
the game's story. Total conversions are the biggest thing modding
communities make (Fire Emblem ROM hacks, Battle for Wesnoth campaigns).

Only if 0054 allows mods to add stories. If Nick said no, close this ticket
with a note instead.

## Nick input

**Sign-off:** Nick starts the example mod's two-chapter campaign (1023 adds
one if it doesn't have it) from the title and plays it through.

## Scope

**In:**
- A mod's manifest may declare campaigns: `campaigns: [(id, name,
  new_game: "path to its new_game.ron")]`; its chapters, battles and
  scenes come from the mod's files.
- `Campaigns` (or `Mod campaigns`) on the title menu when any enabled mod
  declares one; picking one starts New Game with that campaign's first
  chapter and roster, using the normal mode screen and lead select.
- The campaign id is stored in the `Campaign` and the `SaveFile`; loading
  a save of a mod campaign whose mod is off refuses with a message.
- Save slots: shared with the main game but labelled with the campaign's
  name (or kept apart, if 0054 or 0050 decided on separate slots for extra
  modes; follow that).

**Out (do not do):** a world map for mod campaigns (until 1007 is done, mod
campaigns are linear like Chapters 1–3); mod campaigns on the web build.

## Implementation steps

1. Read ADR-0035, 0039, `crates/content/src/chapter.rs` (`new_campaign`),
   `crates/core/src/campaign*`, the flow and title screens, and 1015.
2. Teach `content` to load more than one new-game definition, keyed by
   campaign id (the game's own is `"main"`).
3. Thread the campaign id through `Campaign`, the save (version bump with a
   migration: old saves are `"main"`), and the flow screen.
4. The title entry and a small list screen.
5. A two-chapter test campaign in a test mod.

## Acceptance criteria

- [ ] With the test mod on, its campaign appears and plays chapter 1 into
      chapter 2 (Harness test).
- [ ] Saving and loading a mod campaign keeps its id; an old save loads as
      the main campaign (guard test).
- [ ] With the mod off, the save refuses to load with the message (test).
- [ ] No hard-coded keys or literal screen text.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: campaign id in `Campaign` and the save.
- Integration: the test campaign through the flow screen.

## Completion notes
