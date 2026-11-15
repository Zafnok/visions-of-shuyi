---
id: "1272"
title: "Press and playtest copies are marked for their recipient"
type: infra
milestone: M8 Release
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1272 — Press and playtest copies are marked for their recipient

## Context

The engine pin move of 2026-11-15 passed engine PR #417 (engine ticket
3302): `cargo shuyi export <target> --release --recipient <id> --label
<text> --signing-key <key>` copies a release package into
`dist/recipients/<id>/<target>`, with the recipient's 8-character code
in the program, the pack and one picture named by `shuyi.ron`'s
`export: (overlay_picture: …)`, and records it in a signed ledger;
`export --detect-recipient` finds whose copy a leaked file is. The
`shuyi-ship` skill's step 10 says how, and its limit: a mark is a
deterrent and circumstantial evidence, never proof.

The game will hand out closed builds before its release: press and
publisher copies around the store page (0901, 0903) and playtest builds
to people outside the team.

## Nick input

None for the work. Who gets a copy is Nick's call; the PR tells him
plainly what a mark can and can't show (the skill's limit), and records
nothing as decided.

## Scope

**In:**
- `overlay_picture` in `shuyi.ron`'s `export` part: a picture of the
  game's with at least seven colours (never text), shipped in every
  release package. A bought picture is fine (the package carries it
  already); none is committed.
- `docs/releasing.md`: making a recipient's copy, where the signed
  ledger and the signing key live (the private repository's checkout,
  outside `content/`, never this repository), and checking a leaked
  file with `--detect-recipient`.
- One test copy made and detected locally, its result in the PR.

**Out (do not do):**
- Sending any copy to anyone.
- A key or ledger in this repository.

Estimate: About 80 lines (a `shuyi.ron` line and a page of docs).

## Implementation steps

1. Read the `shuyi-ship` skill's step 10 and the engine's ADR for 3302.
2. Pick the picture, add `overlay_picture`, run an export with
   `--recipient` and detect it.
3. Write the docs; `run-gates`.

## Acceptance criteria

- A local release export with `--recipient test` makes a copy that
  `--detect-recipient` names.
- `docs/releasing.md` says how, and the mark's limit.

## Tests required

None beyond the gates; the test copy's export and detection output is in the PR.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
