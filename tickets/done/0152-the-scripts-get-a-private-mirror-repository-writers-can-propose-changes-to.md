---
id: "0152"
title: The scripts get a private mirror repository writers can propose changes to
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["0159", "0160"]
nick_input: setup
completed: 2026-11-13
---

# 0152 — The scripts get a private mirror repository writers can propose changes to

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

This ticket is the bullet "They need limited access to github repo".
The other bullets: 0153 (research: how to keep the story private), 0764 (a preview
program), 0765 (the guide), 0766 (the ad and the trial).

**GitHub can't limit a collaborator to one folder.** A collaborator on a
repository reads all of it, and on a repository owned by a personal
account (`Zafnok`) every collaborator has write access: there are no
Read or Triage roles outside an organization. CODEOWNERS only asks for
reviews. So "only the scripts" means a separate private repository that
holds only what a writer works on, with the writer as a collaborator
there and nowhere else.

### Where the files are now (0153's pick, B+)

0153 picked B+ (2026-11-10): the story's documents and the game's
content moved to the private assets repository
(`Zafnok/visions-of-shuyi-assets`), which the game builds with at the
commit in `assets-private.rev` (ADR-0117, tickets 0156, 0159, 0160).
What a writer works on is now:

- the scripts: `assets-private/content/dialogue/*.dlg` (`ch01`–`ch08`,
  `side_ledger`, `side_letters`), and the format's reference, which
  stayed public: `assets/dialogue/README.md`;
- the story's documents: `assets-private/story/` (beats, bible,
  outline, characters, beat sheets, ledger, names registry, craft notes,
  the writer's guide), with the story parts of the design documents in
  `story/design/`;
- the public design notes a writer needs: `docs/design/audio.md`,
  `death-and-difficulty.md`, `voices-languages-and-script.md`.

A writer can never join the assets repository: the bought art's licence
forbids passing the files on (ADR-0040), and a collaborator there could
read and write all of it. So the writer gets a **mirror**: a third
private repository, `Zafnok/visions-of-shuyi-story`, holding copies of
those files, written only by the game's CI (0153 §3.4, route (c)):

1. A push to the game's `main` that moves `assets-private.rev` (or
   changes the public files it copies) checks the assets repository out
   at the pin with `PRIVATE_ASSETS_KEY` and copies the files into the
   mirror's `main` (a workflow here, with a token only it holds).
2. A writer clones the mirror, works on a branch, and opens a pull
   request **in the mirror**. The mirror's own CI checks it with a
   checker program (no source code, no secret).
3. When Nick approves that pull request, a session here takes it in with
   `cargo xtask story-import <number>`: it applies the writer's change
   onto the assets repository's `main` (a three-way merge from the files
   the writer started from), with the writer as the commit's author, and
   opens the assets pull request. After it merges, the session moves the
   pin in a game pull request (the `content-change` skill).
4. When the game pull request merges, step 1 copies it back, and closes
   the writer's pull request with a note.

Nobody merges in the mirror; its `main` is written by the game's CI only.
The mirror holds no secret, so a writer with write access reaches nothing
but the files they were given.

Why not the old plans: moving the scripts to a writer's repository of
their own was rejected by 0153 §3.4 (a second deploy key, the engine's
`private-assets` reading two repositories, and Nick merging writer PRs
before any game check sees them). The first version of this ticket kept
the scripts in this repository; 0160 moved them, so the sync now reads
the pin.

The music and sound effects stay public (CC0/CC-BY, free to share), and
the writer never gets the bought art.

Links: ADR-0040 (the private art repository), ADR-0117 (the content in
the private repository), ADR-0011 (story pipeline), ADR-0057 (test
files), ADR-0101 (PR size), 0153's report §3.4, ticket 0723 (`cargo
xtask check-script`, the guide).

## Nick input

**Setup** (only Nick's account can do these). The session gives him
these steps when it starts and builds meanwhile; it checks they are done
before it runs the workflows.

1. **Create the mirror.** Open `https://github.com/new`. Owner:
   `Zafnok`. Repository name: `visions-of-shuyi-story`. Choose
   **Private**. Leave "Add a README", ".gitignore" and "license" unset
   (the first sync fills it). Click **Create repository**.
2. **Read-only workflows there.** In the new repository: **Settings** →
   **Actions** → **General** → *Workflow permissions*: pick **Read
   repository contents and packages permissions**, untick **Allow GitHub
   Actions to create and approve pull requests**, click **Save**.
