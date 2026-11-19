---
id: "0187"
title: Each third-party work is typed once, and the notices and the assets table are written from it
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0187 — Each third-party work is typed once, and the notices and the assets table are written from it

## Context

From the friction sweep of 2026-11-17
([report](../../docs/research/friction-2026-11-17.md), bucket B3
"derive hand-kept copies", with 0258 and the 1237 amendment).

A third-party work (a music track, a sound, a font, a bought art pack)
is typed three or four times, then cross-checked by tests:

- `THIRD_PARTY_ASSETS.md`, the registry ADR-0013 requires: 61 table
  lines (header and rule included) under *Shipped items* (line 99),
  columns Item, Source URL, Version, License, License file, Used for,
  Added by ticket.
- `assets/audio/audio.ron` (47 `source:` credits, each with `title`,
  `author`, `source`, `license` and a `note` that is the version) and
  `assets/data/credits.ron` (11 `source:` credits, with `private` file
  lists for bought art, ADR-0051).
- `assets/NOTICES-game.txt` (292 lines, 48 works, each
  `Title / Author / Source / Version / License`), which `shuyi.ron`'s
  `notices.texts` (line 126) puts in the web build's `NOTICES.txt` whole.
- Tests that keep them equal: `every_third_party_asset_has_a_credit`
  (`crates/content/src/credits.rs:200`, by source link) and
  `the_notices_name_the_games_works_and_license_texts`
  (`crates/tools/src/game_tools/tests.rs:868`).
- Tests that list the bought packs by name, so a new pack edits them:
  `the_embedded_file_loads_and_merges_with_the_audio_manifest`
  (`credits.rs:140-144`, `assert_eq!(ids, ["tiny_tales", "shironejiya",
  "pixel_flag", "mattwalkden"])`, then `covers` asserts per artist) and
  `every_bought_file_has_a_credit`
  (`crates/content/tests/it/private_assets.rs:208-216`, the four art
  titles).

#588 (0851, two music tracks) edited `THIRD_PARTY_ASSETS.md`,
`audio.ron`, `NOTICES-game.txt`, `import.py` and `shuyi.ron`. #718 and
#730 (one bought pack each) edited `THIRD_PARTY_ASSETS.md`,
`credits.ron`, `credits.rs` and `private_assets.rs`. Of the 41 commits
to `THIRD_PARTY_ASSETS.md`, 14 also edited `credits.ron` and 7
`credits.rs`. Due again in the art tickets 0858, 0861, 0862 and 0866 and
the music and sound tickets 0028, 0473, 0605 and 0742 (either order:
done first, this ticket saves them the edits; done after, their rows are
moved here with the rest).

The engine already merges audio and file credits into one list
(`shuyi_assets::credits::merge`), and `shuyi web` writes `NOTICES.txt`
by copying the game's texts. A generator in the engine is an engine
change, so it is proposed separately and not waited on: this ticket
generates the game's files with a game-side command.

Not 0180's job, and 0180 not this one's: 0180 writes `shuyi.ron`'s
disclosure lists (`allowed_files`, `allowed_pack_entries`); this ticket
leaves `shuyi.ron` alone (`notices.texts` keeps naming
`assets/NOTICES-game.txt`). Either order with 0180: if it is done, reuse
its marker splice (`crates/xtask/src/disclosure.rs`, step 4 there) for
the table; if not, write a small one here and 0180 may share it.

## Nick input

None. The credits screen, the notices and the registry say the same
things as before; only where they are typed changes.

## Scope

**In:**
- The fields the table and the notices need, added to the credit
  entries of `credits.ron` and `audio.ron` (and to the game's credit
  types in `trpg_content::credits` and `trpg_content::audio`).
- `cargo xtask notices` (writes) and `cargo xtask notices --check`
  (fails on a stale file): the whole of `assets/NOTICES-game.txt`, and
  the *Shipped items* table of `THIRD_PARTY_ASSETS.md` between marker
  comments.
- The two cross-check tests replaced by one test that runs the check.
- The id-list and title-list asserts replaced by generic checks.
- ADR-0013's registry clause amended (`write-adr`): the table is
  written from the credits.

**Out (do not do):**
- The prose of `THIRD_PARTY_ASSETS.md` (the policy, the bought art's
  purchase record): it stays hand-written, outside the markers.
