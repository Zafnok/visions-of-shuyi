---
id: "0909"
title: The web build's NOTICES.txt names the game's own works and license texts
type: feature
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-01
---

# 0909 — The web build's NOTICES.txt names the game's own works and license texts

## Context

Since the engine pin move of 2026-10-27 (engine PR #139, engine ticket
3039; `docs/engine/pins.md`), `cargo shuyi web` writes `dist/web/NOTICES.txt`
beside the page: the crates compiled in, the engine's vendored scripts
(`mq_js_bundle.js`, `quad-storage.js`, `sapp_jsutils.js`) and their license
texts. It lists none of the game's own third-party works, because
`shuyi.ron` has no `notices` entry yet.

The web build ships the Terminus font (OFL-1.1, whose license must travel
with the font) and the music in `assets/audio/` (CC0-1.0 and CC-BY-4.0;
CC-BY needs each work's title, author and source credited). All of them
are listed in [`THIRD_PARTY_ASSETS.md`](../../THIRD_PARTY_ASSETS.md) and
the music in [`assets/audio/audio.ron`](../../assets/audio/audio.ron). The
web build is already published (`.github/workflows/pages.yml`) and will be
on itch.io (0901). Desktop packages carry `THIRD_PARTY_LICENSES.html` for
the crates (`release.yml`) and the credits screen (0808) credits the music
in game; this ticket covers the web page's file.

The engine's side (`shuyi-ship` skill, step 3): `shuyi.ron` takes
`notices: (texts: ["path", …])`, files from the project's root, each put
in `NOTICES.txt` whole; a missing file fails the build.

## Nick input

None.

## Scope

**In:**
- `notices` in `shuyi.ron`, naming the game's license texts and one
  attribution file.
- That attribution file, listing each shipped non-crate work (title,
  author, source URL, license) as `THIRD_PARTY_ASSETS.md`'s *Shipped
  items* table does, and a test that keeps it in step with that table.

**Out (do not do):**
- Bought art (private, ADR-0040): it isn't in the public web build; when
  `--features private-assets` builds ship, its notices are a separate
  ticket.
- Desktop and Playdate packages' notices, the credits screen (0808).
- Any engine change.

## Implementation steps

1. Write `assets/NOTICES-game.txt` (plain text): a heading, then one
   entry per row of `THIRD_PARTY_ASSETS.md` *Shipped items* that the web
   build ships (the Terminus font and every music track; not the Playdate
   SDK file, not `gamecontrollerdb.txt`, which only native builds compile
   in, and not the engine's scripts, which the engine lists itself):
   name, author, source URL, version, license.
2. Add to `shuyi.ron` (top level, beside `web:`):
   `notices: (texts: ["assets/NOTICES-game.txt", "assets/fonts/Terminus-LICENSE.txt", "assets/audio/licenses/CC0-1.0.txt", "assets/audio/licenses/CC-BY-4.0.txt"])`,
   plus `CC-BY-3.0.txt` if any shipped track uses it.
3. Add a test in `crates/xtask` (beside the other `shuyi.ron` tests in
   `crates/xtask/src/game_tools/tests.rs`) that every license file named
   in `THIRD_PARTY_ASSETS.md` *Shipped items* under `assets/` is in
   `shuyi.ron`'s `notices.texts`, and every music title in
   `assets/audio/audio.ron` appears in `assets/NOTICES-game.txt`.
4. Run `cargo shuyi web --release` and read `dist/web/NOTICES.txt`.

## Acceptance criteria

- [x] `cargo shuyi web --release` writes a `NOTICES.txt` that contains the
      Terminus license, the CC0 and CC-BY texts, and each track's title
      and author.
- [x] The new test fails when a track is added to `audio.ron` without an
      entry in `assets/NOTICES-game.txt`.
- [x] `THIRD_PARTY_ASSETS.md`'s header says the web build's notices come
      from `shuyi.ron`'s `notices`.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the xtask test of step 3.
- Snapshot / integration: none.

## Completion notes

`shuyi.ron` has `notices`, naming `assets/NOTICES-game.txt` and the font's and
the audio's license texts (CC-BY-3.0 included: three tracks use it).
`cargo shuyi web --release` writes all of them into `dist/web/NOTICES.txt`.

Deviations:

- The attribution file also lists the 16 sounds of `assets/audio/sfx/`, not
  only the font and music of step 1: the web build ships them, some are
  CC-BY, and the ticket's scope says each shipped non-crate work. The test
  covers every third-party work of `audio.ron` for the same reason.
- `NOTICES-game.txt` is listed in `crates/content/playdate-bundle.txt`: it sits
  under `assets/`, and the Playdate never reads it.
- The engine writes the same `notices` into every package `cargo shuyi export`
  makes, not only the web build's. That comes with the setting.

Follow-ups: none. No gameplay rules decided.
