# ADR-0123: The writer's preview: an edition without scripts, and a load of some scripts

- **Status:** Accepted
- **Date:** 2026-11-13
- **Related tickets:** 0764; 0152 (the kit, ADR-0122), 0766 (the trial
  packet), engine#2585, engine#2586 (the preview screen), engine#3082 (editions)

## Context

The writer's script preview, `story-preview`, is sent to writers on trial
before one is hired (0766). It plays the scenes of a folder in the game's
own dialogue screen, with the bought art inside it (a build, which ADR-0040
allows). Two things the game had no way to do:

- **Leave the scripts out.** Every build embeds `assets/` with the game's
  content laid over it (ADR-0117), scripts and all. A program sent to an
  applicant must hold only the scenes in its folder: a trial packet is two
  scenes, and the rest of the story must not travel with it.
- **Check some scripts, not all.** `story-check` runs the whole content
  load with the folder's scripts as the game's (`load_with_scripts`). A
  chapter, a battle trigger, a cinematic, a camp event, a support or a
  world-map place that names a scene the scripts lack is an error there,
  and a battle file with one fails to load, so with two scenes nothing
  loads. The preview's own start has no scripts at all.

## Decision

1. **The preview is shuyi.ron's `story-preview` edition** (engine ADR-0100):
   the package `trpg-app`'s second program `story-preview`, the bought art,
   and the content root `crates/content/story-preview`, whose `bundle.txt`
   (the demo list's format, ADR-0119) names `dialogue/`. When an export
   hands that root to the build as `SHUYI_CONTENT`, `crates/content/build.rs`
   leaves what the list names out of **every** table it writes (`assets/`,
   the content, the art, the test files): one filter where a table is
   written, so the pack the export bakes from those tables holds none
   either. The edition's `disclosure` is the project's without the scripts'
   pack entries, and forbids the scripts' paths; the kit job also looks for
   three script lines in the program. No `cfg` is set: the program reads its
   bundle as every build does.
2. **A load of only some scripts** (`trpg_content::load_with_some_scripts`)
   is `load_with_scripts` with the scene-reference checks noting a missing
   scene instead of reporting it: the six checks ask one function,
   `dialogue::scene_missing`, which a thread-local scope (set and put back
   by `with_some_scenes`, panic-safe) turns lenient for that load alone.
   Every other check runs. The scenes noted are returned, and the preview
   says "Only the scenes in this folder are checked" when there are any.
3. **The format's rules on a scene as written** (`FormatRules`): the
   engine's preview runs the format's checks on the scenes it parses, which
   still hold their stand-ins (`{lead|the exile}`); the game's rules read
   that token as the name. The game's own problems are the some-scripts
   load's, less those the format's checks found already. A scene plays with
   its stand-ins written `{lead}` and its line ids kept (`as_shown`), so the
   preview follows the line on screen.

## Consequences

- A writer's program holds no scene but its folder's, and the same rules
  as `story-check` hold its folder's scenes, every one of them when the
  folder is a whole clone of the mirror.
- A new kind of file a script names a scene from must call
  `dialogue::scene_missing`, or a trial packet fails to load in the preview.
- A second content root for an edition is a list in `crates/content/<root>/`
  and a branch in `build.rs`'s `SHUYI_CONTENT` check.

## Alternatives considered

- **A `trpg-content` feature `no-embedded-scripts`** (the ticket's fallback
  before engine#3082): a feature reaches every build that shares the crate
  in one cargo run, and it duplicates what an edition says; the edition is
  pinned, so it lost.
- **Threading a "some scenes" flag through the six loaders' signatures**:
  right in principle, but it changes six loaders, their reference structs
  and some thirty tests for a switch only one load turns on; the scoped
  thread-local keeps them as they are and can't leak past its load.
- **Stub scenes for the missing ids** (load, read the missing ids from the
  messages, load again with stubs): parses messages, and loops as each
  stage's failures uncover the next stage's.
