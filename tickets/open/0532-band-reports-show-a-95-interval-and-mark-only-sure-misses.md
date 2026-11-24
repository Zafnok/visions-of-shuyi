---
id: "0532"
title: "Band reports show a 95% interval and mark only sure misses"
type: feature
milestone: M4 Enemy AI
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["engine#2148"]
nick_input: none
completed:
---

# 0532 — Band reports show a 95% interval and mark only sure misses

## Context

The retrospective of PRs #616–#735
([T2](../../docs/research/retro-2026-11-17-prs-616-735.md#t2-stop-retuning-bands-on-a-moving-army-0531-first-the-arrival-army-off-the-ui-fixtures-upstream-balance-before-chapter-bands)) found that
band verdicts are judged on 100 tries. Then a success rate carries about
±10 points of noise, while a band is 25 points wide. The report still
marks 49/100 against a 50–75% band "✗ too hard", just as it marks 15/100.
#623 (0529) needed a second seed range by hand to see that ch05
Hardcore, 71 on seeds 1–100, was 77 on seeds 101–200.

The mark comes from the engine, not the game: `Band::verdict` and
`PlaytestReport`'s table in `shuyi_core::bots::playtest`
(`crates/core/src/bots/playtest.rs` at the pin `e738ffc2`, lines
236–258 and 780–795). `judge` (line 638) calls it (line 683), and both the game's
`cargo xtask playtest` and the engine's own playtest command that 1207
moves the game to print that table. So this is an engine change (it
passes both tests of the `engine-change` skill §1: any game's bots want
it, and it names nothing of this game). Engine core has
`deny(clippy::float_arithmetic)` and builds without `std`, so the
interval is computed in integers.

The other parts of T2 live elsewhere: a stable arrival army and the
check that re-recording a chapter moves later chapters little are 0531.
The order of the balance tickets is in 0876's `blocked_by` and in
`docs/ROADMAP.md` Lane I. This ticket also adds the rule that keeps that
order to the `work-ticket` skill.

## Nick input

None. The bands, the 20% floor and what a miss does stay Nick's
(`docs/design/playtest-bots.md`); this only says how sure the bots'
count is. The engine part is engine#2148's part D (Nick asked for it,
2026-11-24, with the engine's other bot gaps in one ticket); this ticket
is the game's half, after the pin passes it.

## Scope

**In:**
- Engine (engine#2148, part D; not this PR): a 95% Wilson score interval in whole percents for each rate
  check, shown in the table (`band 50–75% · 95% 35–53%`). New verdicts:
  ✗ too hard only when the whole interval is under the band, ⚠ too easy
  only when it is all over the band. A share outside the band whose
  interval still reaches it gets a "maybe" mark of its own (for example
  `~ maybe too hard`).
- Game: the tests that assert the old marks, `docs/playtesting.md`
  (*Reading the report*), and the `work-ticket` line below.
- `work-ticket` skill §1 *Choose the ticket*, one bullet: a `tuning`
  ticket, or any ticket that re-records the winning playthroughs
  (`crates/ui/tests/it/fixtures/*_winning.ron`), lists in its PR the
  open balance tickets upstream of the chapters it tunes. Any of them
  that waits on no Nick decision goes first, or the PR says why not.
  Ones waiting on Nick don't hold it. Then
  `pwsh -File scripts/sync-agent-skills.ps1`.

**Out (do not do):**
- The bands, the floor and the `blocked` rule (the floor stays judged on
  the share itself, so CI fails exactly as today).
- Autobalance's distance to the bands (`crates/core/src/bots/balance.rs`):
  it keeps aiming at the band on the share itself.
- The arrival army and any CI check on re-recording (0531).
- Retuning any battle (0876 and the Lane I balance tickets).
- Moving the game to the engine's playtest command (1207). Either order
  works: the table is the engine's in both.

## Implementation steps

1. Engine: **done by engine#2148 (part D)**, not here. Steps 1–3 are
   what that ticket builds, kept for reference. If the pin is not past
   engine#2148 yet, move it first (`engine-change` §7, the `engine-pin`
   skill's review). In `crates/core/src/bots/playtest.rs`, add
   `Band::interval(successes, count) -> Option<(u32, u32)>`: the lowest
   and highest whole percent `k` in `0..=100` that passes the score test
   `(100·s − k·n)² · 10_000 ≤ 38_416 · n · k · (100 − k)` (z² = 3.8416),
   in `u128`. The passing `k` form one run, so a scan is enough. `None`
   for 0 tries. If no whole `k` passes (only with thousands of tries),
   use the share's own percent. Check: 44/100 gives 35–53.
2. `RateVerdict` gains `MaybeBelow` and `MaybeAbove`. `Band::verdict`:
   `Below` when the interval's top is under `min`, `Above` when its
   bottom is over `max`, `Within` when the share is in the band, and
   the `Maybe` ones otherwise. Add the interval to `Found::Rate` and to
   the table line (around line 787). Keep reading old JSON history:
   `#[serde(default)]` on the new field.
3. Engine tests in `crates/core/src/bots/playtest/tests.rs`: the
   intervals 44/100 → 35–53, 0/100 and 100/100, 0 tries; every verdict
   on band 30–55 at 100 tries (20 → `Below`, 29 → `MaybeBelow`,
   40 → `Within`, 60 → `MaybeAbove`); the table text.
4. Game, after the pin passes it: update
   `band_check_flags_too_easy_and_too_hard` in
   `crates/bots/src/playtest/tests.rs` (lines 61–86: 29 and 60 become
   `Maybe`, 20 stays `Below`) and `render_matches_the_layout` in
   `crates/tools/src/playtest/tests.rs` (line 271). In
   `docs/playtesting.md`, *Reading the report*, explain the interval
   and the marks, with the noise at 100 and 200 tries.
5. Add the `work-ticket` bullet and sync the skill loaders.

Expected size: about 40 game lines, mostly tests and docs (the engine's
160 are engine#2148's), plus the pin move if this moves it.

## Acceptance criteria

- [ ] The pin is past engine#2148, whose table shows `95% lo–hi%` on
      every rate line.
- [ ] `cargo xtask playtest ch04 --bot all` on the moved pin prints the
      interval; a ✗ or ⚠ appears only when the whole interval is outside.
- [ ] A playtest run on `main` still fails a battle exactly when Casual
      is under its floor (`band_check_flags_too_easy_and_too_hard`
      covers 19 and 20).
- [ ] `docs/playtesting.md` and the `work-ticket` skill carry the new
      text; `scripts/sync-agent-skills.ps1 -Check` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (engine, engine#2148): `Band::interval` and `Band::verdict` at the
  numbers in step 3.
- Unit (game): the two tests in step 4.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
