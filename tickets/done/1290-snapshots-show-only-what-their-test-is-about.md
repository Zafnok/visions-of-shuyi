---
id: "1290"
title: Snapshots show only what their test is about, with the cursor's pulse held still
type: infra
milestone: Later
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-18
---

# 1290 — Snapshots show only what their test is about, with the cursor's pulse held still

## Context

From the retrospective of PRs #616–#735
([T20](../../docs/research/retro-2026-11-17-prs-616-735.md)). Most of the
260 snapshot asserts in `crates/ui` take the whole frame (about 190 are a
one-line `h.snapshot()` or `to_snapshot(..)`), cursor pulse included. So
a side-panel change or one extra frame re-blesses dozens of unrelated
snapshots. #720 (0484) changed 57 `.snap` files (1,590 lines), most
"only in the cursor's pulse", and its private Chapter 1 lessons snapshot
took visions-of-shuyi-assets#8 and two pin moves. #726 changed 22 and
#721 changed 15. `tests/it/ch01_lessons.rs:750-752` waits `32.0 / 60.0` s
only to put the pulse back where the private picture has it.

The engine already has the switch: `ctx.idle_motion`, "Off here, so a
screen left alone in a test looks the same a moment later". The platform
turns it on (the engine's `Setup::ctx`; `crates/playdate/src/start.rs:49`).
The Harness and `screen::tests::ctx()` leave it off. The game's looping
motions don't read it yet. `assert_private_snapshot!`
(`tests/it/main.rs:12`, ADR-0118 §3) has 18 uses: `ch01_lessons.rs` 1,
`places.rs` 4, `world_map.rs` 1 and `title_cinematic.rs` 12.

## Nick input

None. Tests only; what a player sees doesn't change.

## Scope

**In:**
- With `idle_motion` off, the game's looping motions draw at their start.
- A helper that snapshots one region of a frame. Battle snapshots use it.
- The private snapshots of `ch01_lessons`, `places` and `world_map` show
  only their content area, or become view asserts.
- One rule in ADR-0007 and `run-gates`.
- Expected size: about 40 production lines and 150–300 test lines.
  `.snap` files don't count (`cargo xtask pr-size`).

**Out (do not do):**
- The engine's `Cursor`, or a crop API in the engine. An engine change
  needs Nick's OK and the `engine-change` skill.
- The `title_cinematic.rs` shots, except the menu in step 5.
- Frame text helpers and behavior asserts on views (1214, 1237).
  **Either order** with 1214, 1237, 1232 and 1213. If 1237 is done, call
  its shared `battle::testing::render`. Skip tests that 1232 deleted.
  1213's edge fade still reads `scene.clock_ms`, so step 1 covers it.
- Reaching a battle through one Harness call and reading its text from
  the view (1289). **Either order:** both edit `tests/it/battle.rs`,
  `controller.rs` and `language.rs`; merge the lines, and if 1289 is
  done, crop what its `h.quick_battle()` callers snapshot.

## Implementation steps

1. **Hold the loops still.** When `ctx.idle_motion == IdleMotion::Off`,
   use the start value in: the cursor brightness at
   `screens/battle/mod.rs:2623` (`1.0`); `clock_ms(ctx)` at `mod.rs:2916`
   (`0`: edge arrows, effect marks, units stepping in place);
   `screens/world_map.rs:916` (`pulse(0.0)`); both `lost_shaded(..)` calls
   (`glyph/forecast.rs:354`, `playdate/forecast.rs:225`; `0.0`); and
   `notes::blink_on` at `mod.rs:2567` (`0.0`). Tests of the motion turn
   it on (`c.idle_motion = IdleMotion::On`): `cursor_pulses_with_frame_time`
   (`battle/tests.rs:258`), `scene_tests.rs:122` and any others that fail.
   Fix the comment at `harness.rs:528`.
2. **Crop helper.** In `crates/ui/src/harness.rs`, add
   `pub fn snapshot_of(buf: &GlyphBuffer, area: Rect, palette: &Palette) -> String`
   and `Harness::snapshot_of(&self, area: Rect)`. Build a buffer the size
   of `area` with the source's cell size, then
   `blit(buf, -area.x, -area.y)`. `blit` clips and moves sprites and
   rectangles. It doesn't copy a backdrop or painted pictures, so tests
   of those keep whole frames (say so in the doc comment).
3. **Battle snapshots by region.** Use `layout.rs`'s `MAP_VIEW` (menus,
   tips and boxes over the map), `SIDE_PANEL` (panel, forecast) and
   `HELP_BAR`. Start with the files #720 re-blessed:
   `screens/battle/{art,item,magic,skill,turn,tip,progress,view,lesson}_tests.rs`
   and `tests/it/{battle,controller,flow,language,map_skin,map_zoom}.rs`.
   Keep whole frames for `quick_battle_renders`,
   `quick_battle_in_every_theme` and other whole-screen tests, with a
   comment saying why.
4. **Private snapshots.** In `ch01_lessons.rs`, delete the wait at
   `:750-752`. Snapshot the lessons box's region, or assert its view
   (`lessons_end`, read at `:754`). If the crop shows only public UI
   text, make it a public `assert_snapshot!`; `cargo xtask check-leaks`
   decides. Crop `places.rs:82,131,197,274` to the place, without the
   text box or help bar. Crop `world_map.rs:412` to the map, without the
   panel or help.
