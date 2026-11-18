---
id: "0173"
title: Playtest CI skips comment and test edits, plays each input once, and splits its slowest battles
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-17
---

# 0173 — Playtest CI skips comment and test edits, plays each input once, and splits its slowest battles

## Context

Retro T10 ([report](../../docs/research/retro-2026-11-17-prs-616-735.md)):
Playtest used 15,424 of 49,275 job-minutes (31%) from Nov 10 to 17. A
played run is 17 jobs and 127–149 job-minutes, and ch03 and
side_company_ledger (20–33 min each) set how long a PR waits.

- `.github/playtest-levers.sh:31` plays on any `crates/content/src/`
  file: credits.rs, tip.rs and dialogue.rs all played (#718, #728, #682),
  and so did comments and in-file tests (#677, #658). Line 25 misses
  `lang/data_tests.rs` (#725).
- About 37 `main` pushes replayed the merged PR's inputs (about 5,563
  minutes). The seeds are fixed, so the same inputs give the same report.
- Playtest isn't required, and ADR-0014's gate table leaves it out.

0172 makes the private content's pin a lever. This ticket does the rest.

## Nick input

None.

## Scope

**In:** narrower code levers and their test; reuse of a report whose
inputs hash the same (cache, and the merged PR's run on `main`); ch03 and
side_company_ledger split into two try ranges and merged; ADR-0014 and
`docs/playtesting.md` say Playtest is advisory.

**Out (do not do):**
- `assets-private.rev` as a lever, the content's lever paths, `demo.yml`
  (0172). Whichever lands second merges the other's lever-script lines.
- Making Playtest required (a ruleset setting, Nick's).
- The xtask artifact (0178) and `target/` caches (0171).
- Skipping `detect` on a draft PR (0174). Either order: merge the `if:`.
- The engine's playtest workflow (1207). Persona, band or try changes.

## Implementation steps

Expected size: about 90 lines of shell, 120 of workflow, 60 of Rust, 150
of tests and 30 of docs.

1. **Levers** (`.github/playtest-levers.sh`). Add `*_tests.rs` to line
   25. Before line 31, skip the content modules the battle never reads.
   Candidates are credits, tip, title, lang, audio, voice, font, image,
   palette, playdate_images, playdate_palette, portrait, cinematic and
   keymap (`x.rs` and `x/*`). Keep one only if a `grep` shows
   `crates/core`, `crates/bots` and `crates/xtask/src/playtest.rs` never
   use its types. Check dialogue.rs the same way, since a scene can add
   a unit or an item. `bundle.rs` stays a lever.
2. **Comment and test edits.** The script takes optional `BASE HEAD`
   arguments (`playtest.yml:84` passes `$range`). For a `.rs` or `.ron`
   lever, strip both versions (`git show REV:path`) and compare. On a
   match it prints `skips: <path> (comments or tests only)`. Stripping
   drops blank lines and lines that start with `//` (after spaces). It
   also drops each column-0 `#[cfg(test)]` that comes right before a
   column-0 `mod <name> {`, and every line through the next line that
   is exactly `}` (rustfmt puts that brace alone at column 0). A file
   added or deleted on either side always plays.
3. **Input hash** (`.github/playtest-inputs.sh`): one sha256 of
   `git ls-files -s` over `Cargo.toml`, `Cargo.lock`,
   `rust-toolchain.toml`, `shuyi-engine.rev`, `crates/{core,content,bots}`,
   `crates/xtask/Cargo.toml`, `crates/xtask/src/playtest.rs`, `assets/`,
   `assets-test/`, `crates/ui/tests/it/fixtures/`, `playtest.yml` and
   both scripts, plus `git -C assets-private rev-parse HEAD:content`.
   It is wider than the levers on purpose: a miss only costs a play,
   while a wrong hit hides a balance change.
4. **Reuse, in `detect`** (`playtest.yml:49-85`), only when
   `play=true`:
   - Check out the content once (0172 may have added this step already).
   - Output the hash as `inputs`.
   - `actions/cache/restore` `reports/` with key
     `playtest-reports-<inputs>`.
   - On a push to `main` with no hit, find the PR
     (`gh api repos/$REPO/commits/$SHA/pulls`) and its latest Playtest
     run (`gh run list --workflow playtest.yml --event pull_request
     --branch <head> --limit 1`). If that run succeeded and its
     `playtest-inputs` artifact holds the same hash, download its
     `playtest-report-*` into `reports/`. A main push can't restore a
     PR's cache. Give `actions: read` to this job only.
   - On a hit, upload `reports/` as `playtest-report-reused` and output
     `reused` (`cache` or the run id) and `run_play=false`. `build` and
     `play` then run only if `run_play == 'true'`.
5. **Gather** (`playtest`, lines 200–246). Accept a skipped `build` when
   `reused` is set, take the battle list from `reports/*.status` then
   (`build.outputs.battles` is empty), and say where the reports came
   from on the summary's first line. Statuses are still judged, so a reused blocked
   battle still fails. When every battle has a status,
   `actions/cache/save` `reports/` under the key, and upload the hash as
   `playtest-inputs`. Reports hold no build: 0171's check allows a cache
   whose path isn't `target`.
6. **Split.**
   - `crates/xtask/src/playtest.rs` gets `--merge <json>...`. It reads
     `--json` `Record`s and requires the same battle, tier, turn cap,
     army and checks in each. It passes every part's `report.entrants`
     and the first part's checks to `shuyi_core::bots::playtest::judge`,
     which joins parts by name and sorts them by seed. It averages
     `seconds_per_try`, prints `render(&record, None)`, and exits 1 if
     blocked.
   - In `build`, the env `SPLIT_BATTLES: ch03 side_company_ledger`
     makes the matrix `{battle, part, seed, runs}`: two parts of 50
     (seeds 1 and 51) for those battles, one part of 100 for the rest.
     Split parts run with `--runs 50 --seed <seed> --json` and upload
     `playtest-part-*` JSON. A part's own exit code and "Fail if the
     battle is blocked" step are skipped: half the tries can't judge the
     floor, so only `merge` does.
   - A new `merge` job, matrix over the split battles, gets the xtask
     the way `play` does (decrypting it if 0178 is done). It writes
     `reports/<battle>.txt` and `.status`. `playtest` needs it.
   - Each part keeps all three personas: the "faster" check compares two.
7. **Docs.** Update `playtest.yml`'s header (lines 1–27), the script's
   header, and `docs/playtesting.md` lines 182–196. Add a Playtest row
   to ADR-0014's gate table: PR and push, levers only, not required
   (fails on a blocked battle, bands only warn). Add a dated note on
   reuse by input hash.

## Acceptance criteria

- [x] A unix-only test `playtest_levers_skip_comments_tests_and_presentation`
      sits in `crates/xtask/src/main.rs`'s tests (beside 0172's, if it
      has landed). It builds temp git repos and gets `play=false` for a
      comment-only `credits.rs`, a comment in `terrain.ron`, an in-file
      `mod tests` edit and `lang/data_tests.rs`. It gets `play=true` for
      a code line in `crates/core/src/combat.rs`, a new file and a
      `*_winning.ron`. It fails without the stripping (tried, noted).
- [x] A `--merge` test: two parts of a test battle give the same tries,
      checks and `blocked` as one whole run. Mismatched parts are an error.
- [x] The merge diffs of #658, #677, #718, #725 and #728 go through the
      new script. The notes name the real lever for each that still plays.
- [ ] This PR's Playtest shows `play (ch03 1/2)`, `play (ch03 2/2)` and
      `merge (ch03)`, with one ch03 table. After a docs-only push, the
      next run says "reused from cache" and has no `build`.
- [ ] After the merge, the `main` run says "reused from run <id>". If it
      played instead (main moved), the notes say why.
- [x] `zizmor` passes on `playtest.yml`.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the lever-script fixtures, and `--merge` against an unsplit run.
- Workflow: this PR's own runs (split, cache reuse) and the `main` reuse.

## Completion notes

**Done.** `.github/playtest-levers.sh` skips `*_tests.rs` and the content
modules no battle reads, and, given `BASE HEAD` (the workflow passes its
range), skips a `.rs`/`.ron` lever whose edit is only comments, blank
lines or a column-0 in-file test module. `.github/playtest-inputs.sh`
hashes the inputs; `detect` checks out the content, restores
`reports/` from the cache under that hash, and on a push to `main` with no
hit takes the merged PR's last passing run's reports when its
`playtest-inputs` artifact matches. On a hit `build`, `play` and `merge`
are skipped and `playtest` judges the reused reports (a blocked one still
fails). `cargo xtask playtest --merge <json>...` joins parts;
`SPLIT_BATTLES: ch03 side_company_ledger` plays those in two parts of 50
(seeds 1 and 51) and a `merge` job writes their report. ADR-0014 has a
Playtest row (advisory) and a dated note; `docs/playtesting.md` updated.

**Modules skipped, and why.** credits, tip, title, lang, audio, voice,
font, image, palette, playdate_images, playdate_palette, portrait,
cinematic, keymap (`x.rs` and `x/*`): no `x::` path or exported type of
theirs appears in `crates/core`, `crates/bots` or
`crates/xtask/src/playtest.rs`. The battle modules of `crates/content`
name a few of them only to check references or to draw (`battle.rs`'s
`AudioManifest` for music cues, `item.rs`'s `lang::text_width`,
`terrain.rs`'s palette and glyphs, `battler.rs`'s images).
**dialogue stays a lever**: the bots' army reads `content.dialogue` and
`Stage::Joins` (a scene brings a unit in). `bundle.rs` stays.

**Retro PRs through the new script** (their merge commit's diff):
#658 skips (every lever touched only in comments or test modules), #677
skips (a comment in `terrain.ron`), #718 skips (`credits.rs` and a test file), #728 skips (`tip.rs`), #725 **still plays**: it adds
`short_name` to core's `WeaponDef` and edits `assets/data/items.ron` and
`crates/content/src/item.rs`, real levers by path (a field the rules never
read, but the script can't know that).

**Stripping test.** With `strip()` replaced by `cat`, a comment-only edit
of a lever plays (tried by hand in Git Bash); the unix-only Rust test then
fails on its first skip case. The Rust test doesn't run on Windows; the
same cases were run by hand on Windows with the script and all matched.

**This PR's runs.** Playtest run 44223934350 (head 45ea02e6) played
`play (ch03 1/2)` (15 min), `play (ch03 2/2)` (17 min) and `merge (ch03)`,
the same for side_company_ledger, with one table per battle, and saved
`playtest-reports-9628b566…`. The next docs-only push (run 44232479153)
played again, rightly: `main` had moved under the PR (0450 changed
`crates/core`), so the merge commit's inputs hashed differently
(`d772bae2…`). The reuse from cache and on `main` are checked in the PR. Minutes the Nov 10–17 runs would have saved, roughly:
the ~37 main replays (~5,563 job-minutes, per the retro) plus about
4 PR runs at 127–149 each (#658, #677, #718, #728) ≈ 6,100 of 15,424.
The split adds a little (one merge job, two runners' setup for two
battles) but cuts the PR's wait from ch03's 20–33 minutes to about half.

**Friction sweep, 2026-11-17.** The sweep (merged to `main` while this
PR was open) moved steps 1–5 to 1207, to be built on the engine's
workflow, and put 0173 after 1207, so that playtest CI would be built
only once. This PR was already done and green, so Nick chose on
2026-11-18 to land it as built: the savings start now, and 1207 carries
it over (its notes say what to keep).

**Engine friction:** estimate 60 production lines of Rust (the ticket's
figure; I didn't write my own before coding, a slip), measured 93
(1.6×), under 2×. **PR size:** ticket estimate ~450, measured ~800 (the
workflow grew to 300 changed lines with the reuse and merge jobs).

No gameplay rule was decided.
