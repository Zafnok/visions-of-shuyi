---
id: "0929"
title: "A package with the bought art never ships without its voices, and a broken install says what to do"
type: infra
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0929 — A package with the bought art never ships without its voices, and a broken install says what to do

## Context

An audit of the Chapter 1 demo before the writer trial (0766, 2026-11-19) found three gaps in how a package is built and what it tells a player.

**Voices can go missing and nothing turns red.**

- `shuyi.ron`'s `export.beside` has `(path: "voice", optional: true)`, and `docs/releasing.md` (*What a release holds*) says "only `voice` is optional". A package exported with no `voice/` folder beside the repository passes.
- `.github/actions/private-assets/action.yml`, step *Put the voice clips beside the game*, copies them only `if [ -d assets-private/voice ]`. With no folder it skips quietly.
- `scripts/check-release-package.sh` line 47 builds the expected list with `if [ -d voice ]; then tree voice ...; fi`, so with no `voice/` it expects none.
- `.github/workflows/demo.yml` never runs that script. Its *Check the reports* step greps only the edition and the content root.

So a release or demo run whose checkout lost `voice/` ships a silent game, green. That covers the web demo (ADR-0046: web builds ship `voice/` "whenever it is there") as well as Windows. The trial brief tells applicants they will hear the voices in the demo.

**The README beside the program is out of date.** `assets-src/package/README.txt` says, for Windows: "Keep the music folder next to it: the game's music is loaded from there" (Linux says the same). Since 1265 (ADR-0120) the program also needs `trpg-app.pack` beside it, and packages with the art also carry `voice/`. The README names neither.

**A missing pack shows a developer's message.**

- `crates/app/src/main.rs` `startup()` turns the engine's error into the start's error with `.map_err(|e| e.to_string())`. The game shows it in place of the title.
- The engine's `PackFileError::Missing` reads "the game's files, {file}, aren't beside the program (looked in …): bake it again with the game's export, and keep it beside the program" (`shuyi_app::PackFileError`, engine `crates/app/src/pack_file.rs`).
- A player who copied the exe alone is told to "bake it again".

