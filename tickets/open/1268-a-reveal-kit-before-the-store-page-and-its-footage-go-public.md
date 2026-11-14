---
id: "1268"
title: "A reveal kit before the store page and its footage go public"
type: infra
milestone: M8 Release
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1268 — A reveal kit before the store page and its footage go public

## Context

The engine pin move of 2026-11-14 passed engine PR #411 (engine ticket
2816, the engine's ADR-0116): `cargo shuyi reveal prepare` writes a kit,
from a `reveal` part of `shuyi.ron`, that keeps the evidence of what is
the game's and when, before its first public footage: the approved
captures copied whole, the builds by hash (none for a store page alone,
or a release's `SHA256SUMS`), the receipts, salted commitments, a
manifest of every file's SHA-256 sealed by its own digest, an
`EVIDENCE.md` and a `CHECKLIST.md`. `cargo shuyi reveal verify` checks a
copy anywhere, by hashes only. The `shuyi-ship` skill's step 9 says how.

The game's first deliberate reveal is its Steam store page and trailer
footage (0903, after 0906's title check) and the itch.io page going
public (0901). The README's screenshots are already public; the kit
covers what is approved from here on.

## Nick input

None for the work. The kit's `CHECKLIST.md` reminds about copyright
registration (about US$65 in the US); whether to register is Nick's
business decision, not a design one: tell him plainly in the PR, with
the checklist's line, and record nothing as decided.

## Scope

**In:**
- A `reveal` part in `shuyi.ron`: the store URL once 0903 has one, the
  approved captures (the trailer and store screenshots, captured from the
  game, ADR-0040 allows them), and `salts` in the private repository's
  checkout outside `content/` (never in this repository).
- `docs/releasing.md`: when to run `reveal prepare` (before each store
  page or footage goes public), where the kit and its digest are kept
  (the digest in a pushed commit or tag, which dates it; the kit in the
  private repository), and `reveal verify`.
- A first kit for the store page, store-only if no release exists yet;
  with 1267 done, a release's `SHA256SUMS` is its builds.

**Out (do not do):**
- Making the trailer or the store page (0903, 0901).
- Deciding copyright registration.

## Implementation steps

1. Read the `shuyi-ship` skill's step 9 and the engine's ADR-0116.
2. Add the `reveal` part; `cargo shuyi reveal prepare`, then `reveal
   verify` on a copy, `--digest` against the committed digest.
3. `docs/releasing.md`.

## Acceptance criteria

- [ ] `cargo shuyi reveal prepare` writes the kit from `shuyi.ron`, and `reveal verify --digest <digest>` passes on a copy elsewhere.
- [ ] The salts file is in the private repository's checkout and in no tracked file here (`git ls-files` shows none).
- [ ] `docs/releasing.md` has the steps.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

None beyond the gates; the verify run's output is in the PR.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