3. **The token the game's CI uses.** Open
   `https://github.com/settings/personal-access-tokens/new` (a
   *fine-grained* token). Token name: `story-mirror`. Expiration: the
   longest offered (write the date down; the sync's error says how to
   renew it). Resource owner: `Zafnok`. Repository access: **Only select
   repositories** → `visions-of-shuyi-story`. Repository permissions:
   **Contents: Read and write**, **Pull requests: Read and write**,
   **Workflows: Read and write** (the sync writes the mirror's own check
   workflow; Metadata: Read is added by itself). Click **Generate token**
   and copy it.
4. **Hand it to this repository.** `visions-of-shuyi` → **Settings** →
   **Secrets and variables** → **Actions** → **New repository secret**.
   Name: `STORY_MIRROR_TOKEN`. Secret: paste. **Add secret**.
5. **Branch rules** (GitHub Pro has them on a private repository): in the
   mirror, **Settings** → **Rules** → **Rulesets** → **New ruleset** →
   **New branch ruleset**. Name: `main`. Enforcement status: **Active**.
   Bypass list: **Add bypass** → **Repository admin**. Target branches:
   **Add target** → **Include default branch**. Tick **Restrict
   updates**, **Restrict deletions** and **Block force pushes**. Click
   **Create**. (Only the sync, with Nick's token, writes `main`; a writer
   can't push or merge there.)

Adding a writer as a collaborator is 0766's (after the pick). The trial
applicants never get access: they get a packet (0766).

## Scope

**In:**
- The mirror's layout, synced from the game's checkout with the assets
  repository at the pin:

  | Mirror path | From | Taken back by the import |
  | ----------- | ---- | ------------------------ |
  | `dialogue/*.dlg` | `assets-private/content/dialogue/` | yes, to the assets repository's `content/dialogue/` |
  | `dialogue/README.md` | `assets/dialogue/README.md` (the format, public) | no |
  | `story/` | `assets-private/story/` | yes, to its `story/`, except `story/design/` |
  | `design/` | `docs/design/audio.md`, `death-and-difficulty.md`, `voices-languages-and-script.md` | no |
  | `README.md` | `tools/story-mirror/README.md` (a start page; 0765 rewrites it) | no |
  | `.github/workflows/check.yml` | `tools/story-mirror/check.yml` | no |

  A change to a path that isn't taken back (`story/design/`, `design/`,
  the format) is a proposal for Nick: the import refuses it with that
  message. The list lives in one file, `tools/story-mirror/files.txt`,
  read by the sync and the import, so the two never disagree.
- `.github/workflows/story-mirror.yml`: on a push to `main` touching
  `assets-private.rev`, the public files listed or `tools/story-mirror/`,
  and by hand. It checks out the assets repository at the pin (the
  `private-assets` action, content only) and the mirror with
  `STORY_MIRROR_TOKEN`, replaces the mirror's files with the list's,
  commits on top of the mirror's `main` (never a force push) with the
  subject `Game <short sha>: <subject>` and the trailers `Game-Commit:
  <sha>` and `Assets-Commit: <pin>`, and pushes. Then it closes each
  mirror pull request named by a `Story-PR: <n>` trailer in the pushed
  range, with a comment that it is in the game now. No token (a fork, a
  dry run): it warns and stops.
- A checker program a computer with no source can run: a new crate
  `crates/writer` (`trpg-writer`) with the check logic moved from
  `crates/xtask/src/check_script.rs` (a move: no new rule, every message
  stays `trpg_content`'s) and a binary `story-check [folder or file]`
  that prints `file:line: message` and the count, exit 0 / 1, as `cargo
  xtask check-script` does now. It embeds the game's content (chapters
  and battles name the scenes they play), so it is built only where the
  content is checked out and never uploaded as a public artifact
  (ADR-0117 §4). `cargo xtask check-script` calls the crate. 0764 adds
  the preview program to this crate or to `trpg-app` (its call).
- `.github/workflows/story-kit.yml`: builds `story-check` for Linux and
  Windows with the content checked out and uploads each straight to the
  mirror's release `kit` (replacing the files), with
  `THIRD_PARTY_LICENSES.html` (the crates' notices, as every package
  carries, ADR-0013); the release notes name the game commit and the pin.
  By hand, and nightly when `assets-private.rev`, `assets/`,
  `crates/content/` or `crates/writer/` changed since the commit the
  release names. 0764 adds the preview program to it.
