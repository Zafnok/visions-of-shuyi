---
id: "0877"
title: One command shows what each Act 1 chapter still lacks
type: infra
milestone: M7 Chapter 1 & game flow
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0877 — One command shows what each Act 1 chapter still lacks

## Context

The friction sweep of 2026-11-17
([`docs/research/friction-2026-11-17.md`](../../docs/research/friction-2026-11-17.md),
*Built but not backfilled*, "No per-chapter coverage report") found that
every capability was built and filled for Chapter 1 only, and nothing in
the repository shows the gaps after it. Each audit (the retro of PRs
616-735, 0858's audit of PR 481, the sweep itself) rebuilt the same
matrix by grep. What the sweep measured, re-checked on 2026-11-17 in a
current checkout of the private repository:

- 12 portrait sets in `game/portraits/`; 20 speakers placed on screen
  with `@left`/`@right` have none.
- 123 voice clips (96 `ch01_*`, 27 Vix); 0 for Chapters 2-8.
- 3 placed classes (Swordsman, Brawler, Shadowblade) have no tileset unit
  and no battle picture: about 54 of about 250 placements.
- `battlebacks.ron` keys only `mountain` and `peak`, so the 4 indoor maps
  fall back to the fields backdrop.
- 3 seals are defined in `items.ron`; no shop, battle, world map or
  chapter gives one.
- `supports.ron` has `pairs: []` and `camp.ron` has `events: []`.
- `chapters/` holds `ch01`-`ch05` only; 12 of 24 nodes of
  `worldmaps/act1.ron` are `NotBuilt`.

Each piece's own gate passes, so gaps like these stay unseen for weeks.
Some of them are deliberate: dialogue writing is paused until the writer
is hired (0766, `AGENTS.md` rule 3), and whether voices, promotion,
supports and the other gaps are wanted for the Act 1 playtest is asked
in 0184. So the report keeps "deferred by a decision" apart from
"missing".

Bucket B1 of the sweep (Act 1 completeness). Its siblings: 0878 (class
art and the check that warns on a placed class with no art), and the
amendments to 1064, 0757, 0755, 0756, 0237, 0239, 0606, 1067, 1090 and
0522.

## Nick input

None. Verified by tests of the command; its output goes on 1064's
checklist page.

## Scope

**In:**
- `cargo xtask act1-coverage`: a table of chapter (Ch1-Ch8) by column,
  read from the loaded content and, when present, `assets-private/game/`
  and `voice/`:
  - **battle, map, script**: the chapter file names its battle, the
    battle names its map, the chapter's `.dlg` file exists;
  - **faces**: speakers placed on screen in the chapter's scenes that
    have a portrait set, of all placed;
  - **battle art, map art**: units placed in the chapter's battles
    (story battle and the skirmishes and side quests of its world-map
    nodes) whose class or character has a battle picture / a tileset
    unit, of all placed;
  - **voice %**: voiced, stale and missing lines of the chapter's scenes,
    as `cargo xtask voice status` counts them;
  - **backdrop**: maps of the chapter whose terrain or tiles look has its
    own backdrop rather than the default;
  - **music**: the battle names a cue or pool that exists;
  - **seals**: seals the chapter's shops, battles or chapter rewards give;
  - **supports**: support pairs whose characters are both in the army by
    then.
- Each cell is `ok`, `n/m`, `missing` or `deferred (<id>)`.
- A hand-kept list of deferrals, `crates/xtask/src/act1_coverage/deferred.ron`:
  one entry per chapter range and column, with the ticket or decision
  that defers it (for example voice for Ch2-8 deferred by 0766). Entries
  name chapters and columns only, never a character, a line or a private
  file name.
- A section after the table, **Systems with no data**: every shipped
  system whose data table is empty (today `supports.ron` pairs and
  `camp.ron` events), as a warning line each.
- `--list` prints the ids behind each count (speakers without faces,
  classes without art), for a session to work from; the default output
  is counts only, so it can be pasted into a public ticket or page.
- The command exits 0 whatever it finds: it is a report, not a gate.
- `docs/playtesting.md` or `docs/ROADMAP.md`: one paragraph saying what
  the command is for and that 1064 shows its output.

**Out (do not do):**
- Filling any gap: art (0858, 0861, 0862, 0866, 0878), voices (0757),
  seals (0606), supports and camp (1073), the Ch6-8 battles (0852-0854,
  1090).
- Deciding whether a gap is wanted for the playtest: that is Nick's,
  asked in 0184. A deferral is added to `deferred.ron` only when a done
  ticket or a recorded answer defers it.
- Making it a gate or a CI job.
- Ranking tickets: 0182's job. Either order: if 0182 is done, add one
  line to its output naming this command; if not, leave a line in
  0182's Context saying so.
- The engine: the generic part (every id of table X has a file in folder
  Y) may be an engine candidate for `shuyi lint`; run the
  `engine-friction` skill if the code runs past 3× its estimate, and
  file nothing in the engine without Nick's OK.
- Where the command lives if 0188 (split xtask) is done first: put it in
  the crate 0188 gives the game's content tools. Either order.

## Implementation steps

Expected size: about 350 production lines (one module reading the
loaded `trpg_content::Content`, the private folders and the voice
manifest, and a table printer), judged by `crates/tools/src/lines.rs`
and the `status` part of `crates/tools/src/voice.rs`; about 250 lines
of tests.

1. New module `crates/xtask/src/act1_coverage.rs` (and
   `act1_coverage/tests.rs`), registered in `crates/xtask/src/main.rs`
   beside `Some("lines")`.
2. Load the content with `trpg_content::load_embedded()`. Chapters come
   from `chapters/*.ron` (`intro_scenes`, `battle`, `victory_scenes`);
   world-map nodes from `worldmaps/act1.ron`, each assigned to the
   chapter it opens in; scenes from `dialogue/chNN.dlg`.
3. Faces: the `@left`/`@right` speakers of each chapter's scenes against
   `content.portraits`. Battle and map art: resolve each placed
   template or character to its class through `characters.ron`, then
   check `content.battlers` and the `tiny_tales` tileset's `classes` and
   `characters` (a unit drawn by the tileset's `fallback` counts as
   missing). Portraits, battle pictures, the tileset and
   `battlebacks.ron` are all bought art in `assets-private/game/`: without
   it, print the faces, art and backdrop columns as `no art checkout`.
4. Voice: reuse the code behind `cargo xtask voice status` (move what is
   needed into a function both call); without `voice/`, print
   `no voice checkout`.
5. Backdrop: the battle's map terrain and tiles look against
   `battlebacks.ron`'s keys. Seals: items whose kind is a seal, searched
   in shops, battle rewards and drops, chapter rewards. Supports: pairs
   in `supports.ron` whose two characters have joined by that chapter.
6. Read `deferred.ron` and print `deferred (<id>)` where an entry covers
   a cell that is not `ok`.
7. Systems with no data: check each data table the game ships a screen
   for (supports pairs, camp events, and any other list the content
   loads as empty) and print one warning line each.
8. Run it on the current content and paste its default output into the
   PR description.

## Acceptance criteria

- [ ] `cargo xtask act1-coverage` prints the table for Ch1-Ch8, with the
      Systems with no data section.
- [ ] On today's content it reports Ch2-Ch8 voice as `deferred (0766)`
      or `missing`, the 3 classes as missing battle and map art, no seal
      in any chapter, and supports and camp as empty.
- [ ] A cell covered by `deferred.ron` prints `deferred (<id>)`; one not
      covered prints `missing`.
- [ ] Without `assets-private/game/` or `voice/` the command still runs
      and says which columns it skipped.
- [ ] The default output names no character, line or private file.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each column's count on a small content set built in the test
  (a chapter with one placed speaker without a portrait, one class
  without art, one empty support table); the deferral merge.
- Integration: the command runs on the embedded content and prints 8
  chapter rows.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