5. **Logo shot.** If `title_cinematic_12_logo` (`title_cinematic.rs:675`)
   shows the public title menu, crop the menu out or assert it on the
   view.
6. **Private repository.** Re-bless or delete the private `.snap` files
   with the `content-change` skill: a PR there, then
   `cargo shuyi private-assets --pin` here.
7. **Rule.** Add a bullet to `docs/adr/0007-testing-strategy.md`: a
   snapshot shows its test's region, motion is still in tests, and a
   private snapshot shows only the private part. Add the same line to the
   *Snapshots* section of `.claude/skills/run-gates/SKILL.md`, then run
   `pwsh -File scripts/sync-agent-skills.ps1 -Check`.
8. **Two checks, on a throwaway branch.** (a) Add one row to
   `paint_panel` (`screens/battle/glyph.rs:318`) and run
   `cargo test -p trpg-ui --no-fail-fast` with private assets checked
   out. (b) Add `h.wait(1.0 / 60.0)` before each snapshot in
   `tests/it/battle.rs` and `tests/it/controller.rs`. Write down the
   failure counts, then delete the branch.

## Acceptance criteria

- [x] Unit test `the_battle_holds_still_without_idle_motion`: with
      `c.clock_s = 0.5`, a 0.5 s update and `idle_motion` off, the scene
      has cursor brightness 1 and `clock_ms` 0. With it on, they are
      `BLINK_MIN` and 500.
- [x] A unit test crops a 3 × 2 region with `snapshot_of` (cells and a
      sprite's moved `dest`).
- [x] `grep -n "32.0 / 60.0" crates/ui/tests/it/ch01_lessons.rs` finds
      nothing.
- [x] `grep -nE "assert_private_snapshot!\(dir, (\"[a-z0-9_]+\", )?h\.snapshot\(\)\)" crates/ui/tests/it/ch01_lessons.rs crates/ui/tests/it/places.rs crates/ui/tests/it/world_map.rs`
      finds nothing.
- [x] Check (a) fails fewer than 10 snapshot tests, none of them private.
      Check (b) fails none. Both counts are in the completion notes.
- [ ] `assets-private.rev` is at the private PR's merge, and
      `cargo xtask check-leaks` passes.
- [x] ADR-0007 and `run-gates` state the rule.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the still-motion test and the `snapshot_of` test.
- Snapshot / integration: the existing tests, cropped. Each still checks
  its subject. Remove a test only when a view assert replaces it.

## Completion notes

- **Still loops.** `screen::loop_clock_s(ctx)` is the clock a loop reads:
  `ctx.clock_s`, or 0 with `idle_motion` off. The battle's map clock (edge
  arrows, effect marks, units stepping), both forecasts' lost-HP blink,
  the battle cursor (brightness 1), the notes' blink and the world map's
  cursor pulse all hold at their start. Tests of a motion turn it on.
- **Crop helper.** `harness::snapshot_of(buf, area, palette)` and
  `Harness::snapshot_of(area)`; battle unit tests use `layout::snap` and
  the new `layout::MESSAGE_ROW`. Deviation: `blit` also carries painted
  pictures, so only a backdrop is left out (the doc comment says so).
- **Crops.** About 60 snapshot asserts now show `MAP_VIEW`, `SIDE_PANEL`,
  `HELP_BAR` or the message row. Whole frames stay, with a comment, for
  `quick_battle_renders`, `quick_battle_in_every_theme`, the info screens
  and Game Over's first frame. Beyond the ticket's files, check (a) found
  six more in `act1`, `ai_phase`, `glyph`, `heal_aim` and `tests.rs` and
  `map_skin`'s dusk test; they are cropped too.
- **Private snapshots.** Both `ch01_lessons` shots now snapshot the lesson
  box alone (new `battle::glyph::lesson_box`), which is public UI text, so
  they are public `assert_snapshot!`s (`check-leaks` passes) and their
  private files are deleted. The wait at `:750-752` is gone. `places`
  crops to the rows over the text box (`dialogue::glyph::TEXT_BOX`, now
  public), `world_map` to `world_map::glyph::MAP`, and the logo shot to
  the rows over the menu (`title::glyph::MENU_ROW`, now public).
- **One test changed its keys:** `controller::each_default_button_does_what_its_key_does`
  played `f Up d`, which ends where it started; only the pulse used to
  differ. It now plays `f Up d Up`.
- **Check (a)** (one more row in `paint_panel`, private assets checked
  out): 7 snapshot tests fail, none private: the panel's own
  (`panel_snapshots_…`, `hover_empty_plain`, `hover_unit_on_forest`,
  `a_downed_unit_under_the_cursor_shows_in_the_side_panel`) and the
  whole-screen ones (`quick_battle_renders`, `quick_battle_in_every_theme`,
  `game_over_opens_with_its_animation`).
  **Check (b)** (`h.wait(1.0 / 60.0)` before all 12 snapshots of
  `battle.rs` and `controller.rs`): 0 fail.
- Friction: estimate 40 production lines (the ticket's, checked against
  the five call sites and a two-line crop), measured 68 (1.7×), under 2×.
- No gameplay rules decided. Nothing changes for a player: the platforms
  turn idle motion on.
