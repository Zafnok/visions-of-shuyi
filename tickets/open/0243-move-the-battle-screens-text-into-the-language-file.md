---
id: "0243"
title: Move the battle screen's and the class change screen's text into the language file
type: feature
milestone: M1 Engine
model: opus-5.5
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
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
- Turning screens into a view and a skin (0240, 0241, 0242). *Either
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

- [ ] `cargo xtask check-text` reports 0 and its limit is 0.
- [ ] `check-text` has a test for each new kind of literal it counts.
- [ ] No snapshot changes.
- [ ] `cargo xtask lang-status test` still works.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `check_text.rs` tests for the new rules.
- Snapshot / integration: existing snapshots unchanged is the test.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
