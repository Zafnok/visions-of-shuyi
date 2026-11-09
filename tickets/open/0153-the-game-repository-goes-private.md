---
id: "0153"
title: The game repository goes private
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: setup
completed:
---

# 0153 — The game repository goes private

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

This ticket is the bullet "Likely we need to make github repo private".
It comes before the ad (0766 is blocked by it): the ad draws people to
look at the game, and nobody should find the source by following it.

`Zafnok/visions-of-shuyi` is public today (`gh api`, 2026-11-08:
`visibility: public`, 0 forks, Pages on). Much of how the repository is
run assumes that, because public repositories get things for free:

| What | Where | What changes when private (check GitHub's and SonarSource's current docs and prices; these are as of this writing) |
| ---- | ----- | ------------------------------------ |
| Actions minutes | every workflow | Free on public. On a private repository of a personal account: 2,000 minutes a month on GitHub Free, 3,000 on GitHub Pro (about $4 a month), shared by all private repositories (the story mirror, 0152, too). Windows minutes count twice, macOS ten times. Past the limit jobs stop (the default spending limit is $0) |
| Mutation testing on every PR | `mutants.yml`; ADR-0047 (up to 9 runners a PR, `:58`), ADR-0100 (`:33`: "the repository is public, so standard runner minutes are free") | The largest user of minutes |
| Bots playtest | `playtest.yml` (every battle 100 times when a lever changed) | Minutes |
| Release | `release.yml` (Windows, Linux, macOS universal, web) | One release uses macOS minutes at ×10 |
| The browser build | `pages.yml`; README `:8`, `:77` link `https://zafnok.github.io/visions-of-shuyi/` | GitHub Pages on a private repository needs a paid plan (Pro). The site itself stays public. It is the whole game today, with the debug tools (`--debug-tools`) |
| Branch rules | the ruleset `protect main` (active: no deletion, no force push, required checks `ci-result`, `codeql-result`, `mutants (diff)`, `deny`, `machete`, `typos`, `zizmor`, `tickets`) | Rulesets and branch protection on a private repository need Pro; on Free they stop being enforced |
| CodeQL | `codeql.yml`; required check `codeql-result`; ADR-0008 (`:14-15`: "We use them only because this repo is public") | Code scanning on a private repository is GitHub Code Security, sold to organizations, not offered on a personal account's repository. CodeQL's own license allows running it outside GitHub only on open-source code, which this isn't. So it is removed, not moved |
| OpenSSF Scorecard | `scorecard.yml` (uploads SARIF) | Made for public repositories; its upload needs code scanning. Removed |
| zizmor's upload | `security.yml` `zizmor` job (`security-events: write`, SARIF) | Same upload problem. zizmor itself (MIT) still runs and fails the job on a finding, reporting in the log or as annotations instead of code scanning |
| Secret scanning and push protection | repository settings; ADR-0008 | Free on public only. Lost unless paid (an organization's Secret Protection). A secret scan in `security.yml` (gitleaks, MIT, a pinned release binary with its checksum) replaces it |
| SonarCloud | `ci.yml` coverage job's scan (`SONAR_TOKEN`, `:216`, `:236`); `sonar-triage.yml` (`:46`: reads anonymously, "the project is public"); `tools/sonar-triage/`; the `sonar-fix` and `sonar-triage` skills; `AGENTS.md` hard rule 1's `sonar` exception; README badges `:13-14`; `sonar-project.properties`; ADR-0071 | SonarQube Cloud is free for public projects only; a private one is a paid plan priced by lines of code |
| Dependabot | `.github/dependabot.yml` | Version updates and alerts work on private repositories; the dependency graph must be switched on (a setting). Its PRs use minutes |
| The engine's watcher | `engine-watch.yml`, started by the engine's `notify-dependents.yml` with its `DEPENDENTS_DISPATCH_TOKEN` | Still works if that token can reach a private repository; check one run |
| The engine's links to the game | the engine's `AGENTS.md` and tickets link `github.com/Zafnok/visions-of-shuyi/...`; its `crates/xtask/src/tickets.rs:57` reads game tickets with `gh api` | The links stop working for anyone but Nick; the engine's lint still works with Nick's `gh` login. An engine ticket says so in its docs (below) |
| What the repository says about itself | `LICENSE` (the opening line "This repository is public so that people can see how the game is made", §2's "Hosting on GitHub" paragraph, §3c's link); README "Source-available" (`~:284`); `AGENTS.md:144` ("(public)"); ADRs 0008, 0013 (`:90`), 0014 (`:30`), 0032 (`:24`, `:46`), 0040 (`:12`) | Untrue once private. **`LICENSE` is Nick's to change** (`AGENTS.md` rule 6): the session never edits it |
| Existing forks and clones | — | None today (0 forks). A public fork made before the switch would stay public; a clone stays with whoever made it |

Nick's other "likely", moving the script and sound files to the assets
repository: once this repository is private, nobody outside can read
them, so moving them protects nothing more. The music and sound effects
are CC0/CC-BY and free to share anyway; the scripts get a private mirror
for writers instead (0152 says why not a move).

## Nick input

**Decision first, then setup.** At the start the session measures the
minutes and asks Nick (with the question pop-up, one question each,
options with their monthly cost, plus "describe your own"):

1. **GitHub Pro or Free.** Pro keeps the browser build on Pages, keeps
   `protect main` enforced and raises the minutes. Free drops both.
2. **The browser build.** Keep it on Pages as now (needs Pro); take it
   down; or move the public browser build to itch.io (ticket 0901) when
   that is ready. Whether a public build should be Chapter 1 only is a
   later choice (engine#3082, demo editions, builds a smaller program).
3. **SonarCloud.** Pay for a private project, or drop it (the scan, the
   weekly `sonar` issues, the skills, the badges). Clippy, the tests,
   coverage numbers in CI and mutation testing stay either way.
4. **Mutation testing's minutes.** Keep every PR's mutation gate
   (`mutants (diff)`), or run it once a week on `main` only, or on PRs
   with a label. The session shows the minutes each option uses a month
   from the last 30 days.

Before asking, the session counts the last 30 days of Actions time by
workflow and runner OS (`gh run list` and each run's jobs through
`gh api`), multiplies by the OS factor, and shows Nick a short table of
"minutes a month" under each choice against the plan's included minutes.

**Setup** (Nick's clicks), in this order:

1. If Pro: `https://github.com/settings/billing` → **Upgrade** to Pro.
2. When the session's PR is open and green: `visions-of-shuyi` →
   **Settings** → **General** → *Danger Zone* → **Change visibility** →
   **Change to private** → confirm. The PR's CI then runs again on the
   private repository; the PR merges only once that run is green.
3. **Settings** → **Code security** (or *Advanced Security*) → **Dependency
   graph**: **Enable**; **Dependabot alerts**: **Enable**.
4. **Settings** → **Rules** → **Rulesets** → `protect main` → *Required
   status checks*: remove the checks the PR removed (the session lists
   them, e.g. `codeql-result`), add any it added (`secrets`). **Save
   changes**. (On Free the ruleset is kept but not enforced.)
5. If Sonar is dropped: at `https://sonarcloud.io`, the project
   `Zafnok_tactical-rpg` → **Administration** → **Deletion**; and
   **Settings** → **Secrets** → delete `SONAR_TOKEN`.
6. **`LICENSE`**: the session shows Nick the sentences that become
   untrue and a suggested wording. Nick edits the file himself, or
   writes the exact text in chat and says to commit it. Until then the
   PR leaves it as it is and says so.

## Scope

**In:**
- The measurement and the four questions above, before any change.
- Workflows, by Nick's answers:
  - remove `codeql.yml` and `scorecard.yml`;
  - `security.yml`: zizmor without the SARIF upload; a `secrets` job
    (gitleaks) on pull requests and pushes;
  - `mutants.yml`, `pages.yml`, `ci.yml`'s Sonar steps and
    `sonar-triage.yml` as decided; a dropped Sonar takes
    `tools/sonar-triage/`, `sonar-project.properties`, the `sonar-fix`
    and `sonar-triage` skills (and their loaders, `pwsh -File
    scripts/sync-agent-skills.ps1`) and `AGENTS.md` rule 1's `sonar`
    exception with it;
  - the required-check names in `ci.yml`'s `ci-result` (and anything
    that lists them, e.g. `changes.yml`'s comments) kept in step.
- Docs: README (the Pages link, the badges, "Source-available",
  "Releasing"), `AGENTS.md:144` and the map's Sonar rows,
  `docs/releasing.md` (`:13`, the release page is private now: players
  get builds from itch.io), and one line at the top of each ADR whose
  reason was "public" pointing to the new ADR (0008, 0013, 0014, 0032,
  0040, 0047, 0071, 0100). Old ADRs are not rewritten.
- An ADR (the next free number): the repository is private; what was
  dropped, kept or paid and why; who can see what (Nick; the build key;
  writers see only the mirror, 0152).
- After the switch, one run of each remaining workflow is green on the
  private repository (in the PR, or by hand for the scheduled ones), and
  `engine-watch.yml` has run once from the engine's dispatch (or by hand,
  if no engine push came).

**Out (do not do):**
- Editing `LICENSE` (Nick's).
- The engine's repository (it stays public under MIT OR Apache-2.0).
  Its docs' links to the game: write an engine ticket with the
  `engine-change` skill's hand-off, in the engine's `28xx` block (docs),
  saying the game's repository is private and links to it read only for
  Nick. Name it in the completion notes.
- Moving any file to another repository (Context; 0152).
- A Chapter 1-only public build (engine#3082, then a game ticket).
- itch.io (0901).

## Implementation steps

1. Size first: about 900 written lines (workflows 250, docs and ADR
   notes 200, the new ADR 150, this ticket 300). Removing Sonar's
   tooling deletes more; `pr-size` counts deletions and may warn. Say in
   the PR that those lines are removals, and split only if the written
   part passes 2,000.
2. Measure, then ask the four questions. Record the answers in the ADR.
3. Change the workflows and docs. Run zizmor and `actionlint` if the
   gates have it.
4. Open the PR; give Nick setup steps 1–3; when the repository is
   private, rerun the PR's CI and fix what fails.
5. Steps 4–6 with Nick; once merged, run the scheduled workflows by
   hand once.

## Acceptance criteria

- [ ] `gh api repos/Zafnok/visions-of-shuyi --jq .visibility` prints
      `private`.
- [ ] No workflow uploads SARIF or reads SonarCloud anonymously
      (`grep -rn "upload-sarif\|security-events: write" .github` is
      empty, unless Nick chose to pay for something that uses it).
- [ ] The minutes table and Nick's four answers are in the ADR.
- [ ] Every workflow left has a green run on the private repository
      (links in the completion notes).
- [ ] `protect main`'s required checks match the jobs that exist.
- [ ] `LICENSE` is unchanged by the session (or changed by Nick's exact
      text, committed at his word, and the PR says so).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None new beyond the workflows' own runs; xtask tests that read a
  workflow or the skills list (e.g. the doc-crates list in `changes.yml`,
  the skill loaders' check) stay green.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
