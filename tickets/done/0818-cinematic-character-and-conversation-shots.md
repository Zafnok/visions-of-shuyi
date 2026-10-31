---
id: "0818"
title: "Cinematic shots: characters, and conversation snippets"
type: feature
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0036", "0817", "0704", "0711"]
nick_input: answer-first
completed: 2026-10-13
---

# 0818 — Cinematic shots: characters and conversation snippets

## Context

Nick's intro cinematic goes "past some characters" and shows "brief
conversaiton (non spoiler segments) snippets" (ticket 0036). 0817 built the
cinematic player with map pans and the logo. This ticket adds the two shot
kinds that show people.

Nick's answers in 0036 (2026-10-03, `docs/design/title-screen.md`, *Intro
cinematic*):

- **Six bought faces slide past with their names**: the lead, Hollis,
  Tamsin, Aske, Maud, Rue.
- **Snippets play in the game's dialogue screen**, and they are the
  **opening lines of real Chapter 1 scenes**, read from the same script
  file the game plays (`assets/dialogue/ch01.dlg`). No separate cinematic
  script.
- **The lead follows the most recent save**: its name and its face. With
  no save, the default lead.

Character art is bought and drawn as sprite items (0711, ADR-0032,
ADR-0038).
The agent never draws character art.

## Nick input

**Answer first:** ticket 0036 (answered 2026-10-03).

## Scope

**In:**
- Shot kind `Characters`: the listed characters' pictures move across the
  screen as the storyboard describes.
- Shot kind `Talk`: lines of a `.dlg` scene play by themselves in the
  dialogue screen's look (portraits, name plates, text box, typewriter
  text), with no input.
- Both shots show the lead as the most recent save has them (below).
- Validator rules for both; both added to `assets/cinematics/README.md`
  and to `assets-test/cinematics/test.ron`.

**Out (do not do):**
- Choosing which scenes and how many boxes the title cinematic shows
  (0820).
- Writing or changing dialogue. The shots show whatever the script file
  says.
