---
id: "0919"
title: Desktop exports and run --watch pick the game's program again
type: bug
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0919 — Desktop exports and run --watch pick the game's program again

## Context

Found while working 1209. Ticket 0764 (PR #682) gave the package
`trpg-app` a second program, `story-preview` (`crates/app/src/bin/story_preview.rs`,
`[[bin]]` in `crates/app/Cargo.toml`). Since then every engine command that
picks the game's program from `shuyi.ron`'s `export` part refuses:

```text
$ cargo shuyi export windows --without private-assets
export: error[SH1002]: the package `trpg-app` has several programs (story-preview, visions-of-shuyi), and none is named `trpg-app`
$ cargo shuyi run --watch
run: error[SH1002]: the package `trpg-app` has several programs (story-preview, visions-of-shuyi), and none is named `trpg-app`
```

The engine (`shuyi-cli`'s `export.rs`, `program`) takes the bin named as
the package, else the only bin. `shuyi.ron`'s `web` part and an edition may
name a `bin` (`story-preview` does), but the `export` part can't, and the
`demo` edition doesn't. So:

- `release.yml`'s desktop jobs (`cargo shuyi export "$TARGET" --release`)
  fail at the next tag;
- `demo.yml`'s desktop jobs (`--edition demo`) fail;
- `cargo shuyi run --watch` (1209, ADR-0124) can't start the game.

CI on a PR exports only the web (`web.bin` names the program), which is why
#682 was green. `story-kit.yml` names its edition's `bin` and works.

## Nick input

None.

## Scope

**In:**
- The desktop export, the demo's desktop export and `run --watch` pick
  `visions-of-shuyi` again; the `story-preview` edition still builds
  `story-preview`.
- A check that keeps it so (an xtask test, or a CI step that runs the
  program choice without the build).

**Out (do not do):**
- Changing either program's behaviour, name or the saves' folder.
- An engine change without asking first (the `engine-change` skill: Nick
  OKs an engine ticket or PR before it is made).

## Implementation steps

1. Decide, and say why in the PR: (a) **game side**, move the writer's
   preview into its own package (for example `crates/story-preview`,
   package `trpg-story-preview`, with `app_info.rs` reached as it is
   today) and give the `story-preview` edition `package:
   "trpg-story-preview"`, so `trpg-app` has one program again; or (b)
   **engine side**, a `bin` on `shuyi.ron`'s `export` part (and `run
   --watch` reading it) through the `engine-change` skill, after Nick's OK.
   (a) needs nothing from the engine; prefer it unless it costs far more.
2. Make the change; update `shuyi.ron`, `story-kit.yml` and the
   workflow comments that name `trpg-app`'s programs.
3. Run `cargo shuyi export windows --without private-assets` (or the
   host's desktop target) far enough to see it pick `visions-of-shuyi`,
   `cargo shuyi export windows --edition story-preview` to see it pick
   `story-preview`, and `cargo shuyi run --watch` to see the window open.
4. Add the check from **Scope** so a second program can't break the
   desktop export unseen again.

## Acceptance criteria

- [ ] `cargo shuyi export <desktop target>` builds `visions-of-shuyi`, with
      and without `--edition demo`.
- [ ] `cargo shuyi export <desktop target> --edition story-preview` builds
      `story-preview`.
- [ ] `cargo shuyi run --watch` opens the game.
- [ ] A test or CI step fails if the desktop export's program can't be
      chosen.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit or integration (xtask): the program each export picks.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
