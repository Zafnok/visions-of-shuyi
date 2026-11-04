---
id: "1233"
title: "Bring the engine boundary record up to date: today's counts, a decision for every E row, and the tickets that move the rest"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1233 — Bring the engine boundary record up to date

## Context

From the game/engine boundary audit of 2026-11-03. `docs/engine/boundary.md`
is the record of what is engine and what is game ([ADR-0063](../../docs/adr/0063-an-engine-and-a-game-one-boundary-now-two-repositories-later.md));
its rule is that the ticket that moves a module changes its row. Several
rows are stale anyway:

- The summary (`:19-29`) still has 1101's sizes of 2026-10-14 (`ui`
  78.5k, `content` 15.6k, `xtask` 18.3k, "roughly a third goes to the
  engine"), which describe neither repository now.
- `:165` says `bundle.rs` keeps `include_dir!`; since 0123 (#508) every
  build reads `build.rs`'s generated tables. [ADR-0070](../../docs/adr/0070-playdate-front-end.md)
  `:203-205` still keeps "two code paths for one bundle until the
  engine's bundle (1103)". The engine's builder for those tables is
  engine#2215, parked by Nick until MVP; neither record says so.
- `:170` marks `portrait.rs`, `title.rs` and `tileset/look.rs` E with
  no ticket; `look.rs` already re-exports `shuyi_assets::tileset` (1115).
- `:192` keeps the sprite test for "this game's test card", which is
  Shuyi's (`xtask/src/test_card.rs:4`).
- `:206` says the four battle files keep game bits; `camera.rs` is a bare
  `pub use` of the kit's `Camera`.
- `:210-235` keeps the E tests "until those screens move"; 1112 and 1164
  moved them.
- `:263-264` mark `lines`, `check-script`, `test-card` and
  `voice-test-clips` E; `:268` marks the `sfx` synth E, but engine#2225
  is an integer `.sfx.ron` format and says this game doesn't need it.

## Nick input

None.

## Scope

**In:** `docs/engine/boundary.md` and the one stale bullet of ADR-0070.

**Out (do not do):** moving any code; reopening Nick's parking of
engine#2215; the rows of the audit's other tickets beyond naming them.

## Implementation steps

1. Summary: replace `:12-29`'s counts with today's production lines per
   crate from `cargo xtask code-lines --base 4b825dc642cb6eb9a060e54bf8d69288fbee4904`
   (the empty tree: the whole working tree), for the game and, with the
   `cargo xtask engine` checkout at the pin, for the engine's crates; say
   the date and the pin. Drop "roughly a third goes to the engine".
2. `:165`: one code path, `build.rs`'s tables on every target (0123);
   the engine's form of it is engine#2215, parked until MVP. Add one
   dated line under ADR-0070's bullet (`:203-205`) saying the same.
3. Give every remaining E or M row a decision, naming the audit's ticket
   that moves it. **Either order with each ticket named here:** if it is
   done, write the row as moved by it; if not, write "moves with NNNN".
   - `portrait.rs` → 1222; `tileset/look.rs` and `tileset.rs` (`:177`)
     → 1219; `title.rs` → G (the title screen's logo, backdrops and Game
     Over art: this game's files, read for their size).
   - `bundle.rs`, `build.rs` one-bit pictures → 1221.
   - `debug/sprite_test.rs` (`:192`) → 1212.
   - `map_view*` (`:203`) edge arrows and range outlines → 1213.
   - `screens/battle/{camera,cursor,path,walk}.rs` (`:206`): `camera.rs`
     goes with 1230; the cursor, camera and walks with 1217.
   - `screens/credits` (`:208`) → 1211.
   - The E-tests note (`:210-235`): the copies of the engine's screen,
     kit and camera tests go with 1232; the dialogue player's with 1218.
   - xtask: `lines`, `check-script` → G (thin over `trpg_content`'s
     rules, no engine command to move to); `test-card` → E moved (1117,
     `shuyi_cli::test_card`, the game writes its copy); `voice-test-clips`
     → G with `sfx`; `test-tileset` → 1226; `sfx` → G (engine#2225 is a
     different design); `icon` → 1229; `playtest` → 1207.
   - `lib.rs` (`:180`), "run every loader" → 1223 and 1236.
   - `ron_loader` (`:166`), `enums.rs` (`:181`) and `cinema.rs`
     (`:187`): 1230 deletes what they name; it edits those rows.
4. Re-read every row for a claim the code no longer bears out (grep the
   paths it names), and fix it or note it in the completion notes.

## Acceptance criteria

- [ ] `boundary.md` has no 1101 count left, and its summary names the
      date and the pin of its counts.
- [ ] No row is E or M without a moved-by ticket, a "moves with NNNN" or
      a reason it stays.
- [ ] `grep -n "until those screens move" docs/engine/boundary.md` finds
      nothing, and the `bundle.rs` row no longer says `include_dir!`.
- [ ] ADR-0070's bundle bullet names engine#2215.
- [ ] Only `docs/engine/boundary.md`, `docs/adr/0070-playdate-front-end.md`
      and this ticket change.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None (records only); `cargo xtask ticket-lint` and the docs' link check
  in the gates.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
