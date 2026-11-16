---
id: "0170"
title: "Pages builds with nothing private; the bought art's check moves to the playtest page's build"
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: low
status: done
blocked_by: ["0158", "0160"]
nick_input: none
completed: 2026-11-15
---

# 0170 — Pages builds with nothing private; the bought art's check moves to the playtest page's build

## Context

`pages.yml` has failed on every push to `main` since 0160 (PR #676) moved
the game's content to the private repository. Pages checks out the bought
art but never the content (0158), and its art check runs
`private_assets::the_content_loads_with_the_private_assets_and_without_the_test_files`,
the test of a build that ships: the art without the test files. Without
the content the public stand-in `assets/data/new_game.ron` starts the test
chapter, which that build leaves out:

```text
assets/data/new_game.ron: first_chapter: no chapter "test"
assets/data/new_game.ron: roster: no character "test_knight"
```

The test is right; Pages is the wrong place for it. Asked about it, Nick
also wants nothing private on Pages (2026-11-15): a Pages site is public
and the bought art can be pulled out of its build, while the full build
already sits behind the playtest page's password (`playtest-itch.yml`).

## Done when

- [x] `pages.yml` checks out nothing of the private repository: no content,
      no bought art, no voice clips. It builds the public files, the
      placeholder art and the test game, with the debug tools.
- [x] `playtest-itch.yml`, which has the art and the content, runs every
      `private_assets::` test before it builds.
- [x] `docs/releasing.md` and ADR-0040 §6 say so.
- [x] The `private_assets::` tests pass locally with the art and the
      content checked out (8 of 8).
