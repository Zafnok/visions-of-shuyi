---
id: "0158"
title: "The public web build and releases carry no private content; Nick's full build moves to a restricted page"
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0156"]
nick_input: setup
completed: 2026-11-12
---

# 0158 — The public web build and releases carry no private content; Nick's full build moves to a restricted page

## Context

Nick picked B+ in 0153 (`docs/research/0153-keeping-the-story-private.md`
§9): the game's content and story move to the private assets repository
(0159, 0160). That protects nothing while the full game's builds stay
public:

- **GitHub Pages:** `pages.yml` publishes
  `https://zafnok.github.io/visions-of-shuyi/`, the full game with the
  debug tools, open to anyone with the link. Nick plays Act 1 there
  (1064). A Pages site can only be private on Enterprise Cloud (0153
  [S30]).
- **GitHub Releases:** `release.yml` attaches the Windows, Linux, macOS
  and web packages to public releases, and hands them from job to job as
  workflow artifacts, which anyone can download from a public repository
  (ADR-0117 §4). v0.1.0–v0.3.0 already carry the scripts and data of
  their day.

Neither checks out the private content today (the `private-assets`
action's `content` input is off): they embed `assets/`, which still holds
the real content because 0160 hasn't moved it. Once 0160 lands they'd
ship a hollow game, and checking the content out for them would publish
it. This ticket gives each build its place first; 0160 waits for it.

**What 0915 (the Chapter 1 demo, PR #657) already set up, to reuse:**

- the secret `BUTLER_API_KEY` (Nick's itch.io API key, one for every
  project of his account);
- `.github/actions/butler`: butler at a pinned version, checked by its
  SHA-256, on Linux or Windows; its path is the output `butler`;
- the pattern: one job builds with the content and pushes straight to a
  restricted itch.io project, so nothing with the content becomes a
  GitHub artifact (`.github/workflows/demo.yml`);
- the project's `<user>/<project>` in a repository variable;
- `docs/releasing.md`, "The demo's itch.io pages".

The demo stays on its own two pages. **Pages never serves the demo**: the
demo holds Chapter 1's real content and is password-only (0915); a public
demo is Nick's call, as its own ticket.

## Nick input

**Setup** (only Nick's account can do it). The session gives these steps
when it starts and waits for "done":

1. On itch.io: **Dashboard** → **Create new project**. Title: `Visions of
   Shuyi (playtest)`. Kind of project: **HTML**. Pricing: **No payments**.
   Embed options: **Embed in page**, 1280×720, tick **Fullscreen button**.
   Community: **Disabled**. Visibility & access: **Restricted**, then
   **Also allow a password to view page**, with a password different from
   the demo's. **Save**.
2. Tell the session the project's address
   (`https://zafnok.itch.io/<project>`), and the password if the session
   is to test the page (it opens the page; Nick types the password into
   the browser pane).
3. After the first push: on the project's **Edit game** page, tick **This
   file will be played in the browser** for the `web` upload. **Save**.

**Decision** (asked with the question pop-up, with "describe your own"):
the public releases v0.1.0–v0.3.0 carry the scripts and data of their
day. Should their downloads be deleted (they can't be restored), or left
as they are?

## Scope

**In:**
- **Nick's full build on a restricted page:** a workflow
  (`playtest-itch.yml`, or a job beside `pages.yml`'s) that on a push to
  `main`, and by hand, builds the full game for the web with the bought art
  and the content (`./.github/actions/private-assets` with `content:
  "true"`, `required: "true"`) and the debug tools, as Pages has them
  today, then pushes it with `./.github/actions/butler` to the repository
  variable `ITCH_PLAYTEST_PROJECT`, channel `web`. No secret or no
  variable: a notice, and the job ends green. Its own `concurrency`
  group, so pushes in a row upload only the last.
- **Pages:** `pages.yml` keeps checking out no content (as today), so
  after 0160 it serves the public files and the test game, with the
  debug tools. A line in its header says why, and that the demo never
  goes there.
- **Releases:** `release.yml` checks the content out (`content: "true"`,
  `required: true` on a real release, as a sold build needs it), so its
  package jobs can no longer hand packages on as artifacts:
  - if 0901 is done, each package job pushes its package to the store's
    itch.io channel itself, through 0901's step;
  - if not, each package job builds and checks its package and uploads
    nothing; the GitHub Release holds only the notes. 0901 adds the
    pushes later.
  The debug files (`kept-*`) hold no content and may stay artifacts;
  check that with `strings` before keeping them.
- `docs/releasing.md`: where each build goes (Pages, the playtest page,
  the demo pages, releases). 1064: where Nick plays the full web build.
  README: what Pages shows.
- Nick's decision on the old releases carried out (`gh release
  delete-asset`, only on his "yes").

**Out (do not do):**
- Moving content (0159, 0160).
- The demo and its pages (0914, 0915), and putting the demo on Pages.
- The store page and selling (0901).
- Making any release or page public that isn't already.

## Implementation steps

1. Size estimate (rule 10): about 400 changed lines (workflows 150,
   docs 50, this ticket 250).
2. Give Nick the setup and ask the decision; carry on meanwhile.
3. The playtest workflow, built on `demo.yml`'s publish steps. Set the
   repository variable (`gh variable set ITCH_PLAYTEST_PROJECT --body
   zafnok/<project>`). Run it by hand; open the page (Nick types the
   password) and play to Chapter 1's first battle turn (agents' memory:
   *web build testing in browser pane*).
4. `pages.yml`'s header; check its build has no content checked out (the
   run's log shows `content` off).
5. `release.yml` as above; a dry run (`workflow_dispatch`) that uploads no
   package as an artifact (its run page lists only the `kept-*` files, or
   none).
6. The docs, 1064, README; Nick's decision on the old releases.

## Acceptance criteria

- [x] A push to `main` (or a by-hand run) uploads the full web build to
      the restricted playtest project (run linked); without the password
      and logged out, the page shows nothing of the game.
- [x] The playtest page, opened with the password, plays to Chapter 1's
      first battle.
- [x] `pages.yml` checks out no content (the run's log).
- [x] A `release.yml` dry run leaves no package with the content as a
      workflow artifact or on a GitHub Release (run linked).
- [x] `docs/releasing.md` says where each build goes; 1064 names the
      playtest page.
- [x] Nick's answer on the old releases is quoted and carried out.
- [x] zizmor passes; all gates in the `run-gates` skill pass.

## Tests required

- None in Rust. The workflows are checked by their runs (linked) and
  zizmor.

## Completion notes

**Done.**
- `.github/workflows/playtest-itch.yml` (new): on each push to `main` and by
  hand, `cargo shuyi web --release --debug-tools --private-assets` with the
  content checked out (`content: "true"`, `required: "true"`), then
  `butler push dist/web zafnok/visions-of-shuyi-playtest:web` in the same
  job. Its own concurrency group with `cancel-in-progress`. No secret or no
  variable: a notice, green. Variable set:
  `ITCH_PLAYTEST_PROJECT=zafnok/visions-of-shuyi-playtest`.
- Test run from this branch (a temporary branch trigger, removed before
  merge): https://github.com/Zafnok/visions-of-shuyi/actions/runs/43350532711
  (content checked out, 97.61 MiB pushed to channel `web`). Logged out and
  without the password the page shows only "A password is required to view
  this page", no game frame. With Nick's password typed in the browser pane:
  New Game → Chapter 1's scenes → the first battle's player turn, with the
  bought art and the debug tools (Quick Battle, `F2 debug`).
- `pages.yml`: a header on why it checks out no content and that the demo
  never goes there. Its last run on `main`
  (https://github.com/Zafnok/visions-of-shuyi/actions/runs/43348654579)
  logs `content: false` and checks out `game test voice` only.
- `release.yml`: the package jobs check the content out (`required` on a
  real release from this repository). 0901 isn't done, so no package leaves
  its job: the GitHub Release holds the notes and each package's two
  reports. A step checks that only the reports leave and that neither holds
  one of the game's dialogue lines (2,342 searched). Dry run:
  https://github.com/Zafnok/visions-of-shuyi/actions/runs/43349570231 — its
  artifacts are `licenses` and `kept-<target>` (about 11 KB each, the
  reports), no package.
- Docs: `docs/releasing.md` ("Where each build goes", "The playtest page",
  what a release holds), 1064 names the playtest page, README says what the
  browser build will show.

**Deviation: the debug files are not kept.** The ticket expected the
`kept-*` debug files to hold no content. Reading the engine's export
(`crates/cli/src/export/desktop.rs`, `pack`): `<package>.debug` is a copy
of the program before `objcopy --strip-all`, so it embeds everything the
game embeds. It now stays in its job like the package; only the reports
are kept. Follow-up **0917** finds a private home for it before the game
sells (a crash report can't be read without it).

**Nick's decision on the old releases:** asked with the question pop-up;
answer: "Delete the downloads". Done with `gh release delete-asset`: the
12 packages of v0.1.0, v0.2.0 and v0.3.0 (Windows, Linux, macOS, web); the
releases and their notes remain, with no files.

**Left as is:** older workflow artifacts (earlier release dry runs'
packages and debug files, Pages' uploads, an old CI `web` upload) hold the
content as it is in this repository's public files and git history today;
they expire within 90 days. Nothing new is lost by leaving them.

**Notes for Nick:** the playtest page updates by itself after every merge
to `main`; the demo pages are untouched. The page's password was typed in
the session's chat, so you may want to change it once (Edit game →
Visibility & access).

Follow-up tickets: 0917 (a release's debug files kept somewhere private).
