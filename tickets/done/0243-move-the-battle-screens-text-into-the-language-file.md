---
id: "0243"
title: Move the battle screen's and the class change screen's text into the language file
type: feature
milestone: M1 Engine
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-10-11
---

# 0243 — Move the battle screen's and the class change screen's text into the language file

## Context

Second half of 0234, split by that ticket's own rule (it was over 600
changed lines). 0234 moved the text of every screen outside the battle
into `assets/lang/en/ui.ron`
([ADR-0045](../../docs/adr/0045-languages-text-by-key-and-line-ids.md)):
title, layout picker, mode select, lead select, save, results' tally,
game over, credits, key bindings, the dialogue's skip question. What is
left is `crates/ui/src/screens/battle/`, `screens/class_change.rs` (it
shares its stat rows and result pages with the battle) and the words
inside key names. `cargo xtask check-text` lists 123 literals there.

`check-text` sees only part of it. It counts a literal passed straight to
a `print…`/`draw…`/`help_line`/`MenuItem` call and `&str` constants that
look like prose. It does not see text built first and printed later, and
the battle screen has about as much of that again:

- `format!` results kept in a variable or returned from a function:
  `art_list.rs` (`"{left} uses left"`, `"out of range"`, `"no counter"`,
  `"stance: …"`), `map_menu.rs` (`"Rout the enemy"`, `"Defeat {name}"`,
  `"Turn {n}/{limit}"`, `"End turn with {ready} units ready?"`),
  `banner.rs` (`PLAYER PHASE`, `VICTORY`, `Turn {turn}`), `forecast.rs`
  (`"(resist)"`, `"heals {n}"`, `"{n} dmg"`), `items.rs`, `magic.rs`,
  `attack.rs`, `info.rs`, `skills.rs`, `rewind.rs`, `progress.rs`,
  `class_change.rs` (`"Promote {unit} to {class}?"`, `"Tier {n}"`).
- Help-bar labels kept in a variable before `help_line`: `"info"`,
  `"next unit"`, `"move"`, `"end turn"`, `"hide range"`, `"attack"`.
- `match` arms that return a word: `"acted"` / `"ready"`, `"Mounted"`.

## Nick input

None. Nothing a player sees changes.

## Scope

**In:**
- Every player-facing literal in `crates/ui/src/screens/battle/*.rs` and
  `crates/ui/src/screens/class_change.rs`, whether `check-text` lists it
  or not.
- The words inside key names: `NOT_MAPPED` (`! not mapped`) in
  `widgets/help.rs`, `arrows` in `Keymap::cursor_keys_name`
  (`input.rs`), `hold {key}` in the battle help.
- `playing_help` in `battle/progress.rs` (the results screen uses it).
- `check-text` taught to see the kinds of literal listed in Context, and
  its limit set to 0.

**Out (do not do):**
- The debug menu and tools (English only).
- Names and descriptions from data files, tips, dialogue (0235).
- Rewording anything; layout changes. A layout that only fits the English
  word's length: leave it and add a line to 0237's notes.
- Turning screens into a view and a skin (0240, 0241, 0242, 0245). *Either
  order works:* if one of those has converted a file, its text lookups
  are in the logic that builds the view; if not, they are where the
  screen draws.
- The battle help bar's toggles line (0445). *Either order works:* use
  whatever is there when this is built.
- Controller button names and the pad's `D-pad` / `L-stick` (0234 marked
  them `// check-text: not player text`).

## Implementation steps

1. Read how 0234 did it: `screens/save.rs`, `screens/key_bindings.rs`,
   `screens/lead_select.rs` and the new-screen rule in
   `crates/ui/README.md` (*Text the player reads is never a string
   literal*).
2. Whole help lines are one key each where the line is fixed
   (`"battle.help.menu": "{Cursor} choose · {Confirm} confirm · {Cancel}
   back"`). `{Select}` already names the key that picks on the map.
3. A help line whose hints come and go (`help_idle`: `rewind` only when
   it can open; `help_targeting`) gets one key per hint, the key name
   inside it (`"battle.hint.rewind": "{Rewind} rewind"`), joined with
   `widgets::help::SEPARATOR`. A hint for keys that are not one action
   (`←/→ swap`, the two end-turn keys) takes the key names as a value:
   `"battle.hint.swap": "{keys} swap"`.
4. Text made of pieces becomes one key with values: `"Lv {level}"`,
   `"Turn {turn}/{limit}"`. Where English picks a word by count
   (`unit` / `units`), use two keys (`….one`, `….many`), as
   `save.army.one` does.
5. Functions that return text without a `Ctx` (`map_menu::objective_text`,
   `art_list` notes, `progress::stat_row`, `info`'s stat names,
   `class_change::stat_line`) take `&Ctx`, or return a key, whichever
   keeps their callers simplest. Update their tests to compare the text
   the player sees (`ctx.text(KEY)` or the English words), never the key
   by itself: a `!shows(buf, KEY)` assertion would pass for the wrong
   reason.
