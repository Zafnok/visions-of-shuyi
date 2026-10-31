---
id: "0124"
title: Check PR and ticket titles for accidental mentions
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0124 — Check PR and ticket titles for accidental mentions

## Context

The v0.1.0 release notes listed a stranger, the GitHub user `music`, as a
contributor. The notes are generated from PR titles
(`generate_release_notes: true` in `.github/workflows/release.yml`), and
PR #157 was titled "[0710] Dialogue scripts choose the music: `@music`"
with the directive written bare, so GitHub read it as a mention of that
user. The notes were corrected by hand on 2026-10-13.

PR #247 wrote the rule down (`CLAUDE.md` under Environment, `work-ticket`
step 6, `write-ticket` step 3): no bare at-word in a PR title, PR body,
commit message or comment; put it in backticks or drop the at sign. A
written rule is only as good as the session reading it. Nick asked for a
mechanical check (2026-10-13), and tests are the review (ADR-0007).

A PR title is the ticket's title (`work-ticket` step 6), so the check
belongs in two places: the ticket title, where it is caught before any
work starts, and the PR title, which is what the release notes use.

## Nick input

`None.`

## Scope

**In:**
- A pure function in `xtask` that finds bare mentions in a line of text.
- `cargo xtask ticket-lint` reports a bare mention in the `title:` of a
  ticket in `tickets/open/`.
- A new `cargo xtask check-mentions <text>` that the `tickets` CI job runs
  on the PR title.
- A line in the `run-gates`, `work-ticket` and `write-ticket` skills and in
  `CLAUDE.md` saying the check exists.

**Out (do not do):**
- Don't check tickets in `tickets/done/`: 0710's title has the bare
  directive and done tickets are history.
- Don't check PR bodies, commit messages or comments. Only the PR title
  reaches the release notes; the written rule covers the rest.
- Don't add `edited` to the `pull_request` triggers of `ci.yml`: every edit
  of a PR's text would run the whole CI again. The check runs on each push,
  so a title changed after the last push is not checked again; say so in
  the Completion notes and leave it.
- Don't change `release.yml` or how the release notes are generated.
- Don't add a dependency (no `regex` crate): the scan is a few lines over
  `char`s.
- Don't add a new workflow or a new required check; the `tickets` job is
  already behind `ci-result` (ADR-0014).

## Implementation steps

1. Add `crates/xtask/src/mentions.rs` with
   `pub fn bare_mentions(text: &str) -> Vec<String>`, returning each bare
   mention found, with its at sign, in order. A bare mention is an at sign
   that:
   - is followed by an ASCII letter or digit (the mention is the at sign
     plus the run of ASCII letters, digits and `-` after it);
   - is not right after an ASCII letter, digit or `_` (so the address
     `noreply@anthropic.com` is not a mention);
   - is not inside a code span: walk the text and flip an `in_code` flag at
     every backtick; skip everything while it is set.
2. Register the module in `crates/xtask/src/main.rs` and add a
   `check-mentions` arm to the `match` in `main`, written like `check_keys`
   there: exactly one argument (the text) or `usage: cargo xtask
   check-mentions <text>` and exit code 2; no mention prints
   `check-mentions: OK` and returns 0; otherwise print each mention and one
   line of advice ("put it in backticks or drop the at sign: GitHub reads
   it as a mention of that user, and the release notes list them as a
   contributor") and return 1. Add the command to the usage text `main`
   prints for an unknown command.
3. In `crates/xtask/src/tickets.rs`, in `parse_ticket`, when `folder` is
   `Folder::Open` and the frontmatter has a `title`, push one error per
   bare mention: `<path>: title has the bare mention <mention>; GitHub
   would read the PR title as a mention of that user (put it in backticks
   or drop the at sign)`.
4. In `.github/workflows/ci.yml`, job `tickets`: add
   `PR_TITLE: ${{ github.event.pull_request.title }}` to the job's `env`
   (never paste the expression into the script: a PR title is untrusted
   text), and add a step after "Run ticket lint":
   ```yaml
   # No accidental mention of a GitHub user in the PR title: the release
   # notes are generated from it (ticket 0124).
   - name: Check the PR title for accidental mentions
     if: github.event_name == 'pull_request'
     run: cargo xtask check-mentions "$PR_TITLE"
   ```
5. Docs: add `cargo xtask check-mentions "<PR title>"` to the command list
   in `.claude/skills/run-gates/SKILL.md`; in `work-ticket` step 6 and
   `write-ticket` step 3 and in `CLAUDE.md`'s bullet, add that
   `ticket-lint` and CI check titles. Add the check to the gate table in
   ADR-0014 only if that ADR's status allows edits to the table (other
   tickets have added rows); otherwise leave the ADR alone.

## Acceptance criteria

- [ ] `cargo xtask check-mentions "[0710] Dialogue scripts choose the music: @music"`
      exits 1 and names `@music`.
- [ ] The same title with the directive in backticks exits 0, and so does
      `"Mail noreply@anthropic.com about [0001]"`.
- [ ] A ticket in `tickets/open/` whose `title:` has a bare mention makes
      `cargo xtask ticket-lint` fail with the ticket's path; the same title
      in `tickets/done/` does not (0710 stays as it is and the lint passes
      on the repository).
- [ ] The `tickets` CI job runs the new step on pull requests and not on
      pushes to `main`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (`mentions.rs`): a bare mention at the end, at the start and in the
  middle of a line; two mentions in one line, in order; a mention in
  backticks is not reported; a mention after a closed code span is; an
  email address is not; an at sign alone or before a space or punctuation
  is not; a name with a `-` and digits is returned whole; `@dependabot[bot]`
  returns `@dependabot`.
- Unit (`tickets.rs`, beside the existing tests there): an open ticket with
  a bare mention in its title gives the error; a done ticket with the same
  title gives none.
- Property: none.
- Snapshot / integration: none.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
