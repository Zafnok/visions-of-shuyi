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

## Nick input

**Decision** (`ask-nick`), one short message at the end: the reordered
queue in plain words, with the first bundle and each option's cost, and
one question: "Take them in this order, or your own?" Recommend this
order. His answer sets the queue's order. It does **not** answer 0079–0084:
each is asked and recorded in its own ticket's session.

## Scope

**In:**
- Rank the open questions on the 1064 chain and rewrite *Nick's queue*.
- A cost line for each option of 0079, 0080, 0081, 0082 and 0084.
- Three new rules in the `ask-nick` skill.
- The stale Tamsin numbers in `terrain.md` and 0334.

**Out (do not do):**
- Answering 0079–0084, or recording any design answer.
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
   bundles. 0085–0088 already wait in row 10; move 0089's mention from
   row 6 to that row. Leave the done rows alone.
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

   Then run `pwsh -File scripts/sync-agent-skills.ps1` and its `-Check`.
6. **Tamsin.** In `terrain.md` (~:147) and 0334 (~:54), change the
   example to her current base Spd from `characters.ron` (0864: 6, so
   6 → 8). The Spd +2 rule is unchanged. Keep the *asked with* sentence
   (~:125, 15 a hit mounted, 8 on foot) as the record of what was asked.
   Add a dated line under it with the numbers at Def 6. If the trade no
   longer reads the same, add it to step 7's message.
7. Send the message in *Nick input*, then record his order in the queue.

Expected size: about 150–250 changed lines plus this ticket's own file.

## Acceptance criteria

- [ ] *Nick's queue*'s new first row lists the questions that gate 1064
      in the order Nick accepted (or his own), and 0085–0089 sit in a
      later row.
- [ ] Each option of 0079, 0080, 0081, 0082 and 0084 has a *What it
      takes* line.
- [ ] `ask-nick` has the three rules, and `sync-agent-skills.ps1 -Check`
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
