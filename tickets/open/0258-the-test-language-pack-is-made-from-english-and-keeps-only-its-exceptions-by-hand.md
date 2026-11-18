---
id: "0258"
title: The test language pack is made from English and keeps only its exceptions by hand
type: infra
milestone: M1 Engine
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0258 — The test language pack is made from English and keeps only its exceptions by hand

## Context

Found by five lenses of the friction sweep of 2026-11-17
([report](../../docs/research/friction-2026-11-17.md), bucket B3
"derive hand-kept copies", with 0187 and the 1237 amendment).

The `test` language pack's screen text, `assets-test/lang/test/ui.ron`,
is 1,614 lines and 833 entries against 834 keys in
`assets/lang/en/ui.ron` (949 lines). Each entry is
`(key, source: <English>, text: <English in capitals>)`, typed by hand.
Its header keeps two exceptions on purpose: `title.continue` is stale
(its `source` is "Resume", English says "Continue") and `title.credits`
is missing. A script check on 2026-11-17 found that 823 of the 833
entries are exactly their `source` in capitals with `{placeholders}`
kept as written. The other 10 are:

- 6 plural messages (`save.army`, `battle.end_turn_question`,
  `battle.objective.bonus`, `battle.rewind.charges`, `battle.rewind.use`,
  `battle.rewind.hits`) and `battle.lessons_end.text`'s `\n`, which are
  capitals too once the plural syntax (`{count, plural, one {...} other
  {...}}`, `=1`, `#`) and escapes are kept as written;
- `keys.chord_join` (`+` becomes ` PLUS `, from 1034) and
  `key_bindings.press_a_key` / `press_a_button` (`PRESS a KEY…`), which
  are hand-made.

The cost: 69 of the 70 commits that ever touched the English file also
edited the test pack (57 at today's path, 12 at the old
`assets/lang/test/`), and 40 of the last 41 did, about 400 lines a
month. A missed copy fails the build: `crates/content/src/lang.rs:225-226`
(`the_embedded_languages_load`) asserts `status.missing ==
["title.credits"]` and `status.stale == ["title.continue"]`, and
`crates/xtask/src/game_tools/tests.rs:425-437`
(`the_test_pack_has_stale_missing_and_orphaned_text`) asserts the
lang-status report ends `" missing, 2 stale, 1 orphaned"` and its
missing list starts with `title.credits`.

What governs the pack: [ADR-0045](../../docs/adr/0045-languages-text-by-key-and-line-ids.md)
(its *0233* paragraph: "a pack for tests (English in capitals, with one
stale and one missing entry kept on purpose)"; it still names the old
path `assets/lang/test/`).
[ADR-0057](../../docs/adr/0057-test-files-in-their-own-embedded-directory.md)
only says where the test files live and is not changed. The pack's
`lang.ron` already says `made_by: Machine`.

A generator half exists: `crates/ui/src/words.rs:334`, `shouting()`,
builds a capitals pack of the data's names from `content.lang.source()`
at test time, skipping tips and lessons because plain `to_uppercase()`
would break their placeholders. The engine's language tags accept
`en-x-pseudo` (`shuyi_assets` `lang/tag.rs`), and engine#2353 ("a
generated pseudo-language and a Harness walk over every screen for cut
text") plans the engine's version; it is open and not scheduled. This
ticket is the game-side structure until then.

Tickets that put lines in this pack: 0236, 0237 and 0239 (Japanese and
Chinese test lines), 0831, 1095, 1170 and 1185 (new keys). Either
order with each: if this ticket is done first, they add only their
exceptions (the CJK lines) to the override file and nothing for a plain
new key; if they are done first, their entries become overrides here
or are deleted as generated.

## Nick input

None. Tests only; no player ever sees the test pack.

## Scope

**In:**
- One capitals function in `trpg_content::lang` that keeps
  `{placeholders}`, plural and select syntax (argument names, `plural`,
  `select`, selectors such as `one`, `other`, `=1`, and `#`) and
  escapes as written, and capitalizes only the message text, nested
  branches included.
- The test pack's `ui.ron` made at load from English's `ui.ron` with
  that function, wherever the test files are embedded (ADR-0057), so
  the game, the tests, `cargo shuyi check` and `cargo shuyi
  lang-status` all read the same pack.
- An override file of the exceptions only: the stale `title.continue`
  entry, the `title.credits` key left missing, and any hand-made entry a
  test reads.
