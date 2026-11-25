---
id: "0193"
title: "Code waiting to move to the engine is marked, and a PR that edits it warns until the engine ticket is checked"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-24
---

# 0193 — Code waiting to move to the engine is marked, and an edit to it warns

## Context

Nick, 2026-11-24, after 1180 stalled: "how do we ensure such drift
doesn't happen again".

**What happened:**
- Engine#2132 built `TacticsOrder` from the persona bots' `order`
  (`crates/bots/src/persona.rs`) as it stood on 2026-10-24. The pin
  passed it on 10-25, and 1180, which moves the game onto it, was ready.
- Over the next month seven game tickets changed that same code:
  - 0517 (10-27): no wandering once no foe is left;
  - 0510 (11-01): skirmish bands in `targets.ron`;
  - 0523 (11-03): one strike per unit first;
  - 0849 (11-04): seize;
  - 0526 (11-08): covers in a survive;
  - 0851 (11-08): hold bands and no median check on a hold;
  - 0529 (11-10): holds in a survive.
- None of the seven names 1180 or an engine ticket
  (`grep -cE "1180|2132|TacticsOrder|engine#21" tickets/done/<each>`
  gives 0).
- Three audits re-checked 1180 (11-01, 11-03, 11-17). Each looked only at
  `blocked_by` and the pin, never at what had changed in the code since
  the engine ticket.
- Starting 1180 on 11-24 found that `TacticsOrder` couldn't express two
  of the new rules, so the bots' seeded traces couldn't stay equal.
  Engine#2148 was filed for them, and 1180 waits on it again: a month
  lost, plus a second engine PR.

**Why nobody saw it.** The `// engine-friction: engine#NNNN` marker
(`engine-friction` skill §6, `engine-change` §2) flags a *workaround*.
The engine watcher reads it (`trpg_engine_watch::fetch::friction_markers`),
and 1239 will flag one that has gone stale. Code that an engine ticket
will *replace* (the 0506 sweep's seven pieces, a carve's leftovers) gets
no marker. So a session editing it sees nothing, and the engine ticket
and the game ticket that adopts it both go stale.

**The fix, in four parts:**
1. Mark code that is waiting to move.
2. Warn when a PR edits marked code. The warning never fails a check
   (Nick's rule for process checks: `pr-size`, ADR-0101).
3. The editing PR leaves a dated line on the adopting ticket.
4. An audit that re-checks a ready ticket also looks at what changed in
   the marked code.

## Nick input

None.

## Scope

**In:**
- **The marker.** The same comment, with what replaces the code:
  `// engine-friction: engine#NNNN, moves (game MMMM deletes it)`.
  - The watcher's parser (`trpg_engine_watch::friction_markers`,
    `crates/engine-watch/src/lib.rs:463`) reads only `engine#` and four
    digits, so the extra words change nothing for it, nor for 1239.
  - A marker covers its own line through the end of the item after it:
    the first later line at the marker's indentation that is `}` or ends
    with `;`.
- **The warning**, in `cargo xtask pr-size` (`crates/xtask/src/pr_size.rs`),
  on both its outputs: the Claude Code hook (`--hook`) and the CI step
  (`ci.yml`, the `::warning` line).
  - It fires when the branch's diff against its base (`git diff -U0`)
    changes a line covered by a marker.
  - Its text, one line per marker: `<path>:<line> moves to the engine
    with engine#NNNN (game MMMM deletes it): does engine#NNNN still cover
    this change? If not, add it to the engine ticket now (engine-change),
    and add a dated line to MMMM`.
  - A PR that only adds or removes markers doesn't warn.
