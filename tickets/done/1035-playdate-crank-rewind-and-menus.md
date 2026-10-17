---
id: "1035"
title: "Playdate: the crank rewinds, the system menu's three entries, and map menu entries for actions with no button"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: done
blocked_by: ["1034"]
nick_input: none
completed: 2026-10-17
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

- [x] In the Simulator, winding the crank back in Chapter 1's first battle
      opens the rewind and steps back one action per notch; forwards steps
      towards the present; A then confirms as on the PC; B cancels.
- [x] The system menu shows Danger zone, Dialogue log and Options, and
      each works where it can.
- [x] The Playdate's map menu lists the four extra entries when they have
      no button; the PC's map menu is unchanged. (Three: no Dialogue
      log, Nick 2026-10-17; see the notes.)
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit + property: crank notches.
- Integration (harness): rewind by notches, system menu entries, map menu
  entries (Playdate and PC).

## Completion notes

**What was done.** Input had moved to the engine (1107), so the crank is
an engine input source, as the *Carve* note asked (ADR-0084):

- **The crank as two buttons** (`Button::CrankBack`, `CrankForward`,
  appended to `Button::ALL`): a notch wound is a press and a release, so
  it goes through `InputState` and the player's pad bindings like any
  button. `shuyi_ui::input::crank::Crank` turns degrees into notches,
  purely (deviation: in `engine/crates/ui/src/input/crank.rs`, not
  `crates/ui/src/input/crank.rs`, since input is the engine's now); the
  notch size is `keymap.ron`'s `crank: (notch_degrees: 30)` (optional,
  1 to 360). `RawInputEvent::from_crank` makes a frame's events; docked,
  nothing. 8 unit and 2 property tests.
- **Actions** `RewindBack` / `RewindForward` (optional), bound only in
  the `playdate` table (to the crank) and left off the Key bindings
  screen (`input::CRANK_ACTIONS`; the PC's rows are unchanged, a test
  checks). On the map in the player phase a back notch opens the rewind
  on the newest action, as Rewind does; on the rewind screen back is
  down (further back), forwards is up; nothing else changed there.
- **The system menu**: `game::system_menu` (`SystemMenuEntry`: Danger
  Zone, Dialogue Log, Options, named from the language file) and
  `choose(game, entry)`, which hands the top screen the entry's action
  through the engine's new `Game::act` (an action in an update of its
  own, as if its key were pressed). Options had no action, so there is
  `Action::Options`: no key anywhere, not rebindable; the title,
  Preparations and the battle map (browsing or in the map menu) open
  Options on it where their own menus would. The front end adds the three
  entries (`addMenuItem`, renamed with `setMenuItemTitle` if the language
  changes, `src/menu.rs`) and forwards a choice before the next frame.
- **Map menu entries** for Danger Zone, Auto-End and Unit Info, after
  Objective, on a platform without a keyboard, each only while its action
  has no button or chord in the player's bindings
  (`map_menu::shortcuts`). Choosing one closes the menu and replays its
  action through the battle's own path (`Effect::Press`), so it does what
  the key does (and the lessons see it, for 1060). Unit Info is dimmed on
  an empty tile. The PC's map menu is unchanged (tested with keyboard and
  controller).
- Tests: `ui/tests/it/playdate.rs` (crank: three back notches → the third
  newest, one forward → the second, half notches, a big wind stops at the
  oldest, B closes, A asks and A rewinds for a charge, the crank does
  nothing on the question; the map menu's entries on the Playdate and the
  PC; the system menu on the title, in a scene and on the battle map),
  plus unit tests for the rewind screen, the map menu, the system menu,
  `Game::act`, `from_crank`, the keymap's `crank` and the front end's
  menu.

**In the Simulator** (release build, driven by window messages and a
temporary, uncommitted state log, as in 1033 and 1034): the system menu
showed danger zone, dialogue log, options; Options opened over the title
and over the battle map; Dialogue Log opened the log in the opening
scene; Danger Zone toggled the zone on the map. The map menu's Danger
Zone, Auto-End and Unit Info did what their keys do. With three actions
in Chapter 1's first battle, winding the crank back opened the rewind on
the newest and stepped back one action per 30°, stopping at the oldest;
winding forwards stepped towards the present; B closed it; wound back
again, A asked "Use 1 of 3 charges?" and A rewound (3 actions to 2,
3 charges to 2). The SDK reports winding back as negative degrees, as
the code expects. A docked crank could not be tried there (dragging the
Simulator's crank undocks it); unit tests cover it. The debug build's
`pdc` still crashes (ticket 1056), so the check used the release build.

**Deviations.** No Dialogue log entry in the map menu (Nick, 2026-10-17,
asked in this ticket: it only opens during a conversation, when the map
menu can't; recorded in `docs/design/playdate.md`). The crank
accumulator lives in the engine (above).

**Follow-up tickets.** None new. 1060 (lessons on the Playdate) updated:
the shortcut entries already count as pressing their action, and there
is no Dialogue log entry.

**Claude's starting rules** (where the design is silent; Nick can veto):
- The map menu's extra entries sit after Objective: Danger Zone,
  Auto-End, Unit Info.
- Danger Zone and Auto-End close the map menu and do what their keys do
  (the zone then shows on the map; Auto-End shows its on/off notice).
- Unit Info shows the unit under the cursor (any side's); on an empty
  tile it is dimmed.
- One notch is 30° of winding one way (tunable). Turning the other way
  starts a fresh notch: after a step back, the next step forwards takes
  a whole 30°, and a hand resting on the crank never steps.
- While "Use 1 of N charges?" is up, the crank does nothing.
- Outside the player's turn, or mid-move, winding back does what the
  Rewind key does there (nothing); with nothing done yet it opens the
  rewind showing "Nothing to rewind yet", as Rewind does.
- The system menu's Options opens where a menu has Options (the title
  menu, Preparations, the battle map while browsing or in the map menu),
  not over a conversation or mid-move; Danger Zone works on the battle
  map while the cursor is free; Dialogue Log works in a conversation.
