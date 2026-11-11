---
id: "0530"
title: "Playtest CI plays one battle per job, so it finishes inside its time limit"
type: infra
milestone: M4 Enemy AI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: in-progress
blocked_by: []
nick_input: none
completed:
---

# 0530 — Playtest CI plays one battle per job

## Context

`.github/workflows/playtest.yml` (tickets 0506, 0514, 0515) plays every
battle in `assets/battles/` 100 times per persona (casual, normal,
hardcore) in one job, one battle after another, with
`target/release/xtask playtest "$battle" --bot all`. The job has
`timeout-minutes: 150`, and it now runs out of time and is cancelled,
on `main` too: run 42899039208 at c7e46a6a was cancelled at 150 minutes
while playing `side_letters_home`, the 13th battle of 14; PR #610's run
42901820958 the same way. So `main` gets no balance report, and a
blocked battle would go unseen.

Why:

- #607 (ticket 1071) added the Act 1 fixed skirmishes (`aldwater`,
  `millhaven_road`, `salt_road`, `smugglers_cove`): 14 battles now
  where 10 were.
- A CI machine's speed varies about 2×: the same code took 1.69 s to
  3.12 s per casual `ch01` try across runs. Full runs on `main` with 10
  battles already took 74 to 142 minutes.
- Hardcore is the slow persona. In run 42899039208 (a slow machine)
  `ch03` alone took 29 minutes of wall time (49 s a hardcore try on one
  core, on 4 cores), `side_company_ledger` 24 and `ch01` 16.

`playtest` is not a required check (the `protect main` ruleset), so the
timeout blocks no merge; but the report is the point of the job.

Raising `timeout-minutes` alone is not the fix: the run grows by a
battle every few chapters and already took 2.5 hours of one machine.

## Nick input

None. CI only; verified by the workflow's own run on this PR.

## Scope

**In:**
- `.github/workflows/playtest.yml`: one job per battle (a matrix over
  the files in `assets/battles/`, listed when the run starts so a new
  battle is played without editing the workflow), a job that builds
  `xtask` once for all of them, and a last job named `playtest` that
  gathers every battle's report into one summary, in battle order, and
  fails when a battle is blocked or was not played.
- Every battle still plays every persona 100 times in one `--bot all`
  run: the targets' "faster" check (`crates/bots/targets.ron`, Hardcore
  wins sooner than Normal) compares two personas, so a battle's
  personas are never split across jobs. The tries, seeds and numbers
  are the same as before.
- `.github/playtest-levers.sh`'s rule stays: the battle jobs and the
  gathering job run only when `detect` says a lever changed (and on
  "Run workflow"). 0515's concurrency stays as it is.
- `docs/playtesting.md`, where it says how CI plays.
- Ticket 1207 (the move onto the engine's reusable playtest workflow):
  one line that the move keeps this split.

**Out (do not do):**
- Changing the bots, the personas, the number of tries, the targets or
  when the job fails.
- Making the bots faster (a matter for the bots' own tickets).
- Splitting one battle's tries across jobs.
- The engine's reusable workflow (1207 and its engine ticket).

## Implementation steps

1. Add a `build` job (after `detect`, `if:` play): checkout, Rust,
   `rust-cache`, the macroquad system packages, `cargo build --release
   --locked -p xtask`, upload `target/release/xtask` as an artifact
   (one day), and print the battles as a JSON list output:
   the stems of `assets/battles/*.ron`.
2. Add a `play` job: `strategy.matrix.battle: fromJSON(...)`,
   `fail-fast: false`, a `max-parallel` that leaves room for the other
   workflows' jobs, a per-battle timeout with about 2× headroom over
   the slowest battle on a slow machine. Checkout, download the binary
   (`chmod +x`: artifacts lose the mode), play the battle with `--bot
   all` (keeping the exit code), upload the report and the exit code as
   an artifact, and then fail the job if the battle is blocked.
   `xtask` finds the repository from its build path, which is the same
   checkout path on every runner of this repository.
3. Rename the old `playtest` job to the gathering job: `needs:
   [detect, build, play]`, `if: !cancelled()` and play. It downloads
   every report, writes the summary (the table and the game's lines,
   without one line per try, as before) in the order of the battle
   list, and fails if any battle is blocked or has no report (its job
   failed or ran out of time), or if the build failed.
4. Battle names come into `run:` scripts through `env:`, never by
   `${{ }}` inside the script (zizmor's template-injection rule).
5. Update the header comment, `docs/playtesting.md` and 1207.

Expected size: no Rust; about 120 lines of YAML and comments, 20 of
docs, and this ticket.

## Acceptance criteria

- [ ] On this PR (it changes `playtest.yml`, a lever) the workflow plays
      every battle in `assets/battles/`, one job each, and the
      `playtest` job passes, with every battle's table in its summary.
- [ ] Each battle job plays `--bot all` with the default 100 tries.
- [ ] A blocked battle fails its own job and the `playtest` job; a
      battle job that fails or times out without a report fails the
      `playtest` job too ("not played").
- [ ] When `detect` finds no lever, `build`, `play` and `playtest` are
      skipped, as before.
- [ ] `zizmor` (CI's workflow check) passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None in Rust. The workflow's own run on this PR is the test.

## Completion notes

*(Filled in by the session that completes the ticket.)*
