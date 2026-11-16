# ADR-0126: The trial's web preview is the demo's cut without the scripts

- **Status:** Accepted
- **Date:** 2026-11-15
- **Related tickets:** 0771; 0764 and 0767 (the preview and its web build,
  ADR-0123, ADR-0125), 0766 (the writer trial), 0914 (the demo, ADR-0119)

## Context

About five writers on trial (0766) try their scene in the web script
preview on a password-protected itch.io page. Its page of names and faces
(the Unit info key) lists every `{n:<id>}` of `data/names.ron` and every
character, so it named the whole cast, Act 2's included. Nick,
2026-11-15: "probably dont include anyone past ch1 for this part". The
packet's own story pages already stop at the arc summary.

The `story-preview` edition (ADR-0123) is built twice by `story-kit.yml`:
for Windows, the kit the hired writer gets through the private mirror,
and for the web, the page the applicants use. The hired writer writes the
whole game and needs every name.

The Chapter 1 demo edition (ADR-0119) already embeds only what Chapter 1
uses: `crates/content/demo/bundle.txt` keeps Chapter 1's files and cuts
`data/names.ron` and `data/characters.ron` to Chapter 1's entries, and its
export's canaries refuse a package that holds a later id.

## Decision

1. **A `story-trial` edition** in `shuyi.ron`: the `story-preview` package
   and program on the content root `crates/content/story-trial`, whose
   `bundle.txt` lists `dialogue/`. When an export names that root,
   `crates/content/build.rs` sets the `demo_edition` cfg (the bundle is
   the demo's tables) and leaves the list's paths out of every table, as
   it does for `story-preview`. The page of names and faces stays as it is:
   it lists what the pack holds, and the pack holds only Chapter 1's names
   and characters, not later ones hidden at run time.
2. **The web preview is `story-trial`; the Windows (and macOS) kit stays
   `story-preview`** with every name. `story-kit.yml`'s matrix names each
   preview's edition.
3. Its `disclosure` is the preview's files beside the program, the demo's
   pack entries without `dialogue/ch01.dlg`, both editions' forbidden
   paths, and the demo's canaries with the later cast's ids (ids only:
   `shuyi.ron` is public). An xtask test keeps those lists in step and
   checks every canary is in the full game and in no file of the trial's
   bundle.

## Consequences

- Applicants' page names Chapter 1's cast only; the export fails if a
  later id comes back into a shared file, as the demo's does.
- The trial's bundle carries the demo's `edition.txt`, so `is_demo()` is
  true in it. The preview reads nothing that depends on it.
- A trial packet from a later chapter would need its own cut: today the
  trial scene is Chapter 1's (0766).
- What the demo's list keeps, the trial keeps: a name added to the demo
  reaches the trial too.

## Alternatives considered

- **Filter the page at run time to the ids Chapter 1 uses**: the later
  names would still travel in the pack, which anyone with the page can
  read.
- **Build the web preview of `story-preview` with a target check in
  `build.rs`** (cut when the target is `wasm32`): no new edition, but the
  cut would hang on the target rather than on who the build is for, and
  the export's disclosure couldn't name the later cast for the web alone.
- **A second list of Chapter 1's names for the trial**: a copy of the
  demo's that would drift from it.
