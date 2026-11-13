---
id: "0496"
title: "The battle keeps every unseen line of Vix's loaded, so a tip fired early still speaks"
type: bug
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: ["0494"]
nick_input: none
completed:
---

# 0496 — The battle keeps every unseen line of Vix's loaded, so a tip fired early still speaks

## Context

0494 preloads the guide's lines on the battle screen
(`BattleScreen::coming_guide_lines`, `GuideVoice` in
`crates/ui/src/screens/battle/`) with `ctx.preload_voices`, likeliest
first. The platform keeps only the next 3 of that list loaded, and a
clip not loaded 0.3 s after it is asked for is skipped. A tip fires when
the player does something for the first time, in an order the game
can't know, so a tip outside those 3 is fetched only as its box shows.
0494's check in the web demo, with every clip held 0.5 s by the server:
the tip shown on first picking a unit up ("Moving a unit") was asked for
only as it showed (0494's Completion notes). On itch.io a fetch and its
decoding can take that long too, and her box is then silent.

Engine ticket 3213 (`Zafnok/shuyi-engine`, PR #392 holds the ticket)
adds `Ctx::keep_voices`: a set of clips kept loaded until replaced. The
code to replace is marked `// engine-friction: engine#3213`.

## Nick input

None. Verified by the tests and the web demo in the browser pane;
whether her lines sound right goes on the Act 1 playtest's checklist
(1064).

## Scope

**In:**
- ~~The pin moved to an engine commit with 3213 (`engine-change` §7, or the
  open `engine-pin` PR if one is merging; issue #649 tracks the pin).~~
  (done by the pin move of 2026-11-13)
- The battle keeps the guide's lines it may still show
  (`coming_guide_lines`, any order) with `ctx.keep_voices`, sent when the
  list changes and on the first frame back after a screen over the
  battle, and an empty set when the battle screen leaves the stack.
  The `preload_voices` of 0494 goes, with the `engine-friction` mark.
- `docs/voice.md`, "How the game plays them": the battle keeps her lines
  loaded.

**Out (do not do):**
- Rewording any line (AGENTS.md rule 3).
- The tips over screens outside battle (0497).

## Implementation steps

1. ~~Move the pin (`engine-change` §7).~~ (done by the pin move of 2026-11-13)
2. `GuideVoice::show` sends `ctx.keep_voices(&coming)` where it sends
   `preload_voices` today; `coming_guide_lines` needs no order any more
   (keep its doc honest).
3. When `BattleScreen::update` returns a transition that leaves the
   battle (not `Transition::None` or a push over it), send
   `ctx.keep_voices(&[])`.
4. Tests: the unit tests in `guide_voice.rs` and the integration test
   `the_battle_preloads_her_lines_before_her_box_asks_for_one`
   (`crates/ui/tests/it/lessons.rs`) read `KeepVoices`; a new one shows
   the empty set when the battle ends.

## Acceptance criteria

- [ ] On the battle's first frame the kept set holds every unseen tip's
      and every lesson's line the battle may still show (test).
- [ ] Leaving the battle sends an empty set (test).
- [ ] In the web demo served with each clip held 0.5 s, every box of
      Chapter 1's turn 1 at Full has its clip fetched before it shows
      (the server's log, in the Completion notes).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `GuideVoice` sends the set; the set empties on leaving.
- Integration: the battle's kept set at its start.
- Manual: the web demo in the browser pane, as above.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
