---
id: "1287"
title: "Small engine adoptions land in the pin PR, and the rest get a ticket only when the game needs them"
type: infra
milestone: M1 Engine
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1287 — Small engine adoptions land in the pin PR, and the rest get a ticket only when the game needs them

## Context

The retro of PRs #616–#735 ([report](../../docs/research/retro-2026-11-17-prs-616-735.md),
T5) found that the pin review's adoption tickets cost more than they
give:

- **Tiny adoptions get a full ticket.** 1283 was estimated at 5 and
  measured 4 production lines. It still got a 72-line ticket and its own
  PR (#734, 32 changed lines) with a full CI run. ADR-0096 §5 and the
  `engine-pin` skill forbid adopting in the pin PR.
- **New adoptions jump the queue; old ones sit idle.** About 12 were
  done within days, none on the Act 1 path. The same idle list (11
  tickets in #700, 12 in #706 and #732) is copied into every pin PR, and
  nothing acts on it. 1282 (#735) rewrote keys #726 had added that day.
- **A stopgap ported removed engine code.** #638 copied ink into
  `crates/content/src/ink.rs` (+187/−25) and #639 deleted it (−278)
  about 2 hours later.

The skill also says "When it's unclear, write the ticket", so every
*New for games* row with any use becomes a ticket.

This ticket supersedes ADR-0096 §5 and its *Adopt in the pin PR*
alternative. Related: 0182 ranks ready work by 1064's chain, and with it
a ticket off that chain is worked only when someone names it. Either
order works (step 5). 1255 also edits the `engine-pin` skill and
`pins.md`'s header: either order, merge the text.

## Nick input

None.

## Scope

**In:**
- A new ADR that supersedes ADR-0096 §5 with three rules:
  1. An adoption under about 50 production lines is done in the pin PR.
     Its row says `adopted here: <files changed>`.
  2. `adopt → NNNN` is only for an adoption that is on 1064's chain (or
     shrinks a ticket on it), deletes game code a later engine change
     will break, or fixes a named game problem (a friction row, an open
     bug, a missed budget). The ticket's Context says which.
  3. Any other gain gets `available: <what it would let the game do>`
     and no ticket.
- The `engine-pin` skill: those rules, and never copy code the engine
  removed into the game.
- `engine-pin --check` accepts `adopted here:` and `available:`.
- The issue's adoption list shows only tickets on 1064's chain (with
  age and *idle*). The others become one count line.
- One triage of today's 12 idle tickets, in the PR body.

**Out (do not do):**
- Stale `engine#` blockers such as 1228 on engine#2109, and a lint for
  them (0147). "Unblocks NNNN" left `blocked` (1255).
- The watcher's cadence and the pin PR's single push (1255).
- Ranking ready tickets, and `work-ticket` §1's pick rule (0182).
- Doing any of the 12 adoptions.

## Implementation steps

1. **ADR.** Write it with `write-adr` (next free number; 0144 explains
   why to check open PRs). Set ADR-0096's `Status:` line to "Accepted;
   §5 superseded by ADR-NNNN", and update both rows of
   `docs/adr/README.md`. Cite #734, #638/#639 and the idle lists.
2. **Ledger check** (`crates/engine-watch/src/ledger.rs`).
   `is_fixed_only` (about line 207) also returns false for a decision
   with `adopted here:` or `available:`. `gains_undecided`'s message
   (line 233) names all four forms. `adopted` (line 122) must keep
   ignoring `adopted here:`. Add a test for that.
3. **Chain** (`crates/engine-watch/src/lib.rs`). Add
   `pub fn act1_chain(open: &[OpenTicket]) -> HashSet<String>`: the open
   ids reached from `"1064"` through `blocked_by`, without `engine#` ids.
   If 0182 is done, it added this function already: use it. In `fetch::watch` (`fetch.rs`, about line 400), store it in a new
   `Watch` field. Pass it to `render` (`render.rs`, line 116) beside
   `open_ids`, and update both callers (`main.rs` line 55 and
   `crates/xtask/src/engine.rs` line 272).
4. **The issue** (`render.rs`). In `adoptions` (line 218), list only
   chain tickets as now. End with `N more off the Act 1 path, worked
   when someone names them: <ids>`, or nothing when N is 0. Update the
   section's intro (line 152) and the *New for games* text and
   checklist line (lines 148 and 161) with the four forms.
5. **Skill and ledger header.** In `.claude/skills/engine-pin/SKILL.md`:
   - the intro (lines 21–25) and step 4's decisions (lines 101–115) get
     the new rules. Replace "When it's unclear, write the ticket" with
     `available:`. An adoption done here is measured with `cargo xtask
     code-lines --base <the pin commit>`.
   - step 1 (lines 32–36): name only on-chain idle tickets.
   - step 3: if an engine PR removed code the game uses, fix the break
     with its replacement here, or pin the commit before it. Never copy
     the removed code into the game.

   Update the decision list in `docs/engine/pins.md`'s header (lines
   12–22) the same way. If 0182 is not done, add one sentence to
   `work-ticket` §1: "an adoption ticket off 1064's chain is worked
   only when someone names it." Run
   `pwsh -File scripts/sync-agent-skills.ps1`.
6. **Triage, once.** For each of 1180, 1057, 1068, 1182, 1078, 1170,
   1241, 0236, 0237, 0825, 1228 and 1017, write one line in the PR body:
   on 1064's chain or not, and whether it is still wanted. Today 1057 and
   1068 are in 1064's `blocked_by`. Close a ticket only when another
   ticket or engine piece already does its whole job: move it to
   `done/`, say so in its completion notes, and name what does the job.
   Leave the rest as they are.

Expected size: about 70 production lines (ledger about 10, chain and
render about 60), 100 lines of tests, and about 150 lines of ADR, skill
and header text.

## Acceptance criteria

- [ ] A last-section row `fixed: …; adopted here: …` or `available: …`
      for a PR in both *Breaking* and *New for games* passes
      `gains_undecided` (unit test).
- [ ] `adopted("adopted here: x")` is empty (unit test).
- [ ] `act1_chain` follows `blocked_by` through game ids and skips
      `engine#` ids (unit test; 0182's, if it added the function).
- [ ] The adoption list over a fixture names an on-chain idle ticket
      with its age, and puts an off-chain one only in the count line
      (unit test in `crates/engine-watch/src/tests.rs`).
- [ ] `cargo xtask engine-pin --check` passes on `main`'s ledger.
- [ ] The ADR exists, ADR-0096's status line and the index name it, and
      the skill and `pins.md` header give the four decision forms.
      `sync-agent-skills.ps1 -Check` passes.
- [ ] The PR body has one triage line for each of the 12 tickets.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `ledger.rs` (the two new forms, `adopted` ignoring `adopted
  here:`), `lib.rs` (`act1_chain`), `tests.rs` (the split adoption list).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
