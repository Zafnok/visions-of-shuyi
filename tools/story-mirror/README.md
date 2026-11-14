# Visions of Shuyi: the story

The scripts and story documents of *Visions of Shuyi*, a tactical RPG,
copied here for the game's writer. You rewrite the scenes in plain text
files, watch them in a preview program with the game's own faces and
music, and propose each change as a pull request. This repository is a
mirror: the game's own copy is the real one, and every change to it
lands here by itself.

**These files are confidential.** They are shared with you to write the
game; don't pass them on or post them anywhere.

## Read, in this order

1. [`story/cheat-sheet.md`](story/cheat-sheet.md): everything a script
   can say, on one page.
2. [`story/writers-guide.md`](story/writers-guide.md): setting up,
   sending a change, the preview, checking, what you may change, and what
   a rewrite costs.
3. [`story/README.md`](story/README.md), "For a human writer": the story's
   documents, in the order to read them.

The scripts in `dialogue/` now are drafts written by AI. Keep a line, or
don't.

## What is here

| Folder | What | Can you change it? |
| ------ | ---- | ------------------ |
| `dialogue/` | The scripts, one `.dlg` file per chapter or side quest, and `README.md`, the format | The scripts, yes. The format: suggest it in an issue |
| `story/` | Beats, bible, characters, outline, chapter beat sheets, ledger, names, the guide | Yes; the beats, characters, outline and names need Nick's yes |
| `story/design/`, `design/` | Decisions about the game: tone, the lead, audio, death, voices | Propose only: Nick decides |

## Get the kit

The release [`kit`](../../releases/tag/kit) has the programs:

- `story-preview-windows.zip`: the preview. It plays your scenes in the
  game's text box and lists their problems. Once you have your copy on
  your computer (the guide's section 2), extract it outside that folder
  and drag `dialogue/` onto `story-preview.exe`. Windows only for now;
  on a Mac, the guide's section 1 says what to do.
- `story-check-windows.exe`, `story-check-linux`: the checker alone, for
  a terminal (optional).

## Propose a change

1. Make a branch, edit the files, commit, and open a pull request here
   (the guide shows each button, in GitHub Desktop or on github.com).
2. A check runs on the scripts and marks each problem on its line.
3. Nick reads it and approves it, or asks for changes.
4. Once approved, it goes into the game, and the pull request closes by
   itself with a note. Nobody merges here, and nobody pushes to `main`.

## Ask

Open an issue: for a new character or name, a scene played at a new
moment, a rule in your way, or anything the guide doesn't answer.
