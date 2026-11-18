---
id: "1297"
title: "Dependabot's runs build on the public files: no private key for a dependency's new version"
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: []
nick_input: setup
completed:
---

# 1297 — Dependabot's runs build on the public files

## Context

The engine pin move of 2026-11-18 passed engine PR #498 (engine ticket
3407). It changed the engine's own rule for a game's private files: `shuyi
repos` makes one deploy key and no Dependabot secret, and the template's
private-files action warns and builds on the public files on a Dependabot
pull request without a key, where it failed before. The reason: with the
key, a dependency's new version runs its build script and tests next to
the private files before anyone has read it. The engine's pack skill
`shuyi-private-files` now says so.

The game keeps its own action, `.github/actions/private-assets/action.yml`
(ADR-0040, ADR-0117), not the template's. Its `key` input says
"Dependabot's runs have a Dependabot secret of the same name", and with
`required: 'true'` (every job that loads content) it fails a run without
the key. So today every Dependabot pull request here builds with the
private repository's content, story and art checked out. ADR-0040 already
says Dependabot's runs "can't read a private repository, and must still
build and pass every gate": the secret contradicts it.

The game's CI already keeps the engine's other new rule (no job that
checks out the private files caches `target/`: every such `rust-cache`
has `cache-targets: false`).

## Nick input

**Setup:** deleting the Dependabot secret and the deploy key it holds acts
on Nick's GitHub account. The session shows him the two commands (step 3)
and runs them only with his yes, or he runs them.

## Scope

**In:**
- The action: on a Dependabot run (`github.actor == 'dependabot[bot]'`,
  passed in as an input or read from the environment) without a key, warn
  and build on the public files even when `required` is `'true'`, as the
  engine's template does; every other run without a key fails as today.
- The `key` input's description: Dependabot gets no key.
- Delete the Dependabot secret `PRIVATE_ASSETS_KEY` and the private
  repository's deploy key that only it uses (Nick's yes first).
- ADR-0040: a line under its decision that Dependabot's runs get no key
  (engine #498's reason), or a short new ADR if the change is bigger than
  a line.

**Out (do not do):**
- Moving the game onto the engine's template action or `shuyi repos`.
- Changing which jobs need the private files.

Estimate: about 15 lines of YAML and 10 of docs.

## Implementation steps

1. In `.github/actions/private-assets/action.yml`'s `plan` step, let a
   Dependabot run without a key take the warning branch (`::warning::`,
   `export-flag=--without private-assets`, exit 0) whatever `required`
   says; reword the warning to say why.
2. Search `.github/workflows/` for jobs that would then fail on the public
   files for another reason (a step that reads `assets-private/` without
   checking it is there) and guard them the same way.
3. Ask Nick, then: `gh secret delete PRIVATE_ASSETS_KEY --app dependabot
   --repo Zafnok/visions-of-shuyi`, and delete the Dependabot deploy key
   on the private repository (`gh repo deploy-key list` there names it).
4. Update ADR-0040 and the action's comments.

## Acceptance criteria

- [ ] The next Dependabot pull request's checks run on the public files
      with the warning and pass (or fail only for the dependency itself).
- [ ] A run of `ci.yml` on an ordinary pull request still checks out the
      content.
- [ ] No Dependabot secret named `PRIVATE_ASSETS_KEY` is left
      (`gh secret list --app dependabot`).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the action's behaviour is checked by the Dependabot pull
  request's run and an ordinary one (CI is the test here; no Rust code
  changes).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
