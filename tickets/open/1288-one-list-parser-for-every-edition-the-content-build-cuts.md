---
id: "1288"
title: One list parser for every edition the content build cuts
type: infra
milestone: Later
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1288 — One list parser for every edition the content build cuts

## Context

From the retro of PRs #616–#735 ([report](../../docs/research/retro-2026-11-17-prs-616-735.md),
item T18, narrowed by its verifier). `crates/content/build.rs` (591 lines)
reads four edition lists with two parsers. The demo's, the writer's
preview's and the trial's go through `demo_bundle::Plan`
(`crates/content/src/demo_bundle.rs`, included by `#[path]`). The
Playdate's `playdate-bundle.txt` goes through build.rs's own `rules`,
`matches` and `left_out` (lines 528–570), the same "file or folder/" rule
written a second time. The preview's and the trial's plan sits in a
global (`static LEFT_OUT`, line 97) that `table` (line 461) reads. Those two
lists only leave files out: a `keep` or `swap` line in them is checked
against the files and then silently ignored.

`demo_bundle.rs` (469 lines, ADR-0119) also holds a hand-written RON
scanner. `Shape`, `string_at` and `keep_entries` (lines 304–465) find
brackets, fields and `id`s by depth to cut `names.ron` and
`characters.ron`. #650 spent 3 of its 8 commits bounding its loops so a
mutant could not hang the build script.

The verifier dropped the rest of T18: story-trial already composes over
the demo (build.rs 146–149), and `rerun-if-changed` (130–135) keeps
incremental builds from rerunning the script.

**Why the comment strip stays hand-written.** ADR-0119 strips every comment
from the demo's RON files, because they retell later chapters. `ron`
0.12's `Value` can't do it: a parse and re-serialize drops enum variant
names (`deserialize_any` hands a bare `Sword` to `visit_unit`, ron 0.12.3
`de/mod.rs`, `handle_any_struct`). So `strip_comments` and its small
string/comment scanner (`runs`, `block_end`, `quoted_end`) stay. The
bracket-depth cutter stays too, until 1401 replaces it with engine#3444's
cut list.

## Nick input

None.

## Scope

**In:**
- `playdate-bundle.txt` is read by `demo_bundle::Plan`; build.rs's
  `rules`, `matches` and `left_out` go.
- A list that only leaves files out (Playdate, story-preview, story-trial)
  refuses a `keep` or `swap` line.
- The editions are a table in build.rs, not an if-chain. The preview's and
  trial's plan is passed to `table` as an argument, not kept in a static.

**Out (do not do):**
- Rewriting `keep_entries`, `Shape` and `string_at` on `ron`'s
  `RawValue` (this ticket's old step 4): 1401 replaces the whole cutter
  with engine#3444's cut list (its `keep` lines), so a rewrite here would
  be deleted there (the engine's redundancy sweep of 2026-11-28).
- Moving the parser to the engine: 1401 does that (blocked by this
  ticket and engine#3444), once this one leaves a single parser.
- The comment strip (see Context), the list file format, and what any
  list leaves out.
- The one-bit conversion and building only the tables a build uses
  (1221).
- `bundle.rs`'s disk folders (1270). Its set-up order (`use_pack`,
  `read_from_disk`, the `let _ = …set(…)` calls) is not this ticket's.
- Replacing `playdate-bundle.txt` with a Playdate content root (1170 may
  do that). Either order works: if 1170 lands first and the list is gone,
  skip step 2.
- The disclosure lists (0180). Either order: 0180's `cargo xtask
  disclosure` reads the preview's and trial's lists through
  `Plan::parse` and `Plan::left_out`, which this ticket keeps; if 0180 is
  done, its check test stands in for the disclosure tests named below.

## Implementation steps

Expected size: about 60 lines written, about 200 removed (net −140), without the old step 4.

1. **Leave-out-only lists.** In `demo_bundle.rs`, add
   `Plan::only_leaves_out(&self) -> Result<(), String>`. It errors, with the
   file named, when `keeps` or `swaps` is not empty; `+` lines stay allowed.
2. **The Playdate's list.** In `write_file_tables` (build.rs 221), parse
   `PLAYDATE_LIST` with `Plan::parse`, call `only_leaves_out`, and take
   `left_out` from `plan.left_out(&both)`. Its error ("is no file under
   assets/ or assets-private/content/") is the same as the old one. Delete
   `rules`, `matches` and the free `left_out`.
3. **The edition table.** Replace the if-chain in `main` (build.rs
   139–160) with a `const EDITIONS: &[Edition]`. Each `Edition` gives its
   root (`DEMO`, `PREVIEW`, `TRIAL`), whether it sets `demo_edition`, and
   its leave-out list, if any. Keep the panic text for an unknown
   `SHUYI_CONTENT`. Delete `static LEFT_OUT` and `fn leave_out`. Pass
   `Option<&Plan>` down through `write_file_tables`, `write_demo` and
   `playdate_table` to `table`. Call `only_leaves_out` on the preview's and
   trial's plans.
4. (Removed: the cutter stays as it is until 1401 replaces it.)
5. Update the module docs: build.rs's top comment (Playdate list, the
   editions) and `demo_bundle.rs`'s (it now reads every edition's list).
   No new ADR: the list format and ADR-0082/0119's rules don't change.

## Acceptance criteria

- [ ] `crates/content/build.rs` has no `fn rules`, `fn matches`,
      `fn left_out`, `fn leave_out` or `static LEFT_OUT`.
- [ ] A `keep` line added to `story-trial/bundle.txt` or
      `playdate-bundle.txt` stops the build with an error that names the
      file (tried locally, then reverted).
- [ ] The tests in `crates/content/src/demo_bundle/tests.rs` that check
      exact output pass with unchanged expected strings. Error-message
      tests may change their text.
- [ ] `crates/ui/tests/it/demo.rs`, the bundle tests in
      `crates/content/src/bundle.rs`, `crates/playdate/src/chapter_1.rs`'s
      tests and `crates/tools/src/game_tools/tests.rs`'s disclosure tests
      pass.
- [ ] `cargo xtask code-lines` shows `crates/content` smaller than on main.
- [ ] `cargo build -p trpg-content` passes with `SHUYI_CONTENT` set to
      the absolute path of `crates/content/demo`, then of
      `crates/content/story-trial` (the editions `demo.yml` exports).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `only_leaves_out` accepts folder, file and `+` lines and refuses
  `keep` and `swap`. `playdate-bundle.txt` parses and passes
  `only_leaves_out`. `keep_entries` keeps an entry whose struct holds an
  enum variant (`kind: Sword`) unchanged.
- Property: `keep_entries` with every key of a generated map or list keeps
  the text's entries unchanged, and `ron::from_str::<ron::Value>` of the
  output equals that of the input.
- Integration: the existing demo, bundle, Playdate Chapter 1 and
  disclosure tests (above), unchanged.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
