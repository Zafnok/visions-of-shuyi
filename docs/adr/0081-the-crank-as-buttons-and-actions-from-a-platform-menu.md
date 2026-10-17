# ADR-0081: The crank as two buttons, and actions from a platform's own menu

- **Status:** Accepted
- **Date:** 2026-10-17
- **Related tickets:** 1035 (1033, 1034, 1060)
- **Extends:** ADR-0034 (controller input) and ADR-0079 (button chords,
  the Playdate's own buttons).

## Context

The Playdate has a crank and a system menu (behind its Menu button) that
holds three of a game's own entries. Nick's design
(`docs/design/playdate.md`, *Buttons and the crank*): the crank wound back
rewinds, one action per notch, wound forwards steps towards the present;
the system menu holds Danger zone, Dialogue log and Options. Rules that
already hold: no game code reads a key or button, everything reaches
screens as actions through the player's bindings (the `keyboard-input`
skill), and the front end holds no game logic (ADR-0070).

## Decision

1. **The crank is two controller buttons**, `Button::CrankBack` and
   `Button::CrankForward` (`shuyi_assets::keymap`), appended to
   `Button::ALL` so saved bindings and the `ButtonSet` bits keep their
   meaning. A notch wound is a press and a release of one of them, so the
   crank goes through `InputState` and the player's pad bindings like any
   button, and only the `playdate` table binds them. No other code reads
   the crank.
2. **`shuyi_ui::input::crank::Crank`** turns degrees into notches, purely:
   the platform hands it each frame's turn and gets whole notches back.
   What is left of a notch carries to the next frame in the same
   direction, so any split of a one-way turn gives the same notches;
   turning the other way starts a fresh notch, so after a notch back the
   next notch forwards is a whole notch away and a hand resting on the
   crank never gives one. Docked, it gives none and forgets the rest. The
   notch size is the keymap file's `crank: (notch_degrees: 30)`
   (`CrankDef`, optional, 1 to 360). `RawInputEvent::from_crank` makes the
   frame's events, beside `from_pads`.
3. **The game's actions for them.** `RewindBack` and `RewindForward`
   (optional, bound only in the `playdate` table) are this game's: the
   battle opens the rewind on `RewindBack` as on `Rewind`, and the rewind
   screen steps its list on both as on down and up. They are left off the
   Key bindings screen's rows (`input::CRANK_ACTIONS`): no PC device has a
   crank, and the PC's screen keeps its rows.
4. **A platform's own menu hands the game actions.** `Game::act(action)`
   (engine) gives the top screen one action in an update of its own with
   no time passing, as if its key had been pressed: the screen does what
   it does with that key, or nothing. The game's
   `game::system_menu` lists its entries (`SystemMenuEntry`: each one's
   name from the language file and its action) and `choose(game, entry)`
   calls `act`. Opening Options had no action, so the game gained
   `Action::Options`: no key anywhere, not rebindable, sent only by a
   platform menu; the title, Preparations and the battle map open Options
   on it where their own menus would.
5. **Where the front end comes in.** `crates/playdate` reads
   `getCrankChange` and `isCrankDocked` each frame into `from_crank`, adds
   the three entries with `addMenuItem` (renaming them with
   `setMenuItemTitle` when the language changes), and forwards a choice
   (the SDK calls back between two updates) to `system_menu::choose`
   before the next frame, on the game's stack.

## Consequences

- Any game on the engine can bind the crank, and a crank's feel (notch
  size) is data.
- A menu outside the game can only do what an action does on the screen
  in front; a new kind of entry needs an action and the screens that
  answer it.
- The map menu's entries for actions with no button (Danger Zone,
  Auto-End, Unit Info on a platform without a keyboard) choose their
  action the same way, by replaying it through the battle's own path, so
  the lessons can count them (1060).
