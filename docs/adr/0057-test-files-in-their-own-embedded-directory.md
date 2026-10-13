# ADR-0057: Test files live in `assets-test/`, embedded only in debug and debug-tools builds

- **Status:** Accepted
- **Date:** 2026-10-12
- **Related tickets:** 0122, 0803
- **Extends:** ADR-0005 (the asset bundle gains a directory), ADR-0040 (the
  private repository gains a folder), ADR-0023 (what a `debug-tools` build
  holds)

## Context

Everything under `assets/` is embedded in the game (ADR-0005). Until this
ticket that included the files only the tests and the debug tools use: six
test characters and three generic templates, the `test` and `quick`
chapters and battles, a map, four dialogue scripts, two portraits, the
`test` language pack, three tilesets and the sprite test card. Ticket 0803
put the real Chapter 1 beside them, so a sold build would carry both.

Forces:

- **About 300 places in the tests** load the content with
  `trpg_content::load_embedded` (through `Ctx::embedded`, the Harness, the
  bots) and expect the test files. Changing what every test calls is a large
  diff for no gain.
- **The loaders read one global bundle** (`bundle::file`, `bundle::files_in`),
  as ADR-0040's private layer already does.
- **Three data tables are single files** (`data/characters.ron`,
  `data/names.ron`, `data/supports.ron`): the test entries can't be a
  separate file of the same kind unless entries are merged.
- **The debug Quick Battle** (title screen, where `Ctx::debug_tools` is on:
  a debug build and the Pages build, ADR-0023) is made of the test files.
- **Cargo features are shared across a build.** A feature that `xtask` or a
  test turns on is on for every crate built with it.
- **A gate must check the build that ships**, though every gate runs as a
  test build.
- The private repository's `game/` held the bought portraits of
  `test_lord` and `test_knight`. `game/` is embedded whole, so a sold build
  would embed them and then fail to load: their character would be gone.

## Decision

### 1. `assets-test/` mirrors `assets/`

The test files are in `assets-test/`, at the path they had under `assets/`
(`assets-test/maps/test_small.map`). `assets-test/README.md` lists them.
The tools that make test files (`cargo xtask test-card`, `test-tileset`)
write there (`bundle::test_display_path`).

### 2. It is a layer of the bundle, in debug builds and with `test-fixtures`

`bundle.rs` embeds `assets-test/` when
`cfg(any(debug_assertions, feature = "test-fixtures"))`.
`bundle::TEST_FILES` says whether it did.

- A **debug build** has the test files: `cargo run`, `cargo test`, clippy,
  coverage, mutation testing and `xtask` all work as before, with no flag.
- `trpg-content`'s **`test-fixtures`** feature is for release-profile builds
  that need them. `trpg-ui`'s `debug-tools` and `harness` features turn it
  on; `trpg-content`'s own tests and the bots' tests ask for it directly.
- A **release build of `trpg-app` without `debug-tools`** has neither, so
  it embeds no test file. This is the same condition as
  `trpg_ui::DEBUG_TOOLS`: the Quick Battle is offered exactly where its
  files are.

The test layers come **after** the story's: a file is read from the first
layer that has it, and listings merge all layers. A test file never
replaces a story file: no path may be in both, apart from the three data
tables (a test in `bundle.rs` checks it). A test file is shown as
`assets-test/...` in errors.

### 3. The three data tables gain entries by id

`assets-test/data/characters.ron`, `names.ron` and `supports.ron` hold only
the test entries. `bundle::test_part(path)` returns the test files' own
file at a path the story has too; each of the three loaders reads the
story's file, then adds the part's entries:

- **characters**: the part is a whole character file (characters,
  generics, optional speakers), checked on its own. An id the story's
  table already has (a character or speaker for a character or speaker, a
  generic for a generic) is an error.
- **names**: the part is a names file. An id, or a display name, that the
  story's table has is an error.
- **supports**: the part is `(pairs: [...])`; the rules are the story's.
  A pair the story lists is "listed twice", an error.

Nothing else merges. A new table that needs test entries follows the same
pattern (`from_source` for the story, a `with_part` for the test file).

### 4. `bundle::story_only` hides the test files from one thread

