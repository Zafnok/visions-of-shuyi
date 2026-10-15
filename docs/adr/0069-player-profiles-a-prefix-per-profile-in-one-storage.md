# ADR-0069: Player profiles: a key prefix per profile in the one storage, a list kept per computer

- **Status:** Accepted
- **Date:** 2026-10-15
- **Related tickets:** 0836, 0058, 0837, 0839, 0838, 0825, 0826
- **Supersedes:** ADR-0050's storage part (the `settings` key, and saving
  at once on every change); the rest of ADR-0050 stands
- **Amends:** ADR-0031 and ADR-0039 (where their keys live; their formats
  stand)

## Context

Nick chose player profiles (ticket 0058, `docs/design/options.md`, *Where
options are kept*): a profile holds every option, the hand layout and key
bindings, the tips seen, the 30 save slots and the suspend save. The game
opens on the last profile used and makes `Player 1` on a first launch. A
player from before profiles keeps everything, as `Player 1`. The game mode
and tutorial level stay in the campaign's saves; which profile was used
last and the title's backdrop stay with the computer.

Until now each of these was one `Storage` key for the whole computer:
`settings` (ADR-0050), `keybindings` and the old `layout` (ADR-0031),
`tips_seen` (0406), `slot_01`..`slot_30`, `suspend` and `last_save`
(ADR-0039), and `title_backdrop` (0811). `Storage` keys must match
`^[a-z0-9_-]{1,64}$`; native turns one into a file name, web into a
`localStorage` name. About twenty places read or write `ctx.storage`.

## Decision

- **A profile's keys are its own keys with a prefix**: profile `n` keeps
  what it holds under `p<n>-<key>` (`p1-settings`, `p2-slot_03`,
  `ProfileId::key`). The keys inside keep their names, values and formats:
  nothing in `settings`, `keybindings`, `tips_seen` or a save changes, so
  no format version moves.
- **`ctx.storage` is the profile in use** (`trpg_ui::profiles::ProfileStore`,
  a `Storage` that adds the prefix, and lists only that profile's keys
  without it). Every screen that reads or writes saves, settings, keys or
  tips does so through it, as before, and gets the profile's. **What stays
  with the computer** goes through `ctx.storage.computer()`: the list of
  profiles and `title_backdrop`. Anything new is the profile's unless it
  says otherwise.
- **The list of profiles is one RON record under `profiles`** (per
  computer), versioned like ADR-0050's settings:
  `Profiles(version: 1, profiles: [(id: 1, name: "Player 1")], current: 1, next_id: 2)`.
  `current` is the profile used last; the next launch opens on it.
  Numbers are **never reused** (`next_id` only grows), so a key a deleted
  profile left behind can never turn up in a new one. A `current` not on
  the list reads as the first profile; a `next_id` some profile has is
  moved past them all.
- **Start-up** (`Ctx::with_storage`, `profiles::load`):
  - list saved: use it;
  - none saved: make profile 1, named by the text `profile.numbered`
    (`Player 1`), and move the old keys into it (below);
  - saved but unreadable: make the list again from the profile numbers
    found among the stored keys (`Player <n>`), so no save is lost; with
    none, as if none was saved;
  - storage unreadable: profile 1 for this launch, nothing written.
  Each problem is a warning for `app` to log, like the key bindings'.
- **The move from before profiles** runs when no list is saved. For each
  old key (`settings`, `keybindings`, `layout`, `tips_seen`, `suspend`,
  `last_save`, `slot_01`..`slot_30`): copy it to `p1-<key>` unless the
  profile has that key already. Then save the list. Then delete the old
  keys. The saved list is what marks the move as done, so it runs once.
  Stopped before the list is saved, the next launch copies again (a key the
  profile already has, perhaps changed since, is kept); stopped after, an
  old key left over is a copy nothing reads. The value of `last_save` (a
  key such as `slot_03`) is a key inside the profile, so it moves as it is.
- **`Ctx` operations for the Profile screen (0837):**
  `profiles()`, `switch_profile(id)`, `add_profile(name)`,
  `rename_profile(id, name)`, `delete_profile(id)`.
  - A switch points `ctx.storage` at that profile, reloads its key
    bindings and settings (theme, map look and zoom with them) and its
    layout; with no layout saved yet, the layout picker's keys are in use
    and `Game` opens the picker at the next key, as on a first launch.
    Tips seen and saves are read from storage when a screen needs them, so
    they follow at once. The switch applies even if the list can't be
    saved (the next launch then opens on the profile used before).
    Switching is for the title (0837): a battle or campaign under way is
    not reloaded.
  - Add, rename and delete change the list only once it is saved; if it
    can't be, nothing changes. Names are kept trimmed, 1 to 12 characters
    (`MAX_NAME_CHARS`), unique ignoring case; at most 8 profiles
    (`MAX_PROFILES`); the only profile left can't be deleted
    (`options.md`). A new profile starts from the defaults; the one in use
    stays.
  - A delete saves the list without the profile, then deletes every
    `p<n>-` key of it (`profiles::delete_keys`), and no other profile's.
    Deleting the profile in use switches to the first one left.
- **Saving at once is gone** (`options.md`, *Keeping changed options is
  the player's choice*): ADR-0050's `change_settings` saves on every
  change; from ticket 0839, a change made on the Options screen shows at
  once but is written to the profile only when the player keeps it on
  leaving (Keep / Discard / Back). Settings changed outside Options keep
  saving at once: the Zoom map key and the layout picker. Until 0839
  lands, `change_settings` still saves at once.
- **`MemoryStorage` and the test `Harness`** work as before: a harness run
  starts with an empty storage (a first launch), and `into_storage` hands
  back the computer's whole storage for the next launch.

## Consequences

- Every existing caller of `ctx.storage` keeps working and is now per
  profile; only the title's backdrop had to move to `computer()`.
- A test that writes an old key (`settings`, `slot_01`) into a
  `MemoryStorage` before `with_storage` exercises the move and still sees
  the value through `ctx.storage`; one that inspects the raw storage sees
  `p1-` keys.
- The longest key a profile can use is 64 bytes less its prefix (`p1-` is
  3; a `u32` number at most 12). Every key the game has is far shorter;
  `ProfileStore` refuses one that wouldn't fit.
- A player sharing a browser profile across two copies of the game on the
  same origin still shares everything: profiles split players, not
  installs.
- Language (0825) and voices (0826) become settings fields with defaults
  and are per profile with nothing more to do.

## Alternatives considered

- **One storage key per profile holding everything** (settings, keys,
  saves in one record): a save of a few kilobytes rewritten with every
  option change, every format nested in another, and the save screens'
  per-slot reads become one big parse.
- **A directory per profile on native, a prefix on web**: `Storage` has no
  directories (ADR-0009's trait is flat by design), and two schemes would
  need two migrations.
- **Pass a profile to every `save`/`tips`/settings call** instead of
  making `ctx.storage` the profile: twenty call sites to change, and a new
  one would default to the computer's storage, the wrong way round.
- **Reuse a deleted profile's number**: smaller numbers, but a delete that
  stopped part-way would leak its saves into the next profile made.
- **Move the old keys under a fresh number each launch until it works**:
  harder to reason about than "copy, save the list, delete".
