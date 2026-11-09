---
id: "0152"
title: The scripts get a private mirror repository writers can propose changes to
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: []
nick_input: setup
completed:
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
The other bullets: 0153 (the repository goes private), 0764 (a preview
program), 0765 (the guide), 0766 (the ad and the trial).

**GitHub can't limit a collaborator to one folder.** A collaborator on a
repository reads all of it, and on a repository owned by a personal
account (`Zafnok`) every collaborator has write access: there are no
Read or Triage roles outside an organization. CODEOWNERS only asks for
reviews. So "only the scripts" means a separate private repository that
holds only what a writer works on, with the writer as a collaborator
there and nowhere else.

What a writer works on today, all in this repository:

- the scripts: `assets/dialogue/*.dlg` (`ch01`–`ch08`, `side_ledger`,
  `side_letters`; 6,019 lines, 308 scenes) and the format's reference,
  `assets/dialogue/README.md`;
- the story documents: `docs/story/` (beats, bible, outline, characters,
  chapter beat sheets, ledger, names registry, craft notes, the writer's
  guide; about 11,500 lines).

### The plan, and why it is a mirror and not a move

The first plan was to **move** the scripts into a new private repository
and pull them into the game at a pinned commit, like the bought art
(ADR-0040: `shuyi.ron`'s `private_assets`, `assets-private.rev`, the
`PRIVATE_ASSETS_KEY` deploy key). Reading the code changed it, because a
move costs far more here than for the art:

- The art is optional: every gate builds with public placeholders and
  never reads it (ADR-0040 §5). The scripts aren't: the content load
  validates every chapter and battle against the scenes it names
  (`trpg_content::load_with_scripts`, `crates/content/src/lib.rs:271`),
  so **every** build, test, CI job, Dependabot run and each of the 60+
  worktrees would need the private checkout and a key.
- Many tickets edit `.dlg` files without writing words: music and places
  (0741, 0742, 0851–0854), line-id and voice tools (0759, 0762, 1231),
  translations (0718, 0725). Each would become two pull requests, one in
  each repository, plus a pin move.
- `docs/story/` is named in 112 lines of 68 files outside `tickets/`
  (code, skills, workflows: `crates/content/src/names/tests.rs:19`
  includes `docs/story/names.md`; `crates/xtask/src/check_script.rs:341`
  reads the guide; `.github/workflows/changes.yml:60-62`), and
  `assets/dialogue` in 46 more.

So the game repository **stays the one source** of the scripts, and the
new repository is a **mirror** of the files a writer needs, written only
by the game's CI:

1. A push to this repository's `main` that changes the scripts or the
   story documents copies them into the mirror's `main` (a workflow
   here, with a token only it holds).
2. A writer clones the mirror, works on a branch, and opens a pull
   request **in the mirror**. The mirror's own CI checks it with a
   checker program (no source code, no secret).
3. When Nick approves that pull request, a session here takes it in
   with `cargo xtask story-import <number>`: it applies the writer's
   change to `assets/dialogue/` and `docs/story/`, with the writer as the
   commit's author, and opens a game PR that runs every gate.
4. When that PR merges, step 1 copies it back, and closes the writer's
   pull request with a note.

What that keeps: no pin, no key in any build, no checkout in any
worktree, no path changes; the gates and the test files that name real
scene ids (15 `.rs` files under `crates/` by `grep -rlw` of the 308 ids,
2026-11-08, e.g. `crates/app/src/args.rs`,
`crates/ui/tests/it/scene_preview.rs`) keep working untouched, because
nothing moves. The mirror holds no secret, so a writer with write access
reaches nothing but the files they were given. Nobody merges in the
mirror; its `main` is written by the game's CI only.

What it costs: one workflow, one token, an import command and its skill.

Nick's "move ALL assets ... including the script & sound files": with the
game repository private (0153) nobody outside can read the scripts or
the music, so moving them adds nothing. The music (`music/`, 75 MB of
CC0/CC-BY tracks) and the sound effects are free to share anyway; they
stay. The bought art stays where it is (ADR-0040), and the writer never
gets access to that repository: its licence forbids passing the files on.

Links: ADR-0040 (the private art repository, the pattern this departs
from), ADR-0011 (story pipeline), ADR-0057 (test files), ADR-0101 (PR
size), ticket 0723 (`cargo xtask check-script`, the guide, `--scene`).

## Nick input