Related: ADR-0040, ADR-0046, ADR-0117, ADR-0120. Tickets 0910 (the package and README), 0911 (voice in the disclosure list), 0914 (the demo edition), 0915 (the demo's itch.io pages), 1265 (the pack).

For the writer trial (0766) this is a note only, not a blocker: the trial uses the web demo only (Nick, 2026-11-14).

## Nick input

None. The README's new lines and the start error's sentences only say which files to keep together and what to do when they aren't. They are the agent's starting rules, listed in the Completion notes as 0914's were, and Nick can veto them. Nothing here is a game-design choice.

## Scope

**In:**
- The private-assets action fails, with an `::error::` line, when a run asked for the art with voices (`art: true`, `voice: true`) and the checkout has no `assets-private/voice`.
- `scripts/check-release-package.sh` fails a package checked with the `private-assets` argument when the repository has no `voice/` folder.
- `demo.yml`'s *Check the reports* step, for a run built with the art, requires the package's `export-report.json` to list `voice/en/cast.ron`, on both targets.
- `assets-src/package/README.txt`: the Windows and Linux paragraphs name `trpg-app.pack` and the `voice` folder as files to keep next to the program.
- `crates/app/src/main.rs`: an error reading the pack becomes a player's sentence. It names the file and says to keep it next to the program or to download the game again. The engine's own text follows as a detail line.
- `docs/releasing.md`, *What a release holds*: `voice` is optional only for a build without the bought art; the CI checks refuse an art build without it.

**Out (do not do):**
- Making `voice` non-optional in `shuyi.ron`. A build without the bought art (forks, PR checks) has no clips, and the engine's `optional` has no per-feature switch.
- Any engine change. The engine's `PackFileError` text stays; a better engine wording goes through the `engine-change` skill as its own ticket.
- The desktop demo's line after *Export save?*. That is 0930, filed with this one ("The desktop demo tells the player where its exported save went").
- 0177 (CI fetches `voice/` for the voice tests). Either order: both edit the *Put the voice clips beside the game* step's `if:`. Whichever lands second keeps the other's condition and this ticket's failure.
- 0775 (the trial's demo and web preview pushed together, by hand only). Either order: both edit demo.yml's export job, and 0775 gives its matrix an `edition` key. Whichever lands second keeps the other's change. The voice check in *Check the reports* applies only to `edition == 'demo'` builds with the art, never to the `story-trial` edition.
- 0172 (demo.yml's paths), 0175 (dry runs of dispatch-only workflows), 1229 (the exe's icon, which also edits check-release-package.sh). Either order: different lines.
- The trial packet and `START-HERE.txt`. 0766's trial uses the web demo only (Nick, 2026-11-14).
- `HOW-TO-OPEN.txt` (the engine's export writes it).

## Implementation steps

1. **Size first** (rule 9): about 60 production lines.
   - The action's check: 8.
   - The script's check: 8.
   - demo.yml's step: 12.
   - The README: 6.
   - main.rs's message: 25.
   - Plus about 40 lines of tests, 10 of docs and this ticket's file: about 400 changed lines.
   - Measure with `cargo xtask code-lines`.
2. **The action.** In `.github/actions/private-assets/action.yml`, the *Put the voice clips beside the game* step: in place of the silent `if [ -d assets-private/voice ]`, write `echo "::error::assets-private/voice is missing: a build with the bought art ships the voice clips (ADR-0046)"` and `exit 1` when the folder isn't there. The new failure fires only when the action's `voice` input is true (with `art: true`); a run with `voice: false` skips the step as today and never fails here. Read the step's `if:` first; if 0177 has landed, keep its `voice-without-art` condition.
3. **The release check.** In `scripts/check-release-package.sh`, when the third argument is `private-assets` and `voice/` isn't a folder, `fail` with a line naming `voice/` and ADR-0046, before the file lists are compared. Without the argument, keep today's behavior (no art, no clips expected).
4. **The demo's check.** In `.github/workflows/demo.yml`, *Check the reports*: pass `steps.private.outputs.features` in as an env var. When it is non-empty (built with the art) and, once 0775 has landed, the matrix's `edition` is `demo`, `grep -q 'voice/en/cast.ron' "$folder/export-report.json"` must succeed, or the step prints an `::error::` naming the missing voices and fails. Confirm the report's path format against a real report first (the export lists every file of the package with its hash, `docs/releasing.md`).
5. **The README.** In `assets-src/package/README.txt`, add `trpg-app.pack` (the game's files) and the `voice` folder (when there is one) to the Windows and Linux lines that say to keep the music folder next to the program. Say the game won't start without the pack. macOS and the web stay: the macOS app holds them in its Resources, and the web line already says to serve the whole folder.
6. **The start error.** In `crates/app/src/main.rs`, replace `.map_err(|e| e.to_string())` in `startup()` with a function `pack_error(e: &shuyi_app::PackFileError) -> String`:
   - for `Missing`: a sentence naming the file and saying to keep it in the same folder as the program, or to download the game again;
   - for `Unreadable` and `Refused`: a sentence saying the game's files are damaged or from another version, so download the game again;
   - then, on its own line, `e.to_string()` as the detail;
   - `_ =>` for the enum's other variants, since it is `#[non_exhaustive]`.
   The text can't come from `assets/lang` (the text tables are inside the missing pack), so it is a constant in the program. If `cargo shuyi check-text` flags it, add a `// check-text:` reason comment as `crates/ui/src/flow/demo.rs` does.
7. **The docs.** In `docs/releasing.md`, *What a release holds*, say which checks refuse an art build without `voice/`, and that the README names the pack.
8. **The recorded runs** (acceptance below), then the `run-gates` skill.

## Acceptance criteria

- [ ] `assets-src/package/README.txt`'s Windows and Linux paragraphs each name `trpg-app.pack` and the `voice` folder.
- [ ] Unit test `a_missing_pack_tells_the_player_to_keep_it_beside_the_program` (in `crates/app/src/tests.rs`) builds a `PackFileError::Missing` for `trpg-app.pack`. It checks that `pack_error`'s first line names `trpg-app.pack` and holds neither "bake" nor "export".
- [ ] Unit test `a_refused_or_unreadable_pack_says_to_download_the_game_again` checks the first line of both other variants says to download the game again, and that the engine's text follows on a later line.
- [ ] A recorded local run, both outputs pasted in the Completion notes:
  - `cargo shuyi export windows --release` with `voice/` moved aside, then `bash scripts/check-release-package.sh windows dist/export/windows private-assets` exits non-zero with the voice message;
  - with `voice/` back and the package exported again, the script passes.
- [ ] A by-hand Demo run on the branch (*Push the demo* unticked) is green on web and Windows, and its *Check the reports* log shows the voice check ran (run linked in the Completion notes).
- [ ] On the branch, a throwaway commit makes the action look for a misnamed folder. A Demo run with it fails at *Put the voice clips beside the game* with the `::error::` line (run linked). The commit is reverted before merge.
- [ ] Without the art, `bash scripts/check-release-package.sh windows dist/export/windows` (no third argument) and the PR's Demo job still pass with no `voice/`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `pack_error`'s text for each `PackFileError` variant (`crates/app/src/tests.rs`).
- Integration: the recorded `check-release-package.sh` runs with and without `voice/`; the by-hand Demo run and the throwaway red run of the action's check.
- No snapshot or picture: the start error's look doesn't change, only its words.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations, follow-up tickets created, the agent's starting rules for the README and error wording, notes for Nick.)*
