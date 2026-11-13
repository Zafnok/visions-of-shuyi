---
id: "0915"
title: The Chapter 1 demo on a password-protected itch.io page
type: infra
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: ["0914"]
nick_input: setup
completed: 2026-11-12
---

# 0915 — The Chapter 1 demo on a password-protected itch.io page

## Context

Nick, 2026-11-10: a web demo of Chapter 1 for the writers applying
(0766), one "that doesn't leak anything". 0914 builds it. This ticket
puts it where applicants can play it in a browser and nobody else
finds it.

GitHub Pages isn't the place: a Pages site is public to anyone with the
link, and today it serves the full game for Nick's playtests (1064).
itch.io is already the planned store (ADR-0009; 0901 publishes the full
game there), plays a web build in the page, and can keep a project
**Restricted**: only people with its password, or with a download key,
can open it.

**Either order with 0901:** 0901 publishes the full game's releases to
itch.io with butler (`BUTLER_API_KEY`). If 0901 is done, reuse its
butler install step and its secret; if not, this ticket adds them and
0901 reuses them. The demo is a **separate itch.io project** either way,
so the full game's page never shows it and the demo's page never shows
the full game.

## Nick input

**Decision** (asked with the question pop-up, with "describe your own"):
how applicants get in.

- **One password** for everyone: simple; if it is passed around, change
  it on the project's page.
- **A download key for each applicant** (itch.io makes a link per key):
  each one can be turned off on its own, and the key says whose link was
  shared. 0766 already marks each packet with the applicant.

**Setup** (Nick's clicks, about ten minutes):

1. Log in to itch.io (or make an account). **Dashboard** → **Create new
   project**. Title: the working title and "Chapter 1 demo". **Kind of
   project: HTML**. **Visibility & access: Restricted**, then
   **Password** (pick one and keep it for 0766's packets) or **Download
   keys**, as decided. **Save**.
2. If 0901's secret isn't there yet: **Account settings** → **API keys**
   → **Generate new API key**; then in the game's GitHub repository
   **Settings** → **Secrets and variables** → **Actions** → **New
   repository secret**, name `BUTLER_API_KEY`, paste, **Add secret**.
3. Tell the session the project's address (`https://<user>.itch.io/<project>`).
4. After the first push, on the project's edit page: tick **This file
   will be played in the browser** for the `web` upload, set the embed
   size to 1600×1024 (or **Click to launch in fullscreen**), **Save**.

## Scope

**In:**
- A by-hand workflow (`workflow_dispatch`, a `demo` job in `release.yml`
  or its own `demo-itch.yml`, as 0914 left it): builds the demo edition
  with the bought art (the `private-assets` action and
  `PRIVATE_ASSETS_KEY`, as `release.yml` does), checks it passes the
  disclosure gate, installs butler (the official download from
  `broth.itch.zone`, its checksum checked) and runs
  `butler push <web folder> <user>/<project>:web --userversion <version>`.
  The project's address is a repository variable, not in the file.
- No token: the job says so and stops, green, as other jobs with a
  missing secret do.
- `docs/releasing.md`: a short section on the demo (how to push it, how
  to change the password or turn off a key).
- If Nick picked download keys: how to make one per applicant on
  itch.io, written in 0766's packet steps (a line in 0766 if it isn't
  done, else in `docs/releasing.md`).

**Out (do not do):**
- The demo's contents (0914).
- The full game on itch.io (0901), what GitHub Pages serves, and Nick's
  restricted full-game web page (0158; if 0158 is done, reuse its butler
  step and `BUTLER_API_KEY`).
- A public demo page: a public demo is Nick's call later, as its own
  ticket.

## Implementation steps

1. Read 0914's completion notes (the edition's name, the export
   command) and 0901 if done. **From 0914:** `demo.yml` uploads no
   artifact (the demo embeds the private repository's content, and a
   public repository's artifacts are anyone's, ADR-0117 §4), so this
   workflow builds the demo itself (`cargo shuyi export web --release
   --edition demo`, the content and the art checked out with the key) and
   sends the package straight to itch.io.
