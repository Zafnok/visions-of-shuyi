---
id: "0137"
title: Correct model routing with explicit Claude and OpenAI pairs
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-10-31
---

# 0137 — Correct model routing with explicit Claude and OpenAI pairs

## Context

Nick corrected the merged portability migration on 2026-10-31. Its generic
reasoning category suggested Astra for work originally recommended for Opus.
He requested specific models on every ticket line, with both providers:
Fable/Astra, Opus/Sol, Sonnet/Terra and Haiku/Luna. Game ticket 1008 must
recommend Opus/Sol, not Astra. The independent companion is engine#2038;
neither correction needs the other merged or an engine pin move.

## Nick input

None. The model pairs and correction scope are Nick's explicit instruction.

## Scope

**In:** all open, parked and done ticket recommendations; templates,
ticket schema and tests, active guidance, shared instructions, owned skills
and generated issue guidance. Preserve shared skill loading and the game's
exact copies of the pinned engine's skill pack.

**Out (do not do):** gameplay, assets, engine APIs, engine pins, account
settings, new benchmarks, or implementation of a parked ticket. The
companion correction is independent and leaves no work to this ticket.

## Implementation steps

1. Translate each category to its exact model pair; preserve all other
   metadata and ticket scope.
2. Accept only the four approved pairs; reject generic labels, single
   model ids and mismatched pairs.
3. Correct current guides, templates, instructions, skills and generated
   recommendations. Mark old routing decisions as amended.
4. Audit all ticket metadata against the branch base, verify 1008 uses
   Opus/Sol with high effort in the game, and run relevant gates.

## Acceptance criteria

- [x] Every ticket names its correct Claude/OpenAI pair on the model line.
- [x] All other metadata, scope and lifecycle fields stay unchanged.
- [x] High-effort Opus work recommends Sol rather than Astra.
- [x] Lint accepts the four pairs and rejects ambiguous or wrong mappings.
- [x] Templates, skills, instructions and generated guidance agree.
- [x] Skill loaders remain current and relevant gates pass.

## Tests required

- Unit: approved pairs, incorrect pairings, generic labels and single ids.
- Audit: compare all original ticket YAML fields with the branch base.
- Integration: ticket lint, docs, links, skill loaders and affected tooling.

## Completion notes

Corrected all 567 existing ticket recommendations and the template to
explicit Claude/OpenAI pairs. Every one of the 329 former reasoning
tickets recommends opus-5.5 / gpt-6.1-sol; the 215 implementation tickets
recommend sonnet-5 / gpt-6.1-terra. The 23 creative tickets recommend
fable-5.1 / gpt-6-astra. The schema also supports haiku-4.5 / gpt-6-luna
for mechanical work. No other metadata, effort or lifecycle field changed;
ordinary ticket bodies were compared with the base revision too.

1008 now explicitly recommends Opus/Sol at high effort, not Astra.
All 23 Fable/Astra tickets were compared with the pre-import revision
1b48c306aa63b8e7515111c106df928f365328ec: every one already recommended
Fable. Fifteen are done, including the Chapter 1-8 scripts. The eight open
ones are 0718, 0719, 0725 and 0726 (translation), 1072, 1083 and 1084
(side-quest writing), and 1073 (supports and camp events). No coding ticket
was promoted to Fable/Astra, and the engine has no Fable/Astra tickets.

Updated shared instructions, ticket-writing, story-writing, the historical
research's current recommendation, engine-pin's skill and generated issue
text. Both harnesses still use the same shared skills; the 16 pinned
engine skills remain exact. ADR-0103 amends ADR-0102's routing without
changing its shared instruction/discovery layout. Prior routing notes
are marked as historical. No public API or gameplay change.

Validation: full native workspace tests; 398 xtask unit tests and seven
integration tests, plus 28 engine-watch tests; fmt, workspace Clippy,
warning-denied rustdoc, dependency/unused-crate gates, ticket lint,
spelling, pinned skill-pack check, all 29 skills and loader drift checks.
The schema test accepts the four correct pairs and rejects generic labels,
single-provider ids and mismatches, including Opus/Astra. Mutation testing
remains in CI. No runtime, template, pin, asset or public API changes need
new visual or worked-example checks.

Engine friction: none, no gameplay production change (one generated issue
recommendation string). PR size: estimate 1,700 changed lines; this is one
atomic backlog/schema correction. No new gameplay starting rules and no
Nick check after merge. Worked in a fresh worktree from merged main; the
original checkouts remain clean. The independent engine half is engine#2038;
no pin move or merge order is needed.
