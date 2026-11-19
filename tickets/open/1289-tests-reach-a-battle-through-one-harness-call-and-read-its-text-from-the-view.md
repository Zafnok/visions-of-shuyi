---
id: "1289"
title: Tests reach a battle through one Harness call and read its text from the view
type: infra
milestone: Later
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["1237"]
nick_input: none
completed:
---

# 1289 — Tests reach a battle through one Harness call and read its text from the view

## Context

From the [retro of PRs #616-#735](../../docs/research/retro-2026-11-17-prs-616-735.md)
(T19). The UI tests reach the Quick Battle by typing its key script, and
they count the presses. When #726 added one Confirm press at the start
of a battle, it touched 80 files, 62 of them tests. On main (58176da0),
`"Down f Left f"` still appears 36 times in 20 files of `crates/ui`,
and a `fn quick_battle…` is defined in 15 files. Behavior tests also read the
drawn frame through local `shows`/`row` helpers (25 and 23 copies), so a
narrower panel breaks tests that never meant to check the look, against
AGENTS.md rule 8.

`BattleView::shows` (`crates/ui/src/screens/battle/view.rs:377`) already
reads the view. But it is `#[cfg(test)]` (`view.rs:304`), so the
integration tests in `crates/ui/tests/it` can't call it.

Two open tickets cover nearby work. **1237** adds `tests/it/support.rs`
with one `quick_battle()`, still a key literal, and moves the banner and
help asserts in `battle.rs` onto the view. **1214** swaps the cell
helpers for the engine's frame readers (engine#3204). This ticket does
the part neither covers. The press count lives in one Harness method
that reads the battle's state. And text that the battle view carries
gets read from the view.

## Nick input

None. Tests only; nothing a player sees changes.

## Scope

**In:**
- `Harness::to_quick_battle`, `Harness::dismiss_battle_start`,
  `Harness::quick_battle`, `Harness::battle_view` and
  `Harness::battle_shows`, plus `BattleScreen::takes_map_input`.
- `BattleView::texts`/`shows` (and the sub-view `texts`) built under the
  `harness` feature as well as `test`.
- Every `"Down f Left f…"` script in `crates/ui/src` and
  `crates/ui/tests` goes through the new methods.
- In `battle.rs`, `language.rs`, `controller.rs`, `split_keys.rs` and
  `threat.rs` under `tests/it`: asserts on text the battle view carries
  move from the cell `shows` to `h.battle_shows`.
- Expected size: about 70 production lines (Harness about 55, battle
  screen about 10, `cfg` lines). About 400-800 test lines changed,
  mostly removed.

**Out (do not do):**
- Frame text helpers whose asserts stay on cells. 1214 replaces them
  with the engine's readers.
- Look tests and snapshots. They keep reading cells (1290 crops them).
- The pad path (`tests/it/playdate.rs`'s `quick_battle_on_the_playdate`)
  and story battles (`skip_scenes`, the `chNN_*` tests). A
  `new_game_to_first_battle` helper waits until a third copy shows up.
- `crates/tools/src/game_tools/tests.rs` and the `crates/ui/README.md`
  example. They test `frame-png --keys` scripts as text.
- A lint against new local helpers.
- Speeding the tests up (0128).

**Either order with 1214:** if 1214 is done, the cell helpers are
already `h.shows(..)`; move the battle-view asserts from that call. If
not, delete a local `shows` once none of its calls are left. 1214 then
converts what remains.

## Implementation steps

1. **Battle state.** Near `notes_open` (`screens/battle/mod.rs:807`),
   add `pub fn takes_map_input(&self) -> bool`. It is true when
   `self.mode` is `Mode::Idle { .. }`, `self.queue` is empty and no
   tip or lesson box is up. Check `banner()` (`:739`) and
   `notes_open()` for the rules they already use. Either order with
   1291: if it is done, ask its `Guide` for the boxes.
2. **View text outside unit tests.** Change `#[cfg(test)]` to
   `#[cfg(any(test, feature = "harness"))]` on `impl BattleView`
   (`view.rs:304`) and on `note_texts` (`:383`). Do the same for the
   `texts` blocks in `view/{forecast,guides,info,progress,rewind,shop}.rs`
   and for `widgets::menu::english` (`widgets/menu.rs:19`).
3. **Harness** (`crates/ui/src/harness.rs`, after `path()`, `:341`). Add:
   - `to_quick_battle(&mut self) -> &mut Self`: asserts the title is on
     top, presses `Down f` (Quick Battle), asserts
     `["title", "game_flow", "preparations"]`, then presses `Left f`
     (`Fight!`) and asserts `["title", "game_flow", "battle"]`. It uses
     the right-handed keys that every caller has today; say so in its
     doc.
   - `dismiss_battle_start(&mut self) -> &mut Self`: before each press,
     returns if the top screen is `battle` and `takes_map_input()`.
     Otherwise it presses Confirm (`f`). After 8 presses it panics with
     `self.snapshot()`. It never counts panels, so a new start panel
     changes no caller.
   - `quick_battle() -> Self`: `with_layout(RIGHT_HANDED)`, then the
     two calls above.
   - `battle_view(&self) -> Option<BattleView>` (as `tests/it/playdate.rs:471`
     builds it) and `battle_shows(&self, text) -> bool`, which is false
     with no battle (`#[cfg(feature = "harness")]` or `test`).
4. **Callers.** Replace every script from
   `grep -rn "Down f Left f" crates/ui/src crates/ui/tests`. A helper
   that sets something up first keeps its setup and then calls the
   methods: `debug_view.rs`, `map_zoom.rs`'s skin, `split_keys.rs`'s
   rebinding, the setup in `src/map_view/playdate/tests.rs:485`, and
   `battle.rs`'s `quick_battle_objective`, which stops after
   `to_quick_battle`. Delete each `quick_battle` that becomes
   `Harness::quick_battle()`, including 1237's in `support.rs`.
   Harness's own tests (`harness.rs:538-590`) use the methods too.
   `title_cinematic.rs:538` keeps its `z` (skip) and then calls the
   methods. `profiles.rs:447` is a profile menu, not the Quick Battle:
   leave it. `map_skin.rs`'s timed `SCRIPT` (line 36) may keep its keys
   if a timed step table can't call the methods; say why in a comment.
5. **Text from the view.** In the five files in **In**, turn
   `shows(&h, x)` into `h.battle_shows(x)` when the battle view carries
   `x` (banners, objective, tips, lessons, menus, forecast, info,
   message, help). Keep a cell read only when the test checks how the
   text is drawn, and add a comment saying so. Menu lines in `texts()`
   are English: a `language.rs` assert on a translated menu label reads
   `battle_view()`'s menu labels directly.
6. **Measure** with `cargo xtask code-lines` and `cargo xtask pr-size`.

## Acceptance criteria

- [ ] `grep -rn "Down f Left f" crates/ui/src crates/ui/tests` finds
      only `to_quick_battle` in `harness.rs`, `profiles.rs`'s profile
      menu, and any timed step table kept with a comment (step 4).
- [ ] `grep -rnE "^fn quick_battle\(" crates/ui/tests` finds nothing.
- [ ] A Harness unit test shows that `dismiss_battle_start` reaches
      `Mode::Idle` with the cursor on the lord, both from the objective
      page and from the `PLAYER PHASE` banner. Another shows that it
      panics with no battle on the stack.
- [ ] `tests/it/battle.rs` and `language.rs` define no local `shows`, or
      every remaining call has a comment saying why it reads cells.
- [ ] The same tests pass and none is removed
      (`cargo test --no-fail-fast`).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `dismiss_battle_start` and `takes_map_input` in `harness.rs`'s
  tests.
- Integration: the existing `tests/it` tests, unchanged in what they
  check.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
