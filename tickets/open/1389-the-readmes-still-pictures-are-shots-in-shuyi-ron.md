---
id: "1389"
title: "The README's still pictures are shots in shuyi.ron, made again by one command"
type: infra
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1389 — The README's still pictures are shots in shuyi.ron

## Context

The top-level `README.md` shows nine still pictures of the running game
(`docs/img/title.png`, `lead-select.png`, `dialogue.png`,
`battle-forecast.png`, `battle-glyphs.png`, `combat-scene.png`,
`battle-ranges.png`, `spell-menu.png`, `key-bindings.png`). Each is made
by a `frame-png` command written out by hand in a table of
`docs/img/README.md` (*The other pictures*), with shell variables for the
keys (`$N`, `$B`, `$A`). The file says they "have not been drawn again
since 0125 (2026-10-15)" and that the keys follow Chapter 1's start as it
was then, so each must be checked against the game by eye before
committing it. Nothing says when one is stale.

Engine PR #604 (engine ticket 2813), in the pin since the pin move of
2026-11-27 (`docs/engine/pins.md`), gives a game **shots**: a `shots`
part in `shuyi.ron`, each a name (lowercase letters, digits and `-`) and
the `frame-png` arguments after the output path (`Shot { args }`), always
written to `shots/<name>.png` in the project's root.
`cargo shuyi frame-png --shot <name>` writes one, `--shots` writes every
one, and `shuyi_cli::frame_png::kept::check_shots(project, game)` fails
on a shot that is missing, over `MAX_SHOT_BYTES` (256 KiB, "try
`--scale 1`"), another size, or a pixel that differs, naming the command
that makes it again; a file in `shots/` that no shot names fails too.
The engine's Hello, Grid keeps its book's pictures this way.

The intro animation (`docs/img/intro.gif`, `readme-pictures`, ticket
1264) is a GIF joined from four clips and stays as it is.

The README's pictures show the bought art: commit them only as
screenshots of the running game (`AGENTS.md`, *Bought art*; Nick,
2026-10-05). They need `--features private-assets`, which the gates
never turn on (ADR-0040).

## Nick input

None.

## Scope

**In:**
- A `shots` part in `shuyi.ron` with the README's nine stills, each the
  arguments of its row of `docs/img/README.md` with the variables
  written out (or the shortest equivalent keys today), at a scale under
  256 KiB a picture; `battle-glyphs` the glyph look, as its row says.
- The pictures made again with `cargo shuyi frame-png --shots` (with the
  bought art, as `docs/img/README.md` says how), checked against the game
  by eye once, and the README pointing at `shots/<name>.png`; the old
  `docs/img/<name>.png` of these nine removed.
- A test that runs `check_shots` on the game, only with
  `--features private-assets` (the pictures need the bought art), named
  in `run-gates` beside the other private-assets test, and in
  `docs/img/README.md`.
- `docs/img/README.md`: *The other pictures* says the stills are
  `shuyi.ron`'s shots and how to make them again; the table goes.

**Out (do not do):**
- The intro GIF and `readme-pictures` (1264).
- The PR pictures in `docs/img/` named by ticket (`0468-…` and the
  rest): they show a PR's change, once.
- A gate in CI that needs the bought art (ADR-0040: the gates never read
  it).
- Picking other moments for the pictures: the same screens as today.

## Implementation steps

1. Read `shuyi_cli::project::Config::shots` and `frame_png::kept` at the
   pin, and how Hello, Grid's `shuyi.ron` writes its shots and its test
   (`examples/hello-grid` in the engine).
2. Write each still's arguments into `shuyi.ron`'s `shots`, with a
   comment like the other parts' (the command, the folder, the test).
   Run each with `cargo shuyi frame-png --shot <name>` and the bought art
   until it shows the screen its row describes.
3. Move the README's links to `shots/`, delete the nine old files, and
   rewrite *The other pictures* in `docs/img/README.md`.
4. Add the `check_shots` test behind the `private-assets` feature, next to
   `crates/tools`' other tests of the tool contract, and add its command
   to `run-gates` (*The gates never read the bought art*) and to
   `docs/img/README.md`.

Expected size: about 40 lines of `shuyi.ron`, 20 of test, and the docs.

## Acceptance criteria

- [ ] `cargo shuyi frame-png --shots` (with the bought art) writes the
  nine stills to `shots/`, and `README.md` shows them from there.
- [ ] The private-assets test that runs `check_shots` passes, and fails
  when one shot's keys change (checked once by hand).
- [ ] `docs/img/README.md` has no hand-written table of keys for the
  stills.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: `check_shots` on the game, with `--features
  private-assets`.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
