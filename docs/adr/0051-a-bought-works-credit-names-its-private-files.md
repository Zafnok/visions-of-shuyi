# ADR-0051: A bought work's credit names the private files it covers, and a bought file without a credit is refused

- **Status:** Accepted
- **Date:** 2026-10-03
- **Related tickets:** 0829, 0808, 0436, 0711, 0706, 0413, 0437
- **Extends:** ADR-0032 (rules 5 and 6: the row and the credit of bought
  art), ADR-0040 (what `cargo xtask private-assets --pin` checks)

(ADR-0050 is taken by two PRs open on 2026-10-03.)

## Context

- ADR-0032 rule 6: every bought work is credited on the credits screen.
  Ticket 0711 put two bought portraits in the game and 0436 the bought
  map sprites; neither added a credit, and nothing failed. Nick noticed
  on the Pages build: "credits should always update when we introduce an
  asset."
- The check from 0808 compares the rows of `THIRD_PARTY_ASSETS.md` with
  the credits by source link. It catches a row without a credit. It can't
  catch a **file without a row**.
- The gates never read the bought files (ADR-0040), and a PR's CI has no
  key for the private repository. A gate can't look at them.
- Every bought file reaches a build through one command: after pushing to
  the private repository, the session runs `cargo xtask private-assets
  --pin` and commits `assets-private.rev`. The Pages build then runs the
  opt-in private test before it builds the game.
- The credits file allowed only ADR-0013's open licences and had no group
  for pictures.

## Decision

1. **A credit says which bought files are its.** An entry of
   `assets/data/credits.ron` may have `private: ["units/", …]`: paths
   inside `assets-private/game/`, each a file or the start of several. An
   entry with `private` is a bought work: its `license` is written
   `Custom (<seller>)`, as in its `THIRD_PARTY_ASSETS.md` row. Any other
   entry keeps ADR-0013's list. Pictures go under a new group, `Art`.
2. **Every file in `assets-private/game/` needs a credit that covers
   it**, our own data files there too (a tileset file, a portrait's
   `.ron`): they exist only for the art. The one exception is the
   folder's `README.md`. `trpg_content::credits::uncredited` is the rule.
3. **The rule is checked where the files come in:**
   - `cargo xtask private-assets --pin` refuses to write the pin, naming
     the files. This is the check a ticket's session meets.
   - The private test `every_bought_file_has_a_credit` fails the Pages
     build. This is the backstop for a pin written by hand.
   - The public gates check the other half as before: the credit has a
     row with the same link.
4. **Keep the paths narrow** (`portraits/test_lord`, not `portraits/`),
   so the next ticket's files aren't covered by an old entry and its
   session has to open the credit, where it also adds the pack's artist.
5. **One row and one credit for a bundle bought as one**, naming the
   packs in the game and the artists those packs name. A pack from
   another seller gets its own.
6. **A bought work's credit shows only in a build that has its files.**
   `credits::load` looks for bundle files from the private layer under
   the entry's paths. A clone without the art doesn't use it, and the
   gates' snapshots of the credits screen don't change.

## Consequences

- A ticket that imports bought files can't move the pin without touching
  the credit; the `work-ticket` skill and tickets 0706, 0413 and 0437 say
  so.
- A folder-wide path (`units/`) still lets a new sprite in without a new
  look at the credit. Accepted for the map sprites, which come by the
  hundred from the same bundle; rule 4 covers the rest.
- The credit's wording (author line) is data in a public file; only the
  files are private.
- `--pin` now reads this repository's `assets/data/credits.ron`, so it
  fails when that file doesn't load.

## Alternatives considered

- **A credits file inside the private repository, laid over the public
  one** — the credit would vanish from review here, the row test couldn't
  see it, and nothing would force it to exist.
- **Each importer writes the credit** — three importers (portraits, map
  sprites, tilesets), each needing a table from source folder to pack and
  artist; the pin check covers them all and hand-copied files too.
- **A row and a credit per pack** — the bundle's packs share one seller,
  one licence text and one purchase; five near-identical entries today
  and dozens later say no more than one entry with the artists' names.
- **Always show the credit** — a build without the art would credit work
  it doesn't contain, and every public snapshot would name it.
