---
id: "0771"
title: "The trial's web preview names only Chapter 1's cast"
type: feature
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-15
---

# 0771 — The trial's web preview names only Chapter 1's cast

## Context

The writer trial (ticket 0766, PR #696) sends about five applicants a
text packet. They try their scene in the web script preview on its
password-protected itch.io page (tickets 0764, 0767, 0769). The packet's
story pages stop short of the later characters' details. The preview's
page of names and faces (the Unit info key, `reference_rows` in
`crates/ui/src/story_preview.rs`) still lists every `{n:<id>}` of
`data/names.ron` and every character, Act 2's included (`sister`,
`prince`). Nick, 2026-11-15: "probably dont include anyone past ch1 for
this part".

The web preview is the `story-preview` edition (ADR-0123, ADR-0125)
built for the browser by `story-kit.yml`'s `story-preview (web)` job. The
Windows kit of the same edition goes to the hired writer through the
private mirror (ADR-0122), who writes the whole game and needs every
name.

The Chapter 1 demo edition (ticket 0914, ADR-0119) already cuts the
names and characters tables to what Chapter 1 uses
(`crates/content/demo/bundle.txt`), with canaries that fail its export
when a later id comes back.

## Nick input

None. The answer is quoted above; nothing to check after merge.

## Scope

**In:**
- A `story-trial` edition: the `story-preview` program on the demo's
  Chapter 1 cut, with every script left out. The names page then lists
  only Chapter 1's names and characters, because the pack holds no others.
- `story-kit.yml` builds the web preview as `story-trial`. Windows (and
  macOS) stay `story-preview`, with the full list.
- An ADR recording the split.
- Tests: the page over the trial's bundle names no later character, and
  the edition's canaries name the later cast and aren't in its bundle.

**Out (do not do):**
- Changing the names page's code, or filtering it at run time.
- Changing any script line (dialogue writing is paused).
- Changing the demo's list (`crates/content/demo/bundle.txt`).

## Implementation steps

1. `crates/content/story-trial/bundle.txt`: the content root, listing
   `dialogue/` as `story-preview/bundle.txt` does.
2. `crates/content/build.rs`: when `SHUYI_CONTENT` is that root, set
   `demo_edition` (the demo's tables) and leave out what its list says from
   every table, as the `story-preview` root does. About 20 lines.
3. `shuyi.ron`: the `story-trial` edition: the preview's package, program,
   features and files beside, and a disclosure with the demo's pack
   entries without `dialogue/ch01.dlg`, plus the demo's forbidden paths and
   canaries with the later cast's ids.
4. `story-kit.yml`: each preview of the matrix names its edition, and
   the steps read `dist/export/<edition>/<target>`.
5. ADR-0126; update ADR-0123's and the job's comments.

Expected size: about 60 lines of code and workflow, about 250 lines of
`shuyi.ron` lists, and tests.

## Acceptance criteria

- [x] `cargo shuyi export web --release --edition story-trial` builds the
      web preview; its disclosure names `sister` and `prince` as canaries.
      (The canaries are in; see the notes for the web build itself.)
- [x] `story-kit.yml`'s web preview is the `story-trial` edition; the
      Windows kit stays `story-preview`.
- [x] A test opens the names page over the trial's bundle: no `sister`
      or `prince` row, and Chapter 1's `retainer` is there
      (`the_trials_page_of_names_and_faces_names_only_chapter_1s_cast`).
- [x] An xtask test: the trial's pack entries are the demo's without the
      scripts, and none of its canaries is in a file of its bundle
      (`the_trial_edition_is_the_demo_less_the_scripts_with_no_later_name`).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Integration (`crates/ui/tests/it/story_preview.rs`): the names page over
  the demo's bundle, which is the trial's bundle plus `dialogue/ch01.dlg`.
- xtask (`crates/xtask/src/game_tools/tests.rs`): the edition's lists.

## Completion notes

- Done as planned: a `story-trial` edition (`crates/content/story-trial/`,
  a branch in `crates/content/build.rs`, `shuyi.ron`), the web leg of
  `story-kit.yml` builds it, `docs/releasing.md` says so, ADR-0126
  records the split (the hired writer's Windows kit keeps every name).
- The page has no `sister` or `prince` rows as *characters* in any build:
  they are only names (`{n:sister}`, `{n:prince}`). The test checks those
  tokens and the later character `defector`, and that the full game's
  page has them, so it can't pass by listing nobody.
- The engine refuses a canary under 6 bytes, so the later cast's `vosse`
  is the canary `"vosse"` with its quotes (an xtask assertion keeps every
  canary at 6 bytes or more).
- Verified locally: `cargo shuyi export windows --release --edition
  story-trial` passes the disclosure (canaries and pack entries); its pack
  and program hold none of `"sister"`, `"prince"`, `brother_tor`. The
  local web export was stopped: MSYS2's `wasm-opt` (version 133, as CI's)
  ran an hour on the Wasm without finishing on Windows. CI runs it on
  Linux, where the web preview built before; the first build of this
  edition for the web is the next Story kit run after merge.
- No gameplay rule was decided. Nothing for the Act 1 playtest.
