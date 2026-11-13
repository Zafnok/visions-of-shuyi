---
id: "0496"
title: "The battle keeps every unseen line of Vix's loaded, so a tip fired early still speaks"
type: bug
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: ["0494"]
nick_input: none
completed: 2026-11-13
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

- [x] On the battle's first frame the kept set holds every unseen tip's
      and every lesson's line the battle may still show (test).
- [x] Leaving the battle sends an empty set (test).
- [x] In the web demo served with each clip held 0.5 s, every box of
      Chapter 1's turn 1 at Full has its clip fetched before it shows
      (the server's log, in the Completion notes).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `GuideVoice` sends the set; the set empties on leaving.
- Integration: the battle's kept set at its start.
- Manual: the web demo in the browser pane, as above.

## Completion notes

**What changed.** `GuideVoice` (`crates/ui/src/screens/battle/guide_voice.rs`)
sends `ctx.keep_voices` with the guide's lines the battle may still show,
where it sent `preload_voices`: on the battle's first frame, whenever her
box changes (only if the set changed), and on the first frame back after a
screen over the battle. Only lines with a clip go in the set, and an
empty set is sent only to let a kept one go: unlike `preload_voices`,
the engine's `keep_voices` sends an empty list, and the first build sent
one on every battle without clips, which 13 tests that read a battle's
exact audio requests caught. `GuideVoice::leave` sends an empty set; the
battle screen calls it through `BattleScreen::leaving` whenever `update`
or `uncovered` returns a transition other than `None` or a `Push` (so
Game Over's Retry Battle and Title leave it too: `uncovered`'s answer
moved into `BattleScreen::answered` so every return path passes through
`leaving`). `coming_guide_lines` keeps the same lines; its doc says the
order no longer matters. The `engine-friction: engine#3213` mark is
gone. `docs/voice.md` and `crates/ui/README.md` describe `keep_voices`.

**Tests.** Unit (`guide_voice.rs`): the set is sent first and at each
change; again after a screen over the battle; `leaving_lets_her_lines_go`
(empty set, then the set anew); `no_clip_keeps_nothing` (unvoiced lines
left out, nothing sent with none); nothing with voices off. Integration
(`crates/ui/tests/it/lessons.rs`):
`the_battle_keeps_her_lines_before_her_box_asks_for_one` checks the one
set sent on the battle's first frame holds every tip's line and the
lesson's, before her clip; `leaving_the_battle_lets_her_lines_go`
suspends the battle from the map menu, its lesson still open, and reads
one empty `KeepVoices`.

**Web demo check.** `cargo shuyi export web --release --edition demo`
(`dist/licenses/THIRD_PARTY_LICENSES.html` a local placeholder, as in
0494), served by a Python server that holds every clip 0.5 s, logs each
request and sends `Cache-Control: no-store`, in the browser pane. New
Game at "Not familiar" (Full), voices on, the intro scenes skipped. The
24 clips of her lines this battle may show were all requested within
2 s of the battle's first frame (six at a time, the browser's limit),
while the opening scene was still over the map, about 17 s before her
first box. Turn 1 walked to the enemy phase: the start tip, "Moving the
cursor", "Played on the keys", "Finding your units", "Selecting a unit",
the "Moving a unit" tip (the one 0494 saw fetched late), the "Skills"
tip, "Threat arrows", the "Zooming the map" tip and the "Enemy phase"
tip each showed with no request for any clip after the first 2 s. Not
walked: the remaining units' moves and an attack, but no line of hers
was requested after the battle's start, so none could be fetched late.
The demo was built before the clip filter above; it changes nothing
there (all 24 lines have clips, and the set was never empty). Whether
her lines sound right goes on the Act 1 playtest's checklist (1064).

**Engine friction:** estimate 25, measured 53 (2.1×), accepted: about
20 of the added lines are `uncovered`'s body moved into `answered`
(a move counts as added), and 8 filter the set to voiced lines; no
engine assumption in the way. Row in `docs/engine/friction.md`.
Signals: workaround no (the 0494 workaround is removed) · third copy no
· budget no.

**Follow-up tickets:** none.

No gameplay rule was decided.
