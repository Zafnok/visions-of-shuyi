---
id: "1269"
title: "The release and demo desktop exports pick the game's program again"
type: bug
milestone: M8 Release
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1269 — The release and demo desktop exports pick the game's program again

## Context

Ticket 0764 (PR #682) added a second program to `trpg-app`, `story-preview`
(`crates/app/Cargo.toml`, `src/bin/story_preview.rs`), shipped by
`shuyi.ron`'s `story-preview` edition (`bin: "story-preview"`). Since then
`cargo shuyi export <windows|linux|macos> --release` fails on every
target that builds `trpg-app`'s program without an edition naming it:

```
export: error[SH1002]: the package `trpg-app` has several programs
(story-preview, visions-of-shuyi), and none is named `trpg-app`
```

The engine at the pin picks the program named as the package, else the
only one (`crates/cli/src/export.rs`, `fn program`); the `export` part of
`shuyi.ron` has no `bin` (editions do, and an edition's `package` can
replace the target's). Found by ticket 1267's release dry run
(run 43633342376: windows, linux and macos fail at Export, web passes).
The `demo` edition names no `bin` either, so `demo.yml`'s Windows export
fails the same way; the story kit's edition names its `bin` and works.

## Nick input

None.

## Scope

**In:**
- Move the `story-preview` program out of `trpg-app` into a package of its
  own (`crates/story-preview`, package `trpg-story-preview`, binary
  `story-preview`), so `trpg-app` has one program again; the
  `story-preview` edition names that package (`package:
  "trpg-story-preview"`). Its `private-assets` and `packed` features,
  forwarding to `trpg-content` as `trpg-app`'s do.
- Whatever names the old path follows: `story-kit.yml`, the xtask game
  tools (`--story-preview`), `AGENTS.md`'s crate map, the disclosure
  parts if they name the program.
- A test that keeps it so: an xtask test that every package an export
  builds without an edition `bin` has exactly one program or one named as
  the package (read from `cargo metadata`).

**Out (do not do):**
- An engine `export.bin` field (an engine ticket, if the friction review
  wants one; the game doesn't wait for it).
- Anything in `release.yml` (1267).

## Implementation steps

1. Make `crates/story-preview` (its `Cargo.toml` with the workspace's
   fields and lints; move `src/bin/story_preview.rs` to its `src/main.rs`,
   keeping `#[path]` to `crates/app/src/app_info.rs` or depending on what
   it needs). About 30 lines besides the moved file.
2. `shuyi.ron`: the `story-preview` edition gets `package:
   "trpg-story-preview"`; keep `bin`.
3. Update the references (step "In"), then `cargo shuyi app-info --check`.
4. The xtask test.
5. Run `cargo shuyi export windows --release --without private-assets`
   and `--edition story-preview` locally; then a Release dry run and a
   Story kit run on the branch.

## Acceptance criteria

- [ ] `cargo metadata` lists one program for `trpg-app`.
- [ ] A Release dry run builds all four packages, and its `sums` job's
      summary shows `SHA256SUMS` with four lines (1267's two criteria it
      couldn't meet; links in the PR).
- [ ] The story kit's Windows export still builds `story-preview`.
- [ ] The xtask test fails when a second `[[bin]]` is added to `trpg-app`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the xtask test above.
- Integration: the dry runs (links in the PR).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
