---
id: "1254"
title: "The pin review reads behaviour lines, and flags idle adoptions and workarounds the pin ends"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1254 — The pin review reads behaviour lines, and flags idle adoptions and workarounds the pin ends

## Context

The engine watcher (`.github/workflows/engine-watch.yml`) files an
`engine-pin` issue, and the `engine-pin` skill reviews each engine PR the
move passes. The retro of 2026-11-08 found the loop works: every
`Breaking:` line up to engine #302 reached the game within 37 hours. It
also found three gaps.

- **Behaviour changes have no line to read.**
  - Engine 2350 loosened `.dlg` validation and said "nothing in the game
    must change". Pin PR #547 had to add about 45 lines (`AsciiDlg`) to
    keep game ADR-0045.
  - Engine 2315 and 3210 changed the game's tests and snapshots in #572.
  - The engine is adding a `Behaviour:` line (engine#2066).
- **Adopter tickets sit idle after the pin brings their engine piece:**
  - 1209 for 7 days, 1057 for 14, and 1087, 1252, 0863 and 1068;
  - 1180 has had all 7 engine blockers done since 2026-10-22–24, while
    #520, #535, #538 and #544 kept editing the code it replaces.
  
  The issue's "Adoption tickets still open" list also repeats tickets
  (1170 and 0910 twice).
- **A workaround outlives the pin that ends it.** #563 shipped a
  0.08 s-per-character workaround on top of pin #572, which already had
  engine 2255's end-of-clip report.

**Either order with engine#2066:** if engine#2066 is done, read its
`Behaviour:` lines as below; if not, build the rest and leave the
`Behaviour:` part for a follow-up that engine#2066's completion names.

## Nick input

None.

## Scope

**In:**
- The watcher's issue and `cargo xtask engine-pin --report` list each
  engine PR's `Behaviour:` lines in the *Breaking* table, with
  `Breaking:` lines, so each gets a decision in the ledger.
  `engine-pin --check` requires one.
- The issue's adoption list is de-duplicated, and marks an adopter
  ticket whose engine piece has been inside the pin for more than 5 days
  with its age.
- The report lists every `engine-friction: engine#NNNN` marker in
  `crates/` whose engine ticket the move passes, and the open PRs that
  add one. The pin PR removes each marker or names the ticket that does.

**Out (do not do):**
- Doing the adoptions (each adopter ticket does).
- The pin PR's own mechanics (1255).

## Implementation steps

1. Read how `crates/xtask/src/engine.rs` (the `engine-pin --report`
   code) reads `Breaking:` lines from the engine's done tickets, and
   read `Behaviour:` the same way.
2. Change the issue body built in `engine-watch.yml` (or the xtask it
   calls) for the de-duplicated, aged adoption list.
3. Add the marker scan (`grep`-like over `crates/`, and `gh pr list
   --json files` for open PRs).
4. Update the `engine-pin` skill, then run `pwsh -File
   scripts/sync-agent-skills.ps1`.

## Acceptance criteria

- [ ] `engine-pin --report` lists a passed engine PR's `Behaviour:`
      lines, once engine#2066 is done (or the follow-up is named).
- [ ] The adoption list has no repeated ticket, and shows ages.
- [ ] A marker whose engine ticket the move passes is listed.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the report over fixture engine tickets and a fixture `crates/`
  tree with a marker.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
