---
id: "0184"
title: "Nick's questions come in the order the Act 1 playtest needs, each option with its build cost"
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0184 — Nick's questions come in the order the Act 1 playtest needs, each option with its build cost

## Context

The retro of PRs #616–#735 ([report](../../docs/research/retro-2026-11-17-prs-616-735.md),
item T4, confirmed) found three gaps in how design questions reach Nick:

- **Order.** The five Lane I decisions (0079, 0080, 0081, 0082, 0084) were
  filed on 2026-11-08 (#592) with no blockers and were never asked. Each
  gates a balance ticket (0332, 0333, 0610, 0867, 0868) that 0512 and 1064
  wait on. Meanwhile Nick answered 0078, 0083, 0250, 0480, 0481 and 0864,
  and five more decisions were filed (0085–0089) that the playtest doesn't
  need. `docs/ROADMAP.md`, *Nick's queue*, row 6 lists the five among a
  dozen others, under "any order within a row".
- **Cost.** No option says what it takes to build. 0250's key-cap pick
  (#710) became about 6,400 engine lines (engine #456, #463, #468, #477)
  and a new 1064 blocker, 0254.
- **Cross-checks.** #729 (0083) recorded "Tamsin 5 → 7" Spd on foot in
  `docs/design/terrain.md` (~:147) and `tickets/open/0334-*.md` (~:54).
  A few hours later #731 (0864) made her base Spd 6, Def 6. Both files
  still say 5 → 7.

The `ask-nick` skill (`.claude/skills/ask-nick/SKILL.md`) has no step for
any of the three. Nick does not take technical questions, so a cost shown
to him is in design terms: how big the job is and whether the playtest
waits longer, never lines or engine tickets.

2026-11-17 friction sweep ([report](../../docs/research/friction-2026-11-17.md),
*Recommended order* item 3; bucket B2 with 0773): the questions below
come **before** 0079–0084 in the queue's first row. Two findings: the
writer hire gates 13 open tickets and the playtest but sits in no queue
(0184 never named 0766, 0042 or 0077), and Nick's setup steps aren't
batched with his decisions (10 setup tickets on the chain: 0766,
0858/0861/0862/0866, device runs 1051/1057/1059/1068, 0512). Sibling
tickets from the same sweep that these answers feed: 0877 (Act 1
coverage report), 0878 (class art gaps), and the amended 1064, 0606,
0757, 1067.

**Amended by the [feedback-latency retro](../../docs/research/feedback-latency-2026-11-24.md) (2026-11-24): the top item.** Filed questions
wait far longer than Nick takes to answer them: from the 11-08 playtest,
0081 and 0080 waited about 314 h and were answered in 6 min and 1 h 18 m
once asked; 0079, 0082 and 0084 are still unasked after about 362 h;
0095–0099 (the 11-22 session) are unasked with 9–10 builds behind them.
Add to the scope: the session that files a feedback round asks that
round's decisions in the same sitting or the next; a "decision inbox"
step opens every session with Nick (everything ready and unasked, oldest
first, before any build is picked), ordered by how many builds each
unblocks; at most 3 questions per decision ticket (split bigger ones
before asking); 1323–1331 asked together on one listening page; a
decision goes into 1064's `blocked_by` only with a one-line reason the
playtest can't run without it; propose to Nick that 0090 (the writer's
preview following the demo's settings) is decided by an ADR instead
(he may say no). Fix ROADMAP row 6 (0081 shown open; Lane L's decisions
missing). Nick, 2026-11-24: ask the waiting ones "later", in a session of
their own, not right after the retro.

## Nick input

**Decision** (`ask-nick`), one short message at the end: the reordered
queue in plain words, with the first bundle and each option's cost, and
one question: "Take them in this order, or your own?" Recommend this
order. His answer sets the queue's order. It does **not** answer 0079–0084:
each is asked and recorded in its own ticket's session.

### The first bundle (2026-11-17 friction sweep)

Each question below goes in the queue with its facts and the cost of
each option (step 3's scale), options plus "describe your own", and **no
recommendation**: these are Nick's calls. Re-check each number against
`tickets/open` on the day it is asked. If Nick answers one in his reply,
quote his words, dated, in the home ticket named; the design write-up
stays with that ticket.

1. **The writer hire (0766): a date, or a playtest without it?** Facts:
   all 9 of 0766's blockers are done; it waits only on Nick (its six
   decisions, then setup). 13 open tickets wait on it: 8 list it in
   `blocked_by` (0085–0088, 0772, 1073, 1083, 1084) and 5 say "after the
   writer's rewrite" (0719, 0726, 0755, 0756, 0757). 1064 lists 1073,
   1083 and 1084, so the playtest already waits on the hire. Today
   Act 1's support pairs and camp events are empty tables. Options:
   - **Set a date** for 0766's ad and pick. What it takes: small to
     start (0766's own session); the playtest waits longer: yes, until
     the writer delivers 1073, 1083 and 1084's scripts.
   - **Take 1073, 1083 and 1084 off 1064** and play Act 1 on the current
     script, without supports, camp events or the Hollis, Rue and Maud
     side quests; they follow the hire. What it takes: small (ticket
     edits); the playtest waits longer: no.
   Home: 0766 (the date) or 1064 (the blocker list).
2. **Voices after Chapter 1 for the playtest?** Facts: 123 clips (96
   Chapter 1, 27 the guide), about 8% of about 1,140 voiceable lines;
   Chapters 2–8 and both side scripts have none, and a missing clip is
   silence (`docs/voice.md`). Neither 0077 (the voices after Chapter 1)
   nor 0757 (making them) is on 1064. Options:
   - **Chapter 1 voiced, the rest silent.** Small; waits: no.
   - **Voice Chapters 2–8 on the current script**: 0077's listening
     round, then 0757; lines the writer rewords are made again later by
     the same tool. Large; waits: yes.
   - **Voice them after the writer's rewrite**: waits on question 1.
   Home: 0077, then 0757.
3. **Does Act 1 have promotion, and if so where do seals come from?**
   Facts: promote and reclass are built (the rules and the screen,
   touched by 18 tickets) but open only from the debug menu. Three seals
   exist (tier 2, tier 3, reclass), and no shop, battle, chest, world map
   node or chapter gives one. 0606 adds the menu entry but leaves seal
   sources out. `docs/design/progression.md` has "a separate seal per
   tier, rarer at higher tiers" and no source. Options:
   - **Yes**, with seals from shops, chests, story rewards or battle
     rewards (0606's own *Out* list), or a mix. Medium (0606 plus seal
     sources in the data); waits: yes, a little (0606 is already on
     1064).
   - **No promotion in Act 1**: 0606 and 1067 drop it. Small; waits: no.
   Home: 0606 (its *answer-first* question).
4. **Is the Playdate part of the 1064 gate, or a later pass?** Facts:
   Nick put it on the gate (ROADMAP, 2026-10-21, ticket 0129). 14
   Playdate-only tickets sit on 1064 (0512, 1046, 1051, 1057, 1058, 1059,
   1060, 1062, 1067, 1068, 1069, 1075, 1076, 1178), 4 of them device runs
   that need Nick's Playdate (1051, 1057, 1059, 1068). 9 of 21 screens
   have no one-bit look yet (camp, class change, credits, files, game
   over, preparations, results, save, save import). Options:
   - **Keep it on the gate** (as now). Nothing to change; waits: yes,
     for those 14.
   - **Web first, the Playdate pass after**: the 14 move to a later
     Playdate play-through. Small (ticket edits); waits: no.
   Home: 1064.
5. **Art for the Swordsman, the Shadowblade and indoor, fort and bridge
   battle backdrops.** Facts: of about 250 placed units, 44 Swordsmen, 9
   Brawlers and 1 Shadowblade (Dace) draw the fallback figure in the
   default Pictures look. The Brawler's art was decided (`martial03`) and
   0878 imports it; the Swordsman is missing from look-and-feel's
   *Fighters with no Tiny Tales art* table. Backdrops exist only for
   mountains, so the indoor maps (Greywater's kitchen, Veyne Hall, the
   envoy's house) show an open field, and so does every fight on a fort
   or a bridge tile, Chapter 1's boss fight included (Harl stands on the
   fort). 2 of the 8 bought backdrop sets are imported and none is
   indoors. The indoor, fort and bridge backdrops are asked as one
   question. Options for each (picks from art already owned, a new
   free or bought pack, or the fallback for the playtest; for the
   Shadowblade, also Dace's own art): small for owned art, medium for a
   new pack search; waits: no for the fallback. Home: 0878.
6. **The four portrait setups, as one batch** (setup, not a decision:
   the picks are 0070's and 0076's). Facts: 20 speakers placed on screen
   have no portrait; Joss appears in 9 of the 10 scripts. Steps, with a
   time estimate for each (the session times the first and corrects the
   rest): 0881 (split from 0858), a second look at Hendry's combat picture,
   then buy one pack and build his face; 0861, build the
   prior's and Ned's faces; 0862, buy one $3 pack and build Gil's face;
   0866, download one free pack and build Hedda's and Ragna's faces. Six
   generator faces in all. Home: the four tickets.
7. **Park the post-launch decision tickets?** Facts: 0046–0054 and
   0085–0088 are 13 tickets, 1,677 lines, and gate nothing on 1064; the
   queue's rows 7, 9 and 10 already put them after the playtest. Options:
   **park them** (marked so ticket sessions skip them until Nick reopens
   one; 0181 defines the marker): small, waits: no; or **leave them** as
   now. Home: each ticket, through 0181's marker.
8. **0042 (the Japanese font, picking a language, the label), after
   0766.** Facts: 0042 has no blockers and gates 5 open tickets directly
   (0045, 0236, 0718, 0825, 0907); the Japanese translation (0719) waits
   on the writer anyway. Options: ask it right after question 1, or
   later. Small; waits: no (not on 1064's chain). Home: 0042.
9. **Does the Playdate speed work (1057, 1068, 1078) wait for 1170?**
   Facts: 1170 moves the game onto the engine's Playdate backend (ready
   and pinned since 11-10) and deletes much of `crates/playdate`, the
   code those three tickets would speed up. Options: **1170 first**
   (no speed work lost; the device runs wait one ticket) or **speed work
   first** (on the current path's order; part of it redone after 1170).
   A ranking call, not a design one; asked only if Nick wants a say.
   Home: 0182's order.
10. **The save screens' own keys (0259).** Rebinding End turn also moves
    Import today. 0259 asks which default keys the save screens' file
    actions get. Low: it gates nothing on 1064. Home: 0259.

**Setup and decisions in one weekly queue.** Each week's message to Nick
lists the decisions above and the setup steps that are ready, each with
a time estimate. The ready setup steps come from 0182's ranked list (it
already marks `setup` tickets); this ticket only adds them to the
message and the queue, it doesn't rank them again.

## Scope

**In:**
- Rank the open questions on the 1064 chain and rewrite *Nick's queue*.
- A cost line for each option of 0079, 0080, 0081, 0082 and 0084.
- The first bundle above (2026-11-17), ahead of 0079–0084 in the first
  row, each with its facts and option costs.
- Four new rules in the `ask-nick` skill.
- The stale Tamsin numbers in `terrain.md` and 0334.

**Out (do not do):**
- Answering 0079–0084 or the first bundle, or recording any design
  answer beyond quoting Nick's words in the home ticket.
- Ranking setup tickets: 0182 does it.
- A `cargo xtask` command for the ranking. 0182 ranks ready work on the
  1064 chain but only counts decision tickets. Either order: if 0182
  or 1287 is done, use `trpg_engine_watch::act1_chain`; rank the
  decisions with a throwaway script (never committed) and say so in the
  skill.
- 0254's milestone. `M1 Engine` matches its `02xx` block, as with 0250.
- Lane I's build order: the ROADMAP and 0528's and 0876's
  `blocked_by` set it (retro T2); this ticket doesn't change it.

## Implementation steps

1. **Rank.** For each open ticket with `nick_input: decision` (00xx and
   questions inside other tickets, such as 0870), count the open tickets
   in 1064's `blocked_by` closure that wait on it, directly or through
   others. On 58176da0: 0079–0084 gate 3 each (their balance ticket,
   0512, 1064), 0028 gates 2 (0473, 1064), 0074 gates 1 (1064), and
   0085–0089 gate none. Break ties by Lane I's build order, so 0080 and
   0081 come first.
2. **Bundle.** Group questions on the same units or numbers. Their own
   *Out* sections already link them: 0080 with 0081 and 0079 (who hurts a
   guard, and the EXP it earns), then 0082 with 0084 (Chapters 2 and 3's
   maps).
3. **Cost each option** of 0079, 0080, 0081, 0082 and 0084. Add one line
   under each option in its ticket's *Nick input*:
   `What it takes: small | medium | large; the playtest waits longer: yes | no`.
   Small means data only, plus the bots' re-recording. Medium means a new
   rule or menu line in one ticket. Large means a new screen, engine work
   or a new ticket 1064 waits on. Judge it from the building ticket
   (0332, 0333, 0610, 0867, 0868) and the code it names.
4. **Rewrite *Nick's queue*** in `docs/ROADMAP.md` (~:526). A new first
   row holds the questions that gate 1064, in step 1's order and step 2's
   bundles, led by *The first bundle* (questions 1–8, each with its home
   ticket). Give every row a *Setup* line: the ready setup steps from
   0182's list, each with a time estimate. 0085–0088 already wait in
   row 10; move 0089's mention from row 6 to that row. Leave the done rows alone.
5. **`ask-nick` skill**, a new section before *Format for each question*:
   - *Pick what to ask*: rank as in step 1 and ask the top bundle first.
     A question that gates nothing on the 1064 chain waits, unless Nick
     raises it.
   - *What each option takes*: the cost line from step 3, in design words
     only.
   - *Cross-check before recording*: grep `tickets/open`, `docs/design`
     and the tickets merged in the last two days for the same unit, stat
     or rule. Fix numbers that a newer answer made stale. Ask Nick again
     only when the newer answer changes what he picked.
   - *One weekly queue*: decisions and ready setup steps go in one
     message, each setup step with a time estimate; an owner action that
     gates open tickets (as 0766 did) is never left out of it.

   Then run `pwsh -File scripts/sync-agent-skills.ps1` and its `-Check`.
6. **Tamsin.** In `terrain.md` (~:147) and 0334 (~:54), change the
   example to her current base Spd from `characters.ron` (0864: 6, so
   6 → 8). The Spd +2 rule is unchanged. Keep the *asked with* sentence
   (~:125, 15 a hit mounted, 8 on foot) as the record of what was asked.
   Add a dated line under it with the numbers at Def 6. If the trade no
   longer reads the same, add it to step 7's message.
7. Send the message in *Nick input*, then record his order in the queue.

Expected size: about 250–350 changed lines plus this ticket's own file.

## Acceptance criteria

- [ ] *Nick's queue*'s new first row lists the questions that gate 1064
      in the order Nick accepted (or his own), led by the first bundle
      (questions 1–8) with facts, option costs and home tickets, and
      0085–0089 sit in a later row (or are parked, if Nick says so).
- [ ] The queue's rows carry the ready setup steps, each with a time
      estimate.
- [ ] Each option of 0079, 0080, 0081, 0082 and 0084 has a *What it
      takes* line.
- [ ] `ask-nick` has the four rules, and `sync-agent-skills.ps1 -Check`
      passes.
- [ ] `git grep -n "5 → 7" docs/design tickets/open` finds no Tamsin Spd
      line.
- [ ] `cargo xtask ticket-lint` passes. All gates in the `run-gates`
      skill pass.

## Tests required

None (process and docs). `cargo xtask ticket-lint` and the skill sync
check.

## Completion notes

*(Filled in by the session that completes the ticket.)*
