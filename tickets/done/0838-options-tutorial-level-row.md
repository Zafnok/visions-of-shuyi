---
id: "0838"
title: Options can change the campaign's tutorial level
type: feature
milestone: M7 Chapter 1 & game flow
model: implementation
effort: medium
status: done
blocked_by: ["0835", "0839"]
nick_input: sign-off
completed: 2026-10-15
---

# 0838 — Options can change the campaign's tutorial level

## Context

Ticket 0058 (`docs/design/options.md`, *Where options are kept*;
`docs/design/tutorial.md`): the tutorial level (Full / Short / None),
picked at New Game and kept with the campaign by 0835, **can be changed
later in Options** while that campaign is loaded. It is a value row: no
yes/no question of its own; like the other rows it is kept or discarded
when the player leaves Options (0839).

## Nick input

**Sign-off:** after merge, in a battle open Options, set `Tutorial` to
None, and see that tips stop.

## Scope

**In:**
- A `Tutorial: Full / Short / None` row on the Options screen, shown only
  while a campaign is loaded (in a battle, at Preparations); not on the
  title's Options. Its place on the screen is *the agent's starting rule*
  (next to the game mode row).
- Like every Options row, the change shows at once and is kept only when
  the player picks Keep on leaving (0839; Discard puts the level back).
- It changes the running campaign's `tutorial`, kept in the
  campaign's next save, the way the mode row goes through the flow
  (ADR-0050, `Ctx::campaign_mode`).
- Words in the language files (American English; `test` pack too).

**Out (do not do):**
- Lessons themselves (0456, 0457); already-skipped lessons stay in Guides
  (0458) with no change here.

## Implementation steps

1. `Ctx` exposes and changes the campaign's level (as `campaign_mode`).
2. The Options row (view + glyph skin, ADR-0054).
3. Tests.

## Acceptance criteria

- [x] The row shows in a battle and at Preparations, not on the title.
- [x] Changing it changes which tips show from then on, and the level is
      in the next save (test).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot: the Options view with the row.
- Integration: start at Full, set None in Options, the next tip doesn't
  show; save and load keeps None.

## Completion notes

- `Tutorial: Full / Short / None` is a value row on the Options screen,
  shown only while a campaign is loaded (`Ctx::campaign_tutorial` is
  set), right under `Game mode`. Left/right step it (stopping at the
  ends), Confirm steps it round, like the other value rows. It changes
  `Ctx::campaign_tutorial` at once, so tips follow it straight away.
- Leaving Options counts it as a change: Keep keeps it, Discard puts the
  level back as it was when the screen opened.
- The game flow (`sync_mode`) takes `Ctx::campaign_tutorial` into its
  campaign each frame, so the next save (suspend or chapter save) holds it.
  `Ctx` already had the field from 0835; no new method was needed.
- **Skin fix:** the Options panel was a fixed 20 rows. With a campaign the
  rows now need 18 lines, which put `Restore defaults` on the bottom
  border; with the bought pictures and a campaign it already did before
  this ticket (the `map_look_swaps_the_skin_at_once_and_is_remembered`
  snapshot showed it). The glyph skin now grows the panel down to fit its
  rows (never smaller than before), and the message moves under it.
- Tests: unit tests for the row (values, stepping, Keep/Discard, hidden
  without a campaign), a glyph test for the taller panel, and integration
  tests: None set at Preparations stops the battle's start tip
  (snapshot of the screen with the row), Discard in a battle puts it back
  and the title has no row, and a level set in a battle is in the suspend
  save and still None after Continue in the next launch.

**the agent's starting rules** (Nick can veto):
- The row sits right under `Game mode`.
- Left/right stop at Full and None; Confirm goes round
  (Full → Short → None → Full), as on the other value rows.
