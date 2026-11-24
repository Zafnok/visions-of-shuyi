# Retro: why feedback takes more than a day to land (2026-11-24)

## 1. How it was run

This was a sweep-style retro about one thing: how long it takes for feedback to land. It covered three feedback rounds:
- **(a)** Nick's playtest on 11-08, ROADMAP Lane I.
- **(b)** Nick's itch.io play on 11-17, Lane K.
- **(c)** A new player's Chapter 1 session on 11-22, Lane L.

It used 10 agents: 6 lenses, 3 skeptics and 1 synthesizer.
- The 6 lenses were: a per-item timeline; PR lifecycle and CI; the game/engine/private split; per-ticket process overhead; waiting on design decisions; and case studies of small tuning changes.
- The 3 skeptics re-measured every finding against git, gh and the ticket files.

Everything was read-only. All times are UTC.

**Outcome counts:** 43 lens findings.
- 11 confirmed
- 20 weakened (their corrected scope is used below)
- 11 already covered by an existing ticket
- 1 refuted

After merging shared root causes, that leaves 7 themes, 14 new action items and 9 amended existing tickets.

## 2. Headline

- **The engine/assets/game split is a minor cause, not the main one.** Each time an item has to go through the engine, it costs hours, not days.
  - The worst case was the 150% range tint (1343): about 8.5 h from Nick's pick to landing. The engine PR itself took 54 minutes. The rest was waiting for the pin move and for merges.
  - Over 5 engine-backed items in two weeks, the batched pin moves cost about 30–40 h in total.
  - The private content repo cost about 0.5 h in total and was never the thing a feedback item waited on.
- **The biggest delay is design questions that were filed and then never asked.**
  - From the 11-08 playtest: 0083 waited 196 h, 0081 and 0080 waited about 314 h each, and 0079, 0082 and 0084 are still unasked after about 362 h.
  - From 11-22: 0095–0099 are unasked, and 9–10 build tickets wait on them.
  - Once asked, Nick answers within minutes to hours. 0081 went from question to merged in 6 minutes, and 0080 in 1 h 18 m.
  - The 11-17 retro already filed the fix (0184). Nobody has worked it.
- **Second: nothing merges between about 08:00 and 14:00 UTC.** Nick merges every PR himself.
  - A PR that goes green, or goes red, just before that window waits about 10 h.
  - That one window is why a fix that takes 1 h shows up the next day. Examples: 1312's PR #800 was green at 07:55 and merged at 18:01; 1319's PR #801 sat on a red mutants check from 08:12 to 18:03.
  - The wait is shared by every open PR. It does not add up per PR.
- **Third: work that is ready gets passed over.** None of these needs a decision, yet each has waited a day to two weeks:
  - 0485, 0488 and 0491: no PR in about 362 h.
  - 1311 (low-HP tip bug): about 46 h.
  - 0610 (the EXP cut): still marked "blocked" although all three of its blockers are done.
  - 0528: unstarted about 1 day after it was unblocked.
  - The 11-17 retro's fixes for this (0182 to rank ready work, 0147 to flag stale statuses) are also unworked.
- **Most of this feedback is not a 1-hour tune.** Of the 21 tickets from 11-22, only 2 were value changes (the range colours and the tint). The rest are new mechanics, like swimming, barricades, one-way doors, moving tiles and a bigger window, and they need Nick's decisions first.
  - Where the feedback was a bug or polish, needed no decision and needed no engine change, it landed fast. Lane K (11-17) had a median of 2.2 h from ticket to merge.
- **Balance numbers waited on a chain of bot-tooling work, and still run one at a time.**
  - The stat and EXP tickets were blocked about 337 h by 0527 and 0531.
  - Those are done now, but the rule that balance tickets run one at a time, each re-recording the bots' wins, still adds about 1 day per ticket.

## 3. Action items

