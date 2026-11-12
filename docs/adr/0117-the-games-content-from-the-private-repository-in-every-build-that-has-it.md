# ADR-0117: The game's content from the private repository, in every build that has it

- **Status:** Accepted (0156); extends ADR-0040 and ADR-0057
- **Date:** 2026-11-12
- **Related tickets:** 0153, 0156, 0157, 0158, 0159, 0160

## Context

Ticket 0153 (`docs/research/0153-keeping-the-story-private.md`) compared
ways to keep the story private. Nick picked B+: the code stays public with
free CI and every check kept, and the story and the game's data move into
the private assets repository (`Zafnok/visions-of-shuyi-assets`, ADR-0040).
A copy of the public repository then builds the code and a small test
game, not this game.

The bought art already lives there, in `game/`, laid over `assets/` by the
`private-assets` feature. That layer can't carry the content:

- **The gates never read the art** (ADR-0040 §5), so the checks never
  depend on it. The content is the opposite: if the gates read the public
  files instead of the real ones, B+ weakens the checks (0153 §3.2, §7).
- **Every file in `game/` needs a bought-art credit** (ADR-0051), and the
  engine's `private-assets --pin` refuses one without. A folder outside
  `game/` needs no credit and no engine change.

0153 §3.2 recommends its design (iii): every CI job that loads content
checks the real content out over the public files, and a build without it
still builds on what is public.

## Decision

### 1. Two new folders in the private repository

- `content/`: the game's own data and scripts, laid out like `assets/`.
  0160 moves the files. Its `README.md` says what goes there.
- `story/`: the story's documents (0159 moves `docs/story/` and the story
  sections of the design documents). No build reads it.

