---
id: "0927"
title: "Decide and build: a Classic player of the Chapter 1 demo, which has no Preparations, and the switch to Casual"
type: feature
milestone: M8 Release
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0927 — Decide and build: a Classic player of the Chapter 1 demo, which has no Preparations, and the switch to Casual

## Context

A test audit of the Chapter 1 web demo (ticket 0914, `docs/design/demo.md`) was done before the demo goes to applicant writers (0766, on the password page from 0915). It found that a player who picks Classic at New Game can never switch to Casual in the demo, and the game tells them to do it on a screen the demo doesn't have.

- Nick's rule (`docs/design/death-and-difficulty.md`, *Mode changes*, 2026-10-04, ticket 0805): Classic → Casual only, "at the Preparations screen before a battle, never in the middle of one". The code: `crates/ui/src/flow.rs` `push_next` opens the switch only when `self.stage == Stage::Preparations`, and `crates/ui/src/screens/options.rs` `can_switch_mode` / `switch_mode` refuse with `MODE_AT_PREP_MESSAGE` anywhere else.
- That message (`assets/lang/en/ui.ron`, `options.message.mode_at_prep`): "The game mode can be changed at Preparations, before a battle".
- Chapter 1 has one battle, `ch01`, with `preparations: false` (`assets-private/content/battles/ch01.ron`), and the demo ends after it (`crates/content/demo/bundle.txt` sets `next: None`; `flow.rs` goes to the demo's end card). So the demo never reaches Preparations.
- The New Game mode screen (`crates/ui/src/screens/mode_select.rs`; `mode_select.classic_line`: "Classic: a unit that falls in battle dies and is gone for good.") doesn't say whether the mode can be changed later.
- 0805's completion notes (item 10) covered a battle without Preparations ("the row says 'can be changed at Preparations'") but not a build that never reaches one. In the full game a Classic player can switch at Chapter 2's Preparations. A save exported from the demo keeps its mode (the mode is in the campaign) and goes on to Chapter 2 in the full game (`demo.md` rule 5).

How a Classic player of the demo is told, or helped, is how the game's rule works and what the player sees, so it is Nick's call.

Not a blocker for the writer trial (0766): a writer sees the Preparations message only by opening Game mode in Options.

## Nick input

**Decision**, asked before building with the `ask-nick` skill. The answer goes into `docs/design/death-and-difficulty.md` (*Mode changes*) and `docs/design/demo.md`.

> In the Chapter 1 demo there is no Preparations screen, so a player who picks Classic can never switch to Casual, and the Options screen tells them to do it at Preparations. Which do you want?
>
> - **A. Say it up front: the mode is final in the demo.** The New Game mode screen adds one line in the demo, for example "In the demo, the mode can't be changed later." The Options row's message in the demo says the same instead of naming Preparations. Like *Fire Emblem Awakening*, where Classic or Casual is picked at New Game and kept for the playthrough. The full game is unchanged.
> - **B. A battle without Preparations lets you switch at its start.** At the start of a battle that has no Preparations screen (Chapter 1, in the demo and the full game), on the player's first turn before any unit has acted, Options offers Classic → Casual. *Restart battle* returns there. Once a unit acts, the row says to restart the battle. This keeps your rule that a battle never changes mode in the middle. Closer to *Fire Emblem: Three Houses* and *Engage*, which let a Classic player turn Casual on later, one way only (confirm before asking).
> - **C. Both A and B.** The mode screen tells the player, and Chapter 1 still has a place to switch.
> - **D. Describe your own.**

The exact wording of any new line is proposed in the question and kept as Nick answers. Nothing is checked after merge. The result is verified by the tests and `cargo shuyi frame-png` pictures in the PR, and what is left to judge by playing goes on the Act 1 playtest's checklist (1064).

## Scope

**In:**
- Ask the decision above and record the answer in the two design docs.
- Build the chosen option: the text keys in `assets/lang/en/ui.ron` (and the `test` language pack if every key needs one there), the screen logic, and the tests.
- If the chosen option makes the existing message wrong anywhere, replace it there.

**Out (do not do):**
- A Casual → Classic switch, or a switch in the middle of a battle (Nick's rule stands unless the answer changes it).
- Ironman (0049).
- Moving where a switch reaches the battle into one place: 0857's job. Either order works. If 0857 is done, option B's switch goes through its one place. If not, write it into the battle the same way `switch_to_casual` does from Preparations, and add a line to 0857 naming the new call.
- Hiding the Towns row in the demo (0926).
- Adding a Preparations screen to Chapter 1 (content and pacing; not offered unless Nick asks for it under D, in which case file it as its own ticket).

## Implementation steps

1. Run the decision with `ask-nick`, render the mode screen and the Options row for each option with `cargo shuyi frame-png` in both looks (glyphs and pictures), and record the answer.
2. **If B (or C):** the full game's Chapter 1 changes too, so add 0927 to 1064's `blocked_by` and to the Act 1 critical path in `docs/ROADMAP.md`.
3. **If A (or C):** in `crates/ui/src/screens/mode_select.rs`, add the new line to `ModeSelectView` when `ctx.game.demo`, painted by both skins (`glyph`, `playdate`). In `options.rs`, `switch_mode` refuses with a new `MODE_FIXED_IN_DEMO_MESSAGE` key when `ctx.game.demo`, and keeps `MODE_AT_PREP_MESSAGE` otherwise. Add both keys to `ui.ron`. Expected size: about 30 production lines (one view field, two paint lines, one branch).
4. **If B (or C):** open the switch while a battle without Preparations is on the player's first turn and no player unit has acted. The battle screen sets `ctx.game.mode_switch = ModeSwitch::Open` there and `Closed` after the first action, read from the battle's state, never from cells. A yes on the question writes Casual into the campaign and the running battle, as `switch_to_casual` does at Preparations (see Out on 0857). `Restart battle` reopens it. The closed-switch message for such a battle names restarting the battle (a new key). Expected size: about 80 production lines, judged against the Preparations switch in `flow.rs` / `screen.rs`. Measure with `cargo xtask code-lines` (ADR-0092).
5. Update the doc comments in `options.rs` (module doc, `game_mode_row`, `can_switch_mode`, `switch_mode`) and `flow.rs` `push_next` to the new rule.
6. Add the tests below. Put `cargo shuyi frame-png` pictures of the changed screens in the PR.

## Acceptance criteria

- [ ] Nick's answer is in `docs/design/death-and-difficulty.md` (*Mode changes*) and `docs/design/demo.md`, with his words quoted.
- [ ] In the demo's bundle no screen shows text naming Preparations: `crates/ui/tests/it/demo.rs` asserts the Game mode row's message for a Classic campaign in the Chapter 1 battle.
- [ ] If A or C: the demo's New Game mode screen shows the new line and the full game's doesn't (unit tests in `mode_select/tests.rs`). Confirm on Game mode in the demo refuses with the new message (unit test in `options/tests.rs`).
- [ ] If B or C: on turn 1 of a battle with `preparations: false`, before any action, Confirm on Game mode asks, and *Yes* makes both the campaign and the battle Casual (a unit that falls then retreats). After one player action it refuses. After *Restart battle* it asks again. Each is an integration test in `crates/ui/tests/it/` that reads the state, not cells.
- [ ] Casual never switches back, and a battle with Preparations still switches only at Preparations: `classic_switches_to_casual_after_a_confirm_and_never_back` and `classic_cant_switch_where_the_switch_is_closed` pass, updated only for a renamed message.
- [ ] The PR shows `cargo shuyi frame-png` pictures of each changed screen.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `crates/ui/src/screens/options/tests.rs` (the refusal message per build and per battle kind), and `crates/ui/src/screens/mode_select/tests.rs` (the line shown only in the demo) for A/C.
- Property: none needed (the one-way rule is covered by the existing tests).
- Snapshot / integration: `crates/ui/tests/it/demo.rs` (a Classic run in the demo's bundle: what Game mode says and does). For B/C, a flow test of the turn-1 switch, the closing after an action, and *Restart battle* reopening it. Update the mode screen's glyph snapshot if its view changes.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations, follow-up tickets created, notes for Nick.)*
