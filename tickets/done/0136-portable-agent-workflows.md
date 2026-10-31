---
id: "0136"
title: Make tickets and agent workflows provider neutral
type: infra
milestone: M0 Foundation
model: reasoning
effort: high
status: done
blocked_by: []
nick_input: none
completed: 2026-10-31
---

# 0136 — Make tickets and agent workflows provider neutral

## Context

Nick requested this migration across the engine and game on 2026-10-31.
Existing ticket recommendations and agent roles assume one provider.
The companion migration is engine#2037; both repositories can adopt the
workflow independently without an engine pin move.

## Nick input

None. Nick authorized all ticket rewrites and shared harness support.

## Scope

**In:** all open, parked and done ticket recommendations and agent roles;
ticket templates, lint and routing guidance; shared instructions; portable
skills and discovery in Claude Code and Codex; checks for loader drift.

**Out (do not do):** game design, engine APIs, pin changes, new model
benchmarks, provider account settings, starting any parked implementation.
Historical comparisons, quotations and real integration paths keep their
meaning. The companion migration needs no output from this ticket.

## Implementation steps

1. Map old recommendations to mechanical, implementation, reasoning and
   creative capabilities, preserving effort and lifecycle fields.
2. Update the template and ticket lint; test supported capabilities and
   rejection of provider model ids.
3. Use AGENTS.md for shared instructions and a CLAUDE.md import. Remove
   provider-specific actor and execution requirements from owned skills.
4. Generate Codex discovery loaders pointing at the existing shared skill
   sources; check drift locally and in CI. Preserve pinned engine skills.
5. Document the decision and verify the migrated backlog and loaders.

## Acceptance criteria

- [x] Every ticket uses a provider-neutral capability recommendation.
- [x] Existing ticket scope, lifecycle fields and blockers are preserved.
- [x] Both harnesses discover the same workflows and shared instructions.
- [x] Ticket lint accepts capabilities and rejects provider model ids.
- [x] Shared skill loader checks detect missing or outdated loaders.
- [x] Relevant checks pass; any environment limitations are recorded.

## Tests required

- Unit: routing schema accepts all capabilities and rejects model ids.
- Integration: lint the complete migrated backlog and validate skill loaders.
- Docs: spelling and links (and the engine's docs map).

## Completion notes

Migrated all 566 existing tickets, plus the template and backlog guide.
An independent YAML audit compared every lifecycle field, effort and blocker
with main. Recommendations use mechanical, implementation, reasoning or
creative; ticket lint rejects provider model ids. Optional model examples
live in tickets/README.md. Historical quotations and test comparisons remain
evidence, not execution requirements. Story-writing now uses the creative
capability, and engine-pin issues recommend reasoning rather than a model.

Moved the shared rules to AGENTS.md with a CLAUDE.md import. Twenty-nine
Codex loaders point to the same workflows and resources, including the
engine's 16 copied skills, which remain exactly at the pin. Loader metadata
is generated, with angle brackets omitted from discovery descriptions for
Codex validation; no pinned source was edited. All loaders and owned skills
pass skill validation. Isolated checks exercised missing/stale loaders,
metadata drift, body-only updates and unmanaged-file protection. CI and
run-gates check drift; pin updates regenerate the loaders. ADR-0102 amends
ADR-0010's routing and entry point. No gameplay or public engine API change.

Validation: full native workspace tests; fmt, workspace and Playdate Clippy,
rustdoc with warnings denied, web and no-std builds, dependency and workflow
audits, unused dependencies, spelling, input/text checks, pin/pack checks,
skill validation and ticket lint. The tooling suite alone passes 398 unit
and seven integration tests; engine-watch passes 28 unit tests. Git-based
tests needed access outside the restricted sandbox in the new worktree.

Engine friction: none, no gameplay Rust changes (one issue-rendering string).
PR size: estimate 2,300; measured about 3,000 changed lines. This requested
bulk migration keeps archived recommendations and schema valid together;
shared discovery and instructions complete the same provider-neutral switch.
No new starting gameplay rules and no Nick check after merge.
Work is in a fresh worktree from main, branch t0136-portable-agent-workflows;
the original checkout was restored clean to main after a byte-verified move.
The engine adopts its independent half in engine#2037; no pin move is needed.
