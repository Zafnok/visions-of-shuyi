---
id: "1235"
title: "One report helper for the xtask commands: a usage error, an outcome and a problem list each written once"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1235 — One report helper for the xtask commands

## Context

The game/engine boundary audit of 2026-11-03 (finding tooling#5) read
`crates/xtask/src/main.rs`. About 20 command wrappers (`icon`,
`voice_test_clips`, `sfx`, `test_card`, `test_tileset`, `edge_marks`,
`effect_marks`, `map_sprite_import_in`, `battler_import_in`,
`title_art_in`, `tileset_import_in`, `playdate`, `playdate_images_in`,
`playtest`, `portrait_import`, `code_lines`, `pr_size`, `lines`,
`check_script`, …) each repeat the same two blocks: a usage check
(`eprintln!("usage: …"); return 2;`) and an outcome tail
(`Ok(s) => { println!("{s}"); 0 } Err(e) => { eprintln!("<name>: {e}"); 1 }`).
A `report` helper already exists (`main.rs:269`), but only `engine`,
`engine-pin` and `skill-pack` call it. `check_report` (`main.rs:207`),
`ticket_lint` and `skill_pack --check` print a problem list three ways.

This is game-side structure, not an engine gap: the engine's own xtask
has one `report` per command, and `shuyi_cli::run` dispatches Shuyi's
tools. No engine change.

## Nick input

None.

## Scope

**In:**
- Three helpers in `main.rs`, and every wrapper that repeats a block
  calling them. Behavior unchanged: the same exit codes, the same
  stdout and stderr text.
- A table of the commands that take no arguments.
- Folding the tests that only check exit codes into one table test.

**Out (do not do):**
- Changing any command's arguments, messages or exit codes (`icon`
  exits 2 on a failed run today; it keeps doing so).
- Deleting commands. Other open tickets may delete theirs: 1206
  (`playdate`), 1207 (`playtest`), 1224 (`skill-pack`), 1221
  (`playdate-images`), 1229 (`icon`). Route whatever `main.rs` still
  has when this starts; a ticket that deletes a command later deletes
  its call too. 1172 (`code-lines --tree`) changes an argument list;
  whichever lands second rebases.
- Engine ergonomics (game commands listed by `cargo shuyi`).

## Implementation steps

Expected size: about 170 production lines removed and 40 added (net
about −130), and about 70 test lines removed.

1. Add to `main.rs`, beside `report`:
   - `fn usage(text: &str) -> u8`: prints `text` to stderr, returns 2.
   - `fn report_named(name: &str, outcome: Result<String, String>) -> u8`:
     `Ok(s)` prints `s` (with `print!` when it already ends in a newline,
     as `code-lines` and the `--list` outputs do) and returns 0; `Err(e)`
     prints `{name}: {e}` to stderr and returns 1.
   - `fn report_problems(outcome: Result<String, Vec<String>>, head: impl Fn(usize) -> String, after: &str) -> u8`:
     `Ok` as above; `Err` prints `head(n)` (each caller keeps its words:
     `engine-pin: N problem(s)`, `ticket-lint: N error(s)`, skill-pack's
     hint), each problem indented, then `after` if not empty
     (check-mentions' backticks advice), and returns 1. `check_report`
     (engine-pin), `ticket_lint`, `check_mentions` and
     `skill_pack --check` use it; `check_report` goes.
2. A table for the commands with no arguments:
   `const NO_ARGS: [(&str, fn(&Path) -> Result<String, String>); 4]`
   with `test-card` (`test_card::run`), `edge-marks`, `effect-marks` and
   `voice-test-clips` (`voice_test::run`). `dispatch` looks a name up
   there first: extra arguments give `usage("usage: cargo xtask <name>")`,
   else `report_named(name, run(&repo_root()))`. Their four wrappers go.
3. Rewrite the other wrappers' tails and usage checks on the helpers:
   `sfx`, `test_tileset`, `map_sprite_import_in`, `battler_import_in`,
   `title_art_in`, `tileset_import_in`, `playdate`, `playdate_images_in`,
   `playtest`, `portrait_import`, `code_lines`, `pr_size`, `lines`,
   `check_script`, `clean_merged_targets`, `icon` (its error path keeps
   exit 2: `usage(&format!("icon: {e}"))`). Their own flag parsing stays.
4. If 1174 is done, `friction-check` goes through `report_problems` too;
   if not, 1174 wires it that way (its step 4 says so).
5. Tests: replace the per-command "bad arguments give 2" assertions
   (`voice_test_clips_takes_no_arguments`, `sfx_rejects_unknown_args`,
   `test_card_rejects_args`, the `--bogus` lines of the import, playdate
   and portrait tests, `check_mentions_takes_exactly_one_argument`) with
   one table test over `dispatch`: each `(args, expected code)`. Keep the
   tests that run a command for real (playtest, check-script, code-lines,
   pr-size, lines, sfx `--check`, ticket-lint, the repository checks).
6. Measure with `cargo xtask code-lines` and put the numbers in the PR
   (ADR-0092).

## Acceptance criteria

- [ ] Outside the three helpers and `report`, no function in `main.rs`
      prints an outcome itself: `rg -c 'eprintln!\("[a-z-]+: \{e\}"\)' crates/xtask/src/main.rs`
      finds none, and no `Ok(…) => {` arm prints and returns 0.
- [ ] `test-card`, `edge-marks`, `effect-marks` and `voice-test-clips`
      run from `NO_ARGS`, with no wrapper function each.
- [ ] Every command's exit codes are unchanged: the table test covers a
      bad-argument case of each command that has one, and the kept
      real-run tests pass unchanged.
- [ ] `cargo xtask code-lines` shows `trpg-xtask` removing at least 100
      more production lines than it adds (about 130 expected).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `report_named` and `report_problems` return 0 and 1 (a test
  each); the dispatch table test (step 5).
- Integration: the existing real-run tests in `main.rs`, unchanged.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
