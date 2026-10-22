---
id: "1170"
title: "The Playdate front end on the engine's Playdate backend: the game keeps its config, its start and its one-bit skins"
type: feature
milestone: "Engine: carve"
model: opus-5.5
effort: high
status: todo
blocked_by: ["engine#3010", "engine#3028", "engine#3029", "1175"]
nick_input: none
completed:
---

# 1170 — The Playdate front end on the engine's backend

## Context

The first engine-friction sweep (1153, `docs/engine/friction.md`)
measured the Playdate front end (`crates/playdate`, ADR-0070) at 1735
production lines against an estimate of 100 (17×): the game's
`crates/app/src/main.rs` is 66 lines over the engine's desktop backend,
and the Playdate's should be the same plus the context setup in
`start.rs`. About 1,585 lines are platform glue: SDK bindings, the
allocator, the panic handler, the frame driver, storage, the game's own
stack, audio, pictures, and the on-device report.

The engine holds that work in its tickets 3003 (the platform boundary,
`App::frame`, now with a loading phase), 3010 (the Playdate backend and
`shuyi export playdate`, now pointing at the game's code to lift), 3011
(on the device, now with the report a shipped game keeps), 3028 (the
game on a stack of its own) and 3029 (audio). Each lifts code from this
crate. This ticket moves the game onto them once they land.

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> Play the Playdate build on the device after this lands
> (sideload as usual) and say if anything feels different: start-up,
> sound, saves, the system menu.

## Scope

**In:**
- Move the pin past the three engine tickets (`engine-change` skill,
  step 7).
- `crates/playdate` keeps: the game's Playdate config (its name, data
  folder, keymap, system menu labels, the loading card, the stack
  size), `start.rs` (the game's context), and the build glue the
  engine's exporter can't do for it (if any; say what in the
  Completion notes). Everything the backend now does goes:
  `src/pd/*`, `audio.rs`, `storage.rs`, `report.rs`, `pads.rs`,
  `menu.rs`, `pictures.rs`, `stack.rs`, and `shim/` if the backend
  replaced it.
- `cargo xtask playdate` becomes a call to `shuyi export playdate`, or
  goes, with `docs` and `crates/playdate/README.md` updated.
- Start the Playdate build the way 1175 started the desktop one: a
  `shuyi_platform::Start` with the game's loading card, its load and its
  look, which the engine's Playdate backend runs (3003 changed how a
  backend starts; 1175 moved the desktop `main` and the Harness onto it).

**Out (do not do):**
- The one-bit skins in `crates/ui` (the game's look,
  `docs/design/playdate.md`).
- The AI's speed on the device (1051) and the battle frame's time
  (1057).
- The other Playdate tickets (1057; 1059, the battle map at 50 frames a
  second) touch `crates/playdate` too; 1056 (done) strips the debug
  ELF's unwind sections in `crates/xtask/src/playdate.rs` so `pdc`
  packs it, which engine 3010's exporter now also knows (1063 was its
  duplicate, closed). **Either order works:** if one lands first and
  changes a file this ticket deletes, check the engine's backend does
  the same thing and, if it doesn't, add it there (a small engine PR,
  `engine-change` skill) before deleting the game's copy.

## Implementation steps

1. Write the estimate: about 1,600 lines removed, 100 added.
2. Move the pin; build the device and Simulator targets.
3. Replace the front end piece by piece (storage, audio, the stack,
   the report), checking a device build after each with
   `cargo xtask playdate` (or its replacement).
4. Measure start-up and the minute report against the numbers in
   ADR-0070 and 1057; write them in the Completion notes.

## Acceptance criteria

- [ ] `crates/playdate` is under 200 production lines.
- [ ] A save made before the change loads after it (same data folder
      and format).
- [ ] Start-up to the title on the device is no slower than before
      (the report's numbers in the Completion notes).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The existing host tests that still apply (the engine now holds the
  backend's), and `chapter_1.rs` (Chapter 1 played on the host through
  the Playdate's skins).

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
