---
id: "0453"
title: Show what a spell does while choosing it in the spell list
type: feature
milestone: M3 Battle UI
model: implementation
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0453 — Show what a spell does while choosing it

## Context

Ticket 0429 put a description row under the `Skill` and `Arts` menus and
Nick made it the rule for every list where you choose what a unit does
(`docs/design/look-and-feel.md` → *Descriptions in lists*). The spell list
(`Magic` in the action menu, ticket 0410, already done) doesn't follow it
yet. 0429's *Scope* said to add the requirement to 0410, or write this
ticket if 0410 was already done.

What exists after 0429 (in `crates/ui`, apart from the menu widget):

- `MenuItem::with_note(text)` and `Menu::browsing_disabled()`: Shuyi
  Engine's menu widget since 1112, re-exported by the game's
  `src/widgets/menu.rs`. Its glyph skin (also the engine's) paints the
  focused item's note under a rule, in white, the box as wide as the
  widest note. Nothing here changes it.
- `Technique::effect_text(state, words)` in
  `src/screens/battle/art_list.rs`: `plain attack`, an art's text, or an
  active's `effect_text` (`skills.rs`).
- `arts_menu` (`art_list.rs`) and `skill_menu` (`skills.rs`) are the two
  examples to copy.

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> In Quick Battle, open the mage's `Magic` list and move through
> the lines (`Fire`, `Fire + Overcast`, a dimmed one). Can you tell what each
> does without leaving the menu?

## Scope

**In:**
- In `spell_menu` (`src/screens/battle/magic.rs`), give every line a note
  and make the menu `browsing_disabled()`.
- A line with a spell active (`Fire + Overcast`): the active's effect,
  `SpellChoice::technique().effect_text(...)` (e.g. `Spell: +1 strike`;
  check the real text in a test).
- A plain spell line: a `spell_effect_text(words, &SpellDef)` built from the
  spell's data only (`SpellKind`, `element`, `terrain_effect`, and
  `no_counter` from 0317, shown as `no counter` like Guard Break's), e.g. its
  element and what its tile cast does; no hand-written text per spell. If
  a plain line's numbers are already all on the line, the note says what
  the line doesn't (element, terrain effect, heal amount). Read
  `crates/core/src/spell.rs` and `assets/data/spells.ron` for the fields.
- New text keys in `assets/lang/en/ui.ron` and the test pack
  `assets-test/lang/test/ui.ron` (capitals), like 0429's `battle.art.desc.*`.

**Out (do not do):**
- Any change to the Skill or Arts menus.
- Rewording the spell lines themselves.
- Any rule or number change.

## Implementation steps

1. Add `spell_effect_text` in `magic.rs`, unit-tested for every spell in
   `spells.ron` (non-empty, at most 40 characters, as
   `every_art_says_what_it_does` does in `art_list.rs`).
2. In `spell_menu`, `.with_note(...)` on each item and
   `Menu::new(items).browsing_disabled()`.
3. Check every place that reads the spell list's focus
   (`grep -n "SpellMenu" crates/ui/src/screens/battle/*.rs`): with the focus
   able to stop on a dimmed line, Confirm on it must still do nothing.
4. Update the snapshots of the spell list (`magic_tests.rs`) and review
   them.

## Acceptance criteria

- [ ] Unit test: `spell_effect_text` for every spell, non-empty, ≤ 40
      characters; two exact strings spot-checked.
- [ ] Harness test: moving Down in the spell list changes the focused
      line's note (`menu.view().focused()`), onto a dimmed line too, and
      Confirm on the dimmed line leaves the list open.
- [ ] Spell list snapshots show the note row.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `spell_effect_text`.
- Snapshot / integration: the harness test and the snapshots above.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
