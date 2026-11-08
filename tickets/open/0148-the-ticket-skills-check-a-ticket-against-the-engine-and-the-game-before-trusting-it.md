---
id: "0148"
title: The ticket skills check a ticket against the engine and the game before trusting it
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0148 — The ticket skills check a ticket against the engine and the game before trusting it

## Context

The retro of game PRs #513–#580 (2026-11-08) found several slips that
need judgement rather than a lint, so they belong in the skills:

- **Engine-shaped code written in the game.** 0721's `cargo xtask voice`
  (#540) put a generic voice cast reader and clip plan (about 240 lines)
  in the game's xtask. Three days later engine 2250/2251 (engine #291,
  #303, +1,156 lines) rebuilt that logic, and game 1242 will delete the
  game's copy. The ticket never ran `engine-change` §1's row-by-row
  test. #540 reported "workaround no", and the friction ratio (1.6×)
  could not catch it because the estimate already counted the generic
  code.
- **An engine API assumed, not checked:**
  - 0859 (#563) assumed screens knew a clip's length, which brought in
    engine 2255, a workaround and engine 3210 → 3212;
  - #566 unblocked 1170 from the pin ledger alone, and starting it 37
    minutes later re-blocked it (#571) on engine 2388 and 3096.
- **A calibration tuned around a known fault.** #535 re-tuned Chapter 2
  while it knew of a bot fault, though its ticket said to fix bot faults
  first. #538's fix then pushed a band out and 0524 re-tuned.
- **Work in progress shipped beside its fix.** #563 shipped a
  0.08 s-per-character workaround on top of pin #572, which already had
  engine 2255's end-of-clip report. Its removal (0863) is now redo work.
  #563 was also opened before its engine blocker merged, and sat red
  about 10 hours without being a draft.
- **Decisions whose evidence lives outside a repository.** The test
  scripts behind ADR-0109 (#524) exist only in `D:\voice-lab\`.
- **Art waiting on Nick inside a chapter ticket's Scope In:** deferred
  three times (0848, 0849, 0850 → 0862).
- **Pictures linked at the head branch**, which is deleted on merge:
  #516, #531, #534 and #555 have dead links.
- **A 2-line, `effort: low` engine fix sent down "Ticket and
  follow-up".** Engine #305 then #313 took 24 hours where "Doing it now"
  fits.

The engine's twin is engine#2584.

## Nick input

None.

## Scope

**In:**
- `write-ticket` and `engine-friction`: a tool or infrastructure ticket
  says, part by part, engine or game, with `engine-change` §1's test
  written in the ticket. A part another game would write the same way
  is an engine request first. Generic logic over about 100 lines is a
  signal beside the ratio.
- `write-ticket` and `work-ticket`: every engine API a ticket's steps
  call is named and checked at the pinned revision (`cargo xtask engine`
  checkout, or `gh api` at `shuyi-engine.rev`). A ticket is unblocked by
  starting it, or by checking its steps against the pin, never from the
  ledger alone.
- `work-ticket`:
  - a calibration or tuning ticket that meets a bot fault stops, files
    the fault, and blocks on it;
  - a PR whose engine blocker hasn't merged opens as a draft;
  - before moving onto a pin, grep for `engine-friction: engine#NNNN`
    markers the pin ends, and remove them in the same PR or say why not;
  - scripts and data behind a decision are committed (under `tools/` or
    `docs/`), never left on a local disk.
- `write-ticket`: art Nick must build or buy goes in its own blocked
  ticket, not a chapter's Scope In.
- `work-ticket` §6: pictures link at a commit
  (`…/blob/<sha>/…`), not at the branch.
- `engine-change`: a small, low-effort engine fix goes "Doing it now",
  as one engine PR holding its ticket and the fix.

**Out (do not do):**
- Lints for these (0143, 0144, 0147 and 0151 do the deterministic
  ones).
- The engine's skills (engine#2584).

## Implementation steps

1. Edit `write-ticket`, `work-ticket`, `engine-change` and
   `engine-friction` under `.claude/skills/`, each rule in its section
   with its one-line reason and the PR it comes from.
2. Run `pwsh -File scripts/sync-agent-skills.ps1` and `-Check`.

## Acceptance criteria

- [ ] Each bullet of *In* is a rule or step in its skill, with its reason.
- [ ] `pwsh -File scripts/sync-agent-skills.ps1 -Check` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None beyond the gates (skill text).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
