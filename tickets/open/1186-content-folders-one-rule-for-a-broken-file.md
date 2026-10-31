---
id: "1186"
title: "Content folders: one rule for a broken file, so bundle::parsed and the hand fold-backs go, and the battle lights on the folder loader"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#2233"]
nick_input: none
completed:
---

# 1186 — Content folders: one rule for a broken file

## Context

From the retro of the last 50 PRs (2026-10-22). Three tickets moved the
game's content loaders onto the engine within four hours on 2026-10-20:
1173 (#375, the format registry), 1169 (#377, the bundle's folder
loader) and 1176 (#379, the registry's file-by-file read, engine 2219).
The churn itself was fine (the engine's folder APIs were all-or-nothing,
and the workaround lasted three hours). What it left is two engine
loaders with the same name and opposite rules for a broken file, and the
game adapting each to the other:

- `Bundle::load_each` (engine 2208) gives **the whole map or every
  error**. The chapter and battler loaders need each file's result (their
  checks run on the files that parsed, even when another broke), so the
  game wraps it in `bundle::parsed` (`crates/content/src/bundle.rs`),
  with each parse closure reporting its own errors and returning
  `Option`. 1169's friction row: "a third such loader would make it an
  engine variant that keeps what parsed".
- `Formats::load_each` (engine 2219) gives **each file's result**. The
  map, cinematic and dialogue loaders want all-or-nothing, so each folds
  the map back by hand (`crates/content/src/map.rs`, `cinematic.rs`,
  `dialogue.rs`).
- `battle::load_lights` (`crates/content/src/battle.rs`) is still a hand
  loop over `bundle::files_in`, and silently skips a file that isn't
  UTF-8 (1169's grep, by its error message, missed it).

engine#2233 gives `Bundle` the file-by-file variant and one helper that
turns a file-by-file map into the whole map or every error. This ticket
moves the game onto both.

Two routes stay, on purpose: the closure route (the bundle) where a
parse needs other tables (battle references, battler images), the
registry where a format decodes alone.

## Nick input

None. Nothing a player sees changes. One behaviour change: a battle
file that isn't UTF-8 is an error from the lights loader too, where it
was skipped (the battle loader already reports it).

## Scope

**In:**
- Move the pin past engine#2233 (`engine-change` skill, §7).
- The chapter and battler loaders on the file-by-file `Bundle` variant;
  `bundle::parsed` goes; their parse closures return `Result` instead of
  pushing to a shared `errors` and returning `Option`.
- The map, cinematic and dialogue loaders use the engine's helper
  instead of folding by hand.
- `battle::load_lights` on `bundle::load_each` (all-or-nothing is right
  for it).
- The content tests that pin these loaders' error reports stay green
  (the same errors, maybe in a different order: sort if a test compares
  lists).

**Out (do not do):**
- Changing a file format or a check.
- Moving more loaders between the two routes.

## Implementation steps

1. **Read** engine#2233's API, `crates/content/src/bundle.rs`
   (`load_each`, `parsed`, `files_in`), `chapter.rs`, `battler.rs`,
   `map.rs`, `cinematic.rs`, `dialogue.rs`, `battle.rs`
   (`load_lights`, `light_of`) and their tests.
2. **Pin** (as above).
3. One loader at a time, tests green after each: lights, then map,
   cinematic, dialogue, then chapter and battler; delete `parsed`.
4. **Measure** with `cargo xtask code-lines` (estimate about 15 added
   lines, about 50 removed).

## Acceptance criteria

- [ ] `shuyi-engine.rev` is at or past the commit that closed
      engine#2233.
- [ ] `grep -rn "fn parsed\|files_in(BATTLES_DIR)" crates/content/src`
      finds nothing.
- [ ] No content loader folds a file-by-file map into all-or-nothing by
      hand.
- [ ] A battle file that isn't UTF-8 is reported by the lights loader (a
      test).
- [ ] All gates in the `run-gates` skill pass, including
      `cargo xtask ticket-lint --check-engine`.

## Tests required

- Unit: the existing content loader tests (unchanged expectations); the
  lights loader's new error.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
