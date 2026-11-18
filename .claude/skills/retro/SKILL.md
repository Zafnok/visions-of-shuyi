---
name: retro
description: Run a retrospective with many agents — over the merged PRs since the last retro (what went well, what went wrong, what to change), or as a sweep of the codebase for friction (too many touchpoints, engine candidates, features built but never filled in, slow processes) — verify every finding with skeptics, write the report to docs/research/ and file the surviving action items as tickets. Use when asked for a retro, a retrospective, a look back over recent PRs, or a friction or dev-time analysis. Not for one PR's review (code-review) or for a single engine-friction judgment on one ticket (engine-friction).
metadata:
  version: "1.0"
---

# Retro

Two modes, one shape: **gather → find (many lenses) → verify (skeptics) →
synthesize → file tickets**. A finding with no numbers behind it is not a
finding.

| Mode | Asks | Report |
| ---- | ---- | ------ |
| **PRs** (default) | What went well and what went wrong in the PRs merged since the last retro? | `docs/research/retro-YYYY-MM-DD-prs-A-B.md` |
| **Sweep** | Where does dev time go that it shouldn't, across the whole codebase and history? | `docs/research/friction-YYYY-MM-DD.md` |

Ask which mode only if the request doesn't say; "retro" alone means PRs.
Run a sweep after a roadmap milestone closes or every few PR retros.

## 1. Scope

- **PRs mode:** the range starts after the last PR of the newest
  `docs/research/retro-*.md` (its file name ends `prs-A-B`) and ends at the
  newest merged PR. Without a previous retro, take the last ~100 merged PRs.
  List them:
  ```bash
  gh pr list --state merged --limit 200 --json number,title,mergedAt,headRefName \
    --search "merged:>=YYYY-MM-DD"
  ```
  Keep only the numbers in the range. Aim for about 100 PRs a run; split
  a bigger range into two runs.
- **Sweep mode:** the whole repository and its full git history.

Both modes read first, and hand every agent: the previous retro and sweep
in `docs/research/`, `docs/engine/friction.md`, `docs/ROADMAP.md`. An
agent never re-reports what those or `tickets/open/` already cover
unless it found a deeper cause or a much bigger scope, and then it names
the ticket it extends.

## 2. Fan-out

Use the harness's way to run agents in parallel (Claude Code: the
Workflow tool when the user asked for many agents or ultracode is on,
else the Agent tool; elsewhere its subagents; with none, run the lenses
one after another yourself). About 20 agents is the usual size.

