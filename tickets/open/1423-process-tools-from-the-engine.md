---
id: "1423"
title: "The ticket lint, next-id, pr-size and the gates runner from the engine's shuyi-process, with the game's rules as config"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#3448", "engine#3449"]
nick_input: none
completed:
---

# 1423 — The ticket lint, next-id, pr-size and the gates runner from the engine's `shuyi-process`

## Context

The engine's redundancy sweep of 2026-11-28 found that the game's
process crate (0188) keeps old forks of the engine's tools:

- `crates/xtask/src/tickets.rs`: 1,193 lines, 516 before its tests;
- `pr_size.rs`: 968 lines, 446 before its tests;
- `mentions.rs`: 94 lines.

It also found six open tickets that port more of the engine's tools one
at a time: 0131, 0143, 0144, 0145, 0147 and 0181, about 2,000 planned
lines. Each says it ports a done engine twin.

Engine#3448 moves the engine's ticket lint and next-id into a light crate,
`shuyi-process` (only `serde`, `serde_norway` and `serde_json`), set up by
a config file per repository. Engine#3449 adds pr-size, its hook and the
gates runner. Engine#3448 asks Nick whether every game gets them. Those
tickets' Out lines named that as the engine step that needs Nick's OK.

**If Nick says no in engine#3448**, close this ticket with that note, and
0131–0181 port as planned.

## Nick input

None here. Engine#3448 asks Nick.

## Scope

**In:**
- The process crate depends on `shuyi-process` at the pin.
- `tools/process.ron` holds the game's rules:
  - the id blocks;
  - `engine#` as the other repository's prefix;
  - `status: parked`;
  - an empty `blocked_by` allowed on a blocked ticket waiting on an
    engine ticket;
  - the thresholds of ADR-0101 (about 3,000 lines).
- The game's own rules stay as hooks it passes: the Nick-quote warning,
  the playtest headings, and the moved-marker warnings in pr-size.
- Delete the game's `tickets.rs`, `pr_size.rs` and `mentions.rs`, and
  `code_lines.rs`' own scanner if engine#3448's crate carries
  `rust_lines`. Keep their game-only test cases as tests of the hooks.
- **Either order with 0131, 0143, 0144, 0145, 0147 and 0181:** if one is
  done, delete its port along with the rest. If not, the engine's crate
  already does its Rust part, so this ticket's PR also moves it to
  `done/` with the note "Done by 1423". That ticket's skill and doc edits
  are done here.

**Out (do not do):**
- Changing a lint rule: every open and done ticket lints as before,
  apart from the checks the ported tickets planned to add.
- The CI noise fixes (0150).

**Estimated size:** about 2,600 lines changed, nearly all removed (about
2,250 removed, 350 added).

## Implementation steps

1. Read engine#3448's and #3449's completion notes for the config's
   fields and the hook API.
2. Write `tools/process.ron`. Run the engine's lint with it over
   `tickets/`, and compare its findings with the game's lint. Make every
   difference a config field or a hook, or accept it in the completion
   notes.
3. Swap the commands in `crates/xtask` (or the process crate) for thin
   callers, and delete the forks.
4. Update `.claude/settings.json`'s hooks, the `work-ticket` and
   `write-ticket` skills, and AGENTS.md where they name the old files.

## Acceptance criteria

- [ ] `cargo xtask ticket-lint` passes on `main`'s tickets with the
      engine's crate, and fails on the same broken fixtures as before.
- [ ] `cargo xtask pr-size` gives the same counts on the last 10 merged
      PRs, apart from differences the completion notes list.
- [ ] `crates/xtask/src/tickets.rs`, `pr_size.rs` and `mentions.rs` are
      gone, or are only callers under 60 lines each.
- [ ] 0131, 0143, 0144, 0145, 0147 and 0181 are in `done/`, each with
      its note.

## Tests required

- Unit: the game's hooks (the Nick quote, the playtest headings, the
  moved markers) with the old tests' cases.
- Integration: the lint over `tickets/` in CI's `tickets` job.

## Completion notes

*(Filled in by the session that completes the ticket.)*