`bundle::story_only(|| ...)` runs a closure with the test layers hidden
from the calling thread (a thread-local switch). Inside it every `bundle`
function, and so every loader, answers as in a build that ships. Only
tests call it. `crates/content/tests/it/test_files.rs` loads the content
that way and asserts it validates and holds no id or file starting `test`
and no `quick` chapter; it also asserts that the test files change nothing
of the story's content.

### 5. The game refuses to build with test files by accident

`crates/app/src/main.rs` has a compile-time assertion: a build without
`debug_assertions` and without `debug-tools` fails if
`bundle::TEST_FILES` is true. That happens when a release build includes
`xtask` or the tests (`cargo build --release --workspace`), whose features
bring the test files in. Build the game alone: `-p trpg-app`, as
`release.yml` and `cargo xtask web` do. CI's `cargo xtask web --release`
compiles this assertion on every PR.

### 6. The private repository gets `test/`

The bought art of test files moves from the private repository's `game/`
to a new **`test/`** folder (same paths: `test/portraits/test_lord.ron`).
With `private-assets` **and** the test files, `bundle.rs` embeds
`assets-private/test/` as a fourth layer, before `assets-test/`, so a
bought test file replaces its placeholder; it is shown as
`assets-private/test/...`. `bundle::is_private` says whether a shown name comes
from either private folder (the credits screen and the opt-in test use
it). `cargo xtask private-assets` and `.github/actions/private-assets`
check `test/` out too, and `--pin` refuses a file in `test/` without a
credit, like one in `game/` (ADR-0051; a credit's `private` paths cover
both folders).

Layer order, first wins: `assets-private/game/`, `assets/`,
`assets-private/test/`, `assets-test/`.

## Consequences

- A sold build holds only the story: smaller, and nothing named `test_`
  can leak into a save, a credit or a language pack.
- No test changed what it loads or checks. Expected error paths that named
  a moved file now say `assets-test/...`.
- The story's files must validate without the test files, and a gate
  checks it. A story file can no longer lean on a test character, scene or
  map by accident.
- **A release-profile test run of `trpg-app`** (`cargo test --release -p
  trpg-app`) fails at the assertion of §5, because the Harness brings the
  test files in. Nothing runs it today; run the app's tests in the debug
  profile.
- `cargo xtask check-script` and `check-keys` read `assets-test/` as well
  as `assets/`.
- A pin of the private repository from before this ADR has no `test/`
  folder: a debug or debug-tools build with `private-assets` fails to
  compile until `assets-private.rev` is at or after the commit that added
  it (this ticket's PR moves the pin).
- Adding a test file means deciding which directory it belongs in. The
  rule: a file the story's content never refers to, that exists for tests
  or debug tools, goes in `assets-test/`.

## Alternatives considered

- **Extra files handed to the loader by each caller**
  (`load_with_scripts`-style, the ticket's second candidate): every one of
  the ~300 test call sites, the Harness, the bots and the debug tools would
  pass the test files, and the loaders would stop reading one bundle. The
  Quick Battle in a debug build would still need the files embedded
  somewhere.
- **A cargo feature alone, without `debug_assertions`**: `cargo run -p
  trpg-app` would lose the Quick Battle unless every debug command line
  named the feature, and `trpg_ui::DEBUG_TOOLS` (debug builds) would offer
  a Quick Battle whose files are missing.
- **Checking the shipped build in a separate CI job built without the
  feature**: feature unification makes a test build of the workspace
  always have the test files, so the check would need its own compile of
  `trpg-content` on every PR. `story_only` checks the same loaders on the
  same data inside the normal test run, and §5 covers the build flags.
- **Replacing the three tables' files in the test layer with full copies**
  (story and test entries together): the story's entries would exist
  twice, and a test would pass on a stale copy.
- **Leaving the bought test portraits in `game/`**: a sold build would
  embed them and fail to load. **Deleting them**: the Pages build's Quick
  Battle and dialogue test would go back to placeholders.
- **`build.rs` merging the directories into `OUT_DIR`**: rejected in
  ADR-0040 for the same reason (the merge rule would be out of the tests'
  reach).
