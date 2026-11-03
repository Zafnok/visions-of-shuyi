---
id: "1224"
title: "Keep the engine's skill pack with cargo shuyi skills, and delete xtask's skill-pack"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: low
status: blocked
blocked_by: ["engine#2578"]
nick_input: none
completed:
---

# 1224 — Keep the skill pack with `cargo shuyi skills`

## Context

From the game/engine boundary audit of 2026-11-03. 1192 (ADR-0097) copied
the engine's skill pack into `.claude/skills/shuyi-*` with a command of
our own, `cargo xtask skill-pack [--check]`
(`crates/xtask/src/skill_pack.rs`, 114 production lines and about 130 of
tests); 1194 added `check-skills` beside it. Every game on Shuyi needs the
same after a pin move, so the engine now does it:
[engine#2578](https://github.com/Zafnok/shuyi-engine/blob/main/tickets/open/2578-shuyi-skills-write-and-check-a-games-copy-of-the-skill-pack.md)
adds `shuyi skills [--check]`. `cargo shuyi` runs `shuyi-cli` at the pin,
so the copy stays the pin's, as ADR-0097 §2 requires.

## Nick input

None.

## Scope

**In:**
- Move the pin past engine#2578 (`engine-change` skill §7, `engine-pin`
  skill), unless an open `engine-pin` PR has already done so.
- `cargo shuyi skills [--check]` replaces `cargo xtask skill-pack
  [--check]` everywhere; `skill_pack.rs` and its tests go.
- The one test that checks the real repository's copy stays, on the
  engine's `shuyi_cli::skills::install::problems`.
- **Either order with 1235:** if 1235 is done, `skill-pack` is routed
  through its report helper in `main.rs`: delete that call and its row
  too; if not, delete the arm and `fn skill_pack` as they are, and 1235
  routes what is left.

**Out (do not do):**
- Editing a `shuyi-*` skill, or the game's own skills beyond the command's
  name.
- The `.agents/skills/` loaders' generator (`scripts/sync-agent-skills.ps1`)
  stays the game's; only its comment changes.

## Implementation steps

1. **Pin** (as above). Then run `cargo shuyi skills`: it must leave
   `.claude/skills/` as it is (the game holds every pack skill, and
   `shuyi-net`, a module's, is kept because its folder is there). If it
   changes a file, the copy was behind the new pin: commit what it writes
   and run `pwsh -File scripts/sync-agent-skills.ps1`.
2. `crates/xtask/src/main.rs`: delete `mod skill_pack`, the `skill-pack`
   usage line, the `Some("skill-pack")` arm and `fn skill_pack`.
3. Delete `crates/xtask/src/skill_pack.rs`. Keep its test
   `the_real_repo_has_the_pinned_engine_s_pack` (skipped when
   `crate::engine::PATCH_FILE` exists) as a test in `main.rs`'s tests or
   a small `crates/xtask/src/skills_copy_tests.rs`, asserting
   `shuyi_cli::skills::install::problems(...)` is empty.
4. Rename the command in: `.github/workflows/ci.yml` (*Check the engine's
   skill pack*), the `run-gates` skill (the gate list and the pin-move
   paragraph), the `engine-pin` skill (steps 3 and 7), `AGENTS.md` (the
   *Shared instructions* paragraph and the `.claude/skills/` row),
   `docs/engine/README.md` (*The skill pack*), the comment at the top of
   `scripts/sync-agent-skills.ps1`, and `docs/adr/README.md`'s 0097 row.
5. `docs/adr/0097-…`: in Decision §2 and §3, the command is `cargo shuyi
   skills [--check]`, the engine's since engine#2578 (1224); its reasons
   are unchanged.
6. `pwsh -File scripts/sync-agent-skills.ps1 -Check` (a skill's text
   changed, not its description, so no loader should).
7. **Measure** with `cargo xtask code-lines`: about 130 production lines
   removed (114 and main.rs's 25), none added; about 120 test lines
   removed and 10 added.

## Acceptance criteria

- [ ] `shuyi-engine.rev` is at or past the commit that closed
      engine#2578.
- [ ] `crates/xtask/src/skill_pack.rs` is gone; `cargo xtask skill-pack`
      prints xtask's usage.
- [ ] `grep -rn "xtask skill-pack" --exclude-dir=done --exclude-dir=target .`
      finds nothing outside `shuyi-engine/`.
- [ ] `cargo shuyi skills --check` passes, in CI and locally, and the
      real-repository test passes.
- [ ] All gates in the `run-gates` skill pass, including
      `cargo xtask ticket-lint --check-engine`.

## Tests required

- Unit: the real-repository test on the engine's check. The engine's
  tests (engine#2578) cover the differences and the writing.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
