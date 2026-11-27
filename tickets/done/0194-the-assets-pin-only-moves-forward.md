---
id: "0194"
title: The assets pin only moves forward
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-27
---

# 0194 — The assets pin only moves forward

## Context

`assets-private.rev` names the commit of the private repository every
build and CI job checks out (ADR-0040, ADR-0117). Nick runs several ticket
sessions at once, and more than one may move the pin (the `content-change`
skill). When their pull requests merge out of order, the second can put
the pin back to a commit from before the first's: resolving the one-line
conflict by keeping its own line, or running `cargo shuyi private-assets
--pin` from a checkout whose `origin/main` was stale. Main then builds
without the first ticket's content while its code expects it, and the
playtest and every CI job read the wrong game.

Nothing checks the direction today: the private-assets action checks the
pin out whatever it is.

## Nick input

None. (Nick asked for it, 2026-11-27: "ensure assets pin moves are only
ever forward".)

## Scope

**In:**
- `cargo xtask assets-pin --check [--base <ref>]`: the pin in the working
  tree must be the base's pin or a commit after it in the private
  repository's history, else it fails and says how to fix it.
- It warns (never fails) when the new pin is not on the private `main`
  (the content-change rule that a game pull request never merges with the
  pin on a branch commit), and when the base's pin isn't on the private
  `main` (a repair of an earlier slip, which ancestry can't judge).
- The private-assets action can check out the whole history (commits
  only, no file contents) for it; CI's `tickets` job runs the check on
  every pull request and push to `main` (`ci-result` needs that job).
- The `content-change` and `run-gates` skills say so.

**Out (do not do):**
- Changing the engine's `private-assets --pin` (an engine ticket if it is
  ever wanted).
- Branch protection or rulesets: `ci-result` is already required.

## Implementation steps

1. `crates/xtask/src/assets_pin.rs`: the verdict as a pure function over
   "is an ancestor of" and "is on main", and the git around it.
2. The action's `history` input; the step in `ci.yml`'s `tickets` job.
3. The skills.

## Acceptance criteria

- [x] A pin moved to an older commit, or to a commit beside the base's,
      fails the check with the fix.
- [x] An unchanged pin and a pin moved forward pass.
- [x] A pin on a branch commit passes with a warning.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each verdict, from the pure function.
- Integration: a scratch git repository with a line of commits and a side
  branch, checked by the real git calls.

## Completion notes

`cargo xtask assets-pin --check [--base <ref>]` (`crates/xtask/src/assets_pin.rs`)
judges the move by the private history in `assets-private/`: back or
sideways fails with the fix, forward passes, a pin off the private `main`
warns. CI's `tickets` job runs it with `--base HEAD^1` on every pull
request and push to `main`, after the private-assets action checks out the
whole history (new `history` input: `fetch-depth: 0`, `filter:
blob:none`); `ci-result` already needs that job, so a backward move blocks
the merge. Tried on the real private repository: a pin three commits back
failed, the private `main` passed. The content-change and run-gates skills
say so. Estimate ~150 production lines; 130 measured.
