---
id: "0901"
title: Publish releases to itch.io with butler
type: infra
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: ["0107", "0821", "0909"]
nick_input: setup
completed:
---

# 0901 — itch.io publishing

## Context

[ADR-0009](../../docs/adr/0009-distribution.md): itch.io gets Windows/Linux/
macOS downloads and a browser-playable build, pushed automatically on release.

**Machine-made content (added 2026-10-03, either order with 0907):** if the
published build has AI-generated voices or a machine-translated language
(`docs/design/voices-languages-and-script.md`), the itch page says so. If
0907 is done, use its `docs/release/store-disclosure.md`; if not, leave a
line in 0907 to add the paragraph to the page.

**Added 2026-10-11 (ticket 0413).** Shironejiya's terms (the bandits'
combat pictures, `THIRD_PARTY_ASSETS.md`) ask for a text shipped with the
game that has the site's name (白螺子屋), its address
(http://hi79.web.fc2.com/) and the line that the materials in the game may
not be reused. The credits screen has it in Latin letters (the font has no
kanji); the text file bundled with a release must carry it too, with the
name in kanji.

**Either order with 0158:** 0153 picked moving the story and the game's
content into the private assets repository, and 0158 takes the full game
off GitHub Pages and public GitHub Releases: Nick's full web build goes
to a restricted itch.io project, and `release.yml`'s packages, built with
the content, stop being GitHub artifacts. If 0158 is done, its package
jobs build and check each package and upload nothing: add the push to the
store's channels in each of them. If not, do the same once 0158 lands.

**From 0915 (done):** reuse `BUTLER_API_KEY` and
`.github/actions/butler` (butler pinned and checked by its SHA-256, Linux
or Windows), and push from the job that builds the package, as
`.github/workflows/demo.yml` does. The demo's two projects stay separate
from the full game's.

## Nick input

**Setup (≈10 minutes):**
1. Create an itch.io account (if needed) → **Dashboard → Create new project**.
2. Title: working title; **Kind of project: HTML** (lets the web build play in
   the page; downloads can still be attached). Visibility: **Draft** or
   **Restricted** until ready.
3. (Done in 0915: the secret `BUTLER_API_KEY` is already in the
   repository; skip.)
5. Tell the session the project URL (e.g. `https://<user>.itch.io/<game>`).
6. After the first push: on the project's edit page, tick **"This file will be
   played in the browser"** for the `web` upload, set the embed size to
   1600×1024 (or "click to launch in fullscreen"), and save.

## Implementation steps

1. In each of `release.yml`'s package jobs (the package never leaves its
   job, ADR-0117 §4): `./.github/actions/butler`, then
   `butler push <folder> <user>/<game>:<channel> --userversion <version>`
   for channels `windows`, `linux`, `mac`, `web` (the web folder, not the
   zip). The action knows Linux and Windows; add macOS's build (its
   `darwin-amd64` zip and SHA-256) for the `mac` job, and the Linux job runs
   in the sniper container (check `unzip` and `curl` are there).
2. `BUTLER_API_KEY` passed via env; without it the push steps say so and are skipped, green, as `demo.yml` does.
3. Document the itch project settings above in `docs/releasing.md` (beside
   "The demo's itch.io pages").

## Acceptance criteria

- [ ] A release (or `workflow_dispatch` run) pushes all four channels; itch shows them.
- [ ] Web build plays in the itch page (Chrome + Firefox).
- [ ] `docs/RELEASING.md` complete.

## Completion notes

