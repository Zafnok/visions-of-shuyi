---
id: "0427"
title: "Render and play the \"can't do that\" tone and the close whoosh"
type: feature
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["0031"]
nick_input: none
completed:
---

# 0427 — Render and play the "can't do that" tone and the close whoosh

## Context

Ticket 0031 decides two new in-house sounds (`docs/design/audio.md`): the
"can't do that" tone and, if Nick keeps it, a whoosh when a message closes.
Ticket 0425 left placeholders:

- `MenuSound::Denied` plays `menu_cancel`: the game's `AUDIO` config in
  `crates/ui/src/audio.rs` sets `denied_cue: "menu_cancel"`. The menu
  widget plays it on Confirm on a disabled item (`Menu::handle_with_sound`;
  the widget and `MenuSound` are Shuyi Engine's since 1112, re-exported by
  `crates/ui/src/widgets/menu.rs` and `crates/ui/src/audio.rs`).
- Closing tips, banners, the EXP bar and the level-up page is silent
  (`BattleScreen::update` in `crates/ui/src/screens/battle/mod.rs`: the
  `shown_tip`, `progress_key` and `banner` branches).

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> Press Confirm on something you can't do, and close a tip, a
> phase banner and a level-up page. Does each sound right?

## Scope

**In:**
- Port 0031's recipes to `crates/xtask/src/sfx.rs` (as 0213 did), run
  `cargo xtask sfx`, add the cues to `assets/audio/audio.ron` with
  `credit: Own`.
- `MenuSound::Denied` → the new cue.
- Wherever 0031's Q2 says the tone plays. If that changes battle input
  (Confirm on an acted unit), do that here. If it changes the menu widget
  (the highlight landing on greyed-out items), that is the engine's
  widget: follow the `engine-change` skill (an engine PR first, then this
  ticket moves the pin).
- The close whoosh on the closings 0031 lists (skip if Nick dropped it).

**Out (do not do):**
- Any other sound. Dialogue stays silent.

## Implementation steps

1. Read 0031's section of `docs/design/audio.md` and 0213's completion
   notes (how `sfx.rs` ports a page recipe).
2. Add the recipes to `sfx.rs`; run `cargo xtask sfx`; commit the WAVs; add
   the manifest entries.
3. Set `AUDIO.denied_cue` to the new cue; update the game's test
   `menu_sounds_are_their_cues` (same file).
4. Play the whoosh in the battle screen's close branches.
5. Make the Q2 changes, if any, with Harness tests.

## Acceptance criteria

- [ ] `cargo xtask sfx --check` passes with the new WAVs.
- [ ] Unit test: Confirm on a disabled item plays the new cue.
- [ ] Harness test: closing a tip, a banner and a level-up page each plays
      the whoosh once (if kept); reading on in dialogue plays nothing.
- [ ] Starting rules and anything left to judge by playing are in the
      Completion notes (they go on 1064's checklist).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: menu widget; sfx determinism (the existing test covers new files).
- Snapshot / integration: Harness tests on the battle screen.

## Completion notes

*(Filled in by the session that completes the ticket.)*
