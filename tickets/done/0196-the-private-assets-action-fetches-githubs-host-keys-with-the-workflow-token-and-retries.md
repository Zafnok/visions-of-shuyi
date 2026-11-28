---
id: "0196"
title: The private-assets action fetches GitHub's host keys with the workflow token and retries
type: bug
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-27
---

# 0196 — The private-assets action fetches GitHub's host keys with the workflow token and retries

## Context

Ticket 0195 (PR #883, ADR-0135 §8) made the composite action
`.github/actions/private-assets` merge each change's pin file
(`assets-private.d/*.rev`) into the base pin. Its step "Merge the pin
files" gives git the deploy key for that step alone, with an SSH
known-hosts file built from GitHub's host keys:

```bash
curl -fsSL https://api.github.com/meta | jq -r '.ssh_keys[] | "github.com " + .' > "$known"
```

The request carries no token, so it counts against the limit GitHub puts
on unauthenticated API calls, which a shared runner's address shares with
every other job on it. On 2026-11-28 two jobs of PR #875 (`tickets` and
`no-std`, run 46098920744) failed there with `curl: (22) ... error: 403`,
while the same step passed in about 20 other jobs of the run. Every PR
that carries a pin file can now go red at random, and a red that a re-run
fixes is the kind ticket 0150 set out to remove.

## Nick input

None.

## Scope

**In:**
- The "Merge the pin files" step sends the workflow's token
  (`github.token`, through the step's `env`) as an
  `Authorization: Bearer` header, and retries a failed request a few
  times with a short delay (5xx and 403 answers included). `-f` stays, so
  a request that still fails after the retries fails the job.
- Every other step of `.github/workflows/` and `.github/actions/` that
  calls GitHub's API with `curl` without a token gets the same fix
  (on 2026-11-28 there is none: the other `curl` lines download from
  release pages and other hosts, and `gh` sends its token itself).
- An `xtask` test that fails on a `curl` of `api.github.com` in a
  workflow or action file without an `Authorization` header and
  `--retry`, so a new one can't come back.

**Out (do not do):**
- Changing how the known-hosts file is used (the `GIT_SSH_COMMAND`,
  `StrictHostKeyChecking=yes`), the merge in `resolve.sh`, or which jobs
  check the private repository out.
- Downloads from other hosts (`github.com/.../releases`, the Playdate
  SDK, rustup): they aren't API calls and aren't rate-limited this way.
- `scripts/upload-debug-file.sh`, which already sends its own token.

## Implementation steps

1. In `.github/actions/private-assets/action.yml`, step "Merge the pin
   files": add `GH_TOKEN: ${{ github.token }}` to its `env`. Write the
   answer to a temporary file in `$RUNNER_TEMP` (removed by the step's
   `trap`) with
   `curl -fsSL --retry 4 --retry-delay 5 --retry-all-errors -H "Authorization: Bearer $GH_TOKEN" -o "$meta" https://api.github.com/meta`,
   then build the known-hosts file from it with the same `jq` filter. A
   file rather than a pipe, so a retry after a partial answer doesn't
   hand `jq` the same bytes twice (curl truncates a regular output file
   before retrying).
2. Search `.github/` for other `curl` calls of `api.github.com`; fix any
   the same way.
3. Add the test `every_github_api_curl_sends_a_token_and_retries` to
   `crates/xtask/src/main.rs`'s tests beside
   `every_workflow_file_parses`: for each file of `workflow_files()`,
   join lines continued with `\`, and every command line naming both
   `curl` and `api.github.com` must also carry `Authorization:` and
   `--retry`. It checks that it sees a bare `curl` of the API as wrong.

## Acceptance criteria

- [x] The step sends `github.token` and retries; `-f` is kept.
- [x] No other workflow or action file calls `api.github.com` with
      `curl` without a token.
- [x] `cargo test -p xtask every_github_api_curl_sends_a_token_and_retries`
      passes, and fails on the old line.
- [x] `zizmor` and `actionlint` report nothing new on the changed files.
- [x] All gates in the `run-gates` skill that apply pass (ticket-lint,
      typos, check-mentions, the xtask tests).

## Tests required

- Unit: the `xtask` test above, with its own case of a bare `curl`.
- Integration: the PR's own CI run, whose jobs with the key run the step
  when a pin file is present (none on this branch: the step returns
  early; the next pin-file PR runs it).

## Completion notes

- "Merge the pin files" now passes `github.token` in as `GH_TOKEN` and
  fetches `https://api.github.com/meta` with
  `curl -fsSL --retry 4 --retry-delay 5 --retry-all-errors` and an
  `Authorization: Bearer` header, into `$RUNNER_TEMP/private-assets-meta.json`
  (removed by the step's `trap` with the key and the known-hosts file),
  then builds the known-hosts file from it with the same `jq` filter. The
  known-hosts file and `GIT_SSH_COMMAND` are unchanged. `--retry-all-errors`
  is what makes curl retry a 403 at all (plain `--retry` retries only
  timeouts, 408, 429 and 5xx); `-f` still fails the step once the five
  tries are spent, about 20 seconds plus the requests.
- The search of `.github/` found no other `curl` of GitHub's API: the
  other `curl` lines download release files (binaryen, butler), the
  Playdate SDK and rustup, and the API calls elsewhere go through `gh`,
  which sends its token. `scripts/upload-debug-file.sh` already sends its
  own token.
- `every_github_api_curl_sends_a_token_and_retries` (`crates/xtask/src/main.rs`)
  reads every workflow and action file, joins `\`-continued lines and
  fails on a `curl` of `api.github.com` missing `Authorization:` or
  `--retry`. Checked against `main`'s action file: it fails naming the old
  line.
- This branch has no pin file, so its own CI skips the curl (the step
  returns early); the next PR with a file in `assets-private.d/` runs it.
- `zizmor --offline` and `actionlint`: no findings. No follow-up tickets.
