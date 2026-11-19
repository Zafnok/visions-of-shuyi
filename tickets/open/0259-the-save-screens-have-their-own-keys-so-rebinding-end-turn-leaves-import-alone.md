---
id: "0259"
title: "The save screens have their own keys, so rebinding End turn leaves Import alone"
type: feature
milestone: M7 Chapter 1 & game flow
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["0184"]
nick_input: decision
completed:
---

# 0259 — The save screens have their own keys, so rebinding End turn leaves Import alone

## Context

From the friction sweep of 2026-11-17
([report](../../docs/research/friction-2026-11-17.md), *A new input
action edits 6 parallel tables, so screens borrow unrelated actions*;
bucket B7, code structure, with 0335, and the amendments to 0334, 0257,
1291 and 1236).

The Profile screen and the Load Game list borrow three battle actions
for their file keys (`crates/ui/src/screens/files.rs:17`–`27`, ticket
0873): Unit info exports, the world map's Find opens the saves' folder,
and `pub const IMPORT: Action = Action::EndTurn` imports. They were
picked because those keys don't collide on any keymap (*the agent's
starting rules*, `docs/design/options.md`, *Saves outside the game*,
~:358 and ~:367). But a binding belongs to the action, so a player who
moves End turn to another key also moves Import, and a player who
unbinds Find on the world map loses the folder key on the Load Game
list. Their help lines say `{Info} export`, `{EndTurn} import` and
`{WorldFind} open folder` (`assets/lang/en/ui.ron` ~:58–60, ~:177–181).

ADR-0068's key groups let actions of different groups share a key, as
the world map's own shortcuts do (`KeyGroup::WorldMap`,
`crates/content/src/keymap.rs` ~:130; Nick, 1007: "different contexts
can reuse buttons. It just can't reuse in same context"). A `Files`
group gives the save screens their own three actions on the same
default keys as today.

Adding an action is about 45 to 120 production lines today (`ALL` and
`NAMES` `[_; 24]`, a match arm per property, `KEY_ROWS`, `keymap.ron`).
An action-table macro or derive in the engine would shrink that; it is
an engine ergonomics change, proposed to Nick separately and not
needed here.

## Nick input

**Decision, asked in this ticket's session before building**
(`ask-nick`), in the order 0184's queue sets; blocked on 0184 only so
it isn't asked ahead of that order. 0184 itself doesn't answer it.
Default keys and whether an
action is required or optional are Nick's (`keyboard-input` skill,
*This game's actions*). The defaults here reuse the keys the save
screens already answer to (Unit info's, Find's and End turn's), so no
player-facing default moves. What is still his, asked once in plain
words with the Key bindings screen's mockup: should the three save-file
keys (export, open the folder, import) be listed on the Key bindings
screen as optional keys in a group of their own, bound by default to
the keys they use today, like the world map's shortcuts? Or something
else (his own words). Nothing is built before his answer, which is
recorded in `docs/design/controls.md` and `options.md`.

Nothing is checked after merge: tests and a `cargo shuyi frame-png`
picture of the Key bindings screen in the PR; anything left to judge by
playing goes on the Act 1 playtest's checklist (1064).

## Scope

**In:**
- Three actions in `crates/content/src/keymap.rs` (names are the
  implementer's; e.g. `ExportSave`, `OpenSavesFolder`, `ImportSave`),
  each with a doc comment, in `Action::ALL` and `NAMES`, in a new
  `KeyGroup::Files` (`KeyGroup::ALL` gains it), `is_required` as Nick
  answered.
- `assets/data/keymap.ron`: in both layouts the same chords as Info,
  WorldFind and EndTurn have today (`e`/`i`, `x`/`.`, `Space`); in `pad`
  the same buttons (North, West, Start); in `playdate` none (the
  Playdate has no export or import, `options.md` ~:357).
- `crates/ui/src/input.rs`: their `KEY_ROWS` rows and labels in the
  Files group, and the three in `NOT_ON_THE_PLAYDATE`.
- `crates/ui/src/screens/files.rs`: `ACTIONS` and `IMPORT` name the new
  actions; `screens/save.rs` (~:582), `screens/profile.rs` (~:184–186)
  and the declared actions in `screens/actions.rs` (~:217) match them.
- The help lines and any tip `how` that names these keys use the new
  placeholders: `assets/lang/en/ui.ron`, the test pack
  (`assets-test/lang/test/ui.ron`, unless 0258 has made it derived),
  and `assets/data/tips.ron`. If the private repository's
  `content/` names them too (`grep -rn "{EndTurn} import\|{Info} export\|{WorldFind} open" assets-private/content`),
  change it there with the `content-change` skill.
- `docs/design/controls.md` (the new group's table, as Nick answered)
  and the starting rules in `options.md` rewritten to name the new
  keys.

**Out (do not do):**
- Any other default key, or moving these off today's keys: Nick's.
- The engine's action-table macro or derive (an engine ticket only once
  Nick says yes, `engine-change`).
- Key caps on the save screens' help lines: 0257. Either order: whichever
  lands second uses the other's placeholders and styled text.
- The battle's actions and the world map's group.

## Implementation steps

1. Read the `keyboard-input` skill (*Adding an action*), ADR-0068, and
   `files.rs`, `save.rs`, `profile.rs` and their tests.
2. Add the group and the three actions (`keymap.rs`), their defaults
   (`keymap.ron`), their rows (`input.rs`), per Nick's answer.
3. Switch `files.rs`, `save.rs` and `profile.rs`; then the text files.
4. Update `controls.md` and `options.md`.
5. Tests (below), snapshots, a `frame-png` of the Key bindings screen.
   **Expected size:** about 120 production lines, about 150 test lines,
   docs about 40, this file about 300: about 650 changed lines, plus
   snapshots.

## Acceptance criteria

- [ ] `grep -n "Action::EndTurn\|Action::Info\|Action::WorldFind" crates/ui/src/screens/files.rs`
      finds nothing.
- [ ] With End turn rebound to another key, the Load Game list still
      imports on Space and the battle ends the turn on the new key
      (Harness test).
- [ ] With Find unbound on the world map, the Load Game list still opens
      the folder on its default key (Harness test).
- [ ] The Key bindings screen lists the three in the Files group, and
      the Playdate's doesn't (tests).
- [ ] The keymap-matches-design test passes with `controls.md`'s new
      table; `cargo shuyi check-keys` passes.
- [ ] The save screens' help lines read the same words as before with
      the default keys (tests read `plain()`).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the new actions' group, `is_required` and names; `KEY_ROWS`
  lists each rebindable action once.
- Snapshot / integration: the two rebinding tests above; the Key
  bindings screen with the Files group; the save and profile screens'
  help lines.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
