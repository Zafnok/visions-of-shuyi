---
id: "0172"
title: A change to the private content plays the bots and checks the demo
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0172 — A change to the private content plays the bots and checks the demo

## Context

Since 0160 (ADR-0117) the battles, maps, chapters, skirmishes and data
tables live in the private repository's `content/`. A balance change
reaches the game only as a move of `assets-private.rev`. CI's change
detection never learned that (retro T1,
[report](../../docs/research/retro-2026-11-17-prs-616-735.md)):

- `.github/playtest-levers.sh` (0514, last changed in #535) has no case
  for the private content. Lines 26 and 30 still list `assets/battles/*`,
  `assets/maps/*` and `assets/chapters/*`, which hold only a `README.md`.
  Lines 27–30 make the public stand-in tables levers, though the content
  replaces them whole in every CI build (ADR-0121).
- #731 changed a stat in the private characters table. It played only
  because it also re-recorded the `*_winning.ron` fixtures.
- `demo.yml`'s `pull_request` paths (lines 32–36) leave out
  `assets-private.rev`, so a pin-only change gets no demo disclosure check.
  #731 had no Demo run. (`crates/content/demo/**` is already covered by
  `crates/content/**`.) The other workflows with a `paths` filter
  (`story-mirror.yml`) already list the pin.

## Nick input

None. Verified by the tests below and one real draft PR.

## Scope

**In:**
- The playtest's `detect` job turns a pin move into the content paths
  that changed between the two pins, and the lever script judges them.
- Dead public cases out; a public stand-in table plays only when the
  content has no file at its path.
- `assets-private.rev` in `demo.yml`'s `pull_request` paths.
- Tests of the lever script.

**Out (do not do):**
- Narrowing which public code files play (credits, tips, comments,
  `*_tests.rs`), not repeating a play on `main`, caching reports, splitting
  slow battles: 0173. Either order: both edit the lever script and
  `detect`; whichever lands second merges the other's arms and keeps this
  ticket's private-content arms and stand-in rule.
- Moving the playtest onto the engine's workflow (1207). Either order:
  if 1207 is done, add step 2's expansion wherever its workflow builds the
  changed-path list.
- Caching or encrypting the playtest's build (0171, 0178).

## Implementation steps

1. `.github/actions/private-assets/action.yml`: add an input `history`
   (default `"false"`). When `"true"`, the checkout step (lines 119–128)
   passes `fetch-depth: 0` and `filter: blob:none`, so another pin's trees
   can be read without a second clone.
2. `.github/workflows/playtest.yml`, job `detect` (lines 49–85): after
   the checkout, run the private-assets action with
   `key: ${{ secrets.PRIVATE_ASSETS_KEY }}`, `content: "true"`,
   `art: "false"`, `history: "true"`, `required: "false"`. In
   `Classify changed files`, when `git diff --quiet $range --
   assets-private.rev` fails:
   - read the old pin (`git show HEAD^1:assets-private.rev` on a pull
     request, `git show "$BEFORE:assets-private.rev"` on a push) and the
     new one from the working tree;
   - if `assets-private/content` is missing (no key) or
     `git -C assets-private cat-file -e "$old^{commit}"` fails, print why
     and write `play=true`;
   - else append `git -C assets-private diff --name-only --no-renames
     "$old" "$new" -- content | sed 's|^|assets-private/|'` to the list
     piped into the lever script.
3. `.github/playtest-levers.sh`:
   - delete the `assets/battles/*|assets/maps/*` arm and `assets/chapters/*`;
   - add `assets-private/content/battles/*`, `maps/*`, `chapters/*`,
     `skirmishes/*` (the bots read it: `crates/bots/src/skirmish.rs`) and
     `data/*.ron` minus `camp`, `lessons`, `names` and `tips` (keep the
     same tables as the public list);
   - before the public `assets/data/*` arms, skip a stand-in whose
     replacement exists: `[ -f "assets-private/content/${path#assets/}" ]`
     → `continue`. `terrain.ron` has none, so it still plays;
   - `assets-private/content/tests/*` and `README.md`s stay out through the
     existing `*/tests/*` and `*.md` arms. Update the header comment.
4. `.github/workflows/demo.yml`: add `'assets-private.rev'` to the
   `pull_request` paths.
5. Docs: `playtest.yml`'s header (lines 3–4, 15–18), `docs/playtesting.md`
   around line 185, and ADR-0110 line 64: say the content's battles, maps,
   chapters, skirmishes and tables play through the pin.
6. Tests in `crates/xtask/src/main.rs`'s test module, beside
   `workflow_files()` (line 973). Expected size: about 60 lines of
   workflow and script, about 100 of tests.

## Acceptance criteria

- [ ] `levers_follow_the_private_content` (unix only: it runs `bash
      .github/playtest-levers.sh` on fixed inputs in a temp folder with a
      fake `assets-private/content/data/items.ron`) gives `play=true` for
      `assets-private/content/data/characters.ron`, `.../battles/ch01.ron`
      and `assets/data/terrain.ron`; `play=false` for
      `assets/data/items.ron` alone, `.../dialogue/ch01.dlg`,
      `.../tests/knowledge.ron` and `assets-private.rev` alone; and
      `play=true` for `assets/data/items.ron` when the fake content is absent.
- [ ] `every_lever_pattern_matches_a_file` parses the script's case
      patterns and finds a tracked file for each public one (`*` matches
      across `/`); the `assets-private/content/` ones are checked only when
      the content is checked out.
- [ ] A draft PR that only moves `assets-private.rev` back to #731's base
      pin shows `plays: assets-private/content/data/characters.ron` in
      `detect` and starts `export (demo, web)`. Cancel the play after
      `detect`, close the PR unmerged, and link both runs in the
      completion notes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the two xtask tests above.
- Integration: the draft PR's `detect` and Demo runs (workflow logic has no
  local runner).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