- **The backfill.** Mark the code of every open game ticket that waits on
  an `engine#` ticket to delete or replace game code (`grep -l "engine#"
  tickets/open/*.md`, read each one's Scope). At least:
  - 1180's: `persona.rs`'s `order`, `split_attacks`, `covers`, `hold`,
    `advance`, `once_per_action`, `Tally::score`, `Goals`, `danger`,
    `threats`, `seize_distance`; `playtest.rs`'s `Targets` and its
    `checks*`; `movement.rs`'s `attack_tiles`, `threat_area`,
    `threat_tiles`, `area_of`, `zone_of`, `danger_zone`, `threats_at`;
    `ai.rs`'s hostile-target `flow_field`. They are marked for engine#2132,
    engine#2131, engine#2129 or engine#2148 as each one's ticket says.
  - 1207's xtask playtest wrapper.

  Name each in the completion notes. A workaround already marked stays as
  it is.
- **Skills** (the shared sources in `.claude/skills/`, then `pwsh -File
  scripts/sync-agent-skills.ps1`):
  - `engine-change`, *Ticket and follow-up*: the PR that files an engine
    ticket to replace game code marks that code.
  - `engine-friction`, *How a sweep runs*: each filed row marks its code.
  - `work-ticket` §3: when the `pr-size` warning names a marker, check
    the engine ticket covers the change. If it doesn't, add the gap to
    that ticket by `engine-change`, with Nick's OK. Then add a dated
    line to the adopting ticket's Context.
  - `work-ticket` §1: an engine-blocked ticket is ready only when the
    marked code hasn't changed since its engine ticket was written, or
    every change is in its Context (`git log --since <engine ticket's
    date> -- <marked files>`).

**Out (do not do):**
- Failing any check on a marker. This is a warning, like `pr-size`.
- 1239's stale-marker check. Either order with 1239: if 1239 is done, a
  "moves" marker whose engine ticket is done at the pin passes its check
  while the adopting ticket is open (it names `engine#NNNN`), so nothing
  to do; if not, nothing to do (1239 builds the check as written).
- 0181's one PR-checks CI job. Either order with 0181: if 0181 is done,
  the warning comes through `pr-size` wherever that job runs it, so check
  it does; if not, nothing to do (`ci.yml`'s `pr-size` step prints it).
- Adopting any engine piece (1180, 1207 and the rest are their own
  tickets).

## Implementation steps

1. `crates/engine-watch/src/lib.rs`: next to `friction_markers`, a pure
   `marker_spans(source) -> Vec<(usize, usize, String)>` (first line,
   last line, engine id) with the covering rule above.
2. `crates/xtask/src/pr_size.rs`:
   - a pure `changed_lines(diff_u0)` that reads the `@@ -a,b +c,d @@`
     hunks of `git diff -U0 <base>...HEAD`, on the new side;
   - `moved_warnings(spans, changed)`: one line per marker, in path and
     line order;
   - both added to `output` and `hook_answer` beside the size warning.
     The hook keeps its JSON shape.
3. The backfill (Scope). Run `cargo xtask pr-size` on a scratch edit
   inside a marked `fn` and check it warns. Then drop the edit.
4. The skill texts, and sync them.
5. `docs/engine/README.md`, where the watcher's markers are described
   (around line 37): the second use, one paragraph.

Expected size: about 150 production lines and 150 of tests, plus the
skills, the backfill markers (about 40 lines) and this ticket: under 600.

## Acceptance criteria

- [x] `marker_spans` covers a marked `fn`, `struct`, `impl` and a
      one-line item, and nothing past the item's end (unit tests).
- [x] `changed_lines` reads hunks with and without counts, and a
      deletion (`+c,0`) (unit tests).
- [x] `cargo xtask pr-size` and `--hook` print the marker line for a diff
      inside a marked item, and nothing for a diff outside one or one
      that only adds a marker (tests on fixture diffs).
- [x] Every open ticket that waits on an `engine#` ticket to delete game
      code has its code marked. The completion notes list them, with
      1180's and 1207's.
- [x] `engine-change`, `engine-friction` and `work-ticket` carry the
      steps above; `scripts/sync-agent-skills.ps1 -Check` passes.
- [x] The engine watcher's tests still pass: a "moves" marker parses as
      today's.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `marker_spans`, `changed_lines` and `moved_warnings`, in
  `crates/engine-watch` and `crates/xtask`.
- Integration: `pr-size`'s output and hook answer on fixture diffs, in
  `pr_size.rs`'s own `mod tests` (beside the hook's existing tests).
  Add no new file under `crates/xtask/tests/`.

## Completion notes

Done 2026-11-24.

**What was built.**
- `trpg_engine_watch::marker_spans` (`crates/engine-watch/src/lib.rs`),
  returning a `MarkerSpan` (first and last line, engine id, and the
  game ticket of a "moves" marker), not the plain tuple the step named:
  the warning needs MMMM.
- `pr_size::changed_lines`, `moved_warnings` and `moved_markers`, and
  `output` gets the warnings: one line each in the hook's answer (same
  JSON, the lines after the size warning), `warning:` lines in the
  report, and a `::warning title=Moves to the engine::` line each in CI
  (`ci.yml`'s `pr-size` step, unchanged).

**Deviations and the agent's rules (technical, no gameplay).**
- Only a "moves" marker warns; a workaround's marker (no
  `moves (game MMMM deletes it)`) doesn't, since the warning's text
  needs MMMM and a workaround is edited as part of normal work.
