---
id: "0829"
title: "Credit the bought art, and refuse bought files that have no credit"
type: bug
milestone: M7 Chapter 1 & game flow
model: opus-5.5
effort: medium
status: done
blocked_by: []
nick_input: sign-off
completed: 2026-10-03
---

# 0829 — Credit the bought art, and refuse bought files that have no credit

## Context

Nick, 2026-10-03: "our walkers are not yet credited to the tiny tales
pack. why is that? credits should always update when we introduce an
asset. make it so."

Ticket 0436 put the bought Tiny Tales map sprites in the game, and 0711
two bought test portraits before it. Neither added a credit, though
ADR-0032 rule 6 says every bought work is credited and
`THIRD_PARTY_ASSETS.md` says a pack gets its row and credit "in the
ticket that first puts it in the game". Three reasons:

1. **Nothing checked it.** The test from 0808
   (`every_third_party_asset_has_a_credit`) compares the rows of
   `THIRD_PARTY_ASSETS.md` with the credits. It can't see a file that has
   no row, and the gates never read the bought files (ADR-0040).
2. **Ticket 0436 had no step for it** (0706 and 0413 have one for the
   row, and none of them names the credit).
3. **The credits file couldn't hold it:** it has no art group and allows
   only the open licences of ADR-0013, not a bought pack's own licence.

## Nick input

**Sign-off** after merge, on the Pages build: Credits shows an **Art**
heading with the Tiny Tales entry. Comment on the wording (see *Claude's
starting rules* in the completion notes).

## Scope

**In:**
- The credits file can credit a bought pack and says which folders of
  `assets-private/game/` the credit covers.
- The Tiny Tales credit and its row in `THIRD_PARTY_ASSETS.md`.
- The credit shows only in a build that has the bought files.
- A bought file that no credit covers is refused in two places: by
  `cargo xtask private-assets --pin`, and by the opt-in private test the
  Pages build runs.
- The step in the `work-ticket` skill and in the open tickets that bring
  more bought art (0706, 0413, 0437).
- An ADR.

**Out (do not do):**
- A row per pack with each pack's own store page: one row and one credit
  for the bundle, which is how it was bought.
- Checking our own public images against a list (the row test already
  covers third-party public files).
- Moving the credit headings into the language file (0234).
- Any change to the private repository.

## Implementation steps

1. **ADR** (`write-adr`): a credit names the private folders it covers;
   where the check runs; the credit shows only with the files.
2. **Bundle** (`crates/content/src/bundle.rs`): `private_files()`, the
   paths of the private layer (empty without the feature).
3. **Credits** (`crates/content/src/credits.rs`):
   - `CreditGroup::Art`, after `SoundEffects`.
   - `FileCredit::private: Vec<String>`: path prefixes inside
     `assets-private/game/`. An entry with them is a bought work: its
     licence is `Custom (<seller>)`; an entry without them keeps
     ADR-0013's list. Validate: no empty prefix, none starting with `/`.
   - `uncredited(file, paths)`: the paths no credit covers (`README.md`
     in the root is the folder's own note and needs none).
   - `merge` takes the private paths: a bought entry is shown only when
     one of them is under its prefixes.
4. **Screen** (`crates/ui/src/screens/credits.rs`): the heading `Art`.
5. **Data**: the Tiny Tales entry in `assets/data/credits.ron`, the row
   in `THIRD_PARTY_ASSETS.md`, and that file's guidance (one row for a
   bundle, naming the packs in the game).
6. **Checks**:
   - `crates/content/tests/it/private_assets.rs`: every bought file has a
     credit, and the Tiny Tales credit is in the list.
   - `crates/xtask/src/private_assets.rs`: `--pin` reads
     `assets/data/credits.ron` and the files of `assets-private/game/`
     and refuses, naming the files, when one has no credit.
7. **Process**: `work-ticket` skill, tickets 0706, 0413, 0437,
   `assets/data/credits.ron`'s header.

## Acceptance criteria

- [x] With the bought files in the build, Credits shows `Art` and
      `"Tiny Tales" by Megatiles …`; without them the screen is unchanged
      (existing snapshots).
- [x] `every_third_party_asset_has_a_credit` passes with the new row.
- [x] `cargo xtask private-assets --pin` refuses a checkout with a file no
      credit covers and names it (test).
- [x] The opt-in private test fails on an uncredited bought file (run with
      `--features private-assets`).
- [x] Each new validation error has a test with its message.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the new field's validation; `uncredited`; `merge` with and
  without private paths; the heading; `--pin` with covered and uncovered
  files.
- Snapshot / integration: existing credits snapshots unchanged.

## Completion notes

**Done.** ADR-0051 (0050 is taken by two open PRs).

- `assets/data/credits.ron` has the Tiny Tales entry, group `Art`, with
  `private: [...]` naming the two test portrait folders, the tileset file
  and `units/`. `THIRD_PARTY_ASSETS.md` has its row and says how the next
  ticket fills it in.
- `trpg_content::credits`: `CreditGroup::Art`, `FileCredit::private` and
  `covers`, `uncredited`; `merge` takes the build's bought files and shows
  a bought entry only when one is under its paths.
- `cargo xtask private-assets --pin` refuses, naming the files, when a
  file of `assets-private/game/` has no credit. The private test
  `every_bought_file_has_a_credit` does the same on the Pages build.
- The `work-ticket` skill and tickets 0706, 0413 and 0437 say to update
  the credit.
- Nothing changed in the private repository; `assets-private.rev` is the
  same.

**Deviations.** No `bundle::private_files()`: `credits::load` filters the
bundle's files by `display_path`, which is already tested (a function
that is always empty in a gate can't be mutation-tested).

**Claude's starting rules (veto any):**

- The heading is **Art**, after *Sound effects* and before *Fonts*.
- The entry reads `"Tiny Tales" by Megatiles (artists: Rayane Félix,
  Lunatic Red, Kodots Games Studio)`, then `Custom (Megatiles) ·
  https://megatiles.itch.io/`. The names are the ones the packs' licence
  files and our purchase record give; "Custom (Megatiles)" is how the
  asset list writes a seller's own licence.
- One credit for the whole bundle, not one per pack.
- A build without the bought art (a clone of the public repository)
  doesn't show the entry.

**Seen:** `frame-png` with `--features private-assets`: the credits
scrolled to the end show *Art* and the entry between the sounds and the
font. With `"units/"` taken out of the entry, `--pin` and the private
test both failed and named the twelve sprite files.

**Follow-up tickets:** none.
