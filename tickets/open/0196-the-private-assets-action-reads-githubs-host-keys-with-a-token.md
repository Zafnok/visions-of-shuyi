---
id: "0196"
title: The private-assets action reads GitHub's host keys with a token
type: bug
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: ["0195"]
nick_input: none
completed:
---

# 0196 — The private-assets action reads GitHub's host keys with a token

## Context

Ticket 0195 (ADR-0135) gave the private-assets action
(`.github/actions/private-assets/action.yml`) a step, "Merge the pin
files", that runs whenever a pull request has a pin file in
`assets-private.d/`. To let git's ssh trust github.com, it fetches the host
keys with an unauthenticated call:

```bash
curl -fsSL https://api.github.com/meta | jq -r '.ssh_keys[] | "github.com " + .' > "$known"
```

GitHub's API allows 60 unauthenticated calls an hour per IP address, and
hosted runners share addresses. On PR #881 (ticket 0605) the `docs` job
failed on that line with `curl: (22) The requested URL returned error:
403` while every other job of the same run passed the same step: the
runner's address had used its 60 calls. Every pull request that carries a
pin file runs this step in each job that checks out the content, so this
fails CI at random.

## Nick input

None. Verified by CI on the pull request (a run with a pin file, every
job green).

## Scope

**In:**
- The host-key call in `.github/actions/private-assets/action.yml`, and
  any other `api.github.com` call without a token in `.github/` that a
  job makes on every run (`grep -rn "api.github.com" .github`).

**Out (do not do):**
- Anything else in how pin files are merged or landed (0195's design).
- Hard-coding the host keys in the repository: they rotate.

## Implementation steps

1. Send the job's token with the call:
   `curl -fsSL -H "Authorization: Bearer $GH_TOKEN" https://api.github.com/meta`,
   with `GH_TOKEN: ${{ github.token }}` in the step's `env` (a composite
   action can read `github.token`). An authenticated call has the
   repository's own, much higher limit.
2. Add `--retry 3 --retry-all-errors` so a passing network fault doesn't
   fail the job either.
3. Run the same `grep` over `.github/` and do the same to any other call
   that a job makes on every run.
4. Run `zizmor` and `actionlint` (the `run-gates` skill's CI checks on
   workflows) on the changed files.

## Acceptance criteria

- [ ] No job in `.github/` calls `api.github.com` without a token on every run (the `grep` above).
- [ ] A pull request with a pin file passes every job that checks out the content.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the pull request's own CI run, with a pin file (this
  ticket's branch can add one to a throwaway private branch, or ride on an
  open content pull request's).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
