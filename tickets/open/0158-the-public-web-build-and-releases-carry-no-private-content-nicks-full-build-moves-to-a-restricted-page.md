---
id: "0158"
title: "The public web build and releases carry no private content; Nick's full build moves to a restricted page"
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0156"]
nick_input: setup
completed:
---

# 0158 — The public web build and releases carry no private content; Nick's full build moves to a restricted page

## Context

Nick picked B+ in 0153 (`docs/research/0153-keeping-the-story-private.md`
§9). Moving the content into the private assets repository (0159, 0160)
protects nothing while the public builds still embed it, and today they
do (0153 §5.1):

- **GitHub Pages:** `pages.yml` publishes
  `https://zafnok.github.io/visions-of-shuyi/`, the full game with the
  debug tools, with every file of `assets/` embedded (readable with
  `strings`). With the private checkout it would embed the private content
  too. A Pages site can only be made private on Enterprise Cloud (0153
  [S30]). This is where Nick plays the full game (1064).
- **GitHub Releases:** `release.yml` attaches the Windows, Linux, macOS and
  web builds to public releases. v0.1.0–v0.3.0 already carry the scripts
  and data of their day.

So, before either move lands:

- Pages builds **without** the private content, so it serves what is
  public: the test game, or the Chapter 1 demo once 0914 exists;
- Nick's full-game web build goes where only he can open it: a
  **restricted itch.io project** (password), as 0915 does for the demo;
- the full game's builds stop going to public GitHub Releases. The store
  is itch.io (ADR-0009, 0901), and a release run uploads there or keeps
  its builds as workflow artifacts.

## Nick input

**Setup** (only Nick's account can do these). The session gives these
steps when it starts and waits for "done":

1. **A restricted itch.io project for the full game's web build.** On
   itch.io: **Create new project**. Title: `Visions of Shuyi (playtest)`.
   Kind of project: **HTML**. Visibility & access: **Restricted**, then
   set a password and write it down. Leave it unlisted. **Save**.
2. **The upload key.** If 0901 or 0915 already added the secret
   `BUTLER_API_KEY` to this repository, skip this step. Otherwise: itch.io
   → **Settings** → **API keys** → **Generate new API key**, then this
   repository → **Settings** → **Secrets and variables** → **Actions** →
   **New repository secret**, name `BUTLER_API_KEY`, paste, **Add secret**.

**Decision** (asked with the question pop-up, with "describe your own"):
the public releases v0.1.0–v0.3.0 carry the scripts and data of their
day. Should their downloads be deleted (they can't be restored), or left
as they are?

## Scope

**In:**
- `pages.yml`: builds with the bought art as today (the `private-assets`
  feature), but never with `private-content` (0156). Either order with
  0914:
  - if 0914 is done, Pages publishes the Chapter 1 demo edition;
  - if not, it publishes the build without private content (the public
    files and the test game), with the debug tools as now.
- A workflow (or a job in `pages.yml`) that, on a push to `main`, builds
  the full game for the web with `private-assets` and `private-content`
  (`required: true`) and uploads it to the restricted itch.io project with
  butler (`BUTLER_API_KEY`), on its own channel. If 0915 or 0901 already
  installs butler, reuse that step.
- `release.yml`: no build with private content goes to a public GitHub
  Release. Its builds go to itch.io (if 0901 is done, through 0901's
  upload; if not, as workflow artifacts kept 90 days, and the release
  holds only the notes). `required: true` on the content checkout, as a
  sold build needs it.
- The CI check of the web export (`ci.yml` `export`) stays without private
  content.
- 1064 and README updated: where Nick plays the full web build (the
  itch.io page), and what Pages shows.
- Nick's decision on the old releases carried out (`gh release
  delete-asset`, only on his "yes").

**Out (do not do):**
- Moving content (0159, 0160).
- The demo edition itself (0914) and the demo's page (0915).
- Making any release or page public that isn't already.

## Implementation steps

1. Write the size estimate first (rule 10): about 500 changed lines
   (workflows 200, README and 1064 30, this ticket 250).
2. Give Nick the setup, and ask the decision. Carry on meanwhile.
3. `pages.yml` without `private-content`; check the deployed site has no
   private content. The check from 0157 (or `strings` on the `.wasm`, if
   0157 isn't done) finds no real line.
4. The restricted full build and its upload; check the itch.io page
   plays (Nick's password).
5. `release.yml`'s destinations; a dry run (`workflow_dispatch`).
6. README, 1064; Nick's decision on the old releases.

## Acceptance criteria

- [ ] The deployed Pages build contains no private content (check output
      in the completion notes).
- [ ] A push to `main` uploads the full web build to the restricted
      itch.io project (run linked); the page asks for the password.
- [ ] A `release.yml` dry run attaches no build with private content to a
      GitHub Release.
- [ ] 1064 names where Nick plays the full web build.
- [ ] Nick's answer on the old releases is quoted and carried out.
- [ ] zizmor passes; all gates in the `run-gates` skill pass.

## Tests required

- None in Rust. The workflows are checked by their runs (linked) and
  zizmor.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
