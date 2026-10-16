---
id: "0804"
title: Nick playtests Chapter 1; feedback becomes tickets
type: playtest
milestone: M7 Chapter 1 & game flow
model: opus-5.5
effort: medium
status: in-progress
blocked_by: ["0803", "0802", "0602", "0307", "0406", "0107", "0108", "0414", "0417", "0424", "0425", "0710", "0807", "0814", "0215", "0809", "0810", "0413", "0035", "0316", "0231", "0435", "0714", "0715", "0716", "0436", "0437", "0440", "0116", "0822", "0727", "0724", "0447", "0444", "0055", "0448", "0442", "0439", "0811", "0819", "0820", "0830", "0607", "0451", "0452", "0056", "0317", "0057", "0058", "0029", "0429", "0728", "0729", "0730", "0832", "0833", "0834", "0455", "0456", "0457", "0458", "0731", "0835", "0836", "0837", "0838", "0839", "0732", "0733", "0734"]
nick_input: sign-off
completed:
---

# 0804 — Chapter 1 playtest

## Context

The first milestone Nick plays. He said he'll judge the game by playing and
will report comments, concerns and bugs. This ticket packages a build, gets
his feedback, and turns it into tickets. **No code changes in this ticket.**

**Blockers added 2026-09-29**, so Nick's first impression includes what's
already designed: Combat Arts in the attack flow (0414), boss arts (0503, via
0803), his uppercase-label request (0417), battle and menu sounds (0424,
0425), scene and battle music (0710, 0807), no freeze when music loads
(0215), victory/defeat/game-over music (0809) and the battle results screen
(0810), without which step 4's question about the rewind EXP bonus can't be
answered because the bonus is never shown.

**Blockers added 2026-10-01** (Nick, asked whether the playtest should wait
for each: "sure", "yes"):

- The combat scene with bought full-body art (0413) and Harl's combat
  picture (0035).
- Non-attack skills cost uses per battle (0316, from Nick's review of
  0503). 0803 waits for it too, so its winning replay and difficulty
  numbers are made with the final skill costs.
- Game Over offers Rewind while charges are left (0822, from Nick's answer
  on ticket 0408; asked whether it should be in before the playtest:
  "yes").

