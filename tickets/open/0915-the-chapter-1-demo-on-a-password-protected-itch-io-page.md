---
id: "0915"
title: The Chapter 1 demo on a password-protected itch.io page
type: infra
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: ["0914"]
nick_input: setup
completed:
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

- [ ] A by-hand run pushes the demo to the restricted project (the run
      linked in the completion notes).
- [ ] The project's page, opened with the password or a key, plays the
      demo in the browser to Chapter 1's first battle.
- [ ] The page without the password or a key shows nothing of the game
      (checked in a browser with no itch.io login).
- [ ] No workflow but this one and 0901's names `BUTLER_API_KEY`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None beyond the workflow's run and the checks above (zizmor checks the
  workflow).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
