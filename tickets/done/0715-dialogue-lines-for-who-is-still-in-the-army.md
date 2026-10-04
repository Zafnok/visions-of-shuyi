---
id: "0715"
title: Dialogue lines that depend on who is still in the army
type: feature
milestone: M6 Story & dialogue
model: opus-5.5
effort: high
status: done
blocked_by: []
nick_input: none
completed: 2026-10-04
---

# 0715 — Dialogue lines that depend on who is still in the army

## Context

In Classic a fallen companion dies and leaves the roster for good
(`docs/design/death-and-difficulty.md`, "Falling units"). The story already
assumes that: Hollis's sheet asks for another version of his Chapter 8 scene
if he died earlier (`docs/story/characters/retainer.md`, "Classic
permadeath"), and the outline unlocks side quests "with Tamsin recruited and
alive" (`docs/story/outline.md`).

A `.dlg` scene can't do it. A scene is one fixed list of steps
(`assets/dialogue/README.md`), and a chapter names its scenes by id with no
conditions (`assets/chapters/README.md`). Found by 0707: the Chapter 1
victory scene (`ch01_victory` in `assets/dialogue/ch01.dlg`) has all six of
the party speaking, so a companion who died in the battle a minute ago
talks in it. Five companions can each be dead or alive, so a scene per
combination (32) is not an answer; the script needs to mark **lines**.

This ticket adds that to the format, the check and the player. 0716 then
uses it in the Chapter 1 scenes.

## Nick input

None. What a scene says when someone is dead is story writing (0716).

## Scope

**In:**
- A block in `.dlg` scenes whose steps play only if a named character is
  **present**, with an optional other branch for when they aren't.
- "Present" for a scene between battles (the chapter's intro and victory
  scenes): the character is in the campaign's roster. A unit that died in
  Classic has left it by the time the victory scenes play
  (`Campaign::apply_result`); a unit that retreated in Casual is still in
  it.
- "Present" for a scene during a battle (a trigger's scene): the character
  is a unit still on the map, on any side.
- The check, so a script can't put a dead character on screen by mistake
  (step 4).
- An ADR (`write-adr` skill): the block's syntax, what "present" means in
  each place, and what the check guarantees.
- `assets/dialogue/README.md`: the new lines, the rules and an example.
- The debug scene viewer (`crates/ui/src/debug.rs`) plays a scene with
  everyone present.

**Out (do not do):**
- Any Chapter 1 line (0716).
- Conditions on anything but presence: story flags (`Campaign::flags`),
  game mode, support ranks, the reply the player picked earlier. Leave room
  for them in the syntax; don't build them.
- Choosing between whole scenes in a chapter or battle file.
- Changing who leaves the roster or when (`Campaign::apply_result`).

## Implementation steps

1. **Decide the syntax and write the ADR.** A starting point, to change if
   something better turns up:
   ```
   @if sergeant
   sergeant: Pay-day at last.
   @else
   > Nobody speaks for a while.
   @endif
   ```
   Directives at the first column, like `@choice`. `@else` is optional.
   Decide and record: whether a block may sit inside a reply's reaction
   (0716 needs it: one of `ch01_victory`'s replies is answered by Tamsin)
   and whether blocks nest (a line between two companions needs both).
2. **Model** (`crates/content/src/dialogue.rs`): a new `Step` variant
   holding the character and the two step lists. `Step::text`, `has_text`
   (`dialogue/check.rs`) and `print_scene` cover it; the round-trip
   property test (parse → print → parse) must still hold.
3. **Parser** (`crates/content/src/dialogue/parse.rs`): follow how
   `@choice` … `@endchoice` is parsed (`OpenChoice`, `choice_line`,
   `close_choice`), with line numbers kept for the check. Errors, each with
   a test: `@else` or `@endif` without `@if`; a second `@else`; `@if` not
   closed before `@end`; an id that isn't a valid character id.
4. **Check** (`crates/content/src/dialogue/check.rs`), each with a test of
   the message and line:
   - the character in `@if` can speak (`CharacterTable::can_speak`);
   - both branches leave the same characters on the same sides and the
     same caption, as replies must (`Checker::choice`);
   - **a character who can be absent appears only where they are known to
     be present**: placing them or giving them a line outside an `@if` for
     them (or inside its `@else`) is an error. Who can be absent: every
     character of `new_game.ron`'s roster except the lead, plus anyone a
     battle file can recruit. A battle's fall scene is the exception for
     its own unit: find a way to say so that doesn't need the scene to
     know its trigger (for example, the check takes the triggers and
     treats a `UnitFell` scene's unit as present). Record the choice in
     the ADR.
   - The existing scenes (`test*.dlg`, `ch01.dlg`) must still pass. If the
     new rule fails `ch01.dlg` before 0716 has rewritten it, turn the rule
     on per file or keep a short list of scenes still to convert, and say
     so in 0716; don't weaken the rule itself.
5. **Player** (`crates/ui/src/dialogue.rs`, `DialoguePlayer`): takes the
   set of present characters when it is made and walks the matching branch.
   Skipping a scene, the text log and going back (if any) must agree with
   what was played.
6. **Callers**: `Flow::play` (`crates/ui/src/flow.rs`) passes the
   campaign's roster ids; the battle screen's trigger scenes
   (`crates/ui/src/screens/battle/mod.rs`, where it makes
   `DialogueScreen::overlay`) pass the characters of the units on the map;
   the debug viewer passes everyone who can speak.
7. Update `assets/dialogue/README.md`.

## Acceptance criteria

- [x] A scene with an `@if` block plays the first branch when the character
      is in the roster and the other (or nothing) when they aren't (test on
      `DialoguePlayer`).
- [x] In a Classic campaign where a companion died in the battle, the
      chapter's victory scene plays the absent branch; in Casual, after a
      retreat, the present branch (harness or flow test).
- [x] A trigger scene in a battle plays the absent branch for a character
      whose unit has already fallen (test).
- [x] Each error in steps 3 and 4 has a test of its message and line.
- [x] `print_scene` round-trips a scene with blocks.
- [x] ADR written and listed in `docs/adr/README.md`.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: parser errors; check errors; the player's walk through each branch,
  with skip.
- Property: the parse → print → parse round trip with blocks; a random
  present-set never puts an absent character on screen in a scene that
  passes the check.
- Integration: the flow test for a Classic death and a Casual retreat; the
  all-assets test.

## Completion notes

**Done.** ADR-0055 records all of it.

- **Format:** `@if <character>` … `@else` … `@endif` (`Step::If`). Blocks
  nest, stand in a reply's reaction, and may hold a `@choice`. A block's
  lines are written as far in as its `@if`. `@if` takes exactly one
  character id, which leaves `@if flag:<id>` and the like free for later.
- **Parser:** a stack of open blocks replaces the single open choice;
  `ParsedScene::lines` is now a tree (`Lines`/`PartLines`) instead of
  `step_lines` + `choice_lines`.
- **Checks:** in `check_scene`, the block's character can speak and both
  parts leave the same screen and caption. In the new `check_presence`
  (`dialogue/presence.rs`), run by `load_embedded` once the battles,
  chapters, New Game file and supports are loaded: anyone the army can
  lose appears only in a block for them; nobody appears in the `@else` of
  their own block; a block that can only go one way is an error.
- **Player:** `DialoguePlayer::new` takes a `Present` and plays
  `Scene::resolved(&present)`, the scene with every block replaced by the
  part that holds. Skipping, music and voice preloading need no changes
  and can't disagree with what was played. (There is no text log or
  going back yet.)
- **Callers:** the flow passes the campaign's roster; the battle screen
  passes who was on the map when the trigger fired; the debug viewer
  passes `Present::Everyone`.
- `assets/dialogue/README.md` has the new section "Who is still there".

**Deviations from the plan**

- **`core` changed** (not in the steps): `Event::SceneTriggered` now
  carries `present`, the characters on the map at that moment. The battle
  screen holds the state *after* the whole command, so "the units on the
  map" read there is wrong for a scene played before the combat that
  kills someone: they would already be gone in the scene that comes
  before their death. Only the battle knows the moment.
- **The fall-scene exception is general.** The check takes, from each
  trigger, the characters it names (not only `UnitFell`'s unit: also the
  fighters of `CombatStart`, `HalfHp`'s unit, both of `Talk`, who enters
  an area) and treats them as there in that scene. A support
  conversation can count on its pair.
- **Before anyone can have fallen**, the check lets a scene count on the
  starting roster: the first chapter's intro scenes, and its battle's
  scenes at the start of turn 1's player phase. Without this 0716 would
  have to wrap `ch01_intro`, `ch01_prebattle` and `ch01_first_turn` in
  blocks that can never be false.
- **No per-file switch and no list of scenes to convert.** `ch01.dlg`
  passes because its companions aren't in the New Game roster yet. So
  that the rule can't bite by surprise: 0803 (which adds them) is now
  blocked by 0716, and 0716 has a section saying what 0715 built and
  asks for one test that runs the check on `ch01.dlg` with the five
  companions able to be gone.
- The placeholder scripts were converted: the knight's lines in `test`
  (the debug viewer's scene) and `test_victory` are in blocks;
  `test_victory` has a line for when the knight died.
- The property test "a random present-set never shows an absent
  character" is in `content` (on `Scene::resolved`, with generated
  scenes) and in `ui` (on `DialoguePlayer`, over every embedded scene).

**Rules a player would notice** (*Claude's starting rules*, for Nick to
agree or veto):

1. In a scene **during a battle**, "there" means **on the map right
   then**. A companion you left out of the battle counts as not there,
   and an enemy counts as there. (The ticket set this; noted because it
   is what a player sees.)
2. Someone who is **about to fall** is still there in the line said just
   before that fight, and for their own last words. They are gone from
   every scene after.
3. A scene in which **every line belongs to people who are gone is not
   shown at all**: no empty box, and the music change it would have made
   doesn't happen either. A writer who wants something said anyway
   writes an `@else`.

**Follow-ups:** none created. 0716 and 0803 were edited as above.
