---
id: "1267"
title: "The release workflow takes its matrix and version check from export --plan, and records each package's SHA-256"
type: infra
milestone: M8 Release
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1267 — The release workflow takes its matrix and version check from `export --plan`

## Context

The engine pin move of 2026-11-14 passed engine PR #407 (engine ticket
3050): `cargo shuyi export --plan [--tag <tag>]` builds nothing and answers
the game's `ExportPlan`: the packages' version, whether it is a
prerelease, and for each target of `shuyi.ron` its exporter's `host`
(runner), `container` (Linux: the sniper SDK image, `export::SNIPER`) and
`ci` (`build` or `check-only`). With `--tag` it fails (`SH0001`) unless
the tag is `v` followed by the version. The engine's template ships a
`release.yml` that builds its matrix from that plan and publishes a
`SHA256SUMS` with `actions/attest` attestations.

The game's `.github/workflows/release.yml` does the same jobs by hand:
`check-version` compares the tag with `trpg-app`'s version in bash, and
the `package` job's matrix names each target's runner and the sniper
image's digest, copied from the engine's CI. When the engine moves the
sniper image, the game's copy goes stale silently.

The game's packages never leave their job (ticket 0158, ADR-0117 §4), so
the game can't take the template's workflow whole; it takes the plan and
the sums.

## Scope

**In:**
- `check-version` runs `cargo shuyi export --plan --tag <tag> --json`
  (the dry run's `fake_tag` too) instead of its bash comparison, and
  outputs the version, `dry_run` and the matrix (the plan's `exports`,
  with the game's `release_name` per target kept beside it).
- The `package` job's `runs-on` and `container` come from the plan (the
  hosts mapped to `windows-latest`, `ubuntu-latest`, `macos-latest`, as
  the engine's template does); the hard-coded sniper digest goes.
- Each package job writes its package's SHA-256 (the file it leaves
  behind, never uploaded) and the release job puts the lines together
  as `SHA256SUMS` on the GitHub Release; on a real release, `actions/attest`
  attests them (the digests only: no package is published).
- `docs/releasing.md` says what `SHA256SUMS` holds and how to check a
  package against it.

**Out (do not do):**
- Publishing packages anywhere (0158, 0901).
- The itch.io push (0901 keeps the game's own `.github/actions/butler`).
- The reveal kit (1268), which will read these sums.

## Implementation steps

1. Read the engine's `templates/shuyi-default/base/.github/workflows/release.yml`
   and `export.yml` at the pin, and `docs/contract.md`'s `export --plan` row.
2. Estimate first (about 40 lines of the workflow removed, about 40
   added); measure with `cargo xtask code-lines`.
3. Change `check-version` and the matrix; keep `release_name` and the
   per-target steps as they are.
4. Sums and attestation; `permissions` for `attestations: write` and
   `id-token: write` on that job only.
5. A `workflow_dispatch` dry run on the branch, and one with
   `fake_tag: v0.0.0`, which must fail at `check-version`.

## Acceptance criteria

- [ ] `release.yml` names no runner image digest and no version check of its own.
- [ ] A dry run builds all four targets with the plan's hosts and the Linux container.
- [ ] A dry run with `fake_tag: v0.0.0` fails at `check-version` with the plan's message.
- [ ] A dry run's summary shows a `SHA256SUMS` line per package.
- [ ] `actionlint` and zizmor (the `security` workflow) pass.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

None beyond the gates and the dry runs (links in the PR).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
