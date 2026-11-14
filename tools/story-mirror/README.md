# Visions of Shuyi: the story

The scripts and story documents of *Visions of Shuyi*, copied here for
the game's writer. This repository is a mirror: the game's own copy is
the real one, and every change to it lands here by itself.

| Folder | What | Can you change it? |
| ------ | ---- | ------------------ |
| `dialogue/` | The scripts, one `.dlg` file per chapter or side quest, and `README.md`, the format | The scripts, yes. The format: suggest it in your pull request |
| `story/` | Beats, bible, characters, outline, chapter beat sheets, ledger, names, the writer's guide | Yes, but `story/design/` |
| `story/design/`, `design/` | Decisions about the game: tone, audio, death, voices | No: suggest it in your pull request, and Nick decides |

## Proposing a change

1. Make a branch, edit the files, and commit.
2. Open a pull request here. A check runs `story-check` on the scripts
   and marks each problem on its line.
3. Nick reads it and approves it, or asks for changes.
4. Once approved, the change goes into the game, and this pull request
   closes by itself with a note. Don't merge it yourself: nobody merges
   here.

## Checking a script on your computer

The release [`kit`](../../releases/tag/kit) has `story-check` for Windows
and Linux. Put it in this folder and run it there:

```
story-check                    # every script in dialogue/
story-check dialogue/ch01.dlg  # only that file's problems
```

It prints each problem as `file:line: message`, then how many.

*A fuller guide is coming (ticket 0765).*