**Blocker added 2026-10-02:** 0714 (dialogue speakers who aren't units).
The Chapter 1 script (0707) can't pass its check without it, and 0803
needs the script.

**Blockers added 2026-10-02 (by 0707):** 0715 and 0716. In Classic a
companion who dies in the battle still talks in the victory scene; 0715
lets a script mark lines by who is still in the army, and 0716 rewrites
the Chapter 1 scenes with it. Nick hasn't been asked whether the playtest
should wait for these; drop them from `blocked_by` if he says no.

**Blocker added 2026-10-02 (ticket 0039):** 0440, sprite units stepping
on the spot and walking along their path. Nick was asked whether the
playtest waits for it and said yes ("B").

**Blockers added 2026-10-02 (ticket 0038):** 0436 and 0437, the battle
map drawn with the bought unit sprites and tilesets. Nick bought the art
that day and, asked whether the playtest waits for the bought-art map,
answered "A" (yes). They bring 0432 and 0433 (the map skin) with them.
Per-map lighting (0438) and the zoom toggle (0439) were not part of the
question and don't block the playtest.

**Round 1 played 2026-10-12; round 2 waits for its feedback** (Nick,
2026-10-12): "I think we can conclude this round of ch1 playtesting, and
park this ticket until the feedback I gave is resolved bc it impacts my
first impression more than I thought, so let's say we will revisit round 2
once these new blockers are done and add all these as blockers now". Added
to `blocked_by`: 0727, 0724, 0447, 0444, 0055, 0448, 0442, 0439, 0811,
0819, 0820 and 0830 (the table in the Completion notes says which remark
each answers). The title tickets bring their own chain with them: 0012
(the game's title), 0817, 0818 and 0827.

**Round 2 starts at step 2**, with a new build: bump the version, cut
`v0.2.0` the same way, and send the message again. `v0.1.0` exists and is
not cut again. Steps 4 and 5 were not asked in round 1; ask them in
round 2.

**Round 2 played 2026-10-14; round 3 waits for its feedback** (Nick: "ch1
still doesn't feel right... we need to keep iterating on ch1 ... it's my top
priority to get all this feedback done and then replay ch1 over and over to
get the feel correct"). Added to `blocked_by`: 0607, 0451, 0452, 0056,
0057, 0058, 0029, 0429, 0728, 0729, 0730, 0832, 0833 and 0834 (the round 2
table in the Completion notes says which remark each answers). The
decision tickets (0056, 0057, 0058, 0029) add the tickets they create to
this list.

**Round 3 starts at step 2**, with `v0.3.0` cut the same way (the version
bump goes in its own PR from a branch whose name doesn't start with
`t0804`: the ticket lint wants a `t0804-` branch to close the ticket).
Steps 4 and 5 are answered (round 2); in round 3 ask instead about the
four scene music picks once 0029's backgrounds are in (0029 decided
2026-10-15: the talk over its place, darkened, with a short moving
opening where the script asks; built by 0732, 0733 and 0734, added to
`blocked_by`).

## Nick input

**Sign-off:** play Chapter 1 (≈15–25 min) and report anything: bugs, feel,
difficulty, story, looks, controls.

## Implementation steps

1. Make sure `main` is green and deployed to Pages (0108). Cut `v0.1.0` via
   the release workflow (0107): bump version, merge, tag.
2. Send Nick a short message with:
   - Links: browser build (Pages) and Windows zip (GitHub Release).
   - A key cheat-sheet (from `keymap.ron`, not from memory).
   - What to pay attention to (5 bullets max): cursor feel, readability of the
     map, whether combat numbers make sense, difficulty, whether the story hooks him.
   - "Just tell me in plain words; screenshots welcome."
3. For each piece of feedback, apply the `write-ticket` skill's
   "Turning Nick's playtest feedback into tickets" section. Ask at most one
   clarifying question per unclear item.
4. Ask Nick whether the unused-rewind-charge EXP bonus (7% of a level per
   charge, `death-and-difficulty.md`) felt right, and record any change.
5. Ask Nick about what 0803 left for him to judge by playing (its
   completion notes list every starting rule): whether Rue's Cinder Oath
   and Maud's Vigil Light are too strong to have from the start
   (`magic.md`, "Personal signature spells"); the four scene music picks
   he wanted to "feel out" (`audio.md`, "Chapter 1's cues"); and whether
   it feels right that most of the Red Company waits at the bridges and
   the fort instead of charging.
6. Put a summary in Completion notes: feedback verbatim + ticket numbers created.

## Acceptance criteria

- [ ] Nick played and gave feedback.
- [ ] Every feedback item maps to a ticket (or an explicit "won't do" with reason Nick agreed to).
- [ ] `v0.1.0` release exists.

## Completion notes

**Round 1 done, parked until its feedback is built** (see Context).
`v0.1.0` was tagged on `main` at `f106381` (2026-10-12;
`Cargo.toml` already said 0.1.0, so no bump) and
[released](https://github.com/Zafnok/visions-of-shuyi/releases/tag/v0.1.0)
with Windows, Linux, macOS and web packages.

### Feedback, first batch (Nick, 2026-10-12, verbatim) and where it went

| Nick said | Ticket |
| --------- | ------ |
| "the hold F to speed up text is too eager, I press F to advance to next dialogue and already half the next text is displayed bc it interprets a tap as a hold too quickly" | **0727** (new, bug) |
| "Hollis and Veyne first dialogue is too weird... idk what this means 'that eyebrow's had one scar off me already' wtf? sounds weird as hell" | **0724** (open; his words added) |
| "'A bit proud' also sounds weird unless you enhance it like 'mumbles' or \*mumbles\* or something" | **0724** |
| "'pedlar' is that supposed to be 'peddler'? wtf is this olde english or something?" | **0724** (spell it "peddler") |
| "Aske saying 'twelve men. no. fourteen. bad archers' also doesn't sit right with me. bad archers...? so why would they not be counted?" | **0724** |
| "the blue outline is hard to see for friendlies, the red is very clear." | **0447** (new, options shown to Nick first) |
| "the cursor is also a bit hard to see, I think there's an open ticket for this." | **0444** (open; its question to Nick is now answered) |
| "when moving or proposing to move into an enemy's attack range, it should illuminate or draw slim arrows from enemy to the tile or unit you're moving to. Newer fire emblem games do this. So you don't need to rely on danger zone as much." | **0055** (new, decides the details) then **0448** (new, builds it) |
| "is there an open ticket for making combat arts/skills etc into a different menu? cuz rn it's not... u select weapon and it has attack in same menu as the skills" | **0442** (open, unchanged) |
| "I think we can scale the battle map bigger. tile size / overworld unit size. rn it's quite small idk if the original assets are this small or we shrunk them to glyph size but we can make the 2D size bigger" | **0439** (open, the 1× / 2× zoom; now also asks which zoom the game starts at) |
| "game over and title screen should both have a nice animation w our logo (I think there are open tickets for this)" | Title: **0811**, **0819**, **0820** (open). Game Over: **0830** (new) |

Still to ask, in round 2: steps 4 and 5 (the rewind EXP
bonus, the two signature spells, the four scene music picks, enemies
waiting at the bridges and the fort).

### Round 2 (started 2026-10-13)

All 42 blockers are in `tickets/done/`. The version is bumped to 0.2.0
(root `Cargo.toml`, `Cargo.lock`) in its own PR, because the release
workflow refuses a tag that doesn't match `Cargo.toml`; this ticket stays
open in that PR. After the merge: tag `v0.2.0` on `main`, wait for the
release, send Nick the message of step 2 with steps 4 and 5's questions.

The round 1 tickets left these for Nick to judge by playing; they go in
the message:

- 0727: does the text speed-up start soon enough when the key is held
  (0.3 s, *Claude's starting rule*)?
- 0447: one of his units that has acted has a darker rim than before.
- 0055 / 0448: the threat arrows' starting rules (they stay while the
  action menu is open; units that have acted show them too; the glyph
  look has them too).
- 0439: the map starts at 2×; the zoom key switches.

`v0.2.0` was tagged on `main` at `a43818b` and
[released](https://github.com/Zafnok/visions-of-shuyi/releases/tag/v0.2.0)
(2026-10-13) with Windows, Linux, macOS and web packages; Pages deployed
the same commit. The message went out the same day.

### Feedback, round 2 (Nick, 2026-10-14, verbatim) and where it went

| Nick said | Ticket |
| --------- | ------ |
| Intro cinematic: "the dialogue scene with Tamsin and Hollis hangs uncomfortably long, it should only be like 1/3 its current length." | **0833** (new) |
| "the battle scene has walkers moving at speeds that are not correct to reality, and multiple units walking at same time. This is not the how the game plays so it looks unnatural" | **0833** |
| "the combat scene (2 battlers) is never shown, only some in-battle-map fighting which also looks a bit odd." | **0832** (new, the shot) then **0833** |
| Options: "they should probably be per-save-file. I have never saved and yet the browser cached my options settings which feels weird to me." | **0058** (new, decision) |
| "Maud's eyes look too scary like they are cat-like like too skinny, too focused, they stare into my soul... maybe she needs different eyes" | **0730** (new) |
| "seems main character has no lv up portrait idk why" | **0607** (new, bug: the level-up page looks up `lead`, the portraits are `lead_m` / `lead_f`) |
| Script: "the rewrite was too literal. It took the lines line by line and sucked out the quips rather than reworking the script. Aske says 'fourteen men in the fort...' the line is so literal and boring.... I don't like the original quips but this is just matter of fact and weird." | **0728** (new) |
| "I see still no helper text for i.e. Brace what the hell does it do I thought we already took care of this ticket but it still tells me nothing about what it does when I hover over it in skill menu about to select it" | **0429** (open, never done; his words added) |
| "seems the forest tilemap doesn't line up well with our grid and makes it a little confusing where the forest starts and ends visually. it overlaps with road but becomes road, it overlaps with plain but becomes plain. Need better differentiating here." | **0452** (new) |
| "healers should have the same hover over friendly unit, click, means heal, like units have for enemy attacks..." | **0451** (new) |
| "we should have some kind of basic tutorial, I am almost thinking of the annoying fairy from oot or clippy or something... some tutorial mascot thing idk." | **0057** (new, decision) |
| "on special skill Rue's skill is range 1 really? seems a bit weird what is the flavor here? why should mage special skill be range 1?" | **0056** (new, decision): range 1, no counter → **0317** |
| "Tamsin in general feels too OP for the first fight, constantly doubling on enemy and also having highest mov" | **0834** (new, tuning) |
| "we should have a chat log, accessible via key shortcut, optional binding. when a conversation plays, press chat log button and able to scroll thru messages, hover to see the portraits and that dialogue how it was, click to skip back (with option to skip forward again to latest seen point). Also if we add VA then it will play the voice again." | **0729** (new) |
| "probably the conversations should have some kind of background imagery, similar to a cutscene... like even just Dace pacing a castle or whatever... and then flip to the conversation portraits with the background opacity lessened or otherwise darkened... it would set the mood better than portraits against black background" | **0029** (open; his idea added as the leading option) |

**Steps 4 and 5, answered (recorded in the design docs):**

| Asked | Nick | Recorded |
| ----- | ---- | -------- |
| Rewind bonus, 7% of a level per unused charge | "feels fine" | `death-and-difficulty.md`: settled |
| Rue's and Maud's signature spells | "seem ok other than cinder oath concern I raised" | `magic.md`: Vigil Light kept; Cinder Oath → **0056** |
| The four scene music picks | "I want to see how the cutscenes would play out with this ambience" | `audio.md`: still unapproved, asked again with **0029**'s backgrounds |
| Most of the Red Company holding the bridges and the fort | "its ok I guess" | `chapter-1.md`: kept |
| The 0.3 s before held Confirm speeds text up (0727) | "it's fine" | `controls.md`: kept |