- A marker on a file's **first line covers the whole file**, for
  modules that go whole (a `mod x;` line covers only itself). The
  skills and `docs/engine/README.md` say so.
- A comment line never ends an item; a marker after code on its own
  line covers that line when the code ends there.
- The diff is the merge base to the working tree (as the size count
  is), so the hook sees uncommitted edits too; in CI that is
  `<base>...HEAD`.
- `crates/engine-watch/` is left out of the check: its tests hold
  markers as fixtures (as `added_markers` already does).
- The ticket estimated about 40 marker lines; the backfill is 183,
  because 30-odd open tickets wait on an engine ticket to replace
  code. The PR is still about 900 lines.

**Markers added** (183, `moves (game MMMM deletes it)`; "file" is a
first-line marker over the whole file). None of the items had a marker
before.
- **1180**: `persona.rs`: `order`, `split_attacks` (engine#2148, which
  now holds the gaps 2132 missed); `covers`, `hold`, `advance`,
  `once_per_action` (2132); `Tally::score` (2130); `Goals`, `Danger`,
  `danger`, `Threat`, `threats`, `seize_distance` (2131).
  `playtest.rs`: `Targets`, `checks`, `checks_of`, `checks_with`
  (2148), `Faster` (2129). `movement.rs`: `attack_tiles`,
  `threat_area`, `threat_tiles`, `area_of`, `danger_zone`,
  `ai_danger_zone`, `zone_of`, the free `threats_at` (2131; the
  planner's cached `threats_at` stays). `ai.rs`: `flow_field`, and the
  planner's `flow` and `danger` caches (2131).
- **1207**: `crates/tools/src/playtest.rs`, file (engine#2532).
- **1168**: `options.rs`'s `Named` and its eight impls, `choice`
  (engine#2314).
- **1170** (engine#3097): files `pd/allocator.rs`, `pd/game_stack.rs`,
  `pd/panic.rs`, `heap.rs`, `menu.rs`, `pads.rs`, `screen.rs`,
  `stack.rs`, `text.rs`, `ui/src/game/system_menu.rs`; `pd/mod.rs`'s
  `MenuItem`, `add_menu_item`, `set_menu_item_title`; `pd/entry.rs`'s
  `Loading` and `load`.
- **1182**: `game_tools.rs`'s `After` (engine#2540; its option, field
  and loop were already marked).
- **1184** (engine#2371): the one-bit skin files of options,
  key bindings, profile and name entry; `screens/playdate.rs`'s
  `menu_size`, `menu_text`, `menu`, `list`; `battle/playdate.rs`'s
  `item_text`, `has_notes`, `shown`, `menu_size`, `paint_menu`,
  `paint_menu_framed`.
- **1186** (engine#2233): `bundle::parsed`, the `load_all` of `map.rs`
  and `cinematic.rs`, `ScriptsLoad::start`, `battle::load_lights`.
- **1211** (engine#3201): `credits.rs`'s `rows`, `CreditsScreen` and
  its two impls, `CreditsView`, the glyph `paint`, `actions::CREDITS`.
- **1212** (engine#3202): `debug/sprite_test.rs` file,
  `actions::SPRITE_TEST`.
- **1213** (engine#3203): `map_view/edges.rs` file; the glyph and
  sprite `paint_over`, `edge_arrow`, `EDGE_GAP`, `edge_src`,
  `edge_dest`, the one-bit `paint_under`, `marks::danger_edges`.
- **1214** (engine#3204): the harness's ten input helpers
  (`music_load_delay` … `clear_audio`), `turn_tests::assert_in_font`.
- **1215** (engine#3205): `SellConfirm`, `SellConfirmView`, both
  `paint_sell`.
- **1217** (engine#3206): `follow`, `camera_in`, `fit_camera`,
  `cursor::dir`, `walk_steps`, `walk_done`, `walk_pos`.
- **1218** (engine#3207): the sixteen dialogue tests and first
  `proptest!` block it deletes (`crates/ui/src/dialogue/tests.rs`).
- **1219** (engine#2243): `tileset/look.rs` file; `Checker::inside`,
  `grid_rect`, the free `inside`.
- **1220** (engine#2244): `tune.rs`'s `code`, `In`, `field_at`,
  `is_name`, `enemy_spans`, `add_to_list`, `set_field`.
- **1221** (engine#2245): `playdate_images.rs`'s `Kind`, `Lift` and
  its impl, `impl Kind`, `Style`, `style`, `Scale`, `KINDS`,
  `kind_of`, `make`, `with_rim`.
- **1222** (engine#2246): `Portrait`, `Expression`, `Sidecar`,
  `parse_portrait`, `check_image`, `STANDARD_EXPRESSIONS`,
  `expression_problem`.
- **1223** (engine#2247): `battle::light_of`, `item`'s `err_at_item`.
- **1224** (engine#2578): `tools/src/skill_pack.rs` file;
  `skill_pack`, `skill_pack_in`.
- **1225** (engine#2579): the title tests' fence test, `names_word`
  and its test.
- **1226** (engine#2580): `Canvas` and its impl, `blend`.
- **1227** (engine#3091): `ui/src/platform.rs` file;
  `the_playdates_facts_are_the_engines`.
- **1228** (engine#2109): `ease_out`, `ease_in`, `AiAction::camera`.
- **1229** (engine#3049): `icon.rs`'s `ICO_SIZES`, `ico` and its
  test; both `embed_windows_resources` of `crates/app/build.rs` and
  `crates/story-preview/build.rs`.

**Nothing to mark**: 0141, 0142, 0532, 0759 (the code that goes is an
expression inside a test helper that stays), 1012, 1014, 1017, 1078,
1241, 1276 (its code isn't written yet), 1350: they add code or change
tests and docs, and replace no game code. Not Rust, so no marker:
`crates/ui/src/playdate_target.ron` (1227), `assets/icon/icon.ico` and
`crates/content/playdate-bundle.txt` (1229), the playtest workflow
(1207, engine#2021).

**Checked.** A scratch edit inside `split_attacks` made `cargo xtask
pr-size` and `--hook` print the marker's line; dropped after. This
branch itself, which only adds markers, gets no warning.

**Follow-ups.** None filed. 0181's `check-pr` (with 1257's diff
warning folded in) should reuse `moved_warnings` rather than build a
second check: 1257's "a PR that changes a file an open ticket names"
is this, by marker.
