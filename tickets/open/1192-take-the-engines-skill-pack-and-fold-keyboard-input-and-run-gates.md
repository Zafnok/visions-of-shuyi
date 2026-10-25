---
id: "1192"
title: Take the engine's skill pack and fold keyboard-input and run-gates down to the game's own rules
type: infra
milestone: "Engine: carve"
model: sonnet-5
effort: medium
status: todo
blocked_by: ["1189"]
nick_input: none
completed:
---

# 1192 — Take the engine's skill pack and fold keyboard-input and run-gates

## Context

Engine ticket 1134 shipped Shuyi Engine's skill pack: 15 skills in the
engine's `skills/` (ADR-0043 in the engine), installable as the Claude
Code plugin `shuyi@shuyi-engine` (`.claude-plugin/marketplace.json` in
the engine, source `./skills`). `shuyi new` copies the pack into a new
game's `.claude/skills/`. This game wasn't made by `shuyi new`, so it
has none of them.

Two of the pack's skills overlap the game's own:
- `shuyi-input` (every input rule a game keeps) and the game's
  `.claude/skills/keyboard-input/SKILL.md`;
- `shuyi-gates` (the gates a game runs, lint's findings, the fence's
  errors, snapshots, a CI failure that doesn't reproduce locally) and
  the game's `.claude/skills/run-gates/SKILL.md`.

Engine 1134 kept one source for the engine's own pair: the engine skill
holds what is engine-only and links the pack's skill for the rest. 1189
read engine 1134's notes and filed this ticket to do the same here.

## Nick input

None.

## Scope

**In:**
- Taking the pack the way engine 1134 says a game takes it. Choose one
  and record it in the completion notes:
  - the plugin, registered for the project in `.claude/settings.json`
    (`extraKnownMarketplaces` for `Zafnok/shuyi-engine` and
    `enabledPlugins` for `shuyi@shuyi-engine`), so every session in the
    repository has the skills and they follow the engine's `main`; or
  - a copy at the pinned engine, if the plugin can't be pinned and a
    skill on `main` would teach an API the pin doesn't have yet. Then
    the copy lives in `.claude/skills/` and `cargo xtask engine-pin`
    (or the `engine-pin` skill) refreshes it.
- Folding `keyboard-input` down to what holds in this game alone (its
  `Action`s, its keymap file, the Playdate's buttons, help-bar and
  dialogue text naming keys) and linking `shuyi-input` for the rest.
- Folding `run-gates` down to this game's own gates (the Playdate and
  no-std builds, `engine-pin --check`, the private-assets tests, the
  snapshot review) and linking `shuyi-gates` for the rest.
- `CLAUDE.md`'s skills row in *Map of the repo* names the pack.

**Out (do not do):**
- Changing any skill in the engine. A gap in a pack skill is an engine
  ticket (`engine-change` skill).
- The four pack skills that wait on unbuilt engine work (`shuyi-stage-field`,
  `shuyi-import-art`, `shuyi-animate`, `shuyi-net`): take them with the
  rest, don't edit them.
- Running the pack's worked examples (`cargo xtask skill-examples` is
  the engine's).

## Implementation steps

1. Read the engine's `skills/README.md`, `skills/shuyi-input/SKILL.md`
   and `skills/shuyi-gates/SKILL.md` at the pin
   (`gh api repos/Zafnok/shuyi-engine/contents/skills?ref=$(cat shuyi-engine.rev)`).
2. Pick plugin or copy (Scope) and set it up. With the plugin, check in a
   new session that the `shuyi-*` skills are listed.
3. Rewrite `keyboard-input` and `run-gates`: keep each rule that is about
   this game's files or targets; delete each rule `shuyi-input` /
   `shuyi-gates` already states, and link that skill once at the top.
   Every command a gate runs stays runnable from `run-gates` as written.
4. Run `cargo shuyi check-skills .claude/skills` and fix what it reports
   in the game's own skills.
5. Update `CLAUDE.md`'s skills row.

## Acceptance criteria

- [ ] A new session in the repository lists the pack's `shuyi-*` skills.
- [ ] `keyboard-input` and `run-gates` each link their pack skill and
      repeat none of its rules; every gate command is still in `run-gates`.
- [ ] `cargo shuyi check-skills .claude/skills` passes.
- [ ] `cargo xtask ticket-lint` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None new (no Rust code); `check-skills` above.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
