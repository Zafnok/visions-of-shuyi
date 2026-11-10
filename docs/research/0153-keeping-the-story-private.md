# 0153: keeping the story private

Research for ticket 0153, 2026-11-10. Three options are compared against
Nick's bar, "under $30 a month with the same velocity":

- **A**: the whole repository goes private, with CI made cheaper;
- **B**: the code stays public and the story (scripts and story documents)
  moves to the private assets repository;
- **C**: nothing private.

How it was made: a multi-agent workflow (Ultracode, as Nick asked). Eight
researchers each covered one part: minutes, velocity, Linux runners, macOS
and Windows, cheaper GitHub minutes, what going private loses, the move's
effect on code, and the move's process. Each had a verifier that fetched
every price again and counted every number again, a modeller combined the
results, and a critic looked for gaps. The working notes stay outside the
repository. Where a verifier corrected a number, this report uses the
corrected one. Every price was retrieved on **2026-11-10**; sources are
numbered [S1]… at the end. EUR→USD at the ECB rate of that day, 1.1214 [S29].

## Summary

| Option | Cost a month | Speed against today | What is lost | Meets the bar? |
| ------ | -----------: | ------------------- | ------------ | -------------- |
| A, GitHub's runners as today | about $1,100 and up | Slower: a private repository gets half the processor (2 cores instead of 4) | CodeQL and secret scanning can't be bought on a personal account. The game's whole history goes private | No |
| A, same speed and same checks on GitHub | about $1,800 | Same | Nothing; needs a paid organization | No |
| A, cheapest that keeps every check on every PR (rented and third-party machines) | about $370–600 | A little slower when many PRs run at once (2 Macs instead of 5) | Hours of machine upkeep a month by agents | No |
| A, cheapest with 7 checks weakened (macOS and Windows tests nightly, CodeQL and SonarCloud dropped, …) | about $35–55 | PRs about as fast; a macOS or Windows break is found a day later | CodeQL, SonarCloud and its issue queue, tests on 3 systems per PR | No (cost and checks) |
| **B, story to the private assets repository** | **$0** | **Same** | Nothing in CI. A story change becomes two PRs. Everything pushed so far stays public | **Yes** |
| C, nothing private | $0 | Same | Nothing; protects nothing | Yes, but it keeps nothing private |

**Recommendation: B.** No form of A comes near $30. The macOS tests alone
cost more than the whole budget on any machine that keeps them on every
PR. B costs nothing, keeps every check and today's speed, and protects
what matters next: the hired writer's script and new story work. What B
can't do: hide what is already public (65 commits of story, every old PR's
diff). Under both A and B, the public web build on GitHub Pages must stop
carrying the later chapters, or the story stays public through it.

## 1. The bar, measured

Window: **2026-10-11 to 2026-11-10** (30 days; the 10th partial). Measured
with the GitHub API as the ticket asks: runs counted per workflow and week
through `.../actions/workflows/<file>/runs?created=A..B` (`gh run list`
stops at 1,000 runs, about 6 days here), the jobs of a sample timed, and
scaled.

### 1.1 Required checks

The ruleset `protect main` requires 8 checks: `ci-result`, `codeql-result`,
`mutants (diff)`, `deny`, `machete`, `typos`, `zizmor`, `tickets`. Not
required: playtest, Scorecard, Pages, bench. The branch need not be up to
date (ADR-0042).

### 1.2 Push to green

| Measure | Sample | Median | 90th percentile |
| ------- | ------ | -----: | --------------: |
| Push to `ci-result` green, first attempt | 237 of 726 PR runs, evenly spread, 2026-10-11 to 11-10 | 9.5 min | 16.1 min |
| … code PRs | 206 | 10.1 min | 16.7 min |
| … docs-only PRs | 31 | 0.8 min | 1.4 min |
| Push to all 8 required checks green | 60 of 351 merged PRs, #216–#607 | 11.3 min | 29.5 min |
| … code PRs | 51 | 13.2 min | 31.9 min |

The last check to go green was `ci-result` in 41 PRs, `mutants (diff)` in
18 (median 8.0, 90th percentile 25.3 min: it is the tail) and
`codeql-result` in 1. The all-checks figure includes re-runs: its maximum,
87 min, is a `playdate` job that hung.

Of the 300 sampled CI runs, 41 (14%) were cancelled by a newer push. Those
still hold a runner until they stop.

### 1.3 PRs a day and concurrency

- **PRs merged:** 347 in the 30 full days, 2026-10-11 to 11-09: a mean of
  11.6 a day, median 9.5, most 26 (2026-10-15). 6 of them were Dependabot's.
- **Jobs at once:** sampled on the 4 busiest days (2026-10-16, 10-17,
  10-22 and 11-01; 1,379 runs, all jobs). At most **45** at once (Linux
  40, Windows 5, macOS 5). In the busiest hours the 95th percentile was 35
  overall: Linux 30, Windows 3, macOS 2.
- **PR branches running CI at once:** at most 5. In the busiest hour,
  2026-10-17 01:00 UTC, 4 PRs had CI running.
- **Queue time:** median 2–8 s, 90th percentile 17–85 s.
- **GitHub's limit for Pro** is 40 jobs and 5 macOS jobs at once [S4]. The
  public repository went over 40 for only 3.5 of 2,347 busy minutes.
- **The engine shares the account.** Its CI ran about 3,300 runs in the
  window, on some of the same busy days. Whether a private repository's
  limit counts the engine's public jobs isn't documented.

### 1.4 Minutes

The month at the current workflow setup (runs since 2026-10-25; checked
by the verifier against 22 newer runs):

| Runner | Minutes a month | Price if private [S1] | $ a month |
| ------ | --------------: | --------------------- | --------: |
| Linux | 94,142 | $0.006 | 565 |
| Windows | 8,106 | $0.010 | 81 |
| macOS | 6,861 | $0.062 | 425 |
| **Total** | **109,109** | | **1,071** |