| # | Proposed ticket | Bucket | Priority | Size | Nick? | Verdict | Corrected scope | Ticket |
|---|---|---|---|---|---|---|---|---|
| 1 | Merge on green, or a fixed daily merge window, for ticket-scoped PRs: ticket-only, docs-only, tuning and bug PRs with nick_input none. Merge only after every required check reports. | Process / merge | P1 | S | false (Nick's OK as a workflow choice, not a design question) | confirmed | About 10 h per item that misses Nick's session. The wait is shared, not added per PR. Feedback examples: #757, #800, #801, #802. | Declined by Nick (2026-11-24: he keeps merging); 0146 amended to ping on green and red |
| 2 | Drop the "balance tickets one at a time" rule now that 0531 has landed. Batch 0528 and 0610 (and 0870 once answered) into one re-record with a band report. Start 0528. | Balance chain | P1 | M | false | weakened | Real for 0528 and 0876, which need no decision: about 337 h blocked, then about 1 day unstarted. For 0610, 0870 and 0333, the decision wait was the bigger part. Going forward the rule costs about 1 day per ticket. | Done in this PR: the roadmap's one-at-a-time rule dropped, 0610 unblocked; 0876 amended |
| 3 | A session's last step waits for every required check, mutants included, and pushes the fix before it ends. Turn on the PR monitor when a PR opens. | Process / CI | P2 | S | false | weakened | About 9–10 h each on #801 and #802. This overlaps the merge window in item 1 and does not add to it. | 1351 |
| 4 | Once a feedback item's engine PR merges, its own game PR moves the pin to that merge commit right away, without waiting for a batched pin PR. Reconcile this with 1255. | Engine split | P2 | S | false | weakened | About 30–40 h saveable over 5 items: 0254 about 14 h, 0908 about 12 h, 1314 about 7 h, 1343 about 5 h, 1301 about 0 h. Not the 49–55 h first claimed. | 1255 amended |
| 5 | Clean-ups found by a sweep don't become blockers of feedback builds unless the build truly needs them. Unblock 0334 from 0335, or schedule 0335 now. | Ticket hygiene | P2 | S | false | confirmed | About 160 h on 0334 since its decision, and growing. | 0182 amended (0335 ranked next); the rule in 1352 |
| 6 | Build the part of Nick's words that is fully specified (0099's shallow-water half, 0097's breakable barricade). Keep one trimmed decision for the content questions: who swims, which battles, HP values. | Decisions | P2 | S | false | weakened | Saves days on 2–3 tickets, not 5. New mechanics still go through a decision. | 0184 amended (at most 3 questions, split before asking) |
| 7 | A decision goes into 1064's blocked_by only with a one-line reason why the playtest can't run without it. Ask 1323–1331 on one listening page in one sitting. Remove the done 1322 from 1064. | Decisions | P3 | S | false | confirmed | 1064 has 218 blockers (130 on 11-08). 29 of them are open Nick decisions, 9 from one request (#799). | 1352 (the reason line); 0184 amended (one listening page); 1322 removed from 1064 in this PR |
| 8 | Engine: when a skin is touched, expose its look constants (OVERLAY_BLEND, SHOWN_BLEND, GLINT_BASE and the like) as values the game sets, with engine defaults. | Engine ergonomics | P3 | S | false | weakened | Only 1343 is a clear case (about 1 h of ceremony; its wall-clock wait is counted in item 4). Engine #545 already fixed the tint. A full sweep of the kits is not justified. | Engine proposal, waits for Nick's OK (engine steps need it) |
| 9 | Engine tooling: reserve ticket, ADR and pack-format numbers so parallel engine PRs don't clash and get closed and reopened. | Engine split | P3 | S | false | weakened | About 1–2 h over 2 feedback items (#490→#496, #538→#541). | Engine proposal, waits for Nick's OK |
| 10 | Merge main into a branch only for a real conflict. Stop every filing PR from editing 1064's one-line blocked_by list (one file per entry, or generate the list). | Process / CI | P3 | S | false | weakened | About 5 h of CI wall time in total. Hygiene, not on the critical path. | 1352 |
| 11 | Decide 0090 (whether the writer's preview follows the demo's settings) as an agent ADR, not a Nick question. | Decisions | P3 | S | false | weakened | Only 0090. 0092 and 0093 stay with Nick, at low priority. None of them gates 1064. | 0184 amended (proposed to Nick) |
| 12 | Document the `cargo shuyi run --watch` palette workflow for settling colours with Nick live, before the PR. | Tuning | P3 | S | false | weakened | Only the documentation part. A debug tuning tool and EXP-as-data (L) are deferred: they would have saved at most one round, on 1319. | 1353 |
| 13 | Friction-log line, not a ticket: layout snapshots name a colour by its palette role instead of its hex value, so a palette tweak doesn't re-bless 15 files. | Tests | P3 | S | false | weakened | Review noise only, about 0 h of wall clock in this run. A follow-up to 1290. | Report only |
| 14 | `private-assets --pin` refuses a pin that still points at a private branch commit after the private PR squash-merged. | Private split | P3 | S | false | confirmed | Prevents a repeat of #780 (2 lines, 0.5 h). Optional. | Engine proposal (`private-assets` is the engine's tool), waits for Nick's OK |

### Amended existing tickets

- **0184** (decision queue). This is the top item. It was filed 11-18 and never worked. Add these:
  - The session that files a feedback round asks that round's decisions in the same sitting, or the next one.
  - A "decision inbox" step opens every session with Nick: ask everything that is ready and unasked, oldest first, before any build is picked.
  - Order the questions by how many builds each one unblocks.
  - At most 3 questions per decision; split bigger tickets before asking.
  - Fix ROADMAP row 6, which still shows 0081 as open and doesn't list Lane L's decisions.
  - Ask 0079, 0082, 0084 and 0095–0099 now.
- **0182** (rank ready work). Rank the current feedback round's ready tickets first. Today that is 1311, 1320, 0528, 0610, 0485, 0488 and 0491. Tag value-only tweaks when filing so they go first, in one PR per round.
- **0147** (ticket-lint). It already specifies an error for "status: blocked with every blocker done". Do it next. Right now this would catch 0610 and 1317.
- **0146** (green PRs don't wait unseen). Extend it to red PRs with no commit for 2 h.
- **0181** (lighter tickets). Add these:
  - For feedback that states the fix directly, the build PR carries its own new ticket and starts at once, with no separate filing PR.
  - A build blocked on a decision is filed as a stub.
  - A "tweak" class of ticket: 15–30 lines, several tweaks from one round in one PR with one before/after picture.
- **1255** (one PR moves the pin at a time). Reconcile with item 4: a feedback item's own pin move to its engine PR's merge commit comes before the batched pin.
- **0876**. Decide in the ticket whether 0528 really has to block the Ch3–5 retune, or whether to judge it against today's AI.
- **1287** (adoptions off the path). It already covers keeping adoption tickets behind open feedback. No change; just honor it.
- **0150, 0174, 0145** (mutants before push, CI by change class, fast gates). No change. They are low priority for this question: CI is 18–35 minutes per push and is not the bottleneck.

### The previous retro's tickets

The 11-17 retro and sweep filed the fixes this run found most needed,
and none has been worked: **0184** (the decision queue), **0182**
(ranking ready work), **0147** (stale statuses), **0146** (PRs that wait
unseen) and **0181** (lighter tickets). All five are amended above
rather than filed again. Nick, 2026-11-24: keep merging every PR himself;
ask the waiting decisions later, in a session of their own.

## 4. What went well

- **Lane K (11-17) shows the path works.** It needed no decision and no engine change. Median 2.2 h from ticket to merge: 1295 in 1.8 h, 1292 in 2.2 h, 1303 in 1.1 h.
- **Nick answers fast once asked.**
  - 0080's three questions were answered in one round (PR 1 h 18 m).
  - 0081 was merged 6 minutes after the question.
  - 1322's ambient sound took about 2 h over 3 listening rounds.
  - 1343's tint pick shipped about 1 h after the renders.
- **CI is short.** Median 23 minutes per push (p90 32). Most red runs come from the mutants check, not the main CI job (ci-result was red in 4 of 64 runs).
- **A PR is quick while Nick is around.** 31 of 60 PRs merged within 2 h; the median from green to merge is about 0.55–0.8 h.
- **Lane L's first fixes started soon after filing.** They opened within 1.4–2.5 h, and 1343 took 1 h 02 m from first commit to merge.
- **The engine path can be fast.** When a game PR moved the pin past its own engine PR, the engine hop cost about 1 h (#779 merged 47 minutes after engine #512).
- **The private split never held anything up.** Each private PR merged before its game PR, every time.
- **0531 (#797) took the bots' armies off single recorded wins.** That removes the old "one stat change re-records everything" cascade.

## 5. Themes

### T1. Design questions filed but never asked (largest cause)

**What happened**
- Lane I's six decisions were filed 11-09 03:18 (#592), with no blockers.
- They were answered at these points:
  - 0083 at 196 h, after 4 rounds and 7 rejected mockups
  - 0081 at about 314 h (decided 11-21)
  - 0080 at 315 h
  - 0079, 0082 and 0084: never asked, about 362 h now
- They gate 0332, 0333, 0610, 0867 and 0868, and through them 0512 and 1064.
- Lane L repeats it: 0095–0099, filed 11-23 06:11, none asked. They gate 1313, 1315, 1316, 1318, 0336, 0337, 0338, 0536 and 0880.

**The pattern**
- About 11 questions were asked across 16 days, while about 210 PRs merged.
- Asking happens in three clusters (11-16/17, 11-21/22, 11-23/24), one ticket per session.
- The 11-17 retro (T4) filed 0184 for exactly this. In the 7 days since, 2 of its 5 were asked and 0184 itself is still unworked.

### T2. The daily window with no merges

**What happened**
- Every PR from #746 to #813 was merged by hand by Nick.
- Merge hours: 45 in 00–06, 0 in 07–14, 24 in 15–23.
- Green-to-merge waits were 164 of 337 PR-open hours (49%). On feedback PRs:
  - #800: 10.1 h
  - #757: 9.7 h
  - #797: 11.7 h
- Red mutants runs left unattended fall in the same window:
  - #801: 08:12 to 18:03
  - #802: 08:49 to 18:09

**The pattern**
- Each item that misses the window loses about 10 h.
- That is the same 10 h shared by everything in flight, not a cost per PR.

### T3. Ready work not picked, and stale statuses

**What happened**
- 0485, 0488 and 0491 have had no PR in about 362 h.
- 0490 waited 212 h in the queue, then took 2.9 h as a PR. 0487 and 0489 waited 42 h each.
- 1311 is unstarted at about 46 h.
- 0610 has been "blocked" since 11-23, though 0081, 0527 and 0531 are all done.
- 1317 is "blocked" on 1314, which is done.
- After #798 merged, 5 of 11 merged PRs were not feedback work (#799, #806, #807, #809, #812).

**The pattern**
- 0182 and 0147, from the 11-17 retro, cover this and are unworked.

### T4. The balance chain

**What happened**
- 0870, 0610, 0528, 0876, 0332, 0333, 0867 and 0868 waited on 0527 and 0531.
  - 0527 sat 318 h in the queue, then 18.8 h as a PR.
  - 0531 was +8,287 lines and needed two picks by Nick.
  - Both merged by 11-23 03:53, about 337 h after the feedback.
- Since then, 0528 (no decision needed) has not been started.

**The pattern**
- The one-at-a-time re-record rule adds about 1 day per remaining ticket, roughly 7 days in total if kept.

### T5. Crossing into the engine (minor cause)

**What happened**
- 1343: Nick picked 150% at 20:09. Engine #545 ran 20:48–21:42, pin #808 ran 01:11–02:45 (25 engine PRs), and #813 merged at 04:58. About 8.8 h in total; the engine PR itself took 54 minutes.
- 1314: about 2–9 h waiting for #808, plus one commit for a clash with the engine's pack-format number.
- Gaps from an engine merge to the game's pin:

  | Item | Wait |
  |---|---|
  | 0254 | 16.3 h |
  | 0908 | 14.7 h |
  | 1301 | 9.0 h (all of it the merge window) |
  | 1314 | 7.2 h |
  | 1343 | 5.1 h |

- 1320's engine part has been in since 02:45, but no game session has picked it up. That is scheduling, not the split.

**Capacity**
- The engine merged 207 PRs from 11-09 to 11-24, against 199 for the game.
- 17 game PRs were pin moves and 17 were Lane J.
- The estimate that only about 24 game PRs (12%) were feedback work was not re-derived.

### T6. Process weight on each ticket

**What happened**
- #798 filed 21 tickets (+1,877 lines) over 7.5 h before the first fix opened. 9 of those were builds that can't start until a decision lands.
- #799 turned one request into 2 palette decisions, 8 per-chapter decisions and 11 build and data tickets, all added to 1064 (179 blockers went to 217 in one day).
- Decision tickets carry 3–12 questions, while ask-nick caps a message at 3.
- 0334 got a clean-up ticket from the 11-17 sweep (0335) put in front of it.
- Across 11 feedback PRs (6,388 lines): production code 23%, tests 30%, snapshots 24%, docs and tickets 19%.
- Mutants went red in 22% of runs.

**The pattern**
- These cost mostly agent time, not wall clock.
- The exceptions are the 7.5 h filing wait and the 0334 blocker.

### T7. The private content repo (no real cost)

- 15 game PRs moved assets-private.rev, and each private PR merged first.
- The only extra was one 2-line fix-up PR (#780).

## 6. Detail per action item

**1. Merge on green, or a fixed merge window (P1, S)**
- Evidence: 0 merges in 07–14 UTC over 69 PRs. #800 was green at 07:55 and merged at 18:01. #757 opened at 05:33 and merged at 16:59.
- Proposal: auto-merge whitelisted PRs once every required check has reported, with a `held` label to stop one. Or a fixed daily merge window.
- Skeptic notes:
  - Merging is Nick's standing rule, so this needs his OK as a workflow choice.
  - gh's auto-merge once merged with CI still pending, so the merge must wait for the checks explicitly.
  - ADR-0042 rejected a merge queue.
  - The cheaper fallback: sessions end with green PRs ready, and 0146 lists them.
  - Do not add the red-unattended hours on top of this; they are the same window.

**2. Drop the one-at-a-time balance rule (P1, M)**
- Evidence: the eight tuning tickets above, all blocked by 0527 and 0531. 0528 has been unstarted about 21–25 h since they landed.
- Skeptic notes:
  - The 337 h counts fully only for 0528 and 0876. For 0610, 0870 and 0333 the decision wait was the bigger part.
  - 0532 is about the intervals in the band report, so no existing ticket covers this; it is new.

**3. Wait for every check before ending a session (P2, S)**
- Evidence: #801 was red 08:18 and fixed 18:03. #802 was red 08:49 and fixed 18:09. Each fix was a 10–20 minute test commit. Mutants takes a median of 12 minutes, up to 44 minutes for a shard.
- Skeptic notes:
  - This saves the gap only if the session can push before Nick leaves.
  - #802's second red may have come from the engine's main moving.
  - #794 was not feedback work.

**4. The feedback PR moves its own pin (P2, S)**
- Evidence:
  - Engine-merge-to-pin waits are in T5.
  - #779 and #773 show the fast path: about 1–3 h.
  - #802 planned its own pin move and then waited for #808 anyway.
- Skeptic notes:
  - The first claim (a 16 h mean) measured how long engine-pin issues stayed open, not how long an item waited.
  - #770 would have saved nothing: it waited only because of the merge window.
  - This conflicts with 1255 as written.

**5. Sweep clean-ups don't block feedback builds (P2, S)**
- Evidence: 0334 has blocked_by 0083 and 0335. 0335 was added by #751 on 11-18 and has not been started. 0083 was decided 11-17.
- Skeptic notes: 0335 is the blocker, not a fix for it, so this needs a new process rule.

**6. Build what Nick's words already specify (P2, S)**
- Evidence: 0099 says its first half "is decided as it stands", yet it blocks 0336. 0097 has Nick naming barricades breakable.
- Skeptic notes:
  - Several of the remaining questions are real design calls (who swims, which battles, barricade HP).
  - Nick's rule against inventing player-facing defaults stands.
  - 0092's auto-suspend is a real design choice.
  - Lane K was bugs and polish, not new mechanics.
  - Narrowed to building only the fully specified parts.

**7. The 1064 entry test and one listening page (P3, S)**
- Evidence: 1064 has 218 blockers, 29 of them Nick decisions. 1323–1331 means 9 future sittings. 1322 is done but still listed.
- Skeptic notes: 0181 covers the prose cost but not this rule.

**8. Look constants become game-set values when touched (P3, S)**
- Evidence: 1343, and the constants OVERLAY_BLEND 0.75, SHOWN_BLEND 0.6, GLINT_BASE 0.55.
- Skeptic notes:
  - Engine #545 was 19 lines of code plus 25 of tests; the ticket file was 95 of its 139 lines.
  - 3223–3226 were features or layout fixes, not values a game could tune.
  - Shrunk from M to S.

**9. Engine number reservation (P3, S)**
- Evidence: 9 engine PRs were closed and reopened since 11-08.
- Skeptic notes: only 2 were on feedback (#490→#496, #538→#541). The other close reasons were not confirmed. The game side is covered by 0144 and 0131.

**10. Merge main only on a conflict, and the 1064 hotspot (P3, S)**
- Evidence: 31 merges of main in 19 PRs; 14 had no conflict. 1064 conflicted in #751, #757, #795 and #798.
- Skeptic notes: the 14 were not re-counted, and #802's merges were needed for the pin.

**11. 0090 becomes an agent ADR (P3, S)**
- Skeptic notes: 0092 and 0093 are player-facing and stay with Nick. None of the three gates 1064, so the cost to the feedback is near zero.

**12. Document the `run --watch` palette workflow (P3, S)**
- Evidence: hot reload covers only palettes and .dlg scenes. No debug tool tunes a value.
- Skeptic notes: only 2 of the 21 tickets from 11-22 were value changes, and Nick does not spot-check gameplay. Defer the tuning tool.

**13. Colour by palette role in snapshots (P3, friction-log line)**
- Evidence: #801 re-blessed 15 snapshot files (not 18) for a 4-file palette change.
- Skeptic notes: none of #801's extra commits came from snapshots. About 0 h of wall clock.

**14. Squash-pin guard (P3, S)**
- Evidence: #780.
- Skeptic notes: optional.

## 7. Dropped

- **"Platform infrastructure was built before the first animation"** (refuted). The animations 1315, 1316 and 1318 wait on decision 0096, so shipping an effect first would not have made anything move sooner. #802's own work was done by 18:09; the rest was the pin wait.
- **"About 1,900 and 1,100 item-hours" totals.** These summed waits that ran at the same time. Replaced with wall-clock figures.
- **"Engine split costs 8–24 h on 3 of Lane L's 4 shipped items."** It is one clear case (1343) and one partial case (1314). 1320's wait is scheduling.
- **"Merged engine fixes wait 16 h on average."** That was how long the issues stayed open; corrected to 30–40 h saveable over 5 items.
- **"22 h of merge waiting in round (c)."** This counted #800 and #801 twice inside one window.
- **"Longer decision tickets get picked later."** All five Lane I decisions waited because none was asked; 0080 had 3 questions and still waited 315 h.
- **"About 12 non-feedback PRs after #798."** It was 5; 0527, 0531 and 1310 merged before #798 and were blockers on the feedback.
- **"One stat tweak cascades into a retune chain"** as a new finding. Retro T2 and 0531 already fixed this; only 0876's 0528 blocker remains (amended above).
- **M-sized sweep of every engine kit for look constants, and the L tuning tool with EXP as data.** Not supported by the evidence; deferred.
- **The ritual per ticket as a wall-clock cause.** #813 went from creation to merge in 1 h 02 m including CI. It is a small dev-time cost, covered by 0145 and 0181.
