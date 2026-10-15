---
id: "1053"
title: "Playdate: prepare the application to Panic's Catalog"
type: infra
milestone: Post–Chapter 1
model: sonnet-5
effort: medium
status: todo
blocked_by: ["1052"]
nick_input: setup
completed:
---

# 1053 — The Catalog application

## Context

Nick: after itch.io, **apply to Panic's Catalog** (curated, needs an
application) (`docs/design/playdate.md`, *Selling it*, Q6). An open
question there: whether Catalog allows one purchase to cover several
builds (the language builds, 1049), "checked when applying (a store
question, not a design one)".

## Nick input

**Setup:** Nick submits the application from his own Panic developer
account (Claude never submits forms or signs in for him). Claude prepares
everything to paste, and the exact steps.

## Scope

**In:**
- Read Panic's current Catalog submission page and requirements (what
  they ask for: description, pictures, video, build, price, content
  rating, review time) and write them down in the PR.
- Prepare the material in `docs/store/playdate-catalog.md` (or where
  0901 keeps store text): description, feature list, screenshots and a
  short capture of the running game, the build to send, and the answer
  to the "one purchase, several language builds" question if the page
  says; otherwise the question to ask Panic.
- Anything the requirements ask that the game lacks (e.g. accessibility
  notes, a crank-free option, save behaviour on sleep): a ticket each
  (`write-ticket`), and say which block the application.
- The step-by-step for Nick.

**Out (do not do):**
- Submitting, signing in or accepting terms on Nick's behalf.
- Changing the game.

## Implementation steps

1. Read Panic's Catalog pages (built-in browser; read only).
2. Prepare the material and the steps; write the gap tickets.
3. Hand Nick the steps.

## Acceptance criteria

- [ ] `docs/store/playdate-catalog.md` (or equivalent) holds everything
      to paste.
- [ ] Gap tickets exist for missing requirements.
- [ ] Nick has the steps; the completion notes say whether he has
      applied.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

- None.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