6. Key-name words: `HelpKeys` is built by `Ctx::help_keys()`; give it the
   two words (`not mapped`, `arrows`) from `ctx.text`, so `key_name` and
   `cursor_keys_name` return them in the player's language. Tests that
   build `HelpKeys::new(&keymap, device)` directly keep working with the
   English words as that constructor's default, marked
   `// check-text: not player text` with the reason.
7. Extend `crates/xtask/src/check_text.rs` so the literals in Context
   count (at least: a `format!` whose text has a word and a space or a
   capital, outside `text(`/`text_with(`/assert/panic/`expect` calls and
   outside `#[cfg(test)]`), with tests in that file. What it then lists
   that is not player text gets the marker comment.
8. Set `MAX_LITERALS` to 0 and fix its doc comment.

## Acceptance criteria

- [x] `cargo xtask check-text` reports 0 and its limit is 0.
- [x] `check-text` has a test for each new kind of literal it counts.
- [x] No snapshot changes in English. Three snapshots *in the test language* changed, as they must: the battle's words are now the pack's (see the notes).
- [x] `cargo xtask lang-status test` still works.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `check_text.rs` tests for the new rules.
- Snapshot / integration: existing snapshots unchanged is the test.

## Completion notes

**Done.** Every word the battle screen and the class change screen show
is now in `assets/lang/en/ui.ron` (about 290 new keys: `battle.*`,
`class_change.*`, `stat.*`, `class.tag.*`, `keys.*`). Nothing a player
sees in English changed: every English snapshot is byte for byte the same.

How it was built:

- **Text with no `Ctx`.** Most of the battle's text is made by functions
  that only had `Words` (a list's lines, a forecast's notes, the rewind
  log). `Words` gained `text(key)` and `text_with(key, values)`, so those
  functions take `Words` and their tests still compare the English the
  player reads. With no pack in use `Words` reads English from the
  bundle's own `en/ui.ron` (`trpg_content::lang::english_ui`).
- **Menus.** The action menu and the map menu are built in English while
  playing, like the other lists, and `Mode::told` paints them in the
  player's language. The reasons after a dimmed line (`out of range`,
  `no uses left`) are retold the same way.
- **Help bar.** One key per fixed line (`battle.help.*`), one key per hint
  where hints come and go (`battle.hint.*`), as the ticket asked.
- **Key-name words.** `! not mapped` and `arrows` are `keys.not_mapped`
  and `keys.arrows`; `Ctx::help_keys()` hands them to `HelpKeys`.
  `HelpKeys::new` keeps the English words for tests.
- **`check-text`** now also counts: any literal with a capital or a space
  wherever it stands (so a `format!` kept in a variable or returned), and
  a lowercase word that is the value of a `let`, a `match` arm or an `if`,
  or sits in a tuple or an array. Assertions, panics, `expect` and
  attributes are never counted. Its limit is 0. About 25 places it took
  for text that aren't (log warnings, the scene's text form for tests,
  palette names, controller button names) carry
  `// check-text: not player text (why)`.

Deviations:

- **Three test-language snapshots changed** (`the_title_in_the_test_language`,
  `a_tip_in_the_test_language`, `the_info_screen_in_the_test_language`).
  The test pack must hold every key but one (a test checks it), so it got
  the new keys in capitals, and those three pictures show them. Three
  older assertions that said "this stays English" were turned round.
- Two literals outside the ticket's files were moved too, because the
  stricter `check-text` found them: the weapon and armour lines of the
  Preparations stock (`prep.weapon_detail`, `prep.armour_detail`).
- Functions take `Words`, not `&Ctx` or a key (step 5 offered those two):
  it was already there in most of them.

Still English whatever the language, on purpose or out of scope:

- Movement type names (`foot`) on the info and class change screens: they
  come from the terrain data and have no language key (0235's kind of
  text). Not ticketed: say if it should be.
- Weapon rank letters (`E`, `D`…), the debug hint (`F2 debug`), a storage
  error's own words inside `Save {error}`, and the three class change
  refusals that can't happen from the screen (`core`'s own error text).

Layouts sized for the English words, left as they are, for 0237:

- The forecast: `HP` (2 cells), `Hit` (3), `Crit` (4), `Total` (5),
  `no counter` (11), a strike's `8 dmg` (7), and the two columns' width.
- The info screen: stat names in a 4-cell column, the `E`/`A`/`P` marks in
  one cell, the HP bar at a fixed column after `HP 999/999`, every line
  cut to its column.
- List lines whose numbers are glued to their labels (`Mt 5  Hit 90`,
  `Mt5 Hit90 Rng1`): the name is padded, the rest is as long as its words.
- The help bar and the toggles' line share one row.
- The class change screen: a 20-cell column, `Tier 2` padded to 9, and the
  highlight of a gained stat is found by its `+`.
- Labels are centred by their length in bytes in two places (`portrait`).

No follow-up tickets. Nothing for Nick to decide: no rule or wording was
invented.
