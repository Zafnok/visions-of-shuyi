---
id: "0910"
title: "Release packages made by shuyi export, not by hand in release.yml"
type: infra
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["0909", "engine#3075"]
nick_input: none
completed:
---

# 0910 — Release packages made by shuyi export

## Context

Engine PR #183 (ticket 3004), reached by the pin move of 2026-10-30, adds
`shuyi export <target> --release`: it runs `check --target`, refuses on an
error, builds, and packages for `windows`, `linux`, `macos` and `web`. It also
does what `.github/workflows/release.yml` doesn't:

- `NOTICES.txt`, and the player note on opening an unsigned program;
- the build's paths remapped, and the program searched for the home folder,
  so a release can't carry a developer's paths;
- a stripped program, with the debug file kept beside the package;
- the Linux build in Valve's sniper SDK, so its glibc floor is known and old;
- a universal macOS `.app`, signed ad hoc.

`release.yml` does each package by hand in about 200 lines of shell. It can't
move yet, for two reasons that are engine ticket 3075's: an export turns on no
feature of the game's package, and the game's release builds need
`private-assets` (ADR-0040); and a package holds only the program and the
notes, and the game ships `music/`, `voice/` and the license files beside it
(ADR-0046).

## Nick input

None.

## Scope

**In:**
- `shuyi.ron` names the release's features and the files beside the program,
  as 3075 made it possible. `notices` is 0909's.
- `release.yml`'s Windows, Linux, macOS and web jobs call `cargo shuyi export
  <target> --release`, and upload what it makes, under the release's names.
- The program's debug files are kept as release artifacts.
- `docs/` that describes the release is updated.

**Out (do not do):**
- itch.io (0901) and Steam (0903).
- The Playdate's package (1052).
- Changing what a player gets, other than the additions above: the music and
  voice stay beside the program, and Shironejiya's text stays bundled (0901).

## Implementation steps

1. Move the pin past engine#3075 if it isn't (`engine-pin`).
2. Run `cargo shuyi export windows --release` locally, and compare the file
   list with the last release's zip.
3. Change `release.yml` target by target; run it from a test tag, and compare
   each package's files.

Expected size: about 150 lines of `release.yml` removed.

## Acceptance criteria

- [ ] Each target's package has every file the last release's has, plus
      `NOTICES.txt`, and `export-report.json` beside it.
- [ ] The Windows package runs with the music, on a machine with no MinGW.
- [ ] The private art is in the release builds, and a build without the
      secret still fails or builds as `release.yml` says today.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- A test tag's release run, its packages compared by hand in the PR.
