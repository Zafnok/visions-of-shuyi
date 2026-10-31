---
id: "0134"
title: A mutants shard logs its disk and memory
type: infra
milestone: M0 Foundation
model: implementation
effort: medium
status: todo
blocked_by: ["0133"]
nick_input: none
completed:
---

# 0134 — A mutants shard logs its disk and memory

## Context

In PR #483's two mutants runs (40984462041 and 40996161089, ticket
0133), one shard of eight in each run died before its time limit:

- run 40984462041, shard 7: *The runner has received a shutdown
  signal* after 73 minutes;
- run 40996161089, shard 4: *The hosted runner lost communication with
  the server. Anything in your workflow that terminates the runner
  process, starves it for CPU/Memory, or blocks its network access can
  cause this error* after 86 minutes.

Both end as failed or cancelled shards, so `mutants (diff)` is red with
no missed mutant. GitHub gives this message when a hosted runner runs
out of memory or disk. A shard builds each mutant incrementally in two
copies of the tree under `/tmp` (ADR-0073), and those copies grow as
the shard moves through crates; nextest also runs many test processes
at once. Nothing in the log says which one ran out, so the fix can't be
chosen yet.

Ticket 0133 gave big diffs more, smaller shards, which may be enough on
its own. This ticket makes the next loss diagnosable.

## Nick input

None.

## Scope

**In:**
- `.github/workflows/mutants.yml`, job `pr`: a sampler that writes the
  free disk of `/` and `/tmp`, the free memory and the size of the
  mutant copies to the job log about once a minute while cargo mutants
  runs, and the same once after it ends (`if: always()`).

**Out (do not do):**
- Any change to what is tested (`--timeout`, `--jobs`, the nextest
  profile, the shard rule): ADR-0100.
- A fix for the losses before the sampler has shown their cause; when
  it has, write the fix as its own ticket.

## Implementation steps

1. In the step *Run cargo mutants on this shard of the diff*, start a
   background loop before cargo mutants, for example
   `( while sleep 60; do echo "::group::resources $(date -u +%T)"; df -h / /tmp; free -m; du -sh /tmp/cargo-mutants-* 2>/dev/null; echo "::endgroup::"; done ) &`,
   and stop it after (the step's `run` becomes a `|` block; keep
   `CARGO_INCREMENTAL=1` and every cargo mutants flag as they are).
   Check where cargo-mutants puts its copies (`/tmp` by default) and
   name that directory.
2. Add a step after it, `if: always()`, printing `df -h / /tmp` and
   `free -m` once.
3. One sentence in the `run-gates` skill, *Mutation testing (CI only)*:
   a shard that died early has its disk and memory in the `resources`
   groups of its log.

## Acceptance criteria

- [ ] A shard's log shows a `resources` group about once a minute with
      disk, memory and the copies' size, and one after cargo mutants
      ends.
- [ ] The cargo mutants command line is unchanged apart from how the
      step's `run` is written.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None in Rust: a workflow change. The shards only run on a PR with
  Rust changes, so either work this ticket's PR on top of a small Rust
  change of its own ticket, or check the sampler's output on the next
  PR with mutants and say so in the Completion notes.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
