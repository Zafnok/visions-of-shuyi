---
id: "0804"
title: Nick playtests Chapter 1; feedback becomes tickets
type: playtest
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0803", "0802", "0602", "0307", "0406", "0107", "0108", "0414", "0417", "0424", "0425", "0710", "0807", "0814", "0215", "0809", "0810", "0413", "0035", "0316", "0231", "0435", "0714", "0715", "0716", "0436", "0437", "0440", "0116", "0822", "0727", "0724", "0447", "0444", "0055", "0448", "0442", "0439", "0811", "0819", "0820", "0830", "0607", "0451", "0452", "0056", "0317", "0057", "0058", "0029", "0429", "0728", "0729", "0730", "0832", "0833", "0834", "0455", "0456", "0457", "0458", "0731", "0835", "0836", "0837", "0838", "0839", "0732", "0733", "0734", "0250", "0318", "0446", "0449", "0461", "0462", "0463", "0464", "0465", "0466", "0467", "0468", "0736", "0737", "0738", "0739", "0740", "0741", "0742", "0841", "0842", "0843", "0844", "0845", "0846", "0847", "1118", "1050"]
nick_input: sign-off
completed: 2026-10-21
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

**Round 3 played 2026-10-16; round 4 waits for the engine carve and the
Playdate port, then for its feedback.** Nick: "generally it's an 8/10,
where before it was 3 or 4/10. I am quite impressed and we can park
playtest 4 til after engine carve and playdate port considerations. The
rest I feel is very much taste driven rather than anything else." And on
the script: "I don't want to delay engine carve or playdate port any
more. So we can have ch1 playtest 4 come after those milestones with
update script." Added to `blocked_by`: 1118 (the carve's first example,
the last carve ticket) and 1050 (Nick plays Chapter 1 on his Playdate),
plus the round 3 tickets he'd judge in Chapter 1 (the round 3 table in
the Completion notes says which remark each answers). Every round 3
ticket also waits for 1118 and 1050 itself, except three tutorial bugs
(0464, 0467, 0841): the Playdate runs the same tutorial code, so Nick
chose to fix them before 1050 ("wait for nothing but the way I read this,
we should prio engine carve. Make it obvious engine carve is now the
critical path."). They wait for nothing, and 1050 waits for them. The
carve's first tickets (1103, 1114) no longer wait for 0804, which would
have been a loop; `docs/ROADMAP.md` now opens with the carve as the
critical path.

**Round 4 starts at step 2**, with `v0.4.0` (or whatever version the game
is at then) cut the same way, and a checklist like round 3's. Ask first
about the new script (0737).

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
  (0.3 s, *the agent's starting rule*)?
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

### Round 3 (started 2026-10-16)

All 70 blockers are in `tickets/done/`. The version is bumped to 0.3.0
(root `Cargo.toml`, `Cargo.lock`) in its own PR, from a branch that doesn't
start with `t0804`; this ticket stays open in that PR. After the merge: tag
`v0.3.0` on `main`, wait for the release, and point Nick at the checklist.

Instead of a short message, Nick asked for a checklist with answer
choices (Nick, 2026-10-16: "make me a checklist, with options… this will
be a LOT of questions"), published as a private claude.ai page that
saves his answers for the agent to read back:
<https://claude.ai/artifact/8kz8qiMc245mNaBNrytYKh>. It has the
links, the key cheat-sheet from `keymap.ron`, five things to watch,
the known gaps (0735, 0453, 0454, 0450, 0041), and 70 questions in play
order. They cover:

- Every round 1 and round 2 item he hasn't reacted to since it shipped:
  0447's deeper outline and the darker rim of a unit that has acted,
  0055 / 0448's threat arrows, 0439 starting at 2×, 0444's cursor, 0442's
  Arts menu, 0811's backdrops and sparks, 0819's idle fade, 0830's Game
  Over, and 0724's words left for him ("Aye", "Greywater", "Your Former
  Grace").
- Each round 2 remark against the ticket that answers it: 0833 / 0832
  (intro talk length, unit moves, combat scenes), 0730, 0607, 0728, 0429,
  0452, 0451, 0057 (→ 0455–0459, 0731, 0835), 0056 / 0317, 0834, 0729,
  0029 (→ 0732–0734) and 0058 (→ 0836–0839).
- The starting rules those tickets left for him, the intro's cut timing
  and logo moment, whether the intro gets an off setting ("Later" in
  0036), and the four scene music picks over their new places
  (`audio.md`, "Chapter 1's cues").
- One glance for the screens rebuilt underneath in 0242–0249, plus the
  overall feel, the pace and a free box.

Tips seen now belong to the profile, so the checklist asks him to add
a fresh profile for this round; Player 1 already remembers the tips from
rounds 1–2.

`v0.3.0` was tagged on `main` at `9360688` (2026-10-16) and released by
the release workflow; Pages deployed the same commit.

### Feedback, round 3 (Nick, 2026-10-16, verbatim) and where it went

Nick answered all 76 questions on the checklist page (70 questions plus
his notes). The overall verdict, and the priority, are in Context.

**Became tickets** (all wait for 1118 and 1050 except 0464, 0467 and
0841, which come before 1050; ✱ = not in round 4's `blocked_by`, Nick
didn't need it before then):

| Asked | Nick said | Ticket |
| ----- | --------- | ------ |
| The script overall | "Fable still did it line by line. Still 14 archers stuff... We need to toss the script. We can keep the beats we agreed on but the whole script is fairly mid imo. We need to have you get a crash course on fantasy writing that's compelling. Idk a skill repo for writing? Your writing is so bland." | **0736** (research the craft, into the story-writing skill) → **0737** (write it again on a blank page) |
| Hollis in the yard | "like I said, this is AGAIN a line by line rewrite. Toss the whole thing." | **0737** |
| Aske's fort report | "again, rewrite" | **0737** |
| Tamsin over Harl | "we'll rewrite all this crap" | **0737** |
| Where it drags | "In the conversations" | **0736** (a length budget), **0737** |
| The intro as a whole (a note) | "some parts of the intro still don't make sense... the narration ends with "somewhere... chops wood" and it's supposed to be exciting? and then the portraits fly by, it's so disjointed... when it zooms on the lead and the archer(?) then it looks like a romantic moment in the middle of a battle... the narration at the beginning is thinking you read slow asf... the music closes with a swell crescendo thing and everybody is just chilling totally stationary it's very anti-climactic... We can park this work til after engine carve and playdate" | **0063** ✱ (decide the rework) |
| Tamsin's news, 6 s | "just a bit too short if it was 7-8s it would probably be good." | **0843** |
| The intro's combat scenes | "hold at end too long, want hit sounds, and we need the battler for tamsin" | **0843**; Tamsin's picture **0446** (open; his words added) |
| Logo at 1:57 or on the chord | "On the chord, 1:55.8" | **0843** |
| The idle fade into the intro | "it's a little obnoxious the way the fade in and fade out weave together... or just don't fade out the menu and instead go to blacck and fade in the cutscene intro only." | **0843** |
| A setting to turn the intro off | "Yes, an on/off setting" | **0844** |
| Darkness behind the talk | "the background should stay at this brightness (or maybe 50-60%) but why are the faces impacted? they are foreground this is weird." | **0739** (the listener's dimming, decided with him) |
| The sparring opening | "they can spar for maybe 2s... anyway an issue is that they are yellow outlined? don't think I signed off on that, I want them with no outline." | **0738** |
| Dace pacing | "Longer" | **0738** |
| The study's darkness | "Darken less in this scene only" | **0738** |
| The victory talk at the toll-fort | "show who's left standing, and fade to the darkened, rather than being darkened immediately, it looks like early morning not noon." | **0738** |
| The pre-battle place | "that's a chapel? I don't get that ambience whatsoever" | **0741** |
| Narration (a note) | "it's odd to me that narration dialogue boxes have their text centered while talking dialogue boxes have their text left aligned. Make it consistent, pref left alignment." | **0740** |
| Three scene music picks | the letter: "we can experiment some more. the music is ok but not perfect."; the Red Company: "we can experiment more, it's ok."; after the fight: "we can experiment" | **0742** (after 0737) |
| The dialogue log | "the log is well implemented overall. I just don't like the default key on keyboard. Let's see what visual novels or persona use." | **0847** |
| The "b log" hint | "all hotkeys need to be more easily discernable right now they blend in "b log" looks like some word... if it were "<B> log" or "[B] log" or bolded B log or something with a different color for the keybind it would be better" | **0250** |
| The Profile screen | "on first launch, we should have the guide kinda pop off to the right in its own window and be like "Hey! Did you change your profile yet?"... showing only until the player makes a save to their profile (customize options or name) or to the game itself." | **0842** |
| Vix's face | "it generally looks quite good but I am wondering if her ears are cut off or that's just how her portrait is" | **0468** |
| Vix's name | "Vix is fine but what we can also do is let players name her like they do the rival in pokemon with pre-filled choices or with free text. We can do the same with the lead character too." | **0065** ✱ |
| Vix overall | "I think she turned out great actually... maybe if there is a way to slot her into a minor role in the story it would be good. For example in fortune's weave, Fortuna is a clippy, but she just advises on unique maps" | **0064** ✱ |
| The lessons' moments | "the Arts lesson coming right after the Attack lesson is where the tips first start feeling disjointed... I moved Tamsin to attack a bandit, now the tip says open arts, but this is impossible with every other unit on turn 1... Probably if the tip cannot be achieved then it shouldn't yet show. Also, after the tutorial tips all play, we should ask if the player is OK with their position or wants to retry battle with no penalty, or retry tutorial with no penalty." | **0464**, **0465** |
| Turn 1's load | "just make sure the tips are actually achievable on the turn they present themselves otherwise it's ok" | **0464** |
| The new tips | "I didn't get the magic trigger for some reason... also didn't get terrain" | **0467** (bug) |
| Guides | "hard to find, the guide should say something like "you're on your own for the rest, if you need any help, here is where you find the Guide. Good luck!"... Also it would be good if the guide had rendered mini screens showing what does what." | **0465**; pictures **0469** ✱ |
| Tutorial set to None in battle | "No" (the tips and lessons didn't stop) | **0841** (bug) |
| Starting at 2× | "ensure the guide has a tip for zoom on first turn, and also we need the map scroll arrows... Otherwise 2x is ok to start on." | **0466**; arrows **0449** (open; his words added) |
| The Arts menu | "swapping weapons on forecast menu feels quite hidden as a feature it should be more obvious idk how also the guide should probably mention it" | **0463** |
| Forest edges | "much clearer but I found a new issue is that the road abruptly becomes very narrow when it touches a bridge for some reason." | **0461** (bug) |
| Cinder Oath | "She shouldn't counter with it either" | **0318** |
| Anything else | "lead feels a bit weak actually, usually lead is a bit strong." | **0846** |
| Anything else | "for this first ch1 fight probably the obj should be changed to just kill harl rather than all enemies cuz a straggler guard is not that important he should just run away after harl dies" | **0845** |
| Anything else | "anything that attacks should prio an enemy over terrain for its first hover" | **0462** |
| Sparks; threat arrows' look | "it looks fine for now. Probably before release we tune it"; "probably can be polished before final game release but these came out quite well for now." | **0908** ✱ (blocks the Steam release, 0903) |

**No ticket:**

| Asked | Nick said | Why |
| ----- | --------- | --- |
| Anything else | "ensure the threat arrows are accurate to if the enemies have obstacles (like our allied units moving to block them) and not just a simple range indicator." | They already are: an enemy's reach treats every unit of his as a wall, with the moving unit on its new tile (`threats_at`, `crates/core/src/movement.rs`). Written in `turn-structure.md`; told to Nick. |
| Old saves in Player 1 | "I never clicked save before... I have no data points for you." | Nothing to check. |

**Settled** (recorded in the design docs): the backdrops ("All fine"),
idle fade aside; the cuts "land"; units move "like the game now"; no
spoilers; the familiarity question's wording ("Clear") and place ("Both
right"); the lead-select screen unchanged; the yard ("Feels right");
Maud's eyes ("Fixed"); "Aye", "Greywater" and "Your Former Grace" kept;
Vix's voice ("Yes"); holding End turn during a lesson ("Right"); the
lesson box moving ("a really great feature that I didn't even ask for");
both short sets right; the box beside the panel ("Both fine"); the cursor
("much easier to see, great work"); the outline ("Clear") and the acted
rim ("Fine"); when the arrows show ("Right"); the danger zone used
"rarely, this is great"; Attack dimmed when only an art reaches
("Right"); skill descriptions ("Clear"); pointing to heal ("Works how I
wanted"); Tamsin ("Feels right now"); the lead and Aske doubling
("Fine"); the difficulty ("Right"); the lead's level-up face ("Yes");
battle lines over the live map ("Works"); Tamsin's news ("it's ok"); the
study ("Right"); Veyne Hall's music ("Fits"); the keep-changes prompt
("Right"); profile switching ("Yes"); Game Over ("Good"); the rebuilt
screens ("Everything looked normal").

### Folded into the Act 1 playtest (2026-10-21, ticket 0129)

**Closed without round 4.** Nick decided on 2026-10-21 that he no longer
spot-checks gameplay tickets: "any tickets that say 'check after merge'
aren't gonna happen -- I will save all my checking for either design
(i.e. 00 tickets) or for the full act 1 playtest itself. I don't wanna
spot verify any gameplay tickets anymore." So Chapter 1's round 4 and
the Playdate play-through (1050) became one playtest of all of Act 1
(Chapters 1–8) on the web and on his Playdate: **1064**. Its
`blocked_by` carries this ticket's still-open blockers (round 3's
tickets) over, minus 1118 (done) and 1050 (folded in too); every open
ticket that waited for 0804 now waits for 1064 or for nothing (0129's
PR lists which). Round 4's first question, the new script (0737), is
1064's first question.
