---
id: "0156"
title: "A private content layer: the real game content from the assets repository, in every CI job"
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: []
nick_input: setup
completed: 2026-11-12
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

- [x] With `assets-private/content/` checked out, a file there replaces
      `assets/`'s file at the same path (test:
      `crates/content/tests/it/private_content.rs`, with a folder of its
      own; and `bundle.rs`'s `the_content_lies_between_the_bought_art_and_assets`).
      There is no `private-content` feature: see Deviations.
- [x] Without the checkout, `cargo build --workspace` and the tests pass
      (the first full run in this worktree had no checkout; today every test
      reads only public content, so all of them).
- [x] Every job listed under **In** checks out `content` with
      `required: true`, but `export` (see Deviations); no other workflow job
      names `PRIVATE_ASSETS_KEY` but `pages.yml` and `release.yml`'s, which
      already did for the art (0158's).
- [x] The Dependabot secret exists (`gh api
      repos/Zafnok/visions-of-shuyi/dependabot/secrets` lists
      `PRIVATE_ASSETS_KEY`). No Dependabot PR came during the ticket.
- [x] The `content-change` skill and its loader exist;
      `pwsh -File scripts/sync-agent-skills.ps1 -Check` passes.
- [x] `assets-private.rev` points at the assets commit with the two READMEs
      (`0acd7f9`, Zafnok/visions-of-shuyi-assets#1).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the layering test above; one per `xtask` reader that reads
  a file from a made-up `assets-private/content/`.
- By hand (completion notes): one CI run with every content job green and
  its checkout step's time.

## Completion notes

**What was done**

- ADR-0117: the content layer. `content/` (the game's own data and
  scripts) and `story/` (documents) in the private repository; the build
  embeds `assets-private/content/` over `assets/` whenever it is there;
  every CI job that loads content checks it out; nothing that embeds it
  becomes a public artifact; the disk-reading tools read the same view.
- `crates/content/build.rs` writes `content_files.rs` and
  `playdate_content_files.rs` (the Playdate's, pictures made one-bit like
  the bought art's; `private_table` became `playdate_table` for both).
  `bundle.rs` lays the layer between the bought art and `assets/`
  (`layers_with`), marked private in the engine's sense (not handed out,
  counted in the fingerprint, a missing file still shown under `assets/`)
  but not bought art (`bundle::is_private` stays false: no credit).
- `shuyi.ron`: `private_assets.folders` gains `content` and `story`;
  `credits.folders` unchanged.
- `.github/actions/private-assets`: `content` and `art` inputs, and a check
  that the pinned commit has `content/`. The content checkout is in
  `ci.yml` (clippy, test ×3, doc-tests, wasm, no-std, playdate, docs,
  coverage, tickets, bench), `mutants.yml` (plan, baseline, shards, weekly
  baseline and shards) and `playtest.yml` (build, and the battle jobs; the
  battles are listed from `assets/battles/` and the content's).
- `xtask`: `content_layer.rs`. `check-script` and `voice` read the
  dialogue with the content's files over `assets/`'s; `cargo shuyi
  check-keys` and `lint` scan the content too. `lines` and `lang-status`
  read the embedded bundle, which has it.
- The `content-change` skill and its Codex loader; `AGENTS.md`,
  `work-ticket` (fetch the checkout before the first build) and
  `run-gates`.
- The private repository: `content/README.md`, `story/README.md` and its
  README's rows (Zafnok/visions-of-shuyi-assets#1, merged by Nick), and a
  ruleset on its `main` made with `gh api` (id 27133256, "main: pull
  requests only (ADR-0117)": no deletion, no force push, a pull request
  needed; the repository's admin role may bypass it).
- Setup: Nick's old deploy key's private half was lost, so the Dependabot
  secret holds a **second** read-only deploy key ("visions-of-shuyi
  dependabot (read-only)"); the builds' key and its Actions secret are
  unchanged.

**Deviations**

- **No `private-content` feature: the checkout is the switch.** The
  engine's tools build the game with the features `shuyi.ron` lists, and
  `cargo shuyi bench` builds the tool program with `game.features` alone,
  which can't name a feature a build without the checkout lacks; every
  other job and every local gate would have needed the flag in its own
  spelling, and one that forgot it would quietly read the public files
  (ADR-0117, Alternatives). So the action outputs no feature either.
- **Cargo's watch of a folder that may not be there.** A watched path
  that is missing makes cargo rebuild on every build, so with no checkout
  the build makes an empty, git-ignored `assets-private/` and watches it
  (and a checkout without `content/`, its git index). `bundle.rs`'s test
  that every layer is its folder catches a build that missed a checkout
  made after it.
- **`export` checks out no content** (0158 keeps it so: its package is a
  public artifact), and the `wasm` job no longer uploads its web build: a
  public repository's artifacts can be downloaded by anyone signed in.
  `playtest.yml`'s xtask artifact embeds the content; it was already kept
  one day only.
- **cargo-mutants** copies the tree without what `.gitignore` lists, so
  `.cargo/mutants.toml` sets `gitignore = false`.

**For 0157 and 0160**

- With real content over `assets/`, a test that expects a moved file's
  display path to be `assets/...` (`bundle::tests::display_path_prefixes_assets`
  reads `data/palette.ron`) or reads a moved file's public stand-in reads
  the real one in CI. Seen with a made-up content palette.
- A rule of `crates/content/playdate-bundle.txt` must match a file of
  `assets/` or of the content; a rule that names only a moved file fails a
  build without the checkout.

**Size:** estimate about 200 production lines (the build's layer and the
bundle's order, as `private-assets`' were, plus a small reader in xtask),
measured 184 (0.9×, under 2×). PR size: planned about 1,300, measured
about 1,500 with these notes.

**CI:** see the PR for the first run's content jobs and the checkout
step's time.

**Notes for Nick:** nothing to check. On your machine, the full copy in
`D:\tactical-rpg\assets-private\` gets `content/` and `story/` the next
time it pulls its `main`.