Every agent works **read-only**: git, `gh`, grep, file reads and `wc`;
no edits, no commits and no cargo (another session may be building:
the `one cargo at a time` rule). The private content may be read from
`assets-private/` (or Nick's full copy, `D:\tactical-rpg\assets-private\`)
to count coverage, but no agent quotes dialogue or copies art into its
output (ADR-0040, ADR-0117).

### PRs mode: readers, then lenses

1. **Readers**, one per ~10 PRs (about 10 agents). For each PR: the body,
   the diff stat and the diff where it matters, the reviews, the CI runs
   (`gh pr view N --json ...`, `gh pr checks N`, `gh run list --branch`),
   and the ticket it closed. Each returns per PR: what it delivered, on or
   off the Act 1 critical path, size (`cargo xtask pr-size` numbers from
   its CI or the diff stat), follow-up fixes it needed (a later PR fixing
   it), CI reruns and red runs, and anything that went notably **well**
   (shipped clean, removed code, unblocked the path).
2. **Lenses**, each across all PRs in the range (about 6 agents): churn and
   redundancy (work redone or undone), prioritization (critical path vs
   self-generated work), CI cost (minutes, repeats, cancels), process
   overhead (ticket and doc lines vs code, pin-only PRs), technical design,
   test quality. Lenses read the readers' output plus the PRs themselves.

### Sweep mode: lenses only

About 10 agents, one lens each:

| Lens | Looks for |
| ---- | --------- |
| Content touchpoints | How many files and places one new unit, class, item, skill, battle, map, chapter or format touches; hand-synced registries and parallel lists |
| UI touchpoints | What a new screen, Action, text key, help bar, option row or battle command takes; co-change analysis (`git log --name-only`); edits repeated per skin |
| Engine candidates | Game code that names no game concept; engine features the game never adopted; the cost of the pin itself |
| Built, not filled in (media) | Voice, music, sounds, languages, fonts, tips, lessons, cinematics: infra built vs content coverage in % |
| Built, not filled in (visuals, platforms, systems) | Picture skin, Playdate, web, controllers, Steam, saves, editions, bots, skirmish and the other systems: code size vs real users |
| Process overhead | Lines to tickets, ADRs, docs and skills vs code, by week; mandatory steps per ticket; two-repository dances |
| Tooling sprawl | xtask commands, workflows, scripts, `shuyi.ron`: callers, fix-up commits, duplicates of the engine's tools, built ahead of need |
| Tests and gates | Test vs production lines, snapshot churn, tests pinned to tuned numbers, duplicated fixtures, overlapping gates |
| Roadmap vs effort | The history bucketed by area (overall and last 4 weeks); how much of Act 1 plays end to end; open `00xx` decisions gating it |
| Hotspots | Churn × size per file (top 25), god files, logic duplicated across crates, fix-after-fix chains |

### What every finding carries

Title; category; **evidence** (paths with line counts, commit or PR
numbers, grep counts); **cost** in dev time ("every new skill touches 9
files", "6 follow-up PRs"); proposal; size S/M/L; whether it needs Nick
(a game-design question: those go through `ask-nick`, never decided
here); the open ticket that already covers it, if any. PRs-mode readers
and lenses also return what went **well**, with the same evidence rule.

## 3. Verify

One skeptic per lens (or per group of related findings). Each re-measures
the evidence itself and tries to refute it, then gives a verdict:

- **confirmed** — the evidence holds.
- **weakened** — real, but the numbers or the scope were wrong; give the
  corrected scope (the report uses it).
- **refuted** — wrong, or not actually a cost; dropped.
- **already covered** — an open ticket or the previous retro handles it;
  becomes an amendment of that ticket.

Default to weakened or refuted when the evidence is vague. Run the
skeptics as each lens finishes, not after all of them.

## 4. Report

One synthesizer merges the verified findings (many share a root cause),
drops the refuted ones and writes the report:

1. **How it was run** — the mode, the range, the agent counts, the outcome
   (how many confirmed, weakened, covered, refuted).
2. **Headline** — 4–6 plain bullets.
3. **Action items** — a table: proposed ticket, priority, size, Nick?,
   verdict, corrected scope, and the ticket it became (filled in at step 5).
   Then **amended existing tickets**.
4. **What went well** (PRs mode) — the practices to keep, with evidence.
5. **Themes** — a short section per root cause.
6. **Full detail** — each proposal with its skeptic's notes.
7. **Dropped** — each refuted finding in one line with why.

Plain words and numbers: Nick reads it and doesn't review code. American
English. No bare `@word` anywhere (it goes on GitHub).

## 5. Tickets

First:

- **Check the previous retro's tickets.** List which are done, which are
  open, and which are superseded, and say so in the report. Close or merge
  the stale ones before filing more, so the backlog stays clean.
- **File every action item that survived** — ten or twenty is fine
  (Nick, 2026-11-17: as many as a hygienic process needs). Prefer amending
  an open ticket over a near-duplicate.
- **Group them into buckets.** Sort the action items into a few logical
  buckets (for example: Act 1 completeness, hand-kept copies to derive,
  tests, engine adoption, process and tooling, code structure, Nick's
  questions). Each bucket is a heading in the report with its new and
  amended tickets, and a ticket names its bucket's siblings in its
  Context. Several small items of one bucket that touch the same files
  become one ticket; items that would push it past about 2,000 lines
  (ADR-0101) stay separate.
- **A rule out for each rule in.** A retro that adds a rule to
  `AGENTS.md`, a skill or `tickets/README.md` removes or shortens one,
  and when a rule becomes a lint its prose goes.

Then, for each surviving action item, in the report's order:

- New work → a ticket with the `write-ticket` skill (check the number
  against open PRs too, tickets 0131 and 0144). Link the report from its
  Context.
- Covered → amend the open ticket's scope and say so in the report.
- A design question → a `00xx` ticket for `ask-nick`, never an answer.

Fill the "ticket it became" column, then open **one PR** with the report
and the tickets on a branch of its own. The PR
body leads with the headline and the action-item table.
