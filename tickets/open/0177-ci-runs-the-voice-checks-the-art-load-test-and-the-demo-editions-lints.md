---
id: "0177"
title: CI runs the voice checks, the art load test and the demo edition's lints
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0177 — CI runs the voice checks, the art load test and the demo edition's lints

## Context

The retrospective of PRs #616–#735
([report](../../docs/research/retro-2026-11-17-prs-616-735.md), T14)
found content checks that pass in CI without checking anything:

- **Voice.** `every_lesson_and_tip_the_guide_says_has_a_clip`
  (`crates/xtask/src/voice/tests.rs:1171`) returns at once when
  `voice/en/` has no manifest, and
  `the_disclosure_list_names_the_music_voices_and_licenses_a_package_ships`
  (`crates/xtask/src/game_tools/tests.rs:722`) `continue`s past a missing
  `voice/`. CI never has `voice/`: every test job passes `art: "false"`,
  and `.github/actions/private-assets` copies `voice/` only with the art.
  #664's notes say the clip check is skipped in CI.
- **Art.** The 8 tests of `crates/content/tests/it/private_assets.rs`
  (the content loads with the bought art over it, ADR-0040 §5) run only in
  `playtest-itch.yml`, on push to `main`: a break is found after merge.
- **Demo.** `crates/content/build.rs` sets the `demo_edition` cfg when
  `SHUYI_CONTENT` names `crates/content/demo`. `demo.yml` compiles it on
  content PRs, but no clippy and no test ever runs under it.

The retro's fourth point (the 192 `skip_without_the_game()` guards) is
dropped: they go through `there_or_skip` in `crates/content/src/bundle.rs`,
which already panics in CI.

## Nick input

None.

## Scope

**In:**
- CI fetches `voice/` for the Linux `test` leg, and the two voice tests
  fail there when it is missing.
- A `ci.yml` job runs `private_assets::` on same-repo pull requests that
  touch content.
- A `ci.yml` job runs clippy and the tests of `trpg-content` under the
  `demo_edition` cfg on those pull requests.
- ADR-0040 §6 and ADR-0117 §3 say which jobs now fetch what.

**Out (do not do):**
- The `skip_without_the_game()` guards and a local "N skipped" report (CI
  already fails on them).
- The `story-trial` root (same cfg as the demo; one root is enough).
- Caching rules for the new jobs beyond what 0171 sets (0171's job).
- Fetching `voice/` in `mutants.yml`, coverage or the other legs.
- `demo.yml`'s trigger on a pin move (0172), and dry runs of
  `playtest-itch.yml` (0175).
- The disclosure lists themselves (0180). Either order: this ticket
  changes only the voice test's skip.

## Implementation steps

1. **Action.** In `.github/actions/private-assets/action.yml`, add an
   input `voice-without-art` (default `"false"`): with `art: "false"`,
   `"true"` adds `voice` to the sparse folders and runs the existing
   "Put the voice clips beside the game" copy (widen that step's `if:`).
   It sets no `features` output (the art is still out).
2. **Test helper.** In `crates/xtask/src/voice.rs`, add
   `#[cfg(test)] pub(crate) fn voice_or_skip(root: &Path, needed: bool) -> bool`:
   `true` when `root/voice` is a folder; otherwise it panics if `needed`,
   else prints a skip line naming `cargo shuyi private-assets` and
   returns `false`. The two tests call it with
   `std::env::var_os("SHUYI_NEEDS_VOICE").is_some()` (read at run time,
   so only the job that fetched `voice/` fails), in place of the silent
   `return` and `continue`. Fix their doc comments.
3. **Test job.** In `ci.yml`'s `test` job (line 60), pass
   `voice-without-art: ${{ matrix.os == 'ubuntu-latest' }}` and set
   `SHUYI_NEEDS_VOICE` on that leg only.
4. **Content filter.** In `.github/workflows/changes.yml`, add an output
   `content`: `true` on any non-PR event, or when a PR changes
   `assets-private.rev`, `assets/**`, `assets-test/**`, `crates/content/**`,
   `shuyi.ron`, `shuyi-engine.rev` or `Cargo.lock`. Either order with
   0174, which also edits the `classify` step: if it is done, push events
   diff their own commits there, and a pin move to the same private tree
   is `content=false` too.
5. **Art job.** Add `private-assets` to `ci.yml`, as
   `playtest-itch.yml` lines 41–57 do: `content: "true"`, art on,
   `required: "true"`, then `cargo test -p trpg-content --locked
   --features private-assets --test it private_assets::`. Run it when
   `needs.changes.outputs.content == 'true'` and the event is a push or a
   PR whose head repo is this one (forks have no key). No `rust-cache`
   and no artifact (bought art). Keep the `playtest-itch.yml` run.
6. **Demo job.** Add `demo-edition` to `ci.yml` under the same `if:`
   (content only, `art: "false"`), with
   `SHUYI_CONTENT: ${{ github.workspace }}/crates/content/demo`. The
   path must be absolute: `build.rs` resolves it from the crate's folder.
   Run `cargo clippy -p trpg-content --all-targets --locked -- -D warnings`
   and `cargo test -p trpg-content --locked`. Run both locally first. If
   a test checks something the demo leaves out by design, give it
   `#[cfg(not(demo_edition))]` and a comment; never weaken an assertion.
   Its `rust-cache` step uses `key: demo-edition` and the settings 0171
   gives every job with content.
7. Add `private-assets` and `demo-edition` to `ci-result`'s `needs`
   (0150, 0185 and 1250 edit it too; whichever lands later keeps the others'
   entries). If 0174 is done, both jobs also take its draft condition.
8. Amend ADR-0040 §6 and ADR-0117 §3 for the new jobs and `voice/`.
   Add the demo commands to the `run-gates` skill (with `SHUYI_CONTENT`),
   then run `pwsh -File scripts/sync-agent-skills.ps1`. Fix the module
   comment of `private_assets.rs`, which says only the playtest page runs it.

## Acceptance criteria

- [ ] On the PR, a throwaway commit without step 3's `voice-without-art`
      turns the Linux `test` leg red with `voice_or_skip`'s message (link
      the run in the Completion notes, then revert it).
- [ ] The Linux `test` log shows both voice tests ran, not skipped.
- [ ] `private-assets` and `demo-edition` pass on the PR, and a docs-only
      PR skips both.
- [ ] `zizmor` (the `security` workflow) reports nothing new.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `voice_or_skip` on a temp dir with and without `voice/`, and its
  panic when `needed` (`#[should_panic]`; no `set_var`, which is `unsafe`
  in edition 2024).
- Integration: the two voice tests and the new CI jobs themselves.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
