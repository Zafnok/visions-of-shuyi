---
id: "1171"
title: "One owner for the campaign's mode, tutorial level and lessons, and the class change screen's debug unit moves to the debug tools"
type: feature
milestone: "Engine: carve"
model: opus-5.5
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1171 — One owner for the campaign settings, and the debug unit moves

## Context

The first engine-friction sweep (1153, `docs/engine/friction.md`) found
two game-side structures (outcome 3) that need no engine change:

- **Two copies of the campaign settings** (`crates/ui/src/flow.rs`, 539
  lines against 390, 1.4×). The flow's `Campaign` holds the game mode,
  the tutorial level and the lessons done; `ctx.game.campaign_mode`,
  `campaign_tutorial` and `campaign_lessons` hold them again so the
  Options screen and the battle can read and change them. `adopt`,
  `sync_mode` and `keep_lessons` copy them back and forth every frame,
  and the block that clears them is written twice (in `sync_mode` and
  in `update` on `Transition::Pop`). About 50 lines, and a place for
  the two to disagree for a frame.
- **A debug unit in a screen** (`crates/ui/src/screens/class_change.rs`,
  759 lines against 430, 1.8×): `ClassChangeScreen::demo` (42 lines)
  builds a test unit for the debug menu; only `crates/ui/src/debug.rs`
  calls it, and its doc comment still says Preparations doesn't exist.

## Nick input

None.

## Scope

**In:**
- One owner for the three settings. The likely shape: the running
  campaign's settings live in `ctx.game` (for example a
  `CampaignSettings { mode, tutorial, lessons }` that is `None` on the
  title), the flow's `Campaign` reads them from there when it saves,
  and the copying functions go. Pick the shape that removes the most
  copying; say why in the Completion notes.
- `demo` moves to `crates/ui/src/debug.rs` (or a debug submodule), with
  its doc comment brought up to date.

**Out (do not do):**
- The flow hosting its child screens (1158). Either order works: if
  1158 is done first, the `Stage::Preparations` check in `sync_mode`
  is already a stack lookup; carry it over.
- Any change the player sees: switching to Casual in Preparations, the
  tutorial level from Options, and lessons kept in saves behave as now.

## Implementation steps

1. Write the estimate: about 50 lines removed, 15 added.
2. Find every reader and writer: `grep -rn "campaign_mode\|campaign_tutorial\|campaign_lessons" crates/`.
3. Move to one owner; the flow tests (Casual downgrade, tutorial level
   from Options, lessons in a save) stay green.
4. Move `demo`.

## Acceptance criteria

- [ ] `grep -n "fn sync_mode\|fn keep_lessons" crates/ui/src/flow.rs`
      finds nothing, or the Completion notes say why one stays.
- [ ] `grep -n "fn demo" crates/ui/src/screens/class_change.rs` finds
      nothing.
- [ ] Every flow and save test passes unchanged.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The existing flow, Options and save tests, unchanged in what they
  check.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