- `shouting()` built on the same function.
- ADR-0045 amended.
- A row in `docs/engine/friction.md` (game-side structure, follow-up
  engine#2353).

**Out (do not do):**
- `assets-test/lang/test/data.ron` and `dialogue/`: they translate only a
  few names and lines on purpose (0235), and the tests check that.
- The engine's pseudo-language (engine#2353), text stretching or
  accents, and any change to the engine's pack format or checks.
- English text, the real language packs, or what a player sees.
- The CJK test lines themselves (0236, 0237, 0239).

## Implementation steps

Expected size: about 80-120 production lines (the capitals function and
the load hook), about 100 test lines, and about 1,580 lines of
`ui.ron` deleted. `cargo xtask pr-size` counts the deletions, so plan
about 1,900 changed lines; nothing else goes in this PR.

1. **Capitals.** In `crates/content/src/lang.rs` (or a `lang/capitals.rs`
   beside it), add `pub fn capitals(text: &str) -> String`. Walk the text
   once: outside braces, `to_uppercase()`; a `{name}` placeholder stays
   as written; in `{arg, plural, ...}` and `{arg, select, ...}` the
   argument, the keyword, each selector and `#` stay as written, and
   each branch's `{...}` body is capitalized recursively (it may hold
   placeholders such as `{from}`). Escapes are already decoded by RON,
   so `\n` is a newline here. Reuse the engine's message parser if
   `shuyi_assets::lang` exposes one; otherwise write the small walk and
   note it in the friction row.
2. **Where the overrides live.** Keep `assets-test/lang/test/ui.ron` in
   today's format, holding only exception entries. The keys left
   missing on purpose need a list: first check what the engine's pack
   loader, `check` and `lang-status` accept beside a pack (ADR-0045
   says any other file in a language's directory is an error). If a file
   such as `assets-test/lang/test.generate.ron` with
   `(missing: ["title.credits"])` is accepted, use it; otherwise pick
   the nearest accepted place and record it in the ADR amendment.
3. **Load.** Where `trpg_content::lang::load` builds the `test` pack's
   `PackSources`, build its `ui` from `english_ui()`: for every English
   key not in the override file and not in the missing list, an entry
   `(key, source: english, text: capitals(english))`, then the override
   entries as written. Only builds that embed the test files do this;
   a shipping build has no test pack (ADR-0057). Keep the cost to one
   pass at load.
4. **Delete** every entry of `ui.ron` that step 3 makes the same. Keep
   `keys.chord_join` and `key_bindings.press_a_*` only if a test reads
   their hand-made text (`grep -rn "PLUS\|PRESS a" crates`); otherwise
   let them be generated and say so in the completion notes. Rewrite the
   file's header comment to say what it holds now.
5. **Tests that read the file.** `the_embedded_languages_load`
   (`lang.rs:217`) and `the_test_pack_has_stale_missing_and_orphaned_text`
   (`game_tools/tests.rs:408`) keep their asserts; add a comment that
   the missing and stale entries are the override file's. Check
   `lang/data_tests.rs:1055-1060`, which asserts the same two keys.
   `check_finds_a_character_the_atlas_cant_draw`
   (`game_tools/tests.rs:1133`) edits `text: "NEW GAME"` in the file:
   make it add an override entry with the snowman instead.
6. **`shouting()`** (`crates/ui/src/words.rs:334`): use
   `trpg_content::lang::capitals` and drop its tips and lessons filter
   if their placeholders now survive (the tests that use it, in
   `item_tests.rs:613`, `magic_tests.rs:1807` and `mode.rs:2698`, must
   pass unchanged).
7. **ADR-0045:** amend the *0233* paragraph with the `write-adr` skill:
   the test pack's screen text is made from English in capitals at load,
   an override file keeps the deliberate exceptions, its path is
   `assets-test/lang/test/`, and engine#2353 is the later replacement.
8. **Friction row** in `docs/engine/friction.md` for this game-side
   structure, follow-up engine#2353, and measure with `cargo xtask
   code-lines` and `cargo xtask pr-size`.

## Acceptance criteria

- [ ] `assets-test/lang/test/ui.ron` holds only exception entries (under
      about 30 lines); `grep -c "key:"` on it is under 10.
- [ ] A unit test adds a key to a copy of English's text and gets its
      capitals entry with no test-pack edit.
- [ ] `capitals` has unit tests for a placeholder, each of the 6 plural
      messages above, a nested placeholder in a plural branch, `#`, and
      a newline.
- [ ] `the_embedded_languages_load` and
      `the_test_pack_has_stale_missing_and_orphaned_text` pass with the
      same asserts.
- [ ] `cargo shuyi lang-status test` reports the same missing, stale
      and orphaned lists as before the change.
- [ ] ADR-0045 is amended and `docs/engine/friction.md` has the row.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `capitals` (the cases above); the generated pack against a
  small English map with an override and a missing key.
- Integration: the existing language tests (`crates/ui/tests/it/language.rs`,
  `lang.rs`, `game_tools/tests.rs`) unchanged in what they check.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
