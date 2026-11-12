# ADR-0119: The Chapter 1 demo is an edition built from a cut bundle

- **Status:** Accepted (0914)
- **Date:** 2026-11-12
- **Related tickets:** 0914, 0915, 0872, 0873, 1036, 1249, 0156, 0157, 0160; engine 3081, 3082, 3099

## Context

Nick wants a web demo a hired writer can play that holds Chapter 1 and
nothing later (0914). Every build embeds every file under `assets/`
(ADR-0005, `crates/content/build.rs`), so all eight chapters' scripts, maps,
battles, the world map and the side quests are in the Wasm as plain text. A
menu that hides Chapter 2 still ships it.

The engine has **editions** (engine 3082, engine ADR-0100): a named build in
`shuyi.ron` with its own features, its own `disclosure` part (the
final-package gate, engine 3081, with canaries that refuse an export with
`SH3012`) and a **content root**, one folder handed to the build as
`SHUYI_CONTENT`. The engine's model is a separate folder of the demo's
files. Here that would be a second copy of every shared data file
(`characters.ron`, `names.ron`, the language file, the fonts), and those
files name what comes later: `names.ron` holds every character's and place's
name, `characters.ron` every recruit and boss, and their comments retell
the chapters. The
ticket rules out a hand-kept second copy.

The Playdate already embeds `assets/` less a list
(`crates/content/playdate-bundle.txt`, ADR-0082), its loaders told what was
left out (`bundle::left_out`).

## Decision

1. **The demo is an edition**, `demo` in `shuyi.ron`: the web export of
   `trpg-app` with the bought art, `default_features: false`, its content
   root `crates/content/demo`, its own `disclosure` part. `cargo shuyi
   export web --release --edition demo` builds it. Release exports refuse
   debug features (engine 3099), so the edition lists none to keep off.
2. **The content root holds a list, not files.** `crates/content/demo/bundle.txt`
   says what the demo leaves out of `assets/` and how it cuts the shared
   files; `build.rs` builds the demo's table from `assets/` with the
   game's content from the private repository laid over it (ADR-0117,
   whenever `assets-private/content/` is checked out) and from
   `assets-private/game/`, with it, through `src/demo_bundle.rs`, which the
   crate's tests check too. Its lines:
   - a folder or a file left out, `+` a file kept back from a folder's line
     (every chapter's folder but Chapter 1's own files, the world map, the
     skirmishes): a new later file is left out until a line keeps it;
   - `keep <file> [field]: <keys>`: only these entries of a file's top-level
     map or list, or of a struct field's list (a map's key, a list's string,
     a struct's `id`; a key keeps its `.` forms). An allow list, so a new
     character or name is left out until Chapter 1 needs it, and the demo's
     content check says so;
   - `swap <file>: <text> => <text>`: `ch01.ron`'s `next: Some("ch02")`
     becomes `next: None` (wiring, not writing).
   - Every RON file the demo embeds loses its comments (line breaks kept,
     so errors keep their line numbers).
   A line that matches nothing stops the build, as the Playdate's list does.
3. **The cfg comes from the content root.** When `SHUYI_CONTENT` names
   `crates/content/demo` (anything else stops the build), `build.rs` sets
   the `demo_edition` cfg, and `bundle::layers` is the demo's tables: the
   whole of `assets/` is not compiled in. No Cargo feature turns the demo
   on, so `cargo build --features …` can't make half a demo.
4. **The game knows it is the demo from its bundle.** The demo's table adds
   one file, `edition.txt` (`demo`); `bundle::is_demo` reads it and
   `Shuyi::demo` keeps it. A test that plays the demo reads the demo's
   bundle (`bundle::demo_layers`, the `demo-bundle` feature, through the
   Harness) and is the demo, with no flag of its own. In the demo, Chapter
   1 ends on "Thanks for playing, {lead}" and "Export save?"
   (`docs/design/demo.md`), and the title has no Load Game.
5. **The demo's bundle names nothing it left out.** Unlike the Playdate's,
   its layer has no `leaving_out` list: those paths are the later chapters'
   file names. Every loader reads whole folders, so none asks for a missing
   file.
6. **The export checks it.** The edition's `disclosure` part allows the files
   the full game's packages may hold on Windows and the web (an xtask test
   keeps the lists equal), forbids the later chapters' embedded paths, and
   holds canaries that are ids and file names, never script lines
   (`shuyi.ron` is public): each later chapter's first scene id, the side
   quests' ids, the later maps, the characters first met after Chapter 1.
   The xtask test checks each canary is in a file of the full game and in
   no file of the demo. CI's `demo.yml` exports the demo on every pull
   request that changes `assets/`, `crates/content/` or `shuyi.ron`.
7. **The exported save is the full game's.** The demo writes no save of its
   own; *Yes* bundles `SaveFile::chapter_cleared` of the campaign as slot 1
   (0872's `save_files::offer`). The save names Chapter 1, not Chapter 2:
   the full game's own `ch01.ron` names `next`, so its import (0873) loads
   it into Chapter 2.

## Consequences

- A change that makes Chapter 1 read a new file or name fails the demo's
  content test (`the_demos_bundle_loads_chapter_1_alone_and_says_it_is_the_demo`)
  until `bundle.txt` keeps it; a change that pulls a later id into a kept
  file fails that test or the demo's export in CI.
- Every build writes the demo's table and its cut copies into `OUT_DIR`; only
  the demo edition and tests with `demo-bundle` compile them in.
- The demo's music is every track: the web build copies `music/` whole, and
  an edition can't choose its tracks yet (the engine's book says the same of
  the Playdate). Tracks name no story, but they are most of the package's
  size (1249). Follow-up: an engine ticket for an edition's music.
- The bought art is cut by the same list (paths alike), not by `keep`.
- The content in the private repository (ADR-0117) is cut by the same
  list, so the list stays the same when 0160 moves the content there; the
  demo export in CI checks it out with the key, as every job that loads
  content does. The demo edition compiles neither the whole of `assets/`
  nor the content's own table in.

## Alternatives considered

- **A copy of the demo's files in the content root** (the engine's model):
  a second hand-kept copy of every shared file, which drifts.
- **A menu lock or a runtime flag:** ships every chapter; ruled out by the
  ticket and engine 3082's context.
- **Splitting the shared files** (a `characters-ch01.ron` per chapter): a
  format change across every loader and the story pipeline, for what a build
  step does.
- **A Cargo feature `demo`:** could be turned on without the content root,
  embedding the whole game with the demo's ending; the content root is the
  one switch, and the export records it.
- **A drop list for the shared files** (leave out `defector`, `king`…):
  shorter, but every new later character would ship in the demo until
  someone listed it.