- The title screen (0819).
- Combat pictures (0036 didn't choose them).

## Implementation steps

1. **`Talk(scene: "<id>", first: 0, count: 2)`**: text boxes `first ..
   first + count` of a dialogue scene. From the shot's `progress` and
   length, give each box an equal share of the time: reveal its text at
   `Ctx::text_speed` (capped so the whole box is shown for at least the
   last third of its share), then hold. (*the agent's starting values,
   tunable*, unless 0036's storyboard gives its own; list them in the PR
   for Nick.) Drawn from `t` alone (0817's rule), so split the drawing in `crates/ui/src/screens/dialogue.rs` into
   a function that takes the scene state (who is left and right, speaker,
   caption, text, characters revealed) and call it from both the screen
   and the shot. No "next" marker and no help line in the shot. Name and
   pronoun tokens (0708, 0709) resolve as in the dialogue screen.
   Narration boxes and `@caption` lines in the range show as they do in
   the dialogue screen.
   Validator: the scene exists; the range is inside it; no `@choice` in
   the range; every box fits one page (no paging in a cinematic).
2. **`Characters(who: ["<character id>", …], expression: "neutral")`**:
   each face is drawn with 0711's portrait drawing at its normal size and
   crosses the screen in turn, evenly spaced over the shot, in the
   direction the shot gives (the storyboard: right to left), vertically
   centred. Clipped at the screen edges. The character's short name (0712)
   is printed under each face (0036: names are shown). Validator: every character has a portrait with
   that expression.
3. **The lead from the most recent save.** `CinematicPlayer::new` takes
   an optional lead (name and which of the lead's two looks), which the
   caller reads from the save written last. Saves carry no time (only
   `app` reads a clock), so make `save::write` (`crates/ui/src/save.rs`)
   also store the key it wrote under a new `Storage` key `last_save`, and
   add `save::latest_lead(storage)`: the lead of the save `last_save`
   names; if that key is missing or the save can't be read, the suspend
   save's, else the lowest filled slot's, else `None`. Take the name and
   look from the save's campaign the way the dialogue screen does.
   `Talk` resolves the lead's name and pronoun tokens with it and both shots draw that look's portrait.
   With no save: the names table's default name, and the default look
   that 0706 recorded: `lead_m` (as the dialogue screen falls back
   today). The debug tool passes `None`.
4. Add one of each to `test.ron` using the test portraits and the `test`
   scene, and update the 0817 snapshots.

## Acceptance criteria

- [x] Unit: the `Talk` timing (which box, how many characters revealed) at
      the start, mid-reveal, during the hold and at each box change.
- [x] Snapshot: a `Talk` shot mid-reveal and held; a `Characters` shot with
      one face part-way in and one centred.
- [x] The dialogue screen's own snapshots are unchanged.
- [x] Each validator error has a test.
- [x] Harness: with a save whose lead was renamed and uses the other
      look, a `Talk` box that names the lead and a `Characters` shot show
      that name and that portrait; with no save they show the default.
- [x] What is drawn matches 0036's storyboard rules (list in the completion
      notes which rules were applied).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: validator; `Talk` timing.
- Property: for any `t` in the shot, the revealed length never exceeds the
  box's text and never shrinks as `t` grows within one box.
- Snapshot / integration: the snapshots above; the debug "Play test
  cinematic" tool still loops.

## Completion notes

**Done**

- Two shot kinds in the cinematic file (`assets/cinematics/README.md`):
  `Talk(scene, first, count)` and `Characters(who, expression, from)`.
  Both are in `assets-test/cinematics/test.ron`, which is now 20 seconds:
  the pan (0 to 6 s), two text boxes of the test scene (6 to 10 s), three
  faces (10 to 16 s), the logo (16 to 20 s). The debug menu's "Play test
  cinematic" shows them.
- `Talk` is drawn by the dialogue screen's own code: `screens/dialogue.rs`
  was split into `draw_told` (a box playing by itself), `draw_face` (a
  portrait frame with its name) and the pieces both use. The dialogue
  screen's snapshots did not change; a test compares a talk shot cell by
  cell with the dialogue screen at the same box.
- `Characters` uses the same portrait frame (`draw_face`), so the faces
  are at dialogue size.
- `save::write` notes the key it wrote under the storage key `last_save`;
  `save::latest_lead` reads the lead from it. `CinematicPlayer::new` takes
  the lead; the debug tool passes none.
- 0817's snapshots (pan start, pan middle, logo) are unchanged; three were
  added (text typing, text held, faces).

**0036's storyboard rules applied**

- Hard cuts: the new shots start and end on the millisecond, no fade.
- Snippets play in the game's real dialogue screen, by themselves, the
  text typing out.
- The lines come from the script file; the cinematic has no text of its
  own.
- Never past a reply choice (the validator refuses it). Which scenes are
  shown, so "only scenes before the battle", is 0820's choice of ids.
- Faces cross right to left (the default), one after another, each with
  its name under it, at dialogue size.
- The lead follows the most recent save, name and face; without a save,
  the default name and `lead_m`.
- No sound of its own: the shots ask for none (a scene's `@music` line is
  not followed).

**the agent's starting rules (Nick can veto; also in
`docs/design/title-screen.md`)**

- *How a snippet's text appears:* the boxes of a dialogue shot share its
  time equally. A box types out at the text speed from Options, then stays
  until the next. A long box types faster so that it is whole for the last
  third of its time. Example: two boxes in a 16-second shot get 8 seconds
  each; a 60-letter line is typed in 1 second at Normal and stays for 7.
- *How the faces cross:* at one steady speed, in a row. When one face is
  in the middle of the screen, half of the next is already in and half of
  the one before is still leaving. They face the way they move. Each is in
  the lit frame the dialogue screen gives whoever is speaking. They move a
  character cell at a time (about 20 steps a second in the title's
  19-second shot), not pixel by pixel, because the frame and the name are
  made of characters.
- *The name under a face* is the first name ("Hollis", not "Hollis Marr");
  the lead's is the name the player gave.
- *"The most recent save"* is the one written last, a chapter save or a
  suspend save. If it is gone (a suspend save is used up when the battle
  is resumed), the suspend save if there is one, else the lowest save slot
  in use. Saves made before this change carry no note of which was last,
  so they follow that same order.

**Deviations**

- *The portrait rule of the validator.* The ticket says every character
  of a `Characters` shot must have a portrait with the expression. The
  bought portraits are not in this repository (ADR-0040), so the title's
  cinematic could then never pass the gates. The rule is the one scripts
  have: a portrait a character has must have the expression; a character
  without one gets an empty frame and may only use a standard expression.
- *`from` field.* The ticket's sketch had no field for the direction but
  says the shot gives it: `from: Right` (the default) or `Left`.
- *Which boxes `first` and `count` count.* The scene is read with everyone
  there (`@if` blocks play their first part), and the boxes must all come
  before the scene's first reply choice. Counting past a choice would
  depend on the reply picked.
- *One page.* The validator checks a box with every name at its longest.
  A translation can still be longer than the English; the shot then shows
  the first page only.
- `Ctx::text_speed` in the ticket is `ctx.settings().text_speed`.
- The page size (92 by 3) moved to `trpg_content::dialogue` (`BOX_WIDTH`,
  `BOX_LINES`) so the validator and the screen share it.
- The map pan still shows the lead unit as `ctx.lead` has it (0817); only
  the two new shots take the lead given to the player.

**Follow-ups:** none created. 0819 calls `save::latest_lead` when it makes
the title's player; 0820 picks the scenes and the counts.

**For Nick when he plays:** F2 on the title, "Play test cinematic". The
faces and the text timing above are the things to judge; the real six
faces and the real lines come with 0820.
