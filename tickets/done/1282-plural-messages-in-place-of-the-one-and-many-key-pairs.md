---
id: "1282"
title: "Plural messages in place of the one-and-many key pairs"
type: infra
milestone: M1 Engine
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-17
---

# 1282 — Plural messages in place of the one-and-many key pairs

## Context

The engine pin move of 2026-11-17 passed engine PRs #471 (engine ticket
2348) and #476 (3218): screen text is now a subset of ICU MessageFormat
(`{n, plural, one {# unit} other {# units}}`, `select`, `=N`), filled
by the plural rules of the language the text is written in
(`Lang::text_in`, ADR-0127 of the engine). A translated pack can then
say a count its own way (Russian's `few` and `many`), which two English
keys picked by `n == 1` never can.

The game picks between keys by hand today:

- `save.army.one` / `save.army.many` (`crates/ui/src/screens/save.rs`);
- `battle.end_turn_question.one` / `.many`
  (`crates/ui/src/screens/battle/map_menu.rs`);
- `battle.rewind.charges.one` / `.many` and `battle.rewind.use.one` /
  `.many` (`crates/ui/src/screens/battle/rewind.rs`);
- `battle.objective.bonus_one` / `_two` / `_many`
  (`crates/ui/src/screens/battle/objective.rs`: a count of rewards on
  offer, `=1`, `=2` and `other`).

## Nick input

None: the English each screen shows stays word for word.

## Scope

**In:**
- Each pair (or triple) above becomes one key whose English is a plural
  message, chosen by the count the code already passes; the `if n == 1`
  branches go. The translated packs' entries (the `test` pack in
  `assets-test/`, any other in `assets/lang/` or the content) follow.
- Search `crates/ui/src` for any other key chosen by a count (`== 1`
  beside a text key) and fold it in the same way.
- The screens' tests and snapshots show the same English as before.

**Out (do not do):**
- New wording (dialogue writing is paused; screen text keeps its words).
- Ordinals or `select` where the game has no text that needs them.

Estimate: about 40 production lines removed, 10 changed; 11 keys of
`assets/lang/en/ui.ron` become 5.

## Implementation steps

1. Read `shuyi_assets::message` and `Lang::text_in` at the pin.
2. Merge the keys in `assets/lang/en/ui.ron` and the packs; drop the
   branches; keep the values the code passes.
3. `run-gates` (`check-text`, the lang tests, the snapshots).

## Acceptance criteria

- No screen chooses a text key by a count.
- Every screen above shows the same English for 0, 1, 2 and 5 as before
  (a test per message).

## Tests required

A test per merged message for 0, 1, 2 and 5 against today's English.

## Completion notes

Each key group became one plural message (ADR-0129 of the engine), and
the code passes the count it already had; no screen picks a key by a
count any more:

- `save.army` (`{count, plural, one … other …}`);
- `battle.end_turn_question` (by `ready`);
- `battle.rewind.charges` (by `charges`, as before: `1 of 2 charges
  left`) and `battle.rewind.use` (by `left`);
- `battle.objective.bonus` (`=1`, `=2`, `other`), which now also gets
  the count of rewards on offer as `rewards`; no rewards still shows no
  bonus line, before the message;
- found by the search: `battle.rewind.missed` / `.hit` / `.hits`, the
  rewind list's combat summary chosen by the count of hits, folded into
  `battle.rewind.hits` (`=0 {missed}`, `one`, `other`).

14 English keys became 6. The `test` pack's entries follow (the plural
keywords stay lower case, the words in capitals). There are no other
packs in `assets/lang/` or the content. The class-change hint shown when
there is more than one class, and the shops' `×N`, aren't count-chosen
keys and stay.

A test per message checks 0, 1, 2 and 5 against the old English
(`the_*_reads_as_before_for_each_count`); the snapshots didn't change.

No gameplay rules decided.
