---
id: "0178"
title: The playtest's shared xtask leaves its run only encrypted
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["0171"]
nick_input: none
completed:
---

# 0178 — The playtest's shared xtask leaves its run only encrypted

## Context

A revision of an accepted risk, not a new leak. ADR-0117 §4 keeps every
build that embeds the private content out of public artifacts, with one
exception: `playtest.yml`'s `build` job (lines 91–131) builds
`target/release/xtask`, which links `trpg-content` and so embeds the
content, and uploads it as the `playtest-xtask` artifact
(`retention-days: 1`) for the `play` jobs to download (lines 163–166).
On a public repository any signed-in GitHub user can download it for
24 hours after each run, and Playtest ran about 112 times in the week of
the retrospective
([retro T15](../../docs/research/retro-2026-11-17-prs-616-735.md)).
The retro's verifier confirmed the mechanics and asked for an ADR
revision.

The cheapest close: the `build` job encrypts the program with a key
only the repository's secrets give, and each `play` job decrypts it. The
`play` jobs already get `secrets.PRIVATE_ASSETS_KEY` (they check out the
content with it), and a fork's run has neither, so it fails at
`private-assets` (`required: "true"`) either way. Rejected: passing it
through `actions/cache` (0171 shows a fork's pull request can restore
`main`'s caches), and building xtask in each of the about 15 `play` jobs
(minutes and runner slots on every play).

0171 (open) adds a test that fails a job with private files that uploads
a build, with this artifact as its one named exception, and amends the
same §4. This ticket waits for it and narrows that exception to the
encrypted file. `story-check` on the writer's mirror stays as ADR-0122's
Consequences accept it (a private repository, a hired writer).

## Nick input

None. The key is the existing `PRIVATE_ASSETS_KEY` secret (Dependabot's
runs have their own secret of that name, used by both jobs of one run
alike), so no new secret is set up.

## Scope

**In:**
- `playtest.yml`: encrypt in `build`, decrypt in `play`, with a guard
  step that proves the uploaded file holds no content.
- 0171's workflow check: its exception names the encrypted file only.
- ADR-0117 §4 rewritten for the new rule.

**Out (do not do):**
- `story-check` on the mirror (ADR-0122 accepts it).
- Caches (0171), logs that quote a content line (ADR-0117 §4 accepts
  them), the battle reports (`playtest-report-*`, no build in them).
- Making xtask read the content from disk at run time instead of
  embedding it: a larger change, only if encryption fails here.
- A new secret, or any repository setting (Nick's).

## Implementation steps

Expected size: about 40 lines of workflow, 15 of ADR, 5 of test.

1. **Encrypt in `build`.** After `cargo build --release --locked -p
   xtask` (line 125), add a step with
   `env: PASS: ${{ secrets.PRIVATE_ASSETS_KEY }}` that runs
   `openssl enc -aes-256-cbc -pbkdf2 -iter 200000 -salt -pass env:PASS
   -in target/release/xtask -out target/release/xtask.enc`. The key goes
   in through `env:`, never on the command line or in a log.
2. **Guard it** in the same job, before the upload. Take a needle from
   the content at run time, never written into the public workflow:
   `needle=$(head -c 64 "$(ls assets-private/content/battles/*.ron | head -1)")`.
   Fail unless `grep -qaF -- "$needle" target/release/xtask` finds it
   (the guard works) **and** `grep -qaF -- "$needle"
   target/release/xtask.enc` does not, and unless `head -c 8` of the
   `.enc` file is `Salted__` (openssl's header). Print only found/absent.
3. **Upload only the `.enc`.** The `upload-artifact` step (lines 126–131)
   gets `path: target/release/xtask.enc`; keep the name, the one-day
   retention and `if-no-files-found: error`.
4. **Decrypt in `play`.** After the download (lines 163–166), a step with
   the same `env: PASS` runs `openssl enc -d -aes-256-cbc -pbkdf2 -iter
   200000 -pass env:PASS -in target/release/xtask.enc -out
   target/release/xtask`, then removes the `.enc`. The `chmod +x` in
   "Play the battle" (line 173) stays.
5. **Comments.** Rewrite the comment at lines 87–90: the xtask embeds
   the content, so it leaves the job only encrypted with
   `PRIVATE_ASSETS_KEY` (ADR-0117 §4, this ticket).
6. **0171's check.** In the test 0171 adds to
   `crates/xtask/src/main.rs` (`no_job_with_private_files_caches_or_uploads_a_build`),
   the allowed upload for `playtest.yml` job `build` becomes
   `target/release/xtask.enc`, with the reason: encrypted, ADR-0117 §4.
   Any other entry there (such as `release.yml`'s `dist/release/kept/`
   reports) stays as 0171 left it.
7. **ADR-0117 §4.** Replace the third bullet: no build that embeds the
   content leaves a job in the clear; `playtest.yml`'s shared xtask is
   uploaded encrypted with a key derived from the secret (PBKDF2), and
   the job checks that before uploading. Add 0178 to the ADR's related
   tickets and a dated amendment note in its status line.

## Acceptance criteria

- [ ] On the PR's Playtest run (run it by hand if no lever changed), the
      `build` job's guard step prints "found" for the plain program and
      "absent" for the `.enc`, and every `play` job passes.
- [ ] `gh run download <run> -n playtest-xtask` gives only
      `xtask.enc`, whose first 8 bytes are `Salted__` (said in the
      Completion notes).
- [ ] `cargo test -p xtask no_job_with_private_files` passes, and fails
      when the upload's path is set back to `target/release/xtask`
      (tried, and said in the notes).
- [ ] ADR-0117 §4 no longer lists a content-embedding artifact in the
      clear.
- [ ] `zizmor` (`security.yml`) passes on `playtest.yml`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: 0171's workflow check with the narrowed exception.
- Property: none.
- Snapshot / integration: the guard step and one full Playtest run on
  the PR.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
