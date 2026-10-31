---
id: "0837"
title: The title menu's Profile screen switches, adds, renames and deletes profiles
type: feature
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0836", "0839"]
nick_input: sign-off
completed: 2026-10-15
---

# 0837 — The title menu's Profile screen

## Context

Ticket 0058 (`docs/design/options.md`, *Where options are kept*): a
`Profile` item on the title menu (below `Options`, *the agent's starting
rule*) lists the profiles with the current one marked, and switches, adds,
renames and deletes. 0836 builds the profiles underneath and the `Ctx`
operations this screen calls.

## Nick input

**Sign-off:** after merge, add a second profile, change an option and save
in it, switch back to `Player 1` and see your own options and saves.

## Scope

**In:**
- The `Profile` title item and its screen (ADR-0054: logic builds a view,
  a skin paints it; copy `screens/options.rs`).
- **Switch:** picks that profile; the title then shows its options (and
  `Continue` its suspend save).
- **Add:** a name box, then the first-launch setup (the hand-layout
  picker) for the new profile, which starts from the default options.
- **Rename:** the same name box, filled with the current name.
- **Delete:** asks yes/no first, saying the profile's saves go with it,
  focus on No (the *Asking first* rule). Not offered for the only profile.
- Names up to 12 characters, unique; at most 8 profiles (*tunable*,
  `options.md`). No name box exists yet: build a small one (typed
  letters, Backspace deletes, Confirm keeps, Cancel leaves the name as it
  was, like the volume box in `options.md`); on a controller, an
  on-screen letter grid (Nick, PR #264; on Steam and consoles the
  platform's own keyboard replaces it, ticket 0903, not here).
- **Backing out asks first** (`options.md`, *Keeping changed options is
  the player's choice*): Cancel in a name box with a name typed and not
  kept asks keep / throw away / go back, cursor on go back.
- Words in the language files (the agent writes them; American English;
  `test` pack too). Keys named through the keymap (`keyboard-input` skill).

**Playdate:** if 1045 (the Playdate New Game and Options screens) is
done, give the Profile screen a Playdate paint too, chosen by `ctx`'s
`SkinSet` (1037); if not, 1045 does it.

**Out (do not do):**
- The storage and migration (0836).
- Showing the profile anywhere but the title (a later ticket if Nick asks).

## Implementation steps

1. The screen, its view and glyph skin; the title menu item.
2. Wire switch/add/rename/delete to 0836's `Ctx` operations; the add flow
   hands over to the layout picker.
3. Snapshot and scripted tests.

## Acceptance criteria

- [x] The title menu has `Profile`; the screen lists profiles with the
      current one marked.
- [x] Switching changes the options, keys and saves the game uses (test).
- [x] Adding asks a name, runs the layout picker, and starts with default
      options (test).
- [x] Renaming keeps the profile's options and saves (test).
- [x] Deleting asks first with focus on No; the only profile can't be
      deleted (tests).
- [x] Cancel in a name box with an unkept name asks first, cursor on go
      back; with nothing typed it just closes (tests).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot: the screen's view with one and with several profiles; the
  delete question.
- Integration: scripted add → change an option → switch → switch back.

## Completion notes

**Done.**

- `screens/profile.rs` (+ `view.rs`, `glyph.rs`): the screen builds a
  `ProfileView`, the glyph skin paints it (ADR-0054). The profiles are
  listed with the one in use marked `in use`, then `Add a profile`.
  Confirm on a profile opens a small menu: `Use this profile` (not for
  the one in use), `Rename`, `Delete` (not for the only profile).
- `Profile` is on the title menu under `Options`.
- **The name box:** lead-select already had a typed box and a controller
  letter grid. Both moved to `widgets/name_entry.rs` (with its glyph
  painter) and both screens use it, each with its own rules
  (`NameRules`: most characters, digits or not). Lead-select behaves and
  looks as before (its snapshots are unchanged); its four shared texts
  were renamed `name_entry.*`. Cancel is now reported to the screen,
  which closes the box or asks first.
- Tests: view, actions, delete question (focus on No), the most profiles,
  the controller grid, glyph tests, three snapshots (one profile, three,
  the delete question); scripted in `tests/it/profiles.rs`: add, change
  an option, switch to `Player 1` and back (options, keys and saves
  follow), rename keeps options and saves, delete on the screen, Cancel
  in the name box with and without a name typed, a taken name.
- Title tests and snapshots updated for the extra menu row.
- Playdate paint: 1045 isn't done, so 1045 does it (as the ticket says).

**the agent's starting rules** (the design docs were silent):

- **Use this profile** goes straight back to the title, which then shows
  that profile's options and `Continue`.
- **Adding** a profile makes it the one in use, then shows the layout
  picker for it; after picking, you're back on the title.
- Choosing a profile opens a small menu: `Use this profile`, `Rename`,
  `Delete`.
- A profile name may have **digits** (so you can type `Player 3`), as well
  as letters, `-`, `'` and single spaces, like the lead's name. The letter
  grid gets a row of digits for it.
- A name another profile has is refused with `Another profile already
  has that name.`, and the box stays open.
- In the `Keep this name?` question, Keep does what Enter would (and if
  the name is refused, you're back in the box with the message).

**For Nick when playing:** Title → `Profile`. Add a profile, pick its
hands, change an option and keep it, then `Profile` → `Player 1` →
`Use this profile`: your own options and saves are back.
