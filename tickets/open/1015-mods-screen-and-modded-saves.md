---
id: "1015"
title: "Mods screen on the title menu, and saves that remember their mods"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: medium
status: todo
blocked_by: ["1014", "0054"]
nick_input: sign-off
completed:
---

# 1015 — Mods screen, and saves that remember their mods

## Context

1014 loads the enabled mods at startup from storage key `mods`. Players
need a screen to turn mods on and off and order them, and to see why a mod
didn't load. 1011's ADR says a save records the mods it was made with, and
0054 decides whether modded saves are marked or kept apart.

## Nick input

**Sign-off:** Nick installs the example mod (1023) or a test mod, turns it
on from the Mods screen, restarts, and says whether the screen is clear.

## Scope

**In:**
- `Mods` on the title menu (`crates/ui/src/screens/title.rs`), only when at
  least one mod is installed or a problem was found.
- A Mods screen: every installed mod (name, version, author), on or off,
  moved up and down, problems from 1014 shown under the mod; saving writes
  storage key `mods`; a line says changes apply at the next launch.
- `SaveFile` (`crates/core/src/save.rs`) records `mods: Vec<(id,
  version)>`; old saves load as unmodded (bump the save version per
  ADR-0039 and keep the guard test, 0821, passing).
- Loading a save whose mods differ from the enabled ones asks first,
  naming what's missing or changed.
- Whatever 0054 decided for modded saves (a mark on the slot, or a separate
  list).

**Out (do not do):** reloading content without a restart; downloading
mods; Workshop (1024); the web build.

## Implementation steps

1. Read ADR-0017 (screens), ADR-0039 (saves), ADR-0045 (screen text from
   the language file: every new string goes in `assets/lang/en/ui.ron`),
   the `keyboard-input` skill (no hard-coded keys; reuse existing actions
   for move, confirm, cancel; add an Action only if one is missing), 1014's
   problem list and 0054's answers.
2. The screen in `crates/ui/src/screens/mods.rs`, following an existing
   list screen (the save slot screen in `screens/save.rs` is the closest).
3. Save changes as above, with the version bump and a migration test.
4. Snapshot the screen with two test mods, one with a problem.

## Acceptance criteria

- [ ] `Mods` appears on the title only when mods or problems exist
      (Harness test).
- [ ] Turning a mod on and moving it writes the expected `mods` value to
      `MemoryStorage` (Harness test).
- [ ] A save made with a mod records it; loading it without the mod asks
      first (Harness test).
- [ ] An old save loads (guard test).
- [ ] No hard-coded key or literal screen text (`cargo xtask check-keys`,
      `cargo xtask check-text`).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: save round trip with mods.
- Snapshot / integration: the Mods screen; Harness flows above.

## Completion notes
