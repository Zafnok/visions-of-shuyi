---
id: "0153"
title: "Research: keep the story private, by cheaper private CI or by the story in the assets repository"
type: research
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0153 — Research: keep the story private, by cheaper private CI or by the story in the assets repository

## Context

Nick, 2026-11-08, after playing the Act 1 build:

> the writing is absolute dogshit and needs immediate repair at a dialogue level. The arcs and relationships I think are overall fine but the manner of speaking is robotic and awful. This is probably the 3rd or 4th iteraiton of trying to repair this and at this point I am leaning towards just introducing a human writer. They may look at our arcs and beats and decide they like them or hate them and throw them out entirely if they can't work with them for making compelling dialogue. Expanded in bullet below:
>    * So at this time, we should pause all writing since you can't write for shit
>    * And instead I will make an ad on Reddit probably INAT subreddit
>    * In order to make the ad, the game needs several components first:
>       * We need to have an easy to use scriptwriting kit (script like the dialogue script) that a nontechnical person can use
>          * Expose tone (for AI), var names, flags, and emotion (for the portrait)
>          * We should have an easy playtesting process for them to see how their script renders and add skip ahead / trigger event etc so that certain changes can be seen quickly.
>       * They need limited access to github repo
>          * Only able to make PR or issue and only for the scripts/ folder or whatever
>       * Likely we need to make github repo private
>       * Likely we need to move ALL assets to assets repo like including the script & sound files
>       * Basically, I want to show off the game in an extremely limited manner, without risking outright cloning, and to trial run some writers by asking for i.e. the pre-ch1 battle script to be rewritten as a trial run. This should be able to asked to many writers and pick best one. Without risking us getting copied

This ticket was "The game repository goes private". Measured on
2026-11-09, GitHub-hosted CI on a private repository would cost about
**$1,000 a month** (macOS tests about $480, Linux CI $210, the playtest
bots $105, mutation testing $105, Windows tests $90, CodeQL $50), against
Nick's target of about $20. Nick already pays for GitHub Pro.

Nick, 2026-11-10:

> im not convinced it makes sense to spend like $1000 per month on
> keeping it private. Unless you can prove it will cost under $30/mo to
> keep it private with same velocity then I don't think it makes sense

and, asked what to do with the scripts and story documents if the
repository stays public:

> make story files private, they probably should just use the existing
> assets repo, and then make it so that we have a skill to make
> cross-repo PRs like we have for the sprites and such already. However,
> I really want to explore the private CI cost reduction too. So
> probably note both of these options in 0153 and turn it into more of a
> research ticket. Probably should use Ultracode on 0153 when its time.

So this is now a **research ticket**: price and compare the two ways to
keep the story private, recommend one, let Nick pick, and write the
tickets that build it. **Run it as a multi-agent workflow (Ultracode)**,
as Nick asked: for example one agent per option, plus a verifier that
re-checks every price and number against its source.

### The bar

"Under $30 a month with the same velocity". Velocity is measured, not
judged, from the last 30 days of this repository:

- time from a push to `ci-result` green on a PR (median and 90th
  percentile);
- PRs merged a day, and how many PRs' CI runs at once at the busiest
  hour;
- the gates themselves: every check that runs today still runs, on the
  same events (`AGENTS.md` rule 5: never weaken a gate). An option that
  moves a check to nightly or to release only is a change to a gate:
  say so plainly, and it doesn't meet "same velocity" unless Nick says
  it does.

Measure the way `reference-measuring-actions-usage` says (agents'
memory): `gh run list` stops at 1,000 runs, about 6 days here, so count
each workflow's runs a week through the API
(`repos/Zafnok/visions-of-shuyi/actions/workflows/<file>.yml/runs?created=A..B&per_page=1`,
`.total_count`), time the jobs of a sample, and scale.

### Option A: the whole repository private, with CI that costs less

What going private changes (from the first version of this ticket;
check GitHub's and SonarSource's current docs and prices):

