---
id: "1035"
title: "Playdate: the crank rewinds, the system menu's three entries, and map menu entries for actions with no button"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: ["1034"]
nick_input: none
completed:
---

# 1035 — The crank, the system menu and the map menu on the Playdate

## Context

`docs/design/playdate.md`, *Buttons and the crank* (Nick, 0061, Q4 B):

- **Crank, wound backwards:** rewind; the battle steps back one action
  per notch; A keeps that point (one charge), B cancels.
- **Map menu:** as on the PC, plus every shortcut the Playdate has no
  button for (Danger zone, Auto-end, Unit info, Dialogue log).
- *Claude's starting rules (Nick can veto):* winding forwards steps back
  towards the present while choosing; the notch size is tunable; the
  crank does nothing else on the map; the three system-menu entries are
  Danger zone, Dialogue log and Options.

Today the rewind screen (`crates/ui/src/screens/battle/rewind.rs`) opens
with the `Rewind` action and lists past actions newest first; the cursor
moves the highlight (`CursorDown` = further back) and the map behind
shows the battle as it was. Confirm asks "Use 1 of N charges?" and
confirms; Cancel or Rewind closes it. The crank only replaces the
opening and the cursor moves: the rest of that flow is unchanged.

## Nick input

None. Decided in 0061.

## Scope

**In:**
- **The crank as input, through the keymap** (`keyboard-input` skill):
  the front end reads the SDK's crank change each frame
  (`getCrankChange`), and a pure accumulator in `trpg-ui`
  (`input/crank.rs`) turns degrees into **notches** (backwards /
  forwards), with the notch size in `assets/data/keymap.ron`
  (*tunable*; start at 30°). Notches reach the game as two bindable
  controller inputs ("crank back", "crank forward") so they go through
  the player's bindings like buttons; the `playdate` table (1034) binds
  them. Nothing else may read the crank.
- **What a notch does**, as actions: on the battle map in the player
  phase, a back notch opens the rewind screen with the newest action
  highlighted; then each further back notch moves one action further
  back and a forward notch one towards the present (closing nothing).
  Use new optional actions (e.g. `RewindBack`, `RewindForward`) or map
  the notches to the rewind screen's existing cursor moves, whichever
  keeps the rewind screen's logic one piece; the PC's Key bindings
  screen must not gain rows (hide actions no PC device can bind, or bind
  them only in the `playdate` table). A docked crank does nothing.
- **The system menu:** three entries added with the SDK's
  `addMenuItem`: Danger zone, Dialogue log, Options (their names from
  the language file). Choosing one does what the same thing does in the
  game: toggle the danger zone (`DangerZone` action), open the dialogue
  log where it can open (`DialogueLog`), open Options the way the map
  menu or title menu opens it. Entries that can't act on the current
  screen do nothing. Add a `Game` entry point for these (pure, tested)
  so the front end only forwards the choice.
- **Map menu entries for actions with no button:** when the platform has
  no keyboard (1033's setting), the battle's map menu
  (`screens/battle/map_menu.rs`) gains an entry for each of Danger zone,
  Auto-end, Unit info and Dialogue log **that has no button or chord
  bound** on the active device. Unit info shows the unit under the
  cursor, or is greyed out with none. The PC's map menu is unchanged
  (tests).
- Tests read the state and the screens' views, not cells.

**Out (do not do):**
- How these look on the Playdate (1038, 1039): this ticket works with
  whatever skin is drawn.
- Any other use of the crank (scrolling text, menus): Nick's starting
  rule says the crank does nothing else on the map.
- Any PC change.

**Carve (1106, 1107):** either order works. If input moved to the engine
first, the crank accumulator is an engine input source (any game can bind
it) and the actions stay the game's.

## Implementation steps

1. Read `docs/design/playdate.md`, `docs/design/death-and-difficulty.md`
   (*Rewind*), `rewind.rs`, `map_menu.rs`, and the `keyboard-input`
   skill.
2. The crank accumulator and its notch tests (property: any split of
   the same total turn gives the same notches).
3. Crank inputs in the bindings and the `playdate` table.
4. Rewind opening and stepping by notch; harness tests: wind back three
   notches → the rewind screen highlights the third newest action; wind
   forward one → the second; Cancel closes; Confirm asks as on the PC.
5. The system menu: shim calls, the `Game` entry point, tests.
6. Map menu entries; tests for the Playdate and the PC.
7. Simulator check (the Simulator has a crank control).

## Acceptance criteria

- [ ] In the Simulator, winding the crank back in Chapter 1's first battle
      opens the rewind and steps back one action per notch; forwards steps
      towards the present; A then confirms as on the PC; B cancels.
- [ ] The system menu shows Danger zone, Dialogue log and Options, and
      each works where it can.
- [ ] The Playdate's map menu lists the four extra entries when they have
      no button; the PC's map menu is unchanged.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit + property: crank notches.
- Integration (harness): rewind by notches, system menu entries, map menu
  entries (Playdate and PC).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
