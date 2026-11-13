# ADR-0121: The public files after the content's move: stand-ins and the test chapter

- **Status:** Accepted (0160); extends ADR-0117, ADR-0118 and ADR-0057
- **Date:** 2026-11-13
- **Related tickets:** 0153, 0156, 0160

## Context

Ticket 0160 moves the game's content into the private repository's
`content/` (ADR-0117), so a copy of the public repository is the code and
a small test game, not this game (0153 §9). What moved: every chapter,
battle, map, world map, cinematic, opening, skirmish and `.dlg` script;
the data tables (characters, classes, items, skills, spells, arts, AI,
the basic shop, camp, supports, names, new game, lessons, tips); and the
audio manifest, whose notes tell the plot. What stays: fonts, sounds,
music and their licences, palettes, terrain, lighting, the keymap, the UI
text, credits, the public placeholder pictures and each format's
`README.md`.

Without the checkout (a fork, a worktree without access), the build lays
no content over `assets/`. But every load of the game needs the data
tables: the loader stops at the first missing one, and the test files
(`assets-test/`, ADR-0057) only **add** characters, names, supports, camp
events and lessons to the story's tables; they replace none. The tests of
mechanisms read the classes, items, spells, skills and arts the test
files name, the `lead` and its name, the support rules, the tips the tip
box shows and the music cues (0162's notes).

## Decision

### 1. A public stand-in for each table the load needs

`assets/` keeps a **stand-in** at the moved file's path for each data
table and the audio manifest. With the checkout, the content's file
replaces it whole (ADR-0117 §2); without, the build reads it. Each starts
with a note saying so and what it holds, keeps the format's notes, and
has no other comment (the moved files' notes name chapters and people).
A stand-in holds what the test game and the tests of mechanisms need:

| Stand-in | Holds |
| -------- | ----- |
| `data/new_game.ron` | The test chapter (`test`) with the lead and `test_knight` |
| `data/characters.ron` | The lead, the guide (a speaker, outside the story), and the skirmish generator's class templates (`skirmish_*`); no other generic or speaker |
| `data/names.ron` | The lead's default name and the guide's |
| `data/classes.ron` | The whole class tree (the test tileset draws every class; its numbers are `docs/design/progression.md`'s) |
| `data/skills.ron`, `data/spells.ron` | Every skill and spell (the heal list's and the spell texts' tests read the personal ones) |
| `data/items.ron`, `data/arts.ron` | Every item and art but the personal weapons (a wielder is a story character) and their arts: the special weapons' tests read the others |
| `data/basic_shop.ron` | Its plain items |
| `data/ai.ron`, `data/supports.ron` | The same rules; no support pair |
| `data/camp.ron` | No event: the test files add theirs |
| `data/lessons.ron`, `data/tips.ron` | Every lesson and tip (the tutorial, which tells no story; the guides' and the tip box's tests read them) |
| `audio/audio.ron` | Every sound, track, pool and credit: the files are public |

A stand-in **never follows** the real file: a change to the game's data
is made in `content/` alone. A change to a **format** changes the
stand-in too, as it changes the test files.

### 2. Without the checkout, New Game is the test chapter

A debug build, or one with the test files (`test-fixtures`), starts the
test chapter as its New Game: a fork builds, runs and passes every test
but the tests of the content, which skip (ADR-0118). A build that ships
has no test files, so without the checkout its content fails to load and
the game says why: this game can only be shipped from the private
repository. The demo edition is Chapter 1 of the content: `build.rs`
writes its tables empty without the checkout and refuses to build the
`demo` edition.

### 3. The real files are listed in the private repository

A public `README.md` keeps its format and never lists the real files
with what happens in them: the places' and openings' tables are in the
private `content/maps/README.md` and `content/openings/README.md`. The
leak check (`cargo xtask check-leaks`) searches every tracked file but
the tickets.

## Consequences

- A fork has a playable test chapter on public files only, and the
  workspace's tests pass without access.
- Two files at one path for the data tables: a reader of `assets/` sees
  a stand-in, and its note says where the real one is.
- A test that reads a real entry missing from the stand-ins fails
  without the checkout: it is a test of the content (guard it, ADR-0118
  §1 (a)), or its entry belongs in the test files or the stand-in.
- Most stand-ins are close copies of the real tables, less their notes
  and what names the story's people: the tests of mechanisms read nearly
  every class, item, art and spell, and the tutorial. What a fork lacks is
  the story: the chapters, battles, maps, scripts, cast and the notes.
  A new entry need only be added to the content's file.

## Alternatives considered

- **The stand-ins in `assets-test/`.** The test files lie under the
  story's and add their entries by id; with the checkout, a table in both
  places would be read as the test files' part of the real table, and
  every id in both is an error. They are also missing from a build with
  no test files, where a stand-in in `assets/` at least loads. Lost.
- **No stand-ins: the tables only in `content/`.** No build without the
  checkout loads, and nothing of the code runs in a fork. Lost.
- **Invented values in the stand-ins.** The tests of mechanisms count on
  the classes' and items' numbers they were written against; inventing
  others changes what they check. Lost: the stand-ins take the real
  entries they need, which are the design documents' already.
- **Keep the tables public, as they are.** A fork would have this game's
  data, which 0153 B+ moves. Lost.
