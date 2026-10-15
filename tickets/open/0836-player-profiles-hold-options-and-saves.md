---
id: "0836"
title: Player profiles hold the options, keys, tips seen and saves
type: feature
milestone: M7 Chapter 1 & game flow
model: opus-5.5
effort: high
status: todo
blocked_by: []
nick_input: sign-off
completed:
---

# 0836 — Player profiles hold the options, keys, tips seen and saves

## Context

Ticket 0058 (`docs/design/options.md`, *Where options are kept*): Nick
picked **player profiles**. A profile holds every option, the hand layout
and key bindings, the tips already seen, the 30 save slots and the suspend
save. The game opens on the last profile used; the first one, `Player 1`,
is made without asking. The game mode and the tutorial level stay with the
campaign (its saves).

Today every one of these is a single `Storage` key for the whole computer
(ADR-0009): `settings` (ADR-0050, `crates/ui/src/settings.rs`),
`keybindings` and the old `layout` (ADR-0031, `crates/ui/src/screen.rs`),
`tips_seen` (`crates/ui/src/tips.rs`), `slot_01`..`slot_30` and `suspend`
(ADR-0039, `crates/ui/src/save.rs`). `title_backdrop`
(`crates/ui/src/screens/title.rs`) stays per computer.

This ticket is the storage underneath; the title menu's `Profile` item that
switches, adds, renames and deletes is **0837**.

## Nick input

**Sign-off:** after merge, play on Pages: your options and saves from
before are still there (now `Player 1`).

## Scope

**In:**
- A profile list kept per computer: each profile's id, name and the last
  one used. The current profile chosen at start-up (`Ctx::with_storage`),
  `Player 1` made on a first launch.
- Every key listed above read and written **for the current profile**
  (e.g. under a per-profile key prefix; the scheme is Claude's call, keys
  must stay valid `Storage` keys, `^[a-z0-9_-]{1,64}$`).
- **Migration:** a player from before profiles has their `settings`,
  `keybindings`/`layout`, `tips_seen`, slots and suspend moved into
  `Player 1`, once, and nothing lost. A migration that fails part-way
  doesn't lose data on the next launch (write the new keys before deleting
  the old ones).
- A `Ctx` way to switch profile (reload the settings, keys and tips seen,
  and point the save screens at that profile's slots) for 0837 to call,
  plus add/rename/delete operations on the list (delete removes that
  profile's keys).
- An ADR superseding ADR-0050's storage part and amending where ADR-0039's
  and ADR-0031's keys live (`write-adr` skill).
- Bump whatever format version changes; the settings record itself keeps
  its fields.
- Options are written to the profile only when the player keeps them
  (`options.md`; the Options screen's question is **0839**); the ADR
  records that the save-at-once rule of ADR-0050 is gone.

**Out (do not do):**
- The title menu's Profile screen and its name box (0837).
- The tutorial-level row in Options (0838).
- Moving the game mode or tutorial level out of the campaign (they stay).

## Implementation steps

1. Write the ADR: the profile list's record (RON, versioned like
   ADR-0050), the key scheme, the migration, what stays per computer.
2. A `profiles` module in `trpg-ui` (pure, over `Storage`): load or make
   the list, current profile, add/rename/delete, key for a profile.
3. Route `settings`, key bindings, `tips_seen`, slots and suspend through
   the current profile's keys; `title_backdrop` and the list stay global.
4. The one-time migration into `Player 1`.
5. `Ctx::switch_profile` (or similar) reloads everything the profile holds.

## Acceptance criteria

- [ ] First launch makes `Player 1` and runs the layout picker as today.
- [ ] Options, keys, tips seen, slots and suspend are kept per profile:
      changing them in one profile doesn't change another (test).
- [ ] The next launch opens on the last profile used (test).
- [ ] A storage from before profiles loads as `Player 1` with every
      option, key, tip seen, save and suspend save intact (test), and the
      migration runs only once.
- [ ] Deleting a profile removes its keys and no other profile's (test).
- [ ] The ADR is written and the index updated.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the profile list round-trips; key scheme gives valid keys;
  add/rename/delete rules (names unique, up to 12 characters, at most 8
  profiles, the last one can't be deleted; `options.md`).
- Integration: migration from an old `MemoryStorage`; two profiles with
  different settings and saves, switched back and forth through `Ctx`.

## Completion notes

*(Filled in by the session that completes the ticket.)*