| What | Where | What changes when private |
| ---- | ----- | ------------------------- |
| Actions minutes | every workflow | Free on public. On a private repository, Pro includes $18 of minutes a month (Linux 2-core $0.006 a minute, 1-core $0.002, Windows $0.010, macOS $0.062, Nov 2026), shared by all private repositories |
| Mutation testing on every PR | `mutants.yml`; ADR-0047, ADR-0100 (`:33`: "the repository is public, so standard runner minutes are free") | One of the largest users of minutes |
| Bots playtest | `playtest.yml` (every battle 100 times when a lever changed) | Minutes |
| Release | `release.yml` (Windows, Linux, macOS universal, web) | macOS minutes cost ten times as much |
| The browser build | `pages.yml`; README `:8`, `:77` | Pages on a private repository needs Pro (Nick has it). The site itself stays public. It is the whole game today, with the debug tools; the demo (0914, 0915) changes what is public |
| Branch rules | the ruleset `protect main` | Enforced on a private repository with Pro |
| CodeQL | `codeql.yml`; required check `codeql-result`; ADR-0008 (`:14-15`: "We use them only because this repo is public") | Code scanning on a private repository is GitHub Code Security, sold to organizations. CodeQL's own license allows running it outside GitHub only on open-source code |
| OpenSSF Scorecard | `scorecard.yml` (uploads SARIF) | Made for public repositories; its upload needs code scanning |
| zizmor's upload | `security.yml` `zizmor` job (SARIF) | Same upload problem; zizmor itself still runs |
| Secret scanning and push protection | repository settings; ADR-0008 | Free on public only; gitleaks (MIT) in `security.yml` could replace it |
| SonarCloud | `ci.yml`'s coverage scan, `sonar-triage.yml`, `tools/sonar-triage/`, the `sonar-fix` and `sonar-triage` skills, ADR-0071 | Free for public projects only. Nick values it: its issues are filed on demand, so the count filed so far undersells it |
| Dependabot | `.github/dependabot.yml` | Works; its PRs use minutes |
| The engine's watcher | `engine-watch.yml`, started by the engine's `notify-dependents.yml` | Works if its token reaches a private repository |
| The engine's links to the game | the engine's `AGENTS.md`, its tickets, `crates/xtask/src/tickets.rs:57` | Read only for Nick |
| What the repository says about itself | `LICENSE`, README, `AGENTS.md`, ADRs 0008, 0013, 0014, 0032, 0040 | Untrue once private. `LICENSE` is Nick's to change (`AGENTS.md` rule 6) |

What to price, each against the bar:

- **Runners of our own on rented machines**, not GitHub's: dedicated or
  cloud machines (for example Hetzner, OVH, AWS or GCP spot), started
  per job or always on, with GitHub's self-hosted runner or a service
  that runs ephemeral runners on the account's own cloud. How many
  machines it takes to match today's parallel jobs at the busiest hour,
  what that costs, and who looks after them when one breaks.
- **macOS**: the tests run on macOS on every PR today. A rented Mac (for
  example a Mac mini by the month) against what dropping macOS from PRs
  would mean (a gate change: see the bar).
- **Windows with MSVC** on a rented machine (the CI builds with MSVC;
  `AGENTS.md`, *Environment*).
- **Cheaper GitHub minutes**: ARM Linux runners, 1-core runners for jobs
  that don't need two, caching (sccache or similar) that cuts minutes
  without dropping a check.
- What is lost and what replaces it: CodeQL, Scorecard, secret scanning,
  SonarCloud's price for a private project of this size.
- **Never Nick's own PC** (fans; agents' memory): no runner on it.

### Option B: the code stays public, the story moves to the private assets repository

The bought art already lives in `Zafnok/visions-of-shuyi-assets`
(ADR-0040; `shuyi.ron`'s `private_assets`, `assets-private.rev`, the
`PRIVATE_ASSETS_KEY` deploy key, `cargo shuyi private-assets --pin`).
Nick's idea: put the scripts and the story documents there too, and add
a skill for changes that cross the two repositories (a PR in the assets
repository first, then the game PR that moves `assets-private.rev`, as
the `engine-change` skill does for the engine).

What to work out:

- **What moves**: `assets/dialogue/*.dlg` (all of it, or Chapters 2–8
  only, since Chapter 1 is in the demo anyway), `docs/story/`, and the
  story parts of `docs/design/`. Where they go in the assets repository
  (`game/` is embedded into builds; the documents aren't game files).
- **What it costs in speed**, the reasons 0152 gave for a mirror instead
  of a move, each checked and sized:
  - every gate loads the scripts (`trpg_content::load_with_scripts`,
    `crates/content/src/lib.rs:271`): either the public gates run on
    the test scripts (`assets-test/dialogue/`) and a CI job with the key
    checks the real ones, or every job checks out the private copy.
    Which jobs, and whether Dependabot's PRs (no Actions secrets) and
    the 60+ worktrees still pass;
  - `docs/story/` is named in about 112 lines of 68 files outside
    `tickets/` (code, skills, workflows), and real scene ids in about 15
    test files (2026-11-08);
  - tickets that edit `.dlg` files without writing words (music, places,
    line ids, voices, translations) become two PRs and a pin move each.
- **The cross-repository skill**: what exists today for the art (the
  `work-ticket` skill's private-assets steps, ADR-0040, ADR-0051) and
  what a skill for the story adds.
- **The writer's access**: a writer can never be given the assets
  repository (the bought art's license forbids passing the files on,
  ADR-0040). So 0152's mirror would copy from the assets repository
  instead of this one, or the story goes to a third private repository.
  Say which and what 0152 then changes.
- **The public history**: moving files doesn't remove them from this
  repository's history. The scripts and every story document written so
  far stay readable in old commits. Rewriting a public repository's
  history (git filter-repo and a force push) breaks every clone,
  worktree and open PR, and GitHub keeps old PR views. Say what moving
  protects (the hired writer's new script and new story work) and what
  it can't (everything already pushed).

### Option C: nothing private

For comparison: everything stays public. The demo (0914) still keeps
Chapters 2–8 out of what players download; anyone who finds the
repository reads the whole story.

## Nick input

**Decision**, after the research. The session shows Nick the report's
summary as one table (each option's monthly cost, velocity against
today, what is lost) and asks with the question pop-up which option to
build, with "describe your own". No technical choice inside an option
goes to Nick.

## Scope

**In:**
- The measurement above, from this repository's Actions data.
- A report, `docs/research/0153-keeping-the-story-private.md` (the
  folder is new): each option priced and measured against the bar, with
  sources and dates for every price; a recommendation.
- Nick's decision, quoted in the report.
- Follow-up tickets for the picked option (`write-ticket` skill), for
  example: the move and its skill (Option B), or the runners and the
  workflow changes (Option A). Each states how 0152, 0766 and 0915
  change, and the chain is checked both ways.
- If Option A wins: an engine ticket in the engine's `28xx` block (docs),
  with the `engine-change` skill's hand-off, saying the game's
  repository is private and links to it read only for Nick.

**Out (do not do):**
- Making the repository private, moving any file, adding runners or
  changing any workflow: the follow-up tickets do that.
- Editing `LICENSE` (Nick's).
- Running CI, a runner or mutation testing on Nick's PC.
- The engine's repository: it stays public under MIT OR Apache-2.0.

## Implementation steps

1. Read ADR-0040, ADR-0008, ADR-0047, ADR-0071, ADR-0100, 0152 and the
   `work-ticket` skill's private-assets steps.
2. Measure the last 30 days: runs a week per workflow, job minutes by OS,
   push-to-green times, concurrency at the busiest hour.
3. Run the research as a multi-agent workflow: Option A's pricing, Option
   B's move and its costs, and a verifier that re-fetches every price
   and re-counts every number before the report uses it.
4. Write the report. Size: about 400 lines, plus this ticket.
5. Ask Nick. Record his answer.
6. Write the follow-up tickets; update 0152, 0766, 0915 and
   `docs/ROADMAP.md` (Lane J) to point at them.

## Acceptance criteria

- [ ] `docs/research/0153-keeping-the-story-private.md` prices Options
      A, B and C, each against "under $30 a month, same velocity", with a
      source and date for every price.
- [ ] The velocity numbers (push to green, PRs a day, concurrency) come
      from this repository's last 30 days, and the report gives the
      sample's dates.
- [ ] Nick's choice is quoted in the report.
- [ ] The follow-up tickets exist, and 0152, 0766 and 0915 name them
      where they depend on them.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

- None (research). The follow-up tickets carry the tests.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
