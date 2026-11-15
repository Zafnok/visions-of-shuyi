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
| `editor/` | Colors for the scripts in VS Code (optional, below) | Suggest it in an issue |

## Get the kit

The release [`kit`](../../releases/tag/kit) has the programs:

- `story-preview-windows.zip`: the preview. It plays your scenes in the
  game's text box and lists their problems. Once you have your copy on
  your computer (the guide's section 2), extract it outside that folder
  and drag `dialogue/` onto `story-preview.exe`. Windows only; on a Mac
  or Linux, use the web preview below.
- The web preview: the same preview in your browser, at the link and
  password you were sent. It starts with no scenes: drop your `.dlg`
  files on the page, and drop a file again after you save it. The page
  keeps nothing: a reload, or an update to the page (it can change any
  night), clears it. Your files on your computer are the only copy:
  save often and send your changes (the guide's Part 1).
- `story-check-windows.exe`, `story-check-linux`: the checker alone, for
  a terminal (optional).

**Optional, for VS Code users:** the folder `editor/vscode-dlg/` here
colors the scripts and indents a reply's reaction for you. Close VS Code,
copy the whole `vscode-dlg` folder into VS Code's extensions folder
(`%USERPROFILE%\.vscode\extensions\` on Windows, `~/.vscode/extensions/`
on a Mac or Linux) and open VS Code again. To remove it, delete the
folder. Nothing to run: it's a few text files, and its `README.md` says
more. Any plain-text editor still works.

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