`shuyi.ron`'s `private_assets.folders` lists both, so `cargo shuyi
private-assets` checks them out beside `game/`, `test/` and `voice/`.
`credits.folders` stays `["game", "test"]`: neither needs a credit.

### 2. The build lays `assets-private/content/` over `assets/` whenever it is there

`crates/content/build.rs` embeds every file of `assets-private/content/`
(but the `README.md`s, ADR-0105) as a layer of its own **when the folder is
there**. There is no cargo feature: the checkout is the switch.

- **Order**, top first: the bought art (`private-assets`), the content,
  `assets/`, then the test files' bought art and the test files
  (ADR-0057). A file of the content replaces `assets/`'s at its path; the
  bought art replaces both. `bundle::layers_with` is the order, and a test
  lays a folder of its own in it.
- **Shown as** `assets-private/content/<path>` (`CONTENT_DISPLAY_ROOT`).
- **Private, not bought art.** The layer is marked private in the engine's
  sense: its files may not be handed out, they count in the bundle's
  fingerprint, and a file no layer has is still shown under `assets/`. The
  game's `bundle::is_private`, which picks the files that need a bought
  art's credit, stays false for it.
- **The Playdate** gets the same layer: the content less what
  `playdate-bundle.txt` leaves out, its pictures made one-bit into
  `OUT_DIR/onebit-content/`. A rule of that list must match a file of
  `assets/` or of the content.
- **No checkout:** the layer is left out. A fork, or a worktree without
  access, builds and runs on the public files.

**Cargo's watch.** The build watches the content when it is there; else
the checkout's git index, or the checkout folder. A path cargo watches
must exist (while one is missing, cargo rebuilds the crate on every build),
so with no checkout at all the build makes `assets-private/`, empty and
git-ignored, for `cargo shuyi private-assets` to clone into. `bundle.rs`'s
test that every layer is its folder fails, naming `cargo clean -p
trpg-content`, if a build still missed a checkout made after it.

### 3. Every CI job that builds or loads content checks it out

`.github/actions/private-assets` takes `content: "true"` (sparse
`content/`) and `art: "false"` (none of `game/`, `test/`, `voice/`). With
`required: "true"` a run without the key fails, so CI always checks the
real content. The jobs:

- `ci.yml`: clippy, test (×3), doc-tests, wasm, no-std, playdate, docs,
  coverage, tickets, bench;
- `mutants.yml`: plan, baseline, the diff's shards, the weekly baseline and
  shards;
- `playtest.yml`: build and the battle jobs. The battles are listed from
  `assets/battles/` and the content's `battles/`.

Not: fmt, changes and the result jobs, CodeQL (`build-mode: none`),
security, scorecard, sonar-triage and engine-watch (they load no content),
`pages.yml` and `release.yml` (0158 decides what they ship), and `ci.yml`'s
`export` (0158: its web package is an artifact, see §4).

**Dependabot** runs get only Dependabot secrets. Its `PRIVATE_ASSETS_KEY`
is a second read-only deploy key of the private repository ("visions-of-
shuyi dependabot (read-only)"), so the builds' key and its Actions secret
stay as they were.

**cargo-mutants** copies the tree for each mutant, by default without what
`.gitignore` lists, so `.cargo/mutants.toml` sets `gitignore = false`: the
mutants build with the content, as the unmutated run does.

### 4. Nothing that embeds the content becomes a public artifact

Anyone signed in to GitHub can download a public repository's workflow
artifacts. So:

- the `wasm` job no longer uploads its web build;
- `export` checks out no content, and its package stays an artifact for
  `bench`;
- `playtest.yml`'s xtask, which the battle jobs share, embeds the content:
  it was already kept a day only, and stays so.

A failing test's log may still quote a line of the content. That is the
price 0153 §3.2 accepted for checks on the real content.

### 5. The tools that read `assets/` from disk read the same view

`crates/xtask/src/content_layer.rs`: `check-script` and `voice` read the
dialogue folder with the content's files over `assets/`'s, by name;
`check-keys` and `lint` (through `cargo shuyi`) scan the content too, its
keymap left out as `assets/`'s is. `lines` and `lang-status` read the
embedded bundle, which has the content already.

### 6. A change to the content is two pull requests

The `content-change` skill: a branch and a pull request in the private
repository, never a push to its `main`; after it merges, the game's pull
request moves `assets-private.rev` with `cargo shuyi private-assets --pin`,
after checking that every private commit the move carries has its game
pull request merged (0153 §3.3). A ruleset on the private repository's
`main` blocks force pushes and deletions and asks for a pull request, with
Nick able to bypass it (the art importers still push art there).

## Consequences

- Every gate reads the real game, on every runner and for Dependabot.
  The public repository still builds without access.
- Until 0160 moves files, `content/` holds only its README: no build
  changes.
- A local worktree reads the content once `cargo shuyi private-assets` has
  fetched it, with nothing to turn on, so local gates match CI's.
- After 0160, a test that names a moved file's display path or reads a
  moved file's public stand-in reads the real one in CI. 0157 sorts the
  tests that need the real content from those that read the test game.
- Forks' pull requests fail the content jobs (no key). The repository has
  none, and B+ accepts it (0153).
- The build writes outside `OUT_DIR` once: the empty `assets-private/`.
- A content change is two pull requests, and two moves of the pin conflict
  in one line (ADR-0040 §3).

## Alternatives considered

- **A `private-content` cargo feature** (ticket 0156's plan). The engine's
  tools build the game with the features `shuyi.ron` lists (`game`,
  `web`, `export`): `cargo shuyi bench` builds the tool program with
  `game.features` alone, which can't name a feature a build without the
  checkout lacks. Every other job and every local gate would need the
  flag in its own spelling (`--features trpg-content/private-content`,
  `--private-content`, `--with private-content`), and a job that forgot
  it would quietly read the public files. Lost.
- **An environment variable as the switch.** Reaches every cargo a job
  starts, but every local shell would need it too, and it says no more
  than the folder's presence. Lost.
- **The content in `game/`.** Every file would need a bought-art credit
  (ADR-0051), the gates would read the bought art, and the engine's
  `--pin` check would need changing. Lost.
- **Stand-ins checked by the gates, the real content by one keyed job**
  (0153's design i). Mutation testing, the bots and coverage would judge
  the stand-ins: a gate change. Lost.
