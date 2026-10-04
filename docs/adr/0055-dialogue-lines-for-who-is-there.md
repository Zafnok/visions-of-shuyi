# ADR-0055: Dialogue lines for who is there: `@if` blocks, resolved when the scene starts, and a check that nobody who may be gone is shown

- **Status:** Accepted
- **Date:** 2026-10-04
- **Related tickets:** 0715, 0716, 0707, 0705, 0801
- **Extends:** ADR-0005 (the `.dlg` format) and ADR-0011 (the story
  pipeline's validator)

## Context

In Classic a fallen companion is dead and leaves the roster
(`docs/design/death-and-difficulty.md`). A `.dlg` scene was one fixed list
of steps, and chapters and battles name scenes by id with no conditions,
so a companion who died a minute ago spoke in the victory scene (found by
0707). Five companions make 32 combinations: a scene per combination is
not an answer. The script has to mark **lines**.

Two things must hold. The player must play the right lines. And a writer
must not be able to put a dead character on screen by forgetting: the
validator, not a read-through, has to catch it.

## Decision

### 1. The block

```
@if sergeant
sergeant: Pay-day at last.
@else
> Nobody speaks for a while.
@endif
```

- `@if <character>` … `@endif`, with an optional `@else`. The steps
  before `@else` play if the character is **there** (section 2), the steps
  after it if not. Without `@else`, nothing plays instead.
- A block's lines are written **as far in as its `@if`**: at the first
  column in a scene, at two spaces in a reply's reaction. They are not
  indented further (an indented line still continues the line above).
- **Blocks nest**, to any depth: a line between two companions sits in a
  block for each. A block may stand **inside a reply's reaction**, and a
  `@choice` may stand inside a block; a `@choice` inside a block inside a
  reaction is still a nested choice, and refused.
- A block opened in a reaction must close in that reaction. One open at
  `@end` (or at the next option, for a reaction's) is an error on its
  `@if` line.
- `@if` takes exactly **one character id**. That leaves room for other
  conditions later (story flags, the game mode, support ranks) without
  breaking a script: a character id has no `:` and is one word, so
  `@if flag:<id>` or `@if not <character>` can be added as new forms.
  None of them is built.
- Model: `Step::If { character, then, otherwise }` (`otherwise` empty
  without an `@else`). `print_scene` writes it back; an `@else` with
  nothing after it isn't printed, so parse → print → parse holds.
- Source lines: `ParsedScene::lines` is a `Lines` tree (`steps`, and for
  each block among them its parts as `PartLines`): a choice's options, or
  an `@if` block's two parts. It replaces `step_lines` and `choice_lines`.
- Every line in both parts has a line id (ADR-0045), so `Scene::lines()`,
  `cargo xtask lines`, translations and voice clips cover them all. The
  same line in both parts is a repeat (`_2`).

### 2. Who is there

`trpg_content::Present` is `Everyone` or `Only(set of characters)`.

- **Between battles** (a chapter's intro and victory scenes, played by the
  flow): the characters of the **campaign's roster** at that moment. A
  unit that died in Classic has left it by the time the victory scenes
  play (`Campaign::apply_result`); one that retreated in Casual is still
  in it. A recruit is in it from the victory scenes of the battle it
  joined in.
- **During a battle** (a trigger's scene): the characters whose **units
  are on the map at the moment the trigger fires**, on any side. Not the
  roster: a companion left on the bench is not there, and an enemy boss
  is. `core` records it: `Event::SceneTriggered` carries `present`,
  because only the battle knows the moment. A unit about to fall is still
  there for the scenes before its `UnitFell` (its own last words, and the
  scene before the combat that fells it), and gone for every scene after;
  a reinforcement is there from its arrival on. The UI can't work this
  out from the battle's state, which is already the state after the whole
  command.
- **The debug scene viewer**, and a scene played outside a campaign:
  `Everyone`.

### 3. The player

`DialoguePlayer::new` takes the `Present` and plays
`Scene::resolved(&present)`: the scene with every block replaced by the
steps of the part that holds. Nothing else in the player changes, so
reading, skipping, the music a skip leaves, the reply choices and voice
preloading all agree on what the scene is. The lines kept keep their ids.

A scene with **nothing to say** for those there (every line was someone's
who is gone) is **not played**: the flow goes on to the next scene, the
battle screen drops it. Its `@music` lines don't apply either.

### 4. The checks

In `check_scene` (every scene, every loader):

- the character of `@if` can speak (`CharacterTable::can_speak`);
- both parts leave the **same characters on the same sides and the same
  caption**, as replies must. The scene goes on from one state, whoever is
  there. The error is on the `@else` line (the `@if` line without one);
- a reaction's limit of 4 text lines counts a block as its longer part;
- a scene has text if any part of any block has (see section 3 for a
  scene that ends up with none).

In `check_presence` (`dialogue/presence.rs`), which needs the battles, the
chapters, the New Game file and the supports, so `load_embedded` runs it
after those are loaded (skipped if any of them failed):

- **A character the army can lose appears only where they are known to be
  there.** Placing them (`@left`/`@right`) or giving them a line outside
  an `@if` block for them is an error; so is doing either in the `@else`
  of their own block.
- **Who the army can lose** (`Cast::may_be_absent`): every character of
  `new_game.ron`'s roster except the lead, and every character a battle
  can recruit (`UnitFell(recruit: true)`).
- **Scenes that can't play without someone** (`Cast::certain`) need no
  block for them. The check takes them from where the scene is played,
  so no scene has to say what triggers it:
  - a battle trigger's scene: the characters the trigger is about (who
    enters the area, the fighters of `CombatStart`, `HalfHp`'s unit, both
    of `Talk`, and **`UnitFell`'s unit: a death quote is said by the one
    dying**);
  - a support conversation: its pair;
  - before anyone can have fallen: the first chapter's intro scenes can
    count on the whole starting roster, and the scenes its battle plays at
    the start of turn 1's player phase on those of the roster the battle
    places;
  - a scene played from several places keeps only those all of them have;
    any other chapter scene, and a scene nothing plays, has nobody.
- **A block that can only go one way** is an error: an `@if` for someone
  already known to be there or gone at that line (inside their own block
  or its `@else`, in a scene that can't play without them, or the lead,
  who is in every army).

The rule is on by default for every file; there is no list of scenes
still to convert. `ch01.dlg` passes today only because its companions are
not yet in the New Game roster; 0716 puts their lines in blocks, and 0803
(which adds them to the roster) is blocked by 0716.

## Consequences

- A writer who forgets a block gets `file:line` from the all-assets test,
  in every combination of the dead, without reading 32 versions.
- **What the check does not see:** text. A line that *mentions* a dead
  companion, or narration that has them act ("she closes his eyes"), is
  the writer's to put in the block; so is whether the scene still makes
  sense without them. 0716's read-throughs cover that.
- A character who enters in a block must leave in it (or the `@else` must
  put the same character there), because both parts end on the same
  screen. A scene that a companion stays in to its end needs the rest of
  the scene in their block, or a `@right clear` before `@endif`.
- `Event::SceneTriggered` grew a field: code that matches it names
  `present` or uses `..`. Battle saves are histories of commands
  (ADR-0039), so no save changes.
- `DialoguePlayer::new`, `DialogueScreen::new` and `::overlay` take the
  scene by reference and a `Present`.
- The roster only grows by recruits today. When a chapter file can add a
  character to the army (a story join), `Cast::new` must add them to
  those the army can lose.
- Presence in a battle is "on the map", so `@if` in a trigger's scene
  can't tell "dead" from "not brought to this battle". No scene needs to
  yet.

## Alternatives considered

- **A scene per combination, picked by the chapter file** — 32 victory
  scenes for five companions, and every later chapter worse.
- **A condition on each line** (`sergeant?: Pay-day`) — no place for the
  line said *instead*, and a portrait step, a line and its reply have to
  stand or fall together.
- **Deciding the branch as the player reaches it** — the player would
  need a position inside nested blocks, and skipping, the music after a
  skip and voice preloading would each have to walk blocks the same way.
  Resolving once, at the start, makes them agree by construction. Who is
  there can't change while a scene plays.
- **The UI working out who is on the map** from the battle's state — it
  holds the state after the command, so a unit that dies in a combat
  would already be gone in the scene played before that combat.
- **Several characters in one `@if`** (`@if a b`) — nesting says the same
  and gives each its own `@else`.
- **A scene declaring who it needs** (`@needs harl`) for death quotes —
  the trigger already says it; a second statement could disagree.
- **Turning the rule on per file** — a file left off the list is exactly
  the forgotten case the check exists for.
