# Tickets

The backlog lives here as Markdown files. Full rationale:
[ADR-0010](../docs/adr/0010-ticket-workflow-and-model-routing.md).

```
tickets/
  README.md     ← you are here
  TEMPLATE.md   ← copy for new tickets (see the write-ticket skill)
  open/         ← todo / in-progress / blocked
  done/         ← archived; moved here by the PR that completes the ticket
```

## Numbering

`NNNN-short-kebab-title.md`. The hundreds digit is the block:

| Block | Milestone | Notes |
| ----- | --------- | ----- |
| `00xx` | **Design decisions (Nick)** | Questions with options; answers go to `docs/design/` |
| `01xx` | M0 Foundation | Workspace, CI, scanners, SonarCloud, mutation, release plumbing |
| `02xx` | M1 Engine | Glyph buffer, font, input, screens, web build, storage |
| `03xx` | M2 Core rules | Content loading, map, units, movement, combat, turns, items |
| `04xx` | M3 Battle UI | Map view, cursor, move/attack flow, forecast, info screens |
| `05xx` | M4 Enemy AI | |
| `06xx` | M5 Progression | EXP, level up, class change |
| `07xx` | M6 Story & dialogue | Story bible, dialogue engine, portraits, Chapter 1 script |
| `08xx` | M7 Chapter 1 & game flow | Flow, save/load, Chapter 1 map, playtest, options |
| `09xx` | M8 Release | itch.io, Windows polish, Steam |
| `10xx` | Post–Chapter 1 | Created as needed |
| `11xx` | **The engine** | Shuyi Engine, the open-source engine split from the game ([ADR-0063](../docs/adr/0063-an-engine-and-a-game-one-boundary-now-two-repositories-later.md), [`docs/engine/`](../docs/engine/README.md)). Here: 1101 (the study), 1102–1118 and 1154 (the carve), 1131 (the code moves), 1151–1153 and 1157 (living with two repositories: engine friction, the engine-change skill, the first sweep, moving the engine parts of open game tickets), 1158–1172 (what the first sweep filed), 1180 (the game's playtesting on the engine's, from the sweep of 0506's playtest area). 1119–1129, 1132–1135 and 1140–1150 moved to the [engine's repository](https://github.com/Zafnok/shuyi-engine/tree/main/tickets) with their numbers. Not on the Act 1 critical path, except 1177 (the pin move 0449 and 0740 wait for) |
| `12xx`+ | Later | Created as needed |

New tickets (including bugs) take the next free number in their block. Numbers
are never reused.

## Frontmatter

```yaml
---
id: "0303"
title: Movement range and pathfinding
type: feature        # feature | infra | design-decision | content | bug | tuning | research | playtest
milestone: M2 Core rules
model: opus-5.5      # haiku-4.5 | sonnet-5 | opus-5.5 | fable-5.1
effort: high         # low | medium | high | xhigh | max
status: todo         # todo | in-progress | blocked | done
blocked_by: ["0302"] # ticket ids that must be in done/ first; engine#NNNN for the engine's
nick_input: none     # none | decision | answer-first | setup | sign-off
completed:           # YYYY-MM-DD when done
---
```

A ticket of Shuyi Engine's repository is written `engine#NNNN` in
`blocked_by` (for example `["0302", "engine#2007"]`), the mirror of the
engine's `game#NNNN`. It is done when it is in the engine's
`tickets/done/` and the game's pin is at or past the commit that closed it
(`work-ticket` §1). `cargo xtask ticket-lint` checks only its form (four
digits) so the gates never need the network; `cargo xtask ticket-lint
--check-engine` also looks it up, in `shuyi-engine/tickets/` when the local
checkout is there and on GitHub otherwise. How such tickets come about:
the `engine-change` skill.

## Picking the next ticket

A ticket is **ready** when `status: todo` and every id in `blocked_by` has a
file in `done/` (an `engine#NNNN`: as above). Prefer tickets on the Act 1 critical path in
[`docs/ROADMAP.md`](../docs/ROADMAP.md) (*Act 1 critical path*). Tickets without dependencies on each
other can run in parallel sessions (use separate git worktrees).

## Working a ticket

Use the **`work-ticket`** skill. Summary: branch `t<NNNN>-<slug>` → implement
within scope → run gates → fill in Completion notes → `git mv` to `done/` →
PR titled `[NNNN] Title`.

**The one exception:** a GitHub issue labelled `sonar` is its own ticket.
`tools/sonar-triage/` files SonarCloud's findings as batched issues every
week; each is fixed in one PR on a `sonar/<slug>` branch with the
**`sonar-fix`** skill, with no ticket file
([ADR-0071](../docs/adr/0071-sonar-backlog-becomes-batched-github-issues.md)).

## Nick input legend

- `none`: no Nick involvement.
- `decision`: this ticket *is* a question for Nick (the `00xx` block, run with the `ask-nick` skill).
- `answer-first`: blocked on a `00xx` design decision.
- `setup`: Nick must create an account or secret; the ticket lists exact steps.
- `sign-off`: Nick plays or judges the result himself. Since 2026-10-21
  only for playtests (1064, the Act 1 playtest) and non-gameplay
  calls that are his (0906, the title's trademark), never for a
  gameplay ticket (below).

### No checks after merge on gameplay tickets

Nick, 2026-10-21: "any tickets that say 'check after merge' aren't gonna happen -- I
will save all my checking for either design (i.e. 00 tickets) or for
the full act 1 playtest itself. I don't wanna spot verify any gameplay
tickets anymore."

So a `feature`, `bug`, `tuning` or `content` ticket never asks Nick to
look at or play its result. It is verified by tests, `cargo shuyi
frame-png` pictures in the PR and the playtest bots (0506, 0509, and
for all of Act 1, 0512). Its *Claude's starting rules* and anything
left to judge by playing go in its Completion notes, and the Act 1
playtest (1064) collects them into its checklist.

What Nick still answers: design questions, which are `decision`
(a `00xx` ticket, or a question inside a ticket asked **before**
building, with mockups when he judges pictures: a node-glyph mockup,
a pick from bought art, a listening page), `answer-first` and
`setup` (an account, a purchase, sideloading a build to measure it
on his Playdate). Story: Claude writes every script; only story
decisions (a beat, a new named character) go to him.