2. Ask the question; give Nick the setup steps; wait for "done".
3. The workflow; run it by hand; open the page in the browser pane
   with the password (or a key Nick gives the session), start New Game,
   and play to the first battle turn (agents' memory: *web build testing
   in browser pane*).
4. `docs/releasing.md`.

## Acceptance criteria

- [x] A by-hand run pushes the demo to the restricted project (the run
      linked in the completion notes).
- [x] The project's page, opened with the password or a key, plays the
      demo in the browser to Chapter 1's first battle.
- [x] The page without the password or a key shows nothing of the game
      (checked in a browser with no itch.io login).
- [x] No workflow but this one and 0901's names `BUTLER_API_KEY`.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None beyond the workflow's run and the checks above (zizmor checks the
  workflow).

## Completion notes

**What was done:**

- **Nick's decision:** the web demo behind **one password** (download keys
  can't open a browser-only itch.io page: a key unlocks downloads, and such a
  page has none). He then asked for a desktop demo too, with a **download key
  per applicant**: the Windows demo is its own Restricted project with no
  password, so the web demo's password never shows it and each key traces one
  applicant. Projects: `zafnok/visions-of-shuyi-chapter-1-demo-web` (password)
  and `zafnok/visions-of-shuyi-chapter-1-demo-windows` (keys), the repository
  variables `ITCH_DEMO_PROJECT` and `ITCH_DEMO_DESKTOP_PROJECT`.
- **`.github/workflows/demo.yml`:** the `export` job is a matrix: `web` on
  every run, `web` and `windows` on a by-hand run (the Windows job sets up the
  GNU target as `release.yml` does). A by-hand run with *publish* ticked
  pushes each package with butler to its project's `web` or `windows`
  channel, `--userversion <version>-demo+<commit>`. No secret or no variable:
  a notice, and the job ends green after the export. All in one job from the
  build to itch.io: no GitHub artifact (ADR-0117 §4, as 0914 noted).
- **`.github/actions/butler`:** installs butler from `broth.itch.zone` at a
  pinned version (15.32.0) on Linux or Windows, refused unless the zip's
  SHA-256 matches (broth publishes none; the hashes were taken from two
  downloads each). Gives butler's path as an output instead of writing
  `GITHUB_PATH` (zizmor's `github-env`). 0901 and 0158 reuse it.
- **`docs/releasing.md`:** "The demo's itch.io pages": pushing, the
  one-time settings, changing the password, making and revoking a key per
  applicant (with the key options to keep). **0766:** its packet step names
  the web demo's link and password and a key link per applicant.

**Verification:**

- Run: https://github.com/Zafnok/visions-of-shuyi/actions/runs/43339431351
  (both jobs green; first builds on both channels, 89 and 92 MiB).
- The web page with the password: New Game, through the opening scenes, to
  Chapter 1's first battle on the player's turn, with the bought art; no
  console error.
- The Windows key link, with no itch.io login: the download page offers the
  zip, version `0.3.0-demo+ddbd924`.
- Both pages with no itch.io login: the web page shows only its title and
  the password box; the Windows page says the visitor has no access.
- `BUTLER_API_KEY` is named only in `demo.yml` (0901 hasn't started).
- All `run-gates` commands passed locally; zizmor finds nothing new.

**Deviations:**

- The Windows demo and its key-only project (Nick's request mid-ticket).
- The push lives in `demo.yml` (the ticket allowed it there), not a new
  workflow: the package can't leave its job.

**Notes for Nick:**

- The game in the web page's frame is served from an `html-classic.itch.zone`
  address that plays without the password. Only someone who already got in
  sees it, but they could pass it on, and a new password doesn't close it
  (`docs/releasing.md`).
- The page's embed was 640×360 when tested; Nick then set 1280×720 with a
  fullscreen button (checked on the page), which the docs now give.

Friction: none, no game code (a workflow, an action and docs).