**Setup** (only Nick's account can do these). The session gives him
these steps when it starts, waits for "done", then builds.

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
   **Contents: Read and write**, **Pull requests: Read and write**
   (Metadata: Read is added by itself). Click **Generate token** and copy
   it.
4. **Hand it to this repository.** `visions-of-shuyi` → **Settings** →
   **Secrets and variables** → **Actions** → **New repository secret**.
   Name: `STORY_MIRROR_TOKEN`. Secret: paste. **Add secret**.
5. **Only if the account has GitHub Pro** (0153 asks Nick whether to
   buy it; branch rules on a private repository need it): in the mirror,
   **Settings** → **Rules** → **Rulesets** → **New ruleset** → **New
   branch ruleset**. Name: `main`. Enforcement status: **Active**. Bypass
   list: **Add bypass** → **Repository admin**. Target branches: **Add
   target** → **Include default branch**. Tick **Restrict deletions**,
   **Block force pushes** and **Require a pull request before merging**.
   Click **Create**. Without Pro skip this: writers could push to the
   mirror's `main`, but the next sync replaces its files with the game's,
   and nothing reaches the game except through step 3 of the plan.

Adding a writer as a collaborator is 0766's (after the pick). The trial
applicants never get access: they get a packet (0766).

## Scope

**In:**
- The mirror's layout, synced from this repository:

  | Mirror path | From | Taken back by the import |
  | ----------- | ---- | ------------------------ |
  | `dialogue/` | `assets/dialogue/` (the `.dlg` files and the format's `README.md`) | yes |
  | `story/` | `docs/story/` (every file) | yes |
  | `design/` | read-only copies of the design notes a writer needs: `docs/design/setting-and-tone.md`, `audio.md`, `death-and-difficulty.md`, `voices-languages-and-script.md` | no: a change there is a proposal for Nick, and the import refuses it with that message |
  | `README.md` | `tools/story-mirror/README.md` (a start page; 0765 rewrites it) | no |
  | `.github/workflows/check.yml` | `tools/story-mirror/check.yml` | no |

  The list lives in one file, `tools/story-mirror/files.ron` (or
  `.txt`), read by the sync and the import, so the two never disagree.
- `.github/workflows/story-mirror.yml`: on a push to `main` touching
  those paths, and by hand. It checks out the mirror with
  `STORY_MIRROR_TOKEN`, replaces those paths with this commit's files,
  commits on top of the mirror's `main` (never a force push) with the
  subject `Game <short sha>: <subject>` and a `Game-Commit: <sha>`
  trailer, and pushes. Then it closes each mirror pull request named by a
  `Story-PR: <n>` trailer in the pushed range, with a comment that it is
  in the game now. No token (a fork, a dry run): it warns and stops.
- A checker program a computer with no source can run: a new crate
  `crates/writer` (`trpg-writer`) with the check logic moved from
  `crates/xtask/src/check_script.rs` (a move: no new rule, every message
  stays `trpg_content`'s) and a binary `story-check [folder or file]`
  that prints `file:line: message` and the count, exit 0 / 1, as
  `cargo xtask check-script` does now. `cargo xtask check-script` calls
  the crate. 0764 adds the preview program to this crate or to
  `trpg-app` (its call).
- `.github/workflows/story-kit.yml`: builds `story-check` for Linux and
  Windows and uploads them, with `NOTICES.txt` (third-party license
  notices travel with a binary, ADR-0013), to the mirror's release
  `kit` (replacing the files; the release notes name the game commit).
  By hand, and nightly when `assets/`, `crates/content/` or
  `crates/writer/` changed since the commit the release names. 0764 adds
  the preview program to it.
- `tools/story-mirror/check.yml` (the mirror's CI): on a pull request,
  with `permissions: contents: read` and no secret, downloads `kit`'s
  Linux `story-check` with the mirror's own `GITHUB_TOKEN`, runs it on
  `dialogue/` and turns each problem into an annotation on the line.
- `cargo xtask story-import <n> [--dry-run]`:
  - reads the pull request with `gh` (`-R Zafnok/visions-of-shuyi-story`):
    refuses unless the repository owner's latest review **approves** it;
  - refuses a change outside `dialogue/` and `story/`, naming the files;
  - finds the game commit its base came from (the `Game-Commit:`
    trailer of the mirror commit it branched from), maps the paths back
    (`dialogue/x` → `assets/dialogue/x`, `story/x` → `docs/story/x`) and
    applies the diff with a three-way merge (`git apply --3way`: the
    mirror's files are byte copies, so their blobs are this repository's
    blobs at that commit); a conflict stops it with the files named;
  - commits with the writer as author (their GitHub noreply address),
    the subject `Story: <their title>` and a `Story-PR: <n>` trailer.
- A `story-import` skill (`.claude/skills/story-import/SKILL.md`, loaders
  synced with `pwsh -File scripts/sync-agent-skills.ps1`): a mirror pull
  request Nick approved is its **own ticket**, one game PR, like a
  `sonar` or `engine-pin` issue; branch `story/<n>-<slug>`; run the
  import, then `cargo xtask check-script` and every gate; a test that
  quotes a line the writer reworded is updated in the same PR when the
  test's subject is a mechanism, or pointed at a test script in
  `assets-test/dialogue/` when the words were never its subject; a scene
  the writer renamed is followed into `assets/chapters/`,
  `assets/battles/` and the code that names it; the ledger is the
  writer's; the PR's title `[story#<n>] <their title>`.
- `AGENTS.md`: hard rule 1 gains the third exception (a mirror pull
  request Nick approved, worked with `story-import`); the repository map
  gains the mirror and `tools/story-mirror/`; the skills row gains
  `story-import`. `docs/story/README.md` and `assets/dialogue/README.md`
  say the files are mirrored and edited here or through the mirror.
- An ADR (the next free number): the writer's repository is a mirror of
  the game's files, not a second source; the token; the import; why not
  ADR-0040's pin (the reasons above). It amends nothing in ADR-0040.

**Out (do not do):**
- Moving `music/`, sound effects or any other asset (see Context).
- Making this repository private (0153).
- The preview program (0764); the guide's rewrite and the cheat sheet
  (0765); the mirror README's real text (0765; this ticket writes a stub
  that says what the repository is and how to propose a change); the ad,
  the trial packet and adding a writer (0766).
- Any change to a script's words. The writing pause (0766) holds.
- A web or macOS build of the checker (0764 decides the platforms of the
  writer's programs).

## Implementation steps

1. Write the size estimate first (rule 9): about 1,400 changed lines:
   workflows 200, `tools/story-mirror/` 120, `trpg-writer` 150 new (the
   rest of `check_script.rs` moves as a rename), `story-import` 250 and
   its tests 150, the skill 100, the ADR 130, `AGENTS.md` and READMEs
   40, this ticket 300. Above 2,000, split off the import (command and
   skill) as its own ticket blocked by this one.
2. Give Nick the setup steps above. Wait for "done".
3. `tools/story-mirror/files.*`, the stub README and `check.yml`.
4. Move the check into `crates/writer`, add `story-check`, point
   `cargo xtask check-script` at it; its tests move with it. Add the
   crate to the workspace, `deny.toml`'s and the gates' lists where
   crates are listed.
5. `story-mirror.yml` and `story-kit.yml`, pinned actions as the other
   workflows pin them; zizmor clean. Run `story-kit` by hand, then
   `story-mirror` by hand, and check the mirror on GitHub: the files, the
   release, a test pull request (a comma in a copy of a scene on a
   branch) that the mirror's CI marks; then close it.
6. `cargo xtask story-import`, then the skill and its loader.
7. Prove the round trip on that test pull request: approve it (ask Nick
   to click **Approve**, or test the refusal without it), run the import
   with `--dry-run`, check the diff is the comma in
   `assets/dialogue/…`, and throw it away. Don't commit it.
8. `AGENTS.md`, the READMEs, the ADR and its index line.

## Acceptance criteria

- [ ] The mirror holds `dialogue/`, `story/`, `design/`, `README.md` and
      its `check.yml`, and its last commit names this repository's
      commit in a `Game-Commit:` trailer (checked on GitHub; the commit
      linked in the completion notes).
- [ ] The mirror's release `kit` holds `story-check` for Linux and
      Windows and `NOTICES.txt`.
- [ ] A mirror pull request with a curly quote fails the mirror's check
      with an annotation on its line; one without passes.
- [ ] `cargo xtask story-import` refuses a pull request with no
      approval, and one that changes `design/` (tests with a fake `gh`).
- [ ] It applies an approved change three-way onto a file that moved on
      since (a test in a throwaway repository, as `pr_size.rs`'s tests
      make one).
- [ ] `story-check` and `cargo xtask check-script` print the same
      problems for the same file (a test).
- [ ] No workflow but `story-mirror.yml` and `story-kit.yml` names
      `STORY_MIRROR_TOKEN`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the files list read; the path mapping both ways; the trailer
  parsing; the import's refusals.
- Integration: the three-way apply in a throwaway repository; the
  checker's output against `check-script`'s.
- By hand (in the completion notes): one run of each workflow and the
  test pull request above.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
