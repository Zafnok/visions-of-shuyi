---
id: "0158"
title: "The public web build and releases carry no private content; Nick's full build moves to a restricted page"
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: in-progress
blocked_by: ["0156"]
nick_input: setup
completed:
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

- [ ] A push to `main` (or a by-hand run) uploads the full web build to
      the restricted playtest project (run linked); without the password
      and logged out, the page shows nothing of the game.
- [ ] The playtest page, opened with the password, plays to Chapter 1's
      first battle.
- [ ] `pages.yml` checks out no content (the run's log).
- [ ] A `release.yml` dry run leaves no package with the content as a
      workflow artifact or on a GitHub Release (run linked).
- [ ] `docs/releasing.md` says where each build goes; 1064 names the
      playtest page.
- [ ] Nick's answer on the old releases is quoted and carried out.
- [ ] zizmor passes; all gates in the `run-gates` skill pass.

## Tests required

- None in Rust. The workflows are checked by their runs (linked) and
  zizmor.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
