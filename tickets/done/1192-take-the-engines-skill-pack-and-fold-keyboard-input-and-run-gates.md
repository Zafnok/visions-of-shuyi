---
id: "1192"
title: Take the engine's skill pack and fold keyboard-input and run-gates down to the game's own rules
type: infra
milestone: "Engine: carve"
model: implementation
effort: medium
status: done
blocked_by: ["1189"]
nick_input: none
completed: 2026-10-25
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

- [x] A new session in the repository lists the pack's `shuyi-*` skills.
- [x] `keyboard-input` and `run-gates` each link their pack skill and
      repeat none of its rules; every gate command is still in `run-gates`.
- [ ] `cargo shuyi check-skills .claude/skills` passes. *Not met: the
      game's 13 own skills fail only the worked-example rule (completion
      notes); engine#2559 and 1194.*
- [x] `cargo xtask ticket-lint` passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None new (no Rust code); `check-skills` above.

## Completion notes

- **Copy, not plugin** (ADR-0097). The plugin follows the engine's
  `main` (its marketplace entry has no version), and `check-skills` only
  accepts a link to a file in the folder it checks, so a game skill can
  link `../shuyi-input/SKILL.md` only with the pack beside it. The 15
  skills are in `.claude/skills/shuyi-*`, the four waiting ones
  included, unedited. A session lists them because Claude Code loads
  every `.claude/skills/<name>/SKILL.md`; this session started before
  the copy, so it wasn't seen in a fresh session here.
- **`cargo xtask skill-pack [--check]`** (`crates/xtask/src/skill_pack.rs`)
  writes the pack the pinned `shuyi-cli` embeds, and `--check` fails on a
  missing, changed or extra file or an old `shuyi-*` skill; a test does
  the same on the real repository (skipped with the local checkout on).
  Deviation: the ticket said `cargo xtask engine-pin` refreshes it. It
  can't: the xtask that moves the pin was built with the old one, and on
  Windows it can't rebuild itself while running. So `engine-pin` says to
  run `skill-pack` next, the `engine-pin` skill's step 3 runs it, and CI
  runs `--check` beside `engine-pin --check`.
- **`keyboard-input`** keeps the game's files, actions, Nick's defaults,
  the allowed files, help-text helpers, the Playdate's chords, crank and
  system menu; **`run-gates`** keeps every gate command (now also
  `skill-pack --check` and `ticket-lint`), the environment, the
  private-assets tests, `insta` and CI's mutation testing. Each links its
  pack skill once at the top.
- **`check-skills`**: the pack passes, and so do the game's own skills on
  every rule but one. Fixed in all 13: a `Not …` sentence in each
  description, `metadata.version`, links out of `.claude/skills/` named in
  words, and `story-writing` (663 lines) split into
  `references/examples.md` and `references/critique-pass.md`. **Not met:**
  each of the 13 still fails "no worked example (example.ron)". A worked
  example is a script the engine runs on a fresh game, so a skill about
  this repository's workflow can't have one, and faking one (or a
  `waits_for`) would weaken the check. Filed engine#2559 (engine PR #114,
  the ticket alone) so the check asks it only of the pack's skills, and
  **1194** to make `check-skills` a gate once the pin is past it.
- `check-skills` (like any `cargo shuyi` run) first prints cargo errors
  about `{{crate_name}}` in the engine's `templates/`; they are noise from
  cargo reading the engine's git checkout and don't change the result.
- Friction: estimate 90 production lines (by `engine.rs`'s file-agreement
  check), measured 145 in `crates/xtask` (1.6×, under 2×). `code-lines`
  also counts the pack's copied `example/*.rs` (647 lines) as game code;
  they aren't the game's, and only a pin move that changes the pack
  changes them.
- Nick: nothing to check; no gameplay change.
