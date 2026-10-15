---
id: "0128"
title: The slowest ui tests run in seconds, checking just as much
type: infra
milestone: M0 Foundation
model: sonnet-5
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0128 — The slowest ui tests run in seconds, checking just as much

## Context

The unmutated `trpg-ui` test run under nextest on CI has grown from about
95 s (2026-10-04) to 232 s (2026-10-15). Ticket 0127 found that 32 tests
of 5 s or more take 486 of its 814 thread-seconds. They slow down every
`cargo test` (the `test` jobs of `ci.yml`, coverage, the local gates) and
every mutation run that reaches them. Ticket 0127 moves them to the end of
the mutation run; this ticket makes them cheap.

The slowest, from the unmutated run of PR #287's mutation run
(run 38796893781, artifact `mutants-pr-1-…`, `log/baseline.log`), debug
build, seconds:

| Test | s |
| --- | --- |
| `it`: `map_skin::the_skin_never_changes_the_game` (`crates/ui/tests/it/map_skin.rs`) | 90 |
| `screens::battle::path::tests::steered_paths_are_legal_and_follow_the_cursor` | 38 |
| `it`: `title_cinematic::a_press_in_the_silence_keeps_the_menu_past_the_loop` | 26 |
| `dialogue::tests::no_embedded_scene_shows_someone_who_is_gone` | 20 |
| `map_view::sprite::ground::tests::every_tile_of_the_map_has_one_picture_of_its_own_and_none_spills` | 19 |
| `map_view::sprite::tests::sprite_skin_paints_only_the_area` | 19 |
| `map_view::sprite::tests::sprite_units_skin_paints_only_the_area` | 19 |
| `map_view::glyph::tests::glyph_skin_paints_only_the_area` | 19 |
| `map_view::sprite::tests::tiles_lie_inside_the_area_edge_to_edge` | 19 |
| `map_view::sprite::units::tests::a_picture_is_centred_and_stands_on_the_hp_bar` | 19 |
| `screens::title::tests::the_next_launch_never_shows_the_same_backdrop` | 18 |
| `it`: the other `title_cinematic::…` tests and the `dialogue_log::…` tests | 6–12 each |
| `harness::tests::the_title_music_clock_counts_up_and_wraps_at_the_track_length` | 10 |

Many are proptests (256 cases by default) that redo expensive setup in
every case: e.g. `no_embedded_scene_shows_someone_who_is_gone` calls
`trpg_content::load_embedded()` (parses all the game's content) in each of
its 256 cases.

## Nick input

`None.`

## Scope

**In:**
- The tests in the table, and any shared test helper that makes them slow
  (`ctx()`, `sized()`, `sheets()`, content loading, harness runs).

**Out (do not do):**
- Checking less (CLAUDE.md rule 5): no fewer proptest cases, no
  `#[ignore]`, no narrower input strategies, no deleted assertions. If a
  test can only get faster by checking less, leave it and say why in the
  completion notes.
- Changing game code to make a test faster, unless the change is a plain
  speed-up with no change in behaviour (then name it in the PR).
- Turning on optimisation for the workspace crates in the `dev`/`test`
  profiles: it slows every mutant's build (ADR-0073).
- The mutation workflow (ticket 0127).

## Implementation steps

1. Time the suite locally with `cargo test -p trpg-ui -- --report-time`
   or `cargo nextest run -p trpg-ui` (nextest prints each test's time; it
   is not `cargo mutants`, which must not run on Nick's machine). Write
   the before numbers down.
2. For each test in the table, find where the time goes (a quick
   `std::time::Instant` print, or reading the test). Typical fixes:
   - Setup that does not depend on the proptest inputs (loading content,
     building a skin, decoding an atlas) moves out of the case into a
     `std::sync::LazyLock` / `OnceLock` shared by the cases.
   - A harness test that plays many frames one at a time: check whether
     the harness can step in bigger slices with the same assertions
     (ticket 0205's harness, `crates/ui/src/harness.rs`).
   - A check that repaints a whole buffer for every case where one paint
     plus a comparison would do.
3. Keep every assertion and every input the test had. Re-run and write
   the after numbers.

## Acceptance criteria

- [ ] Each test in the table runs in under 5 s locally in a debug build,
      or the completion notes say why it can't without checking less.
- [ ] No test lost cases, inputs or assertions (the PR description lists
      each test changed and what moved).
- [ ] The completion notes have before/after times for each test, and the
      unmutated `ui` test time from the PR's own mutation run if it has one.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit / property: the existing tests, unchanged in what they check.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
