---
id: "0156"
title: "A private content layer: the real game content from the assets repository, in every CI job"
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: high
status: in-progress
blocked_by: []
nick_input: setup
completed:
---

# 0156 — A private content layer: the real game content from the assets repository, in every CI job

## Context

Ticket 0153 compared ways to keep the story private
(`docs/research/0153-keeping-the-story-private.md`). Making the repository
private costs far more than Nick's $30 a month. Nick picked **B+**: the code
stays public with free CI and every check kept, while the story and the
game's data move into the private assets repository
(`Zafnok/visions-of-shuyi-assets`, ADR-0040). Then a copy of the public
repository gets the code and a small test game, not this game.

This ticket builds the plumbing and moves nothing:

- 0159 moves the story;
- 0160 moves the game's data;
- 0157 readies the tests;
- 0158 readies the web builds.

What exists for the bought art, and why this can't simply reuse it:

- `game/` in the assets repository is laid over `assets/` with the
  `private-assets` feature (`crates/content/build.rs`, `PRIVATE`). Gates
  never turn that feature on (ADR-0040 §5), so the checks don't depend on
  the art and the picture tests stay stable. The real content must be the
  opposite: **every gate reads it**, or B+ weakens the checks (0153 §3.2,
  §7). So the content needs a layer of its own, separate from the art.
- Every file in `game/` needs a bought-art credit (ADR-0051), and the
  engine's `cargo shuyi private-assets --pin` refuses files without one
  (`shuyi_cli` `private_assets.rs`, which checks `shuyi.ron`'s
  `credits.folders: ["game", "test"]`). A new top folder outside `game/`
  needs no credit and **no engine change**.
- The key: `PRIVATE_ASSETS_KEY` (a read-only deploy key) is used today
  only by `pages.yml` and `release.yml`, through
  `.github/actions/private-assets`. Dependabot's runs get only Dependabot
  secrets (0153 [S32]), so the key must be stored a second time for them.

The design 0153 recommends (§3.2, design iii):

- the content lives in the assets repository's new `content/` folder (and
  `story/` for documents, read by no build);
- `content/` is laid over `assets/` by path, like `game/`, under a new
  feature (named here `private-content`) that does not bring in the
  bought art;
- every CI job that builds or loads content checks it out with the key and
  turns the feature on, with `required: true`, so CI always checks the
  real content;
- a build without the checkout (a fork, a worktree without access) still
  builds, on the public files that remain.

Until 0159 and 0160 move files, `content/` holds nothing but a README, so
this ticket changes no behaviour.

## Nick input

