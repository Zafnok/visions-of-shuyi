# ADR-0097: The engine's skill pack is copied into the game at the pin

- **Status:** Accepted
- **Date:** 2026-10-25
- **Related tickets:** 1192 (this), engine#1134 (the pack), engine#2559
  and 1194 (`check-skills` as a gate); extends
  [ADR-0063](0063-an-engine-and-a-game-one-boundary-now-two-repositories-later.md) §8

## Context

Shuyi Engine ships a skill pack (engine#1134, the engine's ADR-0043): 15
`shuyi-*` skills in its `skills/` that teach an agent its API, each with a
worked example its CI runs. A game gets them one of two ways: `shuyi new`
copies them into the new game's `.claude/skills/`, or Claude Code installs
them as the plugin `shuyi@shuyi-engine` from the engine's repository.
This game wasn't made by `shuyi new`.

The plugin follows the engine's `main`: its marketplace entry has no
version, so installs follow the commits. The game builds on the pinned
commit (ADR-0063 §8), often days behind `main`. A skill from `main` can
teach an API the pin doesn't have yet. And `shuyi check-skills` accepts a
link only to a file in the folder it checks, so a game skill can link
`../shuyi-input/SKILL.md` only when the pack sits beside it.

## Decision

1. **A copy at the pin.** The pack lives in `.claude/skills/shuyi-*`,
   committed, beside the game's own skills. No plugin is registered.
2. **`cargo xtask skill-pack` writes it** from the pack the pinned
   `shuyi-cli` embeds (`shuyi_cli::skills::Pack::embedded`): it takes
   away every `shuyi-*` folder and writes the pack's skills. xtask builds
   with the pin, so the copy is the pin's. It runs after a pin move (the
   `engine-pin` skill, step 3), as its own command because the xtask
   that moves the pin was built with the old one.
3. **`cargo xtask skill-pack --check` (CI, `run-gates`) fails** on a
   file missing, changed or extra, or a `shuyi-*` folder the pack lacks.
   A test does the same check unless the local engine checkout is on.
4. **One source for each rule.** The game's `keyboard-input` and
   `run-gates` keep only this game's rules and link `shuyi-input` and
   `shuyi-gates` for the rest, as the engine's own pair does. A
   `shuyi-*` skill is never edited here: a gap in one is an engine
   ticket.
5. **The game's own skills meet the pack's format**: a `Not …` sentence
   in each description, `metadata.version`, under 500 lines, no link out
   of `.claude/skills/` (files outside are named in words). The one rule
   they can't meet, a worked example, is the engine's to change
   (engine#2559); `check-skills` becomes a gate in 1194.

## Consequences

- Every session in the repository has the pack, at the API the game
  builds with, with no install step.
- A pin move also refreshes the copy; forgetting fails CI.
- The four skills that wait on unbuilt engine work (`shuyi-stage-field`,
  `shuyi-import-art`, `shuyi-animate`, `shuyi-net`) come with the rest
  and say what isn't built yet.
- The pack's worked examples (`example.ron`, `example/`) sit in
  `.claude/skills/` as data; nothing in the game builds or runs them.
