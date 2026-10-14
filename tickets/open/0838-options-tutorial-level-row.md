---
id: "0838"
title: Options can change the campaign's tutorial level
type: feature
milestone: M7 Chapter 1 & game flow
model: sonnet-5
effort: medium
status: todo
blocked_by: ["0835"]
nick_input: sign-off
completed:
---

# 0838 — Options can change the campaign's tutorial level

## Context

Ticket 0058 (`docs/design/options.md`, *Where options are kept*;
`docs/design/tutorial.md`): the tutorial level (Full / Short / None),
picked at New Game and kept with the campaign by 0835, **can be changed
later in Options** while that campaign is loaded. It is a value row: no
yes/no question, it changes at once.

## Nick input

**Sign-off:** after merge, in a battle open Options, set `Tutorial` to
None, and see that tips stop.

## Scope

**In:**
- A `Tutorial: Full / Short / None` row on the Options screen, shown only
  while a campaign is loaded (in a battle, at Preparations); not on the
  title's Options. Its place on the screen is *Claude's starting rule*
  (next to the game mode row).
- It changes the running campaign's `tutorial` at once, kept in the
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

- [ ] The row shows in a battle and at Preparations, not on the title.
- [ ] Changing it changes which tips show from then on, and the level is
      in the next save (test).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot: the Options view with the row.
- Integration: start at Full, set None in Options, the next tip doesn't
  show; save and load keeps None.

## Completion notes

*(Filled in by the session that completes the ticket.)*