- `tools/story-mirror/check.yml` (the mirror's CI): on a pull request,
  with `permissions: contents: read` and no secret, downloads `kit`'s
  Linux `story-check` with the mirror's own `GITHUB_TOKEN`, runs it on
  `dialogue/` and turns each problem into an annotation on the line.
- `cargo xtask story-import <n> [--dry-run]`:
  - reads the pull request with `gh` (`-R Zafnok/visions-of-shuyi-story`):
    refuses unless it is open and the repository owner's latest review
    **approves** it;
  - refuses a change outside what is taken back, naming the files;
  - fetches the mirror's `main` and the pull request into the assets
    checkout, takes the diff from the mirror commit it branched from (a
    sync commit: it has the `Assets-Commit:` trailer) with full blob ids,
    maps the paths (`dialogue/x` → `content/dialogue/x`, `story/x` →
    `story/x`) and applies it three-way (`git apply --3way --cached` on a
    scratch index from the assets repository's `origin/main`: the
    mirror's files are byte copies, so their blobs are the assets
    repository's at that pin); a conflict stops it with the files named;
    the checkout's working tree and branch are never touched;
  - commits with the writer as author (their GitHub noreply address), the
    subject `Story: <their title>` and a `Story-PR: <n>` trailer, pushes
    the branch `story/<n>-<slug>` to the assets repository and opens its
    pull request; `--dry-run` prints the change and stops before the
    push.
- A `story-import` skill (`.claude/skills/story-import/SKILL.md`, loaders
  synced with `pwsh -File scripts/sync-agent-skills.ps1`): a mirror pull
  request Nick approved is its **own ticket**, two pull requests (the
  assets one, then the game's that moves the pin, the `content-change`
  skill), like a `sonar` or `engine-pin` issue; game branch
  `story/<n>-<slug>`; run the import, then `cargo xtask check-script`
  and every gate; a test that quotes a line the writer reworded is
  updated in the same PR when the test's subject is a mechanism, or
  pointed at a test script in `assets-test/dialogue/` when the words were
  never its subject; a scene the writer renamed is followed into the
  content's chapters and battles and the code that names it; the ledger
  is the writer's; the game PR's title `[story#<n>] <their title>`, and
  its commit carries the `Story-PR: <n>` trailer the sync closes the
  mirror pull request by.
- `AGENTS.md`: hard rule 1 gains the third exception (a mirror pull
  request Nick approved, worked with `story-import`); the repository map
  gains the mirror and `tools/story-mirror/`; the skills row gains
  `story-import`. `docs/story/README.md` and `assets/dialogue/README.md`
  say the files are mirrored for the writer.
- An ADR (the next free number): the writer's repository is a mirror of
  the files at the pin, not a source; the token; the import; the
  checker's embedded content. It amends nothing in ADR-0040 or ADR-0117.

**Out (do not do):**
- Moving `music/`, sound effects or any other asset.
- The preview program (0764); the guide's rewrite and the cheat sheet
  (0765); the mirror README's real text (0765; this ticket writes a stub
  that says what the repository is and how to propose a change); the ad,
  the trial packet and adding a writer (0766).
- Any change to a script's words. The writing pause (0766) holds.
- A web or macOS build of the checker (0764 decides the platforms of the
  writer's programs).

## Implementation steps

1. Write the size estimate first (rule 9): about 1,700 changed lines:
   workflows 230, `tools/story-mirror/` 110, `trpg-writer` 150 new (the
   rest of `check_script.rs` moves), `story-import` 330 and its tests
   220, the skill 110, the ADR 130, `AGENTS.md` and READMEs 40, this
   ticket 350. Above 2,000, split off the import (command and skill) as
   its own ticket blocked by this one.
2. Give Nick the setup steps above.
3. `tools/story-mirror/files.txt`, the stub README and `check.yml`.
4. Move the check into `crates/writer`, add `story-check`, point `cargo
   xtask check-script` at it; its tests move with it. Add the crate to the
   workspace and wherever crates are listed.
5. `story-mirror.yml` and `story-kit.yml`, pinned actions as the other
   workflows pin them; zizmor clean.
6. `cargo xtask story-import`, then the skill and its loader.
7. `AGENTS.md`, the READMEs, the ADR and its index line.
8. When Nick's setup is done and this PR has merged (the workflows run
   from `main`): run `story-mirror` by hand, then `story-kit` (its
   release needs the mirror's first commit), and check the mirror on
   GitHub: the files, the release, a test pull
   request (a curly quote in a copy of a scene on a branch) that the
   mirror's CI marks; then close it. Prove the import on it with
   `--dry-run` (the change is the quote in `content/dialogue/…`). If the
   PR merges before the setup, these runs go in a follow-up ticket.

## Acceptance criteria

- [ ] The mirror holds `dialogue/`, `story/`, `design/`, `README.md` and
      its `check.yml`, and its last commit names the game's commit and
      the pin in `Game-Commit:` and `Assets-Commit:` trailers (checked on
      GitHub; the commit linked in the completion notes).
- [ ] The mirror's release `kit` holds `story-check` for Linux and
      Windows and `THIRD_PARTY_LICENSES.html`.
- [ ] A mirror pull request with a curly quote fails the mirror's check
      with an annotation on its line; one without passes.
- [x] `cargo xtask story-import` refuses a pull request with no
      approval, and one that changes `design/` or `story/design/` (tests
      with a fake `gh`'s answer).
- [x] It applies an approved change three-way onto a file that moved on
      since (a test in throwaway repositories, as `pr_size.rs`'s tests
      make one).
- [x] `story-check` and `cargo xtask check-script` print the same
      problems for the same file (a test).
- [x] No workflow but `story-mirror.yml` and `story-kit.yml` names
      `STORY_MIRROR_TOKEN`.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the files list read; the path mapping; the trailer parsing; the
  import's refusals.
- Integration: the three-way apply in throwaway repositories; the
  checker's output against `check-script`'s.
- By hand (in the completion notes): one run of each workflow and the
  test pull request above.

## Completion notes

**Done (2026-11-13).** Rewritten first for 0153's pick B+ (the sync reads
the assets repository at the pin; the import lands there), then built:

- `tools/story-mirror/files.txt` (what the mirror holds, read by the sync
  and the import), the mirror's stub `README.md` and its `check.yml`.
- `crates/writer` (`trpg-writer`): `check_script.rs`'s checks and report
  moved there, plus `story-check [folder or file]`; `cargo xtask
  check-script` calls it. A test runs both on the same draft and gets the
  same output. A release build run on a local copy of the mirror's layout
  (the sync's copy loop run by hand): `0 problems in 10 scripts`, and a
  curly quote flagged on its line.
- `.github/workflows/story-mirror.yml` and `story-kit.yml` (zizmor clean;
  only they name `STORY_MIRROR_TOKEN`).
- `cargo xtask story-import <n> [--dry-run]` with tests: the files list,
  the path mapping, trailers, the refusals on a fake `gh` answer (no
  approval, Nick's changes requested, someone else's approval, closed, a
  change to `design/` or `story/design/`), and the three-way apply in
  throwaway repositories (a file that moved on since takes the writer's
  line; the same line on both sides is a conflict, named; a pull request
  not from a sync commit is refused; the checkout's files and branch are
  untouched).
- The `story-import` skill and its loader; `work-ticket`'s description
  points to it. `AGENTS.md` (rule 1's third exception, the map, the
  skills row, the crate), `docs/story/README.md`,
  `assets/dialogue/README.md`, ADR-0122 and its index line.
  `.cargo/mutants.toml` leaves out `story_import::gh` and `story-check`'s
  `main`, which only run `gh` or print.

**Deviations.**

- The import works on a scratch index (`git apply --cached --3way` from
  `origin/main`, then `commit-tree`) and fetches the mirror's commits into
  the assets checkout for full blob ids, instead of `gh pr diff` and
  `git apply` in a working tree: GitHub's diff has short ids, and the
  checkout the game builds from stays as it is. It opens the assets pull
  request; the game's pull request that moves the pin is the skill's
  (`content-change`), with the `Story-PR:` trailer the sync reads.
- The notices are `THIRD_PARTY_LICENSES.html` (cargo-about, as every
  package ships them), not `NOTICES.txt`.
- The ruleset (setup step 5) is **Restrict updates** instead of
  **Require a pull request**: with only a pull request required, a writer
  could merge their own; and the token needs **Workflows** write to push
  the mirror's `check.yml`.
- The by-hand runs (the first three acceptance criteria) need Nick's
  setup and the workflows on `main`, so they moved to **0168** (blocked by
  this ticket; 0766 now waits for it too). Nick's setup was not done when
  this PR opened.
- PR size: 2,615 changed lines against the 1,700 estimated (the import's
  tests and the ticket's rewrite ran over). It could have split off the
  import (about 1,000 lines with its skill), but it was written and
  tested by then; it is under ADR-0101's 3,000.
- Engine friction: estimate 400 production lines, measured 587 (1.5×),
  under 2×.

**Follow-up tickets:** 0168 (the first sync, the kit and a test pull
request, with Nick's setup).

**For Nick:** the five setup steps in *Nick input* above (also in 0168).
No game rule was decided here.
