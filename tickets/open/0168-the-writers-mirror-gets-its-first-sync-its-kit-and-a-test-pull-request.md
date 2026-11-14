---
id: "0168"
title: The writer's mirror gets its first sync, its kit and a test pull request
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["0152"]
nick_input: setup
completed:
---

# 0168 — The writer's mirror gets its first sync, its kit and a test pull request

## Context

Ticket 0152 built the writer's mirror (ADR-0122): the sync
(`.github/workflows/story-mirror.yml`), the kit
(`.github/workflows/story-kit.yml`), the mirror's own check
(`tools/story-mirror/check.yml`) and `cargo xtask story-import`. Its
workflows run only from `main` and need Nick's setup (0152's *Nick
input*: the private repository `Zafnok/visions-of-shuyi-story`, its
workflow permissions, the `STORY_MIRROR_TOKEN` secret, the ruleset), so
0152 merged with its by-hand runs not done. This ticket does them.

## Nick input

**Setup:** 0152's five steps, if not done yet (the session checks with
`gh repo view Zafnok/visions-of-shuyi-story` and `gh secret list`). Then
one click: **Approve** on the test pull request (step 4), or the session
proves only the refusal.

## Scope

**In:**
- One run of each workflow by hand and the test pull request below, with
  fixes to them if a run fails (in this ticket's PR).

**Out (do not do):**
- Adding a writer (0766), the preview program (0764), the guide (0765).
- Any change to a script's words: the test pull request is thrown away.

## Implementation steps

1. Check the setup is done (above). If the ruleset or the workflow
   permissions can't be read with `gh api`, ask Nick.
2. `gh workflow run story-mirror.yml`; when it is green, check the mirror
   on GitHub: `dialogue/`, `story/`, `design/`, `README.md`,
   `.github/workflows/check.yml`, and the last commit's `Game-Commit:` and
   `Assets-Commit:` trailers.
3. `gh workflow run story-kit.yml`; check the release `kit` holds
   `story-check-linux`, `story-check-windows.exe` and
   `THIRD_PARTY_LICENSES.html`, and its notes name the commit.
4. In a scratch clone of the mirror: a branch that puts a curly quote in a
   line of `dialogue/ch01.dlg`, pushed, and a pull request. The mirror's
   check fails with an annotation on that line. Push a second commit that
   takes it out: the check passes.
5. `cargo xtask story-import <n> --dry-run`: without Nick's approval it
   refuses; with it, the change shown is that line in
   `content/dialogue/ch01.dlg`. Close the pull request and delete its
   branch; nothing is imported.

## Acceptance criteria

- [ ] The mirror holds `dialogue/`, `story/`, `design/`, `README.md` and
      its `check.yml`; its last commit has `Game-Commit:` and
      `Assets-Commit:` trailers (linked in the completion notes).
- [ ] The release `kit` holds `story-check` for Linux and Windows and
      `THIRD_PARTY_LICENSES.html`.
- [ ] A mirror pull request with a curly quote fails the mirror's check
      with an annotation on its line; without it, it passes.
- [ ] `story-import --dry-run` refuses it unapproved and shows the line
      when approved (or the refusal only, said so).

## Tests required

- By hand, in the completion notes: the runs above, with links.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