By workflow: ci $691, mutation testing $146, playtest $137, CodeQL $51,
security $28, the rest about $18. The ticket's "about $1,000" holds. It
listed mutants at $105; with the weekly full runs they are $146.

Four things raise it:

1. **Smaller machines.** These minutes ran on public-repository runners:
   4 cores and 16 GB for Linux and Windows. A private repository gets
   **2 cores and 8 GB** on the same labels; macOS (3-core M1) is the same
   either way [S3]. The same jobs take longer and bill more minutes. By
   how much wasn't measured; up to about twice on the CPU-bound jobs. The
   playtest job already took up to 136 min against its 150-minute limit,
   so on 2 cores it would likely fail, not just slow down. 4-core runners
   cost $0.012 (Linux) and $0.022 (Windows) a minute and are "only
   available for organizations and enterprises using the GitHub Team or
   GitHub Enterprise Cloud plans" [S5]. Nick's account is personal.
2. **Storage.** The repository holds 94.4 GB of build artifacts (76.6 GB
   is the `web` artifact, kept for the 90-day default). Pro includes 1 GB;
   more is $0.25 per GB a month [S2]. That is about $23 a month today,
   $40 or more as it fills. Shorter keeping times bring it to about $2–7.
   This costs nothing on a public repository.
3. **Job times grew in November.** The test legs nearly doubled from
   October (macOS 5.0 → 9.7 min a run, Windows 6.4 → 11.6). A small
   sample put November's month at up to about $1,500, but larger samples
   disagree, so this report uses $1,071.
4. **Free parts:** Pro includes **$18** of usage a month [S2]. Dependabot's
   own runs are free, and GitHub's billing page lists self-hosted runners
   as free [S2].

A finding outside this ticket: the weekly full mutation run never
finishes. All 5 runs in the window hit GitHub's 6-hour job limit on all
4 shards. That is about 1,445 minutes a week with no result (ticket 0155).

## 2. Option A: the whole repository private

### 2.1 What going private loses

For a private repository owned by a **personal** account on Pro:

| Today (public) | Private | Replacement | What is lost even so |
| -------------- | ------- | ----------- | -------------------- |
| CodeQL; `codeql-result` is required | Can't be bought on a personal account. Code scanning on private repositories is for organizations on Team ($4 a user [S9]) with Code Security ($30 per active committer [S10]): **$34 a month** [S11]. CodeQL's own licence forbids running it in our CI on code that isn't OSI-licensed [S12] | Semgrep CE (10 community Rust rules), plus clippy, cargo-deny and zizmor, which already run | The required check (a gate change), Rust dataflow queries, the alert list. CodeQL has raised **0** alerts on this repository, ever |
| Secret scanning and push protection | Not available on a user-owned private repository at any price. Secret Protection is $19 per committer, organizations only [S10] | gitleaks (MIT) in `security.yml` | GitHub refusing the push; the scan comes after the push. 0 secret alerts ever |
| Scorecard SARIF upload | Needs code scanning | Run it with `publish_results: false`; keep the report as an artifact | The Security-tab view and the public badge (not a required check) |
| zizmor SARIF upload | Needs code scanning | `advanced-security: false`: findings print, and the job fails on medium and above | Alert history. The gate gets stricter, not weaker |
| SonarCloud | Free only for public projects; private is free up to 50k lines. This project is **162,662** lines of code (Sonar's count). Team starts at $34 for 100k lines; the 200k tier's price isn't published [S13] | Self-hosted SonarQube Community Build on a rented host (~€6–9). It analyses `main` only, with no PR analysis, and Rust comes from a community plugin | More than $34 a month, or no PR analysis. `sonar-triage` needs a `SONAR_TOKEN` either way |
| Pages | Allowed with Pro, but **the site stays public**: a private Pages site needs an organization on Enterprise Cloud [S30] | — | The public site serves the full game with every script (§5.1) |
| Rulesets, required checks | Kept with Pro | — | Nothing |
| Dependabot | Kept; its own runs are free | — | Nothing (its PRs' CI uses minutes) |
| The engine's watcher | Works: its token names the game's repository, whatever the visibility | — | The engine's 118 files that link to the game become dead links for everyone but Nick |

Files that become untrue: `LICENSE` ("This repository is public so that
people can see how the game is made", Nick's file), `README.md`,
`AGENTS.md` and ADRs 0008, 0013, 0014, 0032, 0040, 0047, 0071 and 0100
(listed with line numbers in the working notes; they would go into A's
follow-up tickets).

### 2.2 Linux runners

Prices a minute for one 4-core job, the size of today's public runner
(Linux minutes today: 94,142 a month):

| Runner | $ a minute | $ a month | Works on a personal account? | Source |
| ------ | ---------: | --------: | ---------------------------- | ------ |
| GitHub 2-core (half today's size) | 0.006 | 565 and up (jobs run longer) | yes | [S1] |
| GitHub 4-core | 0.012 | 1,130 | **no**: Team organization only | [S1], [S5] |
| Ubicloud standard-4 (4 vCPU, 16 GB) | 0.0025 | 233 | probably; not stated | [S14] |
| RunsOn on AWS spot, 4 vCPU | ~0.0013 + €300/yr licence + ~$7 a month control | ~157 | probably; not stated | [S20] |
| Namespace 4 vCPU (8 GB) | 0.004 prepaid / 0.006 | 377–565 | yes | [S15] |
| Blacksmith 4 vCPU | 0.008 (derived) | ~753 | **no**: organizations only | [S16] |
| WarpBuild 4 vCPU (16 GB) | 0.008 | 753 | unclear | [S17] |
| BuildJet 4 vCPU | 0.008 | 753 | not stated | [S18] |
| Depot 2 vCPU | 0.006 | — | **no**: organizations only | [S19] |
| Cirrus Runners | $150 per runner, flat | 40 runners: 6,000 | organizations | [S21] |
| Hetzner AX102 (16 cores, 128 GB), 8 jobs each, always on | €159 a month + €129 setup | 5 for the peak: $892; 4 for the 95th percentile: $713 | yes | [S22] |
| Hetzner Cloud CX33 (4 shared vCPU, **8 GB**), a server per job | €0.0144 an hour | maybe $30–320: depends on how many jobs share a billed hour | yes | [S23] |

Notes:

- **GitHub's self-hosted charge.** GitHub announced a charge of $0.002 a
  minute on self-hosted runners for private repositories from 2026-03-01,
  then postponed it with no new date: "We're postponing the announced
  billing change for self-hosted GitHub Actions to take time to re-evaluate
  our approach" [S6]. If it returns, it covers every row above except
  GitHub's own runners, because the third-party services register as
  self-hosted. At these minutes that adds about **$190–300 a month**.
- **Hetzner Cloud is unproven here.** Every Hetzner Cloud plan showed
  "currently unavailable" on 2026-11-10 [S23]. The per-project server
  limit wasn't checked, and its machines have half today's memory. One PR
  push starts 20–25 Linux jobs, many under a minute. If each takes its own
  server for a billed hour, the cost follows the number of jobs, not their
  minutes.
- **A warm build folder** on a machine that stays up saves about 1–2 min
  of a 10.8-min test job; most of the job is running the tests. Machines
  that stay up also carry risks: one job can see another's leftovers and
  secrets [S36], disks fill, and a stale cache can hide a failure.

### 2.3 macOS

The tests run on macOS on every code PR: 6,861 minutes a month, $425 at
GitHub's $0.062 [S1]. Rented options:

| Option | $ a month | Jobs at once | Source |
| ------ | --------: | -----------: | ------ |
| GitHub-hosted | 425 | 5 | [S1] |
| Namespace Team ($100 with 2,500 macOS minutes, then $0.06) | ~362 | 6 | [S15] |
| Blacksmith M4, $0.08 | ~537 | not stated; organizations only | [S16] |
| WarpBuild, $0.08 | ~549 | unlimited | [S17] |
| Cirrus Runners, $150 a runner | 150 each | 1 per runner; organizations | [S21] |
| Scaleway Mac mini M4-S, €149 | 167 | 2 (two macOS VMs per Mac, Apple's licence) | [S25] |
| Scaleway Mac mini M1-M, €75 | 84 | 1 | [S25] |
| MacStadium M4.S | 149 | 1–2 | [S26] |
| AWS EC2 mac2, dedicated host | 475 | 2 | [S27] |

The cheapest Mac that keeps the check on every PR is **$84** for one job
at a time. On a busy day 7 macOS jobs start within 15 minutes, so the
last waits about 40 min. **Every option that keeps macOS on every PR costs
more than $30 on its own.**

How often only macOS failed in the window: 3 of 1,078 CI runs, all the same
random-input test (`print_parse_round_trip`, ticket 0759). It is a bug
that can fail on any system; it failed once on Linux alone too. Real
macOS-only failures: 0. So moving macOS to nightly would rarely catch a
problem a day late. It is still a gate change.

### 2.4 Windows (MSVC)

8,106 minutes a month. GitHub's 2-core runner: $81 and up (2 cores); 4-core
$0.022 needs an organization [S1]. Cheapest that keeps the check: Azure
spot Windows (D4as_v5, $0.075 an hour [S28]) with WarpBuild's $0.002 a
minute on your own cloud [S17]: **about $26–34**, plus 2–3 min of boot per
job, on the Windows leg that is often the slowest. Others: Namespace about
$97 [S15]; Hetzner AX42 with a Windows Server licence, €79 + €27.90 a
month each [S22], [S24]. Cross-compiling the MSVC build from Linux
(`cargo-xwin`) can't run the tests on Windows: a gate change. The game has
Windows-only code (the `%APPDATA%` save folder test, the exe's icon in
`build.rs`).

### 2.5 Cutting minutes without dropping a check

Levers that keep every check on the same events, at private prices:

| Lever | Saves a month |
| ----- | ------------: |
| Drop `wasm`: `export` makes the same build | $16–28 |
| One Linux test run: fold `test (ubuntu)` into `coverage` | $36–63 |
| One `checks` job for the 9 small jobs (fewer rounded-up minutes) | $46–56 |
| One `changes` job per PR | $4–11 |
| The 1-core `ubuntu-slim` ($0.002 [S7]) for jobs that build nothing | $16–21 |
| Cache keys (`save-if`, a shared mutants key) | $2–5 |
| Shorter setup | $3–12 |
| **Together** | **$123–196** |

Levers that need a trial (sccache, ARM runners [S8], faster test steps)
could save up to about another $490, mostly unproven. None of them touches
macOS's $425 except faster test steps. **GitHub-hosted runners can't reach
$30**: after every lever Linux alone is still over $380.

### 2.6 The cheapest A, every check kept

| Part | Choice | $ a month |
| ---- | ------ | --------: |
| Linux | RunsOn on AWS spot, or Ubicloud standard-4, after the levers | 120–180 |
| macOS | 1 Scaleway M4-S (2 jobs at once; today 5) | 167 |
| Windows | Azure spot + WarpBuild | 26–34 |
| CodeQL (`codeql-result`) | An organization on Team + Code Security; the only way to keep the check | 34 |
| Secret scanning | gitleaks (push blocking is lost: a feature, not a check) | 0 |
| SonarCloud | Team, 200k-line tier (price unpublished) | ≥ 34 |
| Storage | Shorter keeping times | 2–7 |
| Pro's included usage | | −18 |
| **Total** | | **≈ $365–440**; up to about **$600** with Namespace for Windows and Ubicloud premium-4 for Linux |

On top of the money:

- **Speed:** about the same at the median, slower in bursts (2 Mac slots
  instead of 5, Windows boot time).
- **Upkeep:** an agent session keeps the machines, images and keys going,
  about **6–15 hours a month**, and holds cloud credentials. When a runner
  breaks, every required check waits and nothing merges.
- **One-time:** the repository moves to an organization (URLs, the
  engine's links, the keys and secrets), plus about 6–8 tickets of CI work.
- **Risk:** if GitHub's self-hosted charge returns, add about $170–260.

**It doesn't meet the bar: about 12–20× over.**

### 2.7 The cheapest A with checks weakened (gate changes)

Listed because the ticket asks what it would take, not as a proposal. Each
is a gate change (`AGENTS.md` rule 5) that meets "same velocity" only if
Nick says so:

1. macOS tests on every PR → nightly on `main` and at release;
2. Windows tests on every PR → nightly;
3. CodeQL removed (Semgrep CE instead);
4. SonarCloud removed (coverage gated by `cargo llvm-cov --fail-under-lines`;
   the `sonar` issue queue ends);
5. push protection → gitleaks after the push;
6. CI no longer runs again on the push to `main`, and playtest runs on PRs
   only;
7. the weekly full mutation run fixed so it finishes (it is broken today).

The cost:

| Part | $ a month |
| ---- | --------: |
| Linux on Hetzner Cloud servers started per job (unproven, see §2.2) | 21–34 |
| macOS and Windows nightly on GitHub (30 × ~8 min × $0.062, 30 × 12.7 min × $0.010) | 19–23 |
| Jobs that build nothing, on `ubuntu-slim` | ~10 |
| Storage | 2–7 |
| Pro's included usage | −18 |
| **Total** | **≈ $35–55**, more if the Linux servers bill per job |

Even with seven checks weakened, it doesn't come in under $30 with any
confidence.

## 3. Option B: the story moves to the private assets repository

The code stays public, so CI stays free and every check keeps running as
today.

### 3.1 What moves, and where

| What | Size | Moves? | Where |
| ---- | ---- | ------ | ----- |
| `assets/dialogue/ch02.dlg`–`ch08.dlg`, `side_ledger.dlg`, `side_letters.dlg` | 9 files, 5,588 lines, 287 scenes | yes | the assets repository, in a folder of its own beside `game/` |
| `assets/dialogue/ch01.dlg` | 392 lines, 21 scenes | **no**: it is in the Chapter 1 demo (0914) and the trial packet (0766) anyway | stays |
| `assets/dialogue/README.md` (the format) | 566 lines | no: it is the format, not the story, and tests read it | stays |
| `docs/story/` | 39 files, 11,528 lines | yes | the assets repository's `story/` |
| Story sections of `docs/design/` | about 14 sections in 10 files (`setting-and-tone.md`, `audio.md`'s cue notes, `death-and-difficulty.md`, `turn-structure.md`'s chapter rules, `look-and-feel.md`'s bosses, `world-structure.md`, `title.md`, …) and `docs/design/playdate/` still to check | yes, the sections only | `story/design/` |
| `voice/` | already private | — | — |
| `music/` | 48 tracks, 80 MB, CC0/CC-BY | no: it isn't story, and its licences allow sharing | stays |

The scripts go in **a folder of their own**, not in `game/`, because:

- ADR-0051 makes every file in `game/` carry a bought-art credit, and the
  engine's `--pin` refuses files without one; a separate folder needs no
  engine change;
- the scripts can be read without the bought art (ADR-0040 §5: the checks
  never read the art).

### 3.2 How the checks keep reading the real scripts

Every check loads the scripts: `trpg_content::load_with_scripts`
(`crates/content/src/lib.rs:271`) validates each chapter and battle against
the scenes it names. Three designs were sized:

| Design | What changes | The checks |
| ------ | ------------ | ---------- |
| (i) Public checks on stand-in scripts; one keyed job checks the real ones | One CI job; stand-ins (one short scene per id the data names, ~670 lines now, ~1,150 when Chapters 6–8 are wired in) | Mutation testing, the playtest bots and coverage would judge the stand-ins: **a gate change** |
| (ii) Every job checks out the private scripts; no stand-ins | About 20 job definitions; every worktree must check out the private repository before it builds (50 of 367 have it); a fork can't build | All kept |
| **(iii) Stand-ins in the public repository; every CI job that loads content checks out the real scripts over them** | About 20 job definitions (a 2–4 s checkout each, measured); stand-ins | **All kept**, on the real scripts and the same events. A worktree or fork without the checkout still builds, on the stand-ins |

**Design (iii)** keeps every gate and leaves the public repository
buildable, so it is the one this report recommends. Under (i) and (iii)
both, Dependabot's PRs only get the key if Nick adds it a second time as a
Dependabot secret [S32] (one setup step). Without it, their checks would
run on the stand-ins.

The stand-ins name every scene id that public data names (167 ids today,
such as `ch03_confession`). They must keep the mechanics the playthrough
tests use (`@joins defector`, `@joins shieldbearer`), and nothing else.

### 3.3 What it costs

- **Money:** $0. CI stays on free public runners, and the extra checkouts
  cost 2–4 s each. The assets repository gets no workflow.
- **Speed:** push to green unchanged; concurrency unchanged.
- **One-time work:** about 4–6 sessions in three PRs:
  1. The story layer: the folder, the feature, the keyed checkouts, the
     stand-ins and their generator, the `story-change` skill and the ADR.
     About 1,100–1,500 lines.
  2. The cut: delete `docs/story/` and the later chapters' scripts, split
     the design sections, point about 117 lines that name `docs/story` and
     50 that name `assets/dialogue` at the new place. About 18,500 lines by
     `cargo xtask pr-size`, almost all deletions, so it passes the
     3,000-line warning with that reason (ADR-0101). About 1,300–1,800
     lines to review.
  3. The public web build (§5.1).
- **Ongoing:** a change to a moved script becomes **two PRs**, the assets
  repository's first and then the game's, which moves `assets-private.rev`
  (as the `engine-change` skill does for the engine):
  - In the window, 25 PRs touched the scripts, 24 of them with code or
    data too; the pin moved 17 times.
  - Under B the pin would have moved about 40 times, and 8 of those PRs
    (20%, one every 3.7 days) would have met another pin move. Each is a
    one-line conflict.
  - Since the writing pause (2026-11-08) one script PR has merged, so 40
    is an upper bound.
  - Open tickets that edit later-chapter scripts without writing words
    (0852, 0853, 0854) become two PRs each. Others (0077, 0491, 0759, 0764,
    0765, 0766, 0867, 0868, 0870, 0914, 1090, 1209, 1231, 1260) read the
    moved files and need the checkout.
- **A new rule for the assets repository:** today art is pushed straight
  to its `main` (0 PRs ever). Script changes rename and remove scenes, so
  an unmerged change there could ride into another session's pin move and
  break its build. Story changes there go through branches and PRs, and
  the skill checks what a pin move carries.

### 3.4 The writer's access

A collaborator on a personal account's repository always gets write access
to all of it [S33], and the bought art's licence forbids passing it on
(ADR-0040). So a writer never joins the assets repository. Of three routes,
the simplest is the one 0152 already plans: **its mirror**, synced by the
game's CI. Under B it copies from the assets repository at the pin, with
the existing `PRIVATE_ASSETS_KEY`. 0152 then changes:

- the sync's trigger: a pin move;
- its sources: the assets repository's script folder and `story/`;
- `story-import`: opens the assets PR first, then the game PR that moves
  the pin.

That adds about 150 lines to its ~1,400. A third private repository just
for the story was rejected. It would need a second deploy key, the engine's
`private-assets` to handle two repositories, and Nick merging writer PRs
before any game check sees them.

### 3.5 What moving can't protect

Moving files doesn't remove them from the public history:

- **What stays readable:** 65 commits on `main` touched the scripts or
  `docs/story/` (64 PRs), and GitHub keeps 582 read-only PR refs. Every
  script and story document so far stays readable at old commits and in
  old PRs' diffs.
- **Outside the moved folders:** the plot also sits in tickets (66 story
  tickets), the design documents' other sections, 8 UI snapshots, scene ids
  in public data, and code comments.
- **Rewriting history** (git filter-repo and a force push [S31]) breaks
  every clone, the 373 worktrees, the open PRs and 570 commit ids cited in
  tickets. It still can't touch the PR refs or GitHub's cached views:
  "GitHub Support won't remove non-sensitive data" [S31]. Not recommended.
- **Exposure measured today:** 0 forks, 0 stars, 5 page views in 14 days.
  Neither Software Heritage nor the Wayback Machine has a copy.

So B protects **what is written after the move**: the hired writer's
scripts, the rewrite of Chapters 2–8, new story documents. It doesn't
protect what is public now. Option A would hide the history on github.com
too, but not copies made before, and not the public Pages build.

## 4. Option C: nothing private

$0, the same speed, no work. The Chapter 1 demo (0914) still keeps
Chapters 2–8 out of what players download. Anyone who finds the
repository reads all 10 scripts, the 39 story documents and every ticket,
and the hired writer's work would be public as it lands. The trial
packet's "confidential" note would cover only the applicant's own entry.

## 5. What every option needs

### 5.1 The public web build

`pages.yml` publishes `https://zafnok.github.io/visions-of-shuyi/`: the
full game with the debug tools, built with the private assets and every
script embedded. It is readable with `strings`, and it is where Nick plays
the full game (1064). A private Pages site needs Enterprise Cloud [S30], so
**under A and B alike the site keeps the story public** until:

- Pages builds without the later chapters (the demo edition, 0914, or the
  stand-ins);
- Nick's full-game web build moves somewhere only he can open, such as a
  restricted itch.io page like 0915's;
- `release.yml`'s public GitHub Releases stop shipping the later chapters.

The releases v0.1.0–v0.3.0 already carry the scripts of their day. Under
B, ticket 0158 does this.

### 5.2 Independent of the option

- The weekly full mutation run never finishes: ticket 0155.

## 6. Recommendation

**Option B, with design (iii).** It meets the bar: $0 a month, today's
push-to-green time, every check on the real scripts on the same events.
It protects the writer's work and new story from the move on, which is
what the writer search (0766) needs. Its costs are agent time: about 4–6
sessions once, then a second PR for each later-chapter script change.

No form of A meets the bar:

- The cheapest that keeps every check is about $370–600 a month, plus 6–15
  hours of machine upkeep.
- Weakening seven checks still lands at $35–55.
- A's real gain over B is hiding the history on github.com, which matters
  little with 0 forks and a handful of views, and costs about $4,400–7,200
  a year at the least.

C keeps nothing private, which isn't what Nick asked for.

## 7. B against A with checks weakened

Nick, 2026-11-10, asked for this comparison:

> lets do a comparison between B and A (checks weakened). Give me a
> comprehensive breakdown (comprehensive meaning cover all facets, NOT a
> fluff essay that my eyes will glaze over reading). Include exactly the
> importance of the checks that we're weakening, if we can weaken them only
> a little (i.e. only moving certain code private and thus weakening), etc.

Same window, 2026-10-11 to 11-10. Counts are from the Actions API and
`git log origin/main`.

### 7.1 What each weakened check has caught

| Check | What it guards against | What it caught in 30 days | Price to keep it under A | A partial keep | How much it matters |
| ----- | ---------------------- | ------------------------- | ------------------------ | -------------- | ------------------- |
| macOS tests on every PR | A break that shows only on a Mac | **0** real ones. 3 of 1,078 runs failed on macOS alone, all one random-input test that fails on any system (0759). The game has no macOS-only code, and the engine's CI tests macOS on its own changes | $84 (one rented Mac, queues) to $425 (GitHub) | Only on PRs that touch dependencies, the engine pin, `crates/app`, build scripts, `xtask` or CI (132 of 352 PRs, 38%), plus nightly: fits one $84 Mac | Low today |
| Windows tests on every PR | A break that shows only on Windows, the main platform | **0** Windows-only failures. Windows-only code is in 3 files: the save folder, the exe icon, `xtask`'s cleanup | $26–34 (Azure spot) to $81+ (GitHub) | The same filter plus nightly: about $7 | Low–medium: Windows is where the game sells first |
| CodeQL (required) | Security flaws found by tracing data through the code | **0** failures; **0** alerts ever | $34 a month, and the repository moves to an organization | Semgrep CE (10 Rust rules), about $0 | Low for an offline single-player game |
| SonarCloud (not a blocking check) | Style and complexity findings, which feed the `sonar` issue queue | 14 issues filed since 2026-10-19, 7 closed. All style: needless `'static`, wildcard imports, complexity. Its dashboard shows 0 bugs, 0 vulnerabilities | ≥ $34 (the 200k-line tier's price is unpublished) | Analyse at most 50k lines on the free private plan (`core` 30k plus `bots`, `engine-watch` and `playdate` = 41k; `main` only; unverified for a personal account's organization), or at most 100k for $34 (every crate except `ui`) | Low as a check. Partly kept, the queue goes on |
| Push protection | A key or token pushed by mistake is refused before it lands | **0** secret alerts ever | $19 a month, organization only | gitleaks scans after the push, about $0 | Low |
| CI again on the push to `main` | Two PRs that each passed alone but break together (the branch need not be up to date) | **12 real breaks** on `main` (tests on all three systems, or clippy), plus 5 flakes | 16,000 Linux minutes: about $5–40 | — | **High: keep it** |
| Playtest again on the push | The balance report on `main` | 0 failures. It is a report, not a check | about $1–9 | — | None as a check |
| Weekly full mutation run | A full mutation sweep | Nothing: it never finishes (ticket 0155) | 0–4,500 min | Fix it | None until fixed |

Mutation testing on PRs stays under every option: it failed 139 PR runs
in the window, so it is the check that catches the most.

### 7.2 How much to weaken: the price of each level

Linux is the largest line. "Untested" is Hetzner Cloud servers started per
job ($0.0144 an hour, half today's memory, "currently unavailable" when
checked [S23]). "Tested" is RunsOn on AWS spot or Ubicloud [S14], [S20],
both in wide use. Their speed against today's runners was measured by
neither. GitHub's postponed self-hosted charge (§2.2) would add $100–260
to every row.

| Level | What changes | $ a month, untested Linux | $ a month, tested Linux |
| ----- | ------------ | ------------------------: | ----------------------: |
| A, all 7 weakened | macOS and Windows nightly; no CodeQL, no Sonar, gitleaks; no CI on the push to `main`; playtest on PRs only | 35–60 | 115–155 |
| **A, lightly weakened** | Keep CI on the push to `main`. macOS (one rented Mac) and Windows (Azure spot) only on platform PRs, plus nightly. Semgrep for CodeQL, Sonar on a free-sized part, gitleaks, playtest on PRs only | **120–145** | **215–265** |
| A, nothing weakened | §2.6 | — | 365–600 |
| B | — | **0** | **0** |

No level of A is under $30. Only the heaviest one gets near it, and only on
machines no one here has tried.

### 7.3 Moving only certain parts private

GitHub makes a whole repository public or private; it can't hide a folder.
"Only certain parts private" therefore means splitting files out into the
private assets repository, as B does. A larger split was sized too:

| Split | What goes private | Checks | $ a month | Work | What someone copying the public repository gets |
| ----- | ----------------- | ------ | --------: | ---- | ----------------------------------------------- |
| **B: the story** | Chapters 2–8 scripts, side quests, `docs/story/`, story sections of the design docs | All kept (design iii) | 0 | About 4–6 sessions | The code, all game data, Chapter 1 and the music. A game playable through Chapter 1, with the later battles but no story |
| **B+: the story and the game's data** | Also maps, battles, chapters, units, classes, items and balance numbers: everything in `assets/` except fonts, music and sound | All kept, if every CI job checks out the private copy | 0 | Large, roughly 3–5× B (estimate): 78 test files load the shipped content, 64 name chapter ids, and many of the 235 picture tests draw real maps. Every worktree needs the private checkout. The public build becomes the test game (`assets-test/`) | The code and a small test game; no real content |
| A: everything | The code too | Weakened, or $365+ | 35–600 | §7.4 | Nothing new on github.com; clones made before, and the old Pages build |

Only A hides the code itself: the rules, the screens, the tools. The engine
under it is public under MIT anyway.

### 7.4 Side by side

| | B | A, checks weakened |
| - | - | ------------------ |
| Cost a month | $0 | $35–60 on untested machines; $115–155 tested; $120–265 if lightly weakened |
| Work up front | 3 PRs, about 4–6 sessions | Rented runners and their autoscaler, the machine images, the workflow changes (about 6–8 tickets); the docs that say "public" |
| Work every month | A later-chapter script change becomes 2 PRs and a pin move: up to about 40 pin moves a month, 1 in 5 meeting another (a one-line fix). Near 0 during the writing pause | 2–6 hours of an agent session on the machines; agents hold cloud keys. When a runner breaks, nothing merges until it is fixed |
| Push to green | Today's: 9.5 min median, 16.1 at the 90th percentile | About today's on PRs if the rented machines match today's speed (unmeasured). With CI on the push to `main` dropped, the 12-a-month breaks there show up in the next PRs instead |
| Checks | All kept, on the real scripts | 7 weakened (5 if lightly) |
| What stays public | The code, the game data, Chapter 1, the music; the story written up to the move (65 commits, every old PR's diff); plot in tickets and design docs | On github.com, nothing. Clones made before it and the old Pages build stay out there |
| Against a copy of the game | Weak: the code and data can be copied, though `LICENSE` forbids it; the story from the move on can't | Strong: nothing to copy |
| The hired writer | Works in 0152's mirror, copied from the assets repository | The mirror is still needed: a collaborator on a personal account's repository can write to all of it [S33] |
| Nick's full-game web build | Moves off public Pages to a page only he opens (§5.1) | The same: Pages stays public either way [S30] |
| Dependabot and forks | Nick adds the key once more, as a Dependabot secret. Forks build on the stand-ins | Nothing to do |
| What Nick sets up | One Dependabot secret | Cloud accounts with his payment card (Hetzner or AWS, Scaleway, Azure), the `LICENSE` wording, the visibility switch |
| Undoing it | Easy: B can still become A later | Easy to make public again, but the private months' cost is spent |

### 7.5 Reading of it

- **Two checks matter:** mutation testing, which stays under every option,
  and **CI on the push to `main`**, which caught 12 real breaks in 30 days.
  Keep that one under any A.
- The rest have caught nothing here in 30 days (macOS, Windows, CodeQL,
  push protection) or are a report and not a check (Sonar, playtest on
  push). Weakening them costs little safety today.
- Weakening them doesn't get A under $30 on machines anyone has tried. It
  lands at $115–265 a month.
- If what matters most is that the game can't be copied, B+ does it for
  $0 at a few times B's work. Only A hides the code itself, at $120 or more
  a month.

## 8. A better GitHub plan?

Nick, 2026-11-10: "are you sure github doesnt have any better plans for
private repos... like github pro max or something".

There isn't one above Pro for a personal account. GitHub's plans and the
CI usage each includes a month [S37]:

| Plan | Price | CI included a month | Notes |
| ---- | ----- | ------------------: | ----- |
| Free | $0 | $12 | |
| Pro (Nick's) | $4 | $18 | The top plan for a personal account |
| Team | $4 a user | $18 | An organization. Can buy CodeQL ($30) and secret scanning ($19); 4-core runners |
| Enterprise Cloud | about $21 a user [S38] | **$300** (≈ 50,000 Linux minutes) | An organization. Private Pages sites. The $21 may be a first-year rate, and whether it needs a minimum number of seats isn't clear (secondary sources only) |

**Enterprise is the best of them, and still over $30.** This month needs
about $1,071 of minutes on GitHub's runners, so Enterprise would bill
about $770 over its $300, on half-size runners. Its best use, at the
"lightly weakened" level of §7.2, works out as follows:

- macOS and Windows run on platform PRs and nightly, inside the $300;
- the rest of the $300 buys about 30,000 Linux minutes;
- the other ~35,000 Linux minutes run on rented machines.

That totals about **$100 a month** on tested machines, or **$35–45** on
untested ones, plus the 2-core slowdown on GitHub's half. CodeQL still
costs $30 extra there, and the repository moves into an organization.

**GitHub for Startups** gives up to 20 Enterprise seats free for a year,
but it is for startups funded by one of its partner investors (Series B or
earlier). One listing says new applications closed in 2026 [S39]. It
doesn't fit a self-funded game.

## 9. Nick's decision

Nick, 2026-11-10, after §7 and with §8 asked:

> are you sure github doesnt have any better plans for private repos...
> like github pro max or something... its just so weird. Otherwise B+
> seems reasonable

§8's answer is that no plan brings A under $30, so the pick is **B+: the
story and the game's data move to the private assets repository**. The
code stays public with free CI and every check kept, as §7.3 describes.
Follow-up tickets:

| Ticket | What it does |
| ------ | ------------ |
| 0155 | The weekly full mutation run finishes (independent of the pick) |
| 0156 | The private content layer: a `content/` folder in the assets repository embedded over `assets/` without the bought art; the public build falls back to the test game; every CI job that loads content checks it out; the `content-change` skill; the ADR |
| 0157 | Tests and picture tests that need the real content: run where the private copy is, and nothing that shows real content stays in a public file |
| 0158 | The web builds: Pages and GitHub Releases stop carrying real content; Nick's full-game web build moves to a restricted itch.io page |
| 0159 | The story documents move: `docs/story/` and the story sections of the design docs |
| 0160 | The game's content and every `.dlg` script move; the public build left starts the test chapter |

0152 (the writer's mirror) then copies from the assets repository, after
0159 and 0160. 0766 waits for them instead of 0153.

## Sources

All retrieved 2026-11-10.

- [S1] GitHub, Actions runner pricing: https://docs.github.com/en/billing/reference/actions-runner-pricing (Linux 1-core $0.002, 2-core $0.006, 2-core arm64 $0.005, 4-core $0.012; Windows 2-core $0.010, 4-core $0.022; macOS $0.062; each job rounded up to the minute; larger runners organizations only)
- [S2] GitHub, Actions billing: https://docs.github.com/en/billing/concepts/product-billing/github-actions (Pro: $18 of usage, 1 GB artifacts, 10 GB cache; storage $0.25 per GB-month; free for Dependabot; self-hosted runners free)
- [S3] GitHub-hosted runners, sizes for public and private repositories: https://docs.github.com/en/actions/reference/runners/github-hosted-runners
- [S4] GitHub Actions limits (Pro 40 jobs, 5 macOS; Team 60/5; 6-hour job limit): https://docs.github.com/en/actions/reference/limits
- [S5] Larger runners: https://docs.github.com/en/actions/concepts/runners/larger-runners and the pricing page in [S1]
- [S6] Self-hosted charge, announced and postponed: https://github.blog/changelog/2025-12-16-coming-soon-simpler-pricing-and-a-better-experience-for-github-actions/ ; https://github.com/resources/insights/2026-pricing-changes-for-github-actions
- [S7] ubuntu-slim generally available: https://github.blog/changelog/2026-01-22-1-vcpu-linux-runner-now-generally-available-in-github-actions/
- [S8] Arm64 standard runners in private repositories: https://github.blog/changelog/2026-01-29-arm64-standard-runners-are-now-available-in-private-repositories
- [S9] GitHub pricing (Team $4 a user a month): https://github.com/pricing
- [S10] GitHub security plans (Code Security $30, Secret Protection $19 per active committer a month): https://github.com/security/plans
- [S11] Who can buy code scanning: https://docs.github.com/en/get-started/learning-about-github/about-github-advanced-security ; https://docs.github.com/en/get-started/learning-about-github/githubs-plans
- [S12] CodeQL CLI terms: https://github.com/github/codeql-cli-binaries/blob/main/LICENSE.md
- [S13] SonarQube Cloud plans: https://www.sonarsource.com/plans-and-pricing/sonarcloud/ ; https://docs.sonarsource.com/sonarqube-cloud/administering-sonarcloud/managing-subscription/subscription-plans/ ; the project's own measure via `api/measures/component` (ncloc 162,662)
- [S14] Ubicloud: https://www.ubicloud.com/docs/about/pricing ; https://www.ubicloud.com/docs/github-actions-integration/runner-types
- [S15] Namespace: https://namespace.so/pricing ; personal accounts: https://namespace.so/blog/completing-the-circle-with-github-runners
- [S16] Blacksmith: https://www.blacksmith.sh/pricing ; https://docs.blacksmith.sh/introduction/quickstart (organizations only)
- [S17] WarpBuild (and BYOC $0.002 a minute): https://www.warpbuild.com/pricing
- [S18] BuildJet: https://buildjet.com/for-github-actions/pricing
- [S19] Depot: https://depot.dev/pricing ; https://depot.dev/docs/github-actions/quickstart
- [S20] RunsOn (licence €300 a year for commercial use; AWS spot prices per minute): https://runs-on.com/pricing/
- [S21] Cirrus Runners: https://cirrus-runners.app/pricing/
- [S22] Hetzner dedicated servers: https://www.hetzner.com/dedicated-rootserver/matrix-ax/
- [S23] Hetzner Cloud: https://www.hetzner.com/cloud/cost-optimized/
- [S24] Hetzner Windows Server licence: https://docs.hetzner.com/robot/general/pricing/windows-2025-pricing
- [S25] Scaleway Apple silicon (public catalogue; the pricing page refused automated reading): https://api.scaleway.com/product-catalog/v2alpha1/public-catalog/products?product_types=apple_silicon
- [S26] MacStadium: https://www.macstadium.com/pricing
- [S27] AWS EC2 Mac dedicated hosts: https://aws.amazon.com/ec2/instance-types/mac/ ; https://b0.p.awsstatic.com/pricing/2.0/meteredUnitMaps/ec2/USD/current/dedicatedhost-ondemand.json
- [S28] Azure retail prices (Windows spot D4as_v5, effective 2026-07-01): https://prices.azure.com/api/retail/prices
- [S29] ECB reference rate, 1 EUR = 1.1214 USD: https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml
- [S30] Private Pages sites need Enterprise Cloud: https://docs.github.com/en/pages/getting-started-with-github-pages/changing-the-visibility-of-your-github-pages-site
- [S31] Removing sensitive data: https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/removing-sensitive-data-from-a-repository
- [S32] Dependabot runs get only Dependabot secrets: https://docs.github.com/en/code-security/dependabot/troubleshooting-dependabot/troubleshooting-dependabot-on-github-actions
- [S33] A personal account's collaborators get write access: https://docs.github.com/en/account-and-profile/setting-up-and-managing-your-personal-account-on-github/managing-personal-account-settings/permission-levels-for-a-personal-account-repository
- [S34] Tart (macOS VMs) licence: https://tart.run/licensing/
- [S35] cargo-xwin: https://github.com/rust-cross/cargo-xwin
- [S36] Self-hosted runner security: https://docs.github.com/en/actions/reference/security/secure-use
- [S37] Usage included with each plan (Free $12, Pro $18, Team $18, Enterprise Cloud $300): https://docs.github.com/en/billing/reference/product-usage-included ; https://docs.github.com/en/get-started/learning-about-github/githubs-plans
- [S38] Enterprise "Starting at $21 USD per user/month" (GitHub's pricing page as quoted by third parties; GitHub's page itself wasn't read): https://github.com/pricing ; https://costbench.com/software/developer-tools/github/ ; https://www.vendr.com/marketplace/github
- [S39] GitHub for Startups: https://github.blog/news-insights/product-news/github-for-startups-is-generally-available/ ; https://www.founderpass.com/partners/github

Not verified: Ubicloud's, RunsOn's and WarpBuild's support for personal
accounts; SonarQube Cloud's 200k-line price; the speed of any rented
machine against today's runners; Apple's two-VM licence term (quoted from
secondary sources); Hetzner Cloud's availability and hourly rounding.
None of these changes the conclusion: every A row is over $30 before any
of them matters.