- `assets-src/audio/import.py` (download links and checksums, not
  credits) and `shuyi.ron` (0180).
- The engine's `shuyi notices` or `shuyi_assets::credits` (proposed as
  an engine ticket, with Nick's OK first).
- Any license decision, a new work, or what the credits screen shows.
- `LICENSE` (Nick's call, AGENTS.md rule 6).

## Implementation steps

Expected size: about 200 production lines (the command and the
renderers), about 100 test lines, about 350 lines of credit data moved
from the table into RON, and the table's 60 lines rewritten by the
command. Plan about 900 changed lines.

1. **Fields.** One table row is one *use* of a credited work: Terminus
   has two rows (the font and the test tileset's glyphs) and one credit.
   Add to each credit an optional `uses: [(item, version, license_file,
   used_for, tickets)]` list holding the table's columns as written
   today; Source URL and License come from the credit itself. A bought
   pack's `license_file` is the plain-text private path the table uses
   now (`private: library/...`). `audio.ron`'s `note` stays the notices'
   Version line. Keep `hidden: true` as it is.
2. **Move the rows.** Copy each of the table's rows into its credit's
   `uses`, matching by source link as `asset_sources()`
   (`credits.rs:181`) does. A row with no credit today is a bug: stop
   and report it rather than inventing a credit.
3. **Command.** New module `crates/xtask/src/notices.rs`, registered in
   `crates/xtask/src/main.rs`. It loads both files through
   `trpg_content` (the merged list the credits screen reads) and
   renders: `assets/NOTICES-game.txt` whole, in today's layout and order
   (header, then one block per shipped work); the table between
   `<!-- generated by cargo xtask notices -->` and `<!-- end generated -->`
   in `THIRD_PARTY_ASSETS.md`, in today's row order (keep an explicit
   order field if the data's order differs). `--check` renders to memory
   and fails with the first differing line and the command to run.
4. **First run is a no-op.** The first write must leave both files byte
   for byte as they are, but for the two marker lines. Fix the data,
   not the renderer, where they differ.
5. **Tests.** Replace `every_third_party_asset_has_a_credit` and
   `the_notices_name_the_games_works_and_license_texts` with
   `the_notices_and_the_assets_table_are_what_xtask_notices_writes`
   (it runs `--check`), keeping their license-text checks (every
   shipped row's text is in `notices.texts`).
6. **Generic bought-art checks.** In `credits.rs`, replace the `ids`
   list and the per-artist `covers` asserts with: every credit with a
   `private` list is in `CreditGroup::Art`; no two credits' `private`
   entries overlap (neither is a prefix of the other), which is what
   the `!covers` asserts were guarding. In
   `crates/content/tests/it/private_assets.rs`, replace the title list
   with: every credit with `private` covers at least one bought file in
   the build, and `credits::uncredited` is empty (already there).
7. **Docs.** Rewrite the paragraphs of `THIRD_PARTY_ASSETS.md` (lines
   9-41) and the header comments of `credits.ron` and
   `NOTICES-game.txt` that say to edit three files: a new work is one
   credit entry, then `cargo xtask notices`. Amend ADR-0013's registry
   clause (line 77) and ADR-0051 (line 18 describes the 0808 check this replaces).
8. **Friction.** This is the third copy of one list, so run the
   `engine-friction` skill and add its row to `docs/engine/friction.md`
   (expected outcome: game-side structure now, engine feature proposed).
   Measure with `cargo xtask code-lines` and `cargo xtask pr-size`.

## Acceptance criteria

- [ ] `cargo xtask notices` on the tree as merged changes nothing.
- [ ] Adding one music credit to `audio.ron` with a `uses` row and
      running the command updates both files with no hand edit;
      `the_notices_and_the_assets_table_are_what_xtask_notices_writes`
      fails until then.
- [ ] `grep -n "tiny_tales\", \"shironejiya" crates` and the art-title
      list in `private_assets.rs` find nothing.
- [ ] A new bought pack's credit needs no test edit.
- [ ] ADR-0013 is amended and `docs/engine/friction.md` has the row.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the renderers on a two-credit fixture (one with two uses, one
  hidden); the marker splice keeps the prose; the overlap check fails on
  `battlers/` against `battlers/guard.`.
- Integration: the check test on the real files; the private test in
  `crates/content/tests/it/private_assets.rs` (runs with the checkout).

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
