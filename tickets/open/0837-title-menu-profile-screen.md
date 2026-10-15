---
id: "0837"
title: The title menu's Profile screen switches, adds, renames and deletes profiles
type: feature
milestone: M7 Chapter 1 & game flow
model: opus-5.5
effort: medium
status: todo
blocked_by: ["0836", "0839"]
nick_input: sign-off
completed:
---

# 0837 — The title menu's Profile screen

## Context

Ticket 0058 (`docs/design/options.md`, *Where options are kept*): a
`Profile` item on the title menu (below `Options`, *Claude's starting
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
- Words in the language files (Claude writes them; American English;
  `test` pack too). Keys named through the keymap (`keyboard-input` skill).

**Out (do not do):**
- The storage and migration (0836).
- Showing the profile anywhere but the title (a later ticket if Nick asks).

## Implementation steps

1. The screen, its view and glyph skin; the title menu item.
2. Wire switch/add/rename/delete to 0836's `Ctx` operations; the add flow
   hands over to the layout picker.
3. Snapshot and scripted tests.

## Acceptance criteria

- [ ] The title menu has `Profile`; the screen lists profiles with the
      current one marked.
- [ ] Switching changes the options, keys and saves the game uses (test).
- [ ] Adding asks a name, runs the layout picker, and starts with default
      options (test).
- [ ] Renaming keeps the profile's options and saves (test).
- [ ] Deleting asks first with focus on No; the only profile can't be
      deleted (tests).
- [ ] Cancel in a name box with an unkept name asks first, cursor on go
      back; with nothing typed it just closes (tests).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot: the screen's view with one and with several profiles; the
  delete question.
- Integration: scripted add → change an option → switch → switch back.

## Completion notes

*(Filled in by the session that completes the ticket.)*