**Setup** (only Nick's account can do this). The session gives these
steps when it starts and waits for "done":

1. Open `https://github.com/Zafnok/visions-of-shuyi/settings/secrets/dependabot`.
2. Click **New repository secret**. Name: `PRIVATE_ASSETS_KEY`. Secret:
   the same private key as the Actions secret of that name (the deploy
   key's private half, which Nick saved when ticket 0110 made it). If it
   is lost, make a new deploy key: the session gives the steps, and both
   secrets get the new one.
3. Click **Add secret**.

## Scope

**In:**
- An ADR (the next free number) for the content layer: where the content
  lives, the feature, which jobs check it out, what a build without it
  does, and why it differs from ADR-0040 §5. It extends ADR-0040 and
  ADR-0057 and amends nothing in ADR-0051.
- `crates/content/build.rs`: a `private-content` feature that embeds
  `assets-private/content/` over `assets/`, after checking it is there,
  the same way `private-assets` embeds `game/`. Both features can be on
  together, art over content. The Playdate table gets the same layer.
- `shuyi.ron`: `private_assets.folders` gains `"content"` and `"story"`.
  `credits.folders` stays `["game", "test"]`.
- `.github/actions/private-assets`: a `content` input that checks out
  `content` (sparse) and outputs the `private-content` feature;
  `required: true` fails without the key.
- Every workflow job that builds or loads content uses it, with
  `required: true`: `ci.yml` (clippy, test ×3, doc-tests, wasm, no-std,
  playdate, docs, coverage, tickets, export, bench), `mutants.yml` (plan,
  baseline, shards, weekly), `playtest.yml`. `pages.yml` and `release.yml`
  are 0158's. Jobs that load no content (fmt, changes, the result jobs,
  CodeQL with `build-mode: none`, security, scorecard, sonar-triage,
  engine-watch) don't.
- `cargo xtask` commands that read `assets/` from disk (`check-script`,
  `voice`, `lines`, `check-keys`' paths in `shuyi.ron`, `lang-status`) read
  the layered view: `assets-private/content/` over `assets/` when it
  exists.
- A `content-change` skill (`.claude/skills/content-change/SKILL.md`, its
  loader synced with `pwsh -File scripts/sync-agent-skills.ps1`), modelled
  on `engine-change`, for a change to files in `content/` or `story/`:
  - a branch and PR in the assets repository, never a push to its `main`;
  - the game PR after it merges, moving `assets-private.rev` with
    `cargo shuyi private-assets --pin`;
  - before pinning, check that every assets commit the move carries has
    its game PR merged (0153 §3.3: a renamed scene can't ride into another
    session's pin);
  - link both PRs, with no bare `@`;
  - what to do when two PRs move the pin (ADR-0040 §3).
- `AGENTS.md`'s map of the repo, `work-ticket`'s private-assets steps and
  the `run-gates` skill: the content layer, and that a worktree checks it
  out with `cargo shuyi private-assets` before building.
- The assets repository: `content/README.md` and `story/README.md` saying
  what goes there, and a ruleset on its `main` (block force pushes and
  deletions; require a PR, with Nick as bypass). This repository's own
  steps for that are in the skill; the session can make the ruleset with
  `gh api` (Nick's token) and says so in the completion notes.

**Out (do not do):**
- Moving any file (0159, 0160), the tests (0157), Pages and the releases
  (0158).
- Any engine change. If one turns out to be needed, follow the
  `engine-change` skill and say why in the ADR.
- The writer's mirror (0152).

## Implementation steps

1. Write the size estimate first (rule 9, rule 10): about 1,300 changed
   lines. That is `build.rs` 120 and its tests 80; the action 40; the
   workflows 150; xtask's layered reads 150 and their tests 100; the skill
   200; the ADR 150; `AGENTS.md` and the skills 60; this ticket 250. Above
   2,000, split off the `xtask` reads.
2. Give Nick the setup above; carry on with the steps that don't need it.
3. The ADR.
4. `build.rs` and the feature; a test (in `crates/content/tests/it/`, as
   `private_assets.rs` is) that a file in `content/` replaces the file at
   the same path in `assets/`. The test makes its own folder, so it needs
   no private checkout.
5. `shuyi.ron`, the action, the workflows. Push, and check that every job
   listed above checks out `content` and passes.
6. The `xtask` readers.
7. The `content-change` skill, its loader, `AGENTS.md`, `work-ticket`,
   `run-gates`.
8. The assets repository's READMEs (through the skill's own steps: a
   branch and PR there, then the pin move here) and its ruleset.

## Acceptance criteria

- [ ] With `--features private-content`, a file in
      `assets-private/content/` replaces `assets/`'s file at the same path
      (test).
- [ ] Without the checkout, `cargo build --workspace` and the tests that
      read no real content pass (the job list in the ADR says which).
- [ ] Every job listed under **In** checks out `content` with
      `required: true`; no other workflow job names `PRIVATE_ASSETS_KEY`
      (grep).
- [ ] A Dependabot PR's CI checks out `content` (or, if none comes during
      the ticket, the Dependabot secret exists: `gh api
      repos/Zafnok/visions-of-shuyi/dependabot/secrets` lists it).
- [ ] The `content-change` skill and its loader exist;
      `pwsh -File scripts/sync-agent-skills.ps1 -Check` passes.
- [ ] `assets-private.rev` points at the assets commit with the two READMEs.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the layering test above; one per `xtask` reader that reads
  a file from a made-up `assets-private/content/`.
- By hand (completion notes): one CI run with every content job green and
  its checkout step's time.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
