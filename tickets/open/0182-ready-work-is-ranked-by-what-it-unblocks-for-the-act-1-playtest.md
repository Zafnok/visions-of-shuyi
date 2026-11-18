---
id: "0182"
title: Ready work is ranked by what it unblocks for the Act 1 playtest
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0182 — Ready work is ranked by what it unblocks for the Act 1 playtest

## Context

The retro of PRs #616–#735 ([report](../../docs/research/retro-2026-11-17-prs-616-735.md),
T3) found that nothing ranks ready tickets against the Act 1 playtest
(1064). Sessions took whatever was filed last: 47 of 100 PRs went to
privacy, release, legal and writer work, and 2 went to the critical
path. Ready path tickets such as 0852–0854, 1065, 1090 and 0606 sat
idle all week. `work-ticket` §1 says only "prefer the lowest number on
the Act 1 critical path", and picking by hand from 243 open files is
error-prone.

The retro also found the backlog contradicting the ROADMAP: 1068 is on
the path (1064 and 0512 list it) but was blocked by 1170, which
`docs/ROADMAP.md` lists under *Off the path*. Nothing checks that. The
data fix for 1068 (`blocked_by: []`, an either-order note on 1170) is
made when this ticket is filed; if 1068 still lists 1170, step 6 makes
it. This ticket adds the check so it can't come back.

Readiness is already defined (`tickets/README.md`, *Picking the next
ticket*); this ticket uses that rule as is.

## Nick input

None.

## Scope

**In:**
- `cargo xtask ready-tickets`: the ready tickets in 1064's transitive
  `blocked_by`, ranked (steps 2–3). Named so it is not confused with
  0131's `ticket-next` (the next free number).
- `ticket-lint`: an error when a ticket the ROADMAP's *Off the path*
  paragraph names is in 1064's transitive `blocked_by`.
- `work-ticket` §1 and `tickets/README.md` *Picking the next ticket*:
  with no ticket named, take the top of `ready-tickets`.

**Out (do not do):**
- `status: blocked` against `blocked_by` (0147 does it). The `parked`
  status (0181): it isn't `todo`, so a parked ticket is never ready.
- Ranking Nick's decisions and stating their cost (0184). Here
  decision tickets are only counted.
- Whether a ready `engine#NNNN` is past the pin (`work-ticket` §1
  keeps that check; this tool never uses the network).
- Rules on which kinds of off-path work may run (adoptions: 1287;
  writer tooling: 0773).
- Changing any ticket's scope or priority.

## Implementation steps

1. In `crates/xtask/src/tickets.rs`, add `title` and `nick_input`
   reads to `TicketFields` (filled in `parse_ticket`; `completed` is
   read already), and a `pub(crate) fn load(repo_root: &Path) ->
   (Vec<Ticket>, Vec<String>)` (tickets, folder read errors) that does
   the folder loop now at the top of `run`, so `run` and the new module
   share it. Make `Ticket`, `Folder` and `TicketFields` `pub(crate)`.
2. New module `crates/xtask/src/ready.rs`:
   - The chain is `trpg_engine_watch::act1_chain(open: &[OpenTicket])
     -> HashSet<String>` (xtask already depends on the crate): open ids
     reached from `"1064"` through `blocked_by`, skipping `engine#` ids
     (done ids are not in `open`). **Either order with 1287**, which
     needs the same walk for the pin issue: if 1287 is done, call it;
     if not, add it to `crates/engine-watch/src/lib.rs` here, with its
     unit test, and 1287 reuses it.
   - Ready = `status: todo`, and every `blocked_by` id is a done game
     ticket. Any `engine#` id means not ready.
   - Rank the ready chain tickets by, in order: (a) how many open chain
     tickets wait on it, directly or not, most first; (b) ready since:
     the latest `completed` date among its blockers, oldest first (no
     blockers counts as oldest); (c) id.
   - Leave `nick_input: decision` tickets out of the list. Print one
     line under it instead: `N decision tickets on the path wait for
     Nick (ask-nick)`.
   - Print another line naming chain tickets held only by `engine#` ids
     (today that would have shown 1170 under 1068).
3. Output: one row per ticket with rank, id, the count from (a), the
   ready-since date, `setup` when `nick_input: setup`, and the title.
   `--all` adds the ready tickets that are not on the chain, below a
   heading, sorted by id.
4. Wire it in `crates/xtask/src/main.rs`: a `Some("ready-tickets")`
   arm in `dispatch` (beside `code-lines`, line 110) and a line in
   `USAGE`.
5. The lint: in `tickets::run`, read `docs/ROADMAP.md`. Take the text
   from `**Off the path**` to the next `\n## `. Collect bold ids
   `**NNNN**` and bold ranges (`**1211**–**1229**` and `**1203–1206**`
   both occur; en dash or hyphen), and keep those that are game ticket
   ids. Report each one in `act1_chain`: "`<path>`: on the Act 1 path
   (1064 waits on it through `<blocker>`) but listed Off the path in
   docs/ROADMAP.md", where `<blocker>` is 1064 or a chain ticket whose
   `blocked_by` names it.
   A missing ROADMAP or paragraph is an error too.
6. In the same PR, bold every game ticket id in that paragraph that is
   plain today (0042, 0043, 0045, 0054, 0236, 0237, 0239, 0718–0726, 0825,
   0826, 0901, 0903, 0906, 0907, 0908, 1011–1024, 1048, 1049,
   1052–1054, 1183–1187). Leave engine numbers and "after
   1064" plain. If the lint fails on `main`, fix the data, never the
   rule (1068 → 1170 is the one case found when this was written).
7. Edit `.claude/skills/work-ticket/SKILL.md` §1, the "Otherwise list
   candidates" bullet: run `cargo xtask ready-tickets`, take the top
   row, and say in one sentence why. A ticket off the chain is worked
   only when the user names it. Mirror it in `tickets/README.md` and
   run `pwsh -File scripts/sync-agent-skills.ps1 -Check`.

Expected size: about 180 production lines (`ready.rs` about 130, the
lint about 50) and 200 lines of tests.

## Acceptance criteria

- [ ] `cargo xtask ready-tickets` on `main` lists only tickets that are
      ready and in 1064's transitive `blocked_by`. The PR body shows
      the output.
- [ ] No `nick_input: decision` ticket is in the list, and the
      decision count line is printed.
- [ ] `cargo xtask ticket-lint` fails on a fixture where an on-path
      ticket is blocked by a ticket in *Off the path* (the 1068/1170
      case), and passes on `main`.
- [ ] `work-ticket` §1 and `tickets/README.md` name the command, and
      `sync-agent-skills.ps1 -Check` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (`ready.rs`, and `act1_chain` in engine-watch if added here):
  the chain skips done and `engine#` ids; an
  `engine#` blocker makes a ticket not ready; ranking by count, then
  ready-since, then id; decision tickets left out and counted.
- Unit (`tickets.rs`): bold ids and both kinds of range are read from
  a fixture paragraph; plain numbers and engine numbers are not; the
  off-path error names the chain blocker; a missing paragraph is an
  error.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
