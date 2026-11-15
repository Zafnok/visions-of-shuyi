# The trial: rewrite Chapter 1's scenes

Thank you for trying out. This page says what to do, how to send it back
and what I'll be looking at.

## What the game is

A tactics RPG in the manner of Fire Emblem: a small party, turn-based
battles on a grid, and a story told in text boxes with portraits between
the battles. Act 1 (eight chapters) is playable. The demo you have the
link to is the title screen, the opening and Chapter 1.

## Why I'm looking for a writer

The battles, the story's arcs and the beats are there. The dialogue isn't:
every script so far is an AI draft, and it reads robotic. I want someone
who can make these people sound like people talking.

## What to do

Rewrite Chapter 1's scenes in `dialogue/ch01.dlg`. These are the five
scenes the demo plays outside the battle's map, in the order they play:

- `ch01_intro`: the opening captions, then the letter;
- `ch01_prebattle`: the Red Company; the party gathers;
- `ch01_first_turn`: a line at the start of the battle;
- `ch01_victory`: after the fight;
- `ch01_tbc`: the scene that ends the chapter.

What's in the file now is the **AI draft**: rewrite it, or throw it away
and start from the beats. Keep each scene's `@scene` id and its `@end`, so
the game can still find it. Inside a scene, everything is yours: the
lines, who speaks, how much, the order. Keep the lead's reply choices (one
each in `ch01_intro`, `ch01_prebattle` and `ch01_victory`), but their
wording is yours too.

`ch01_victory` has `@if` blocks: a companion who fell in the battle is
gone, so the scene has to work without them. The cheat sheet's
*Conditions* section shows how, and the preview's Switches tab plays the
scene with someone missing.

The story pages in `story/` are there to help, not to bind you. If a beat
or a character's voice card gets in the way of a good scene, change it and
say why in your note. I'd rather read that than a scene forced to fit.

Plan on about **five or six hours**. If you run long, send what you have:
two scenes that sing beat five that are merely fine.

## How to see it

Nothing to install: everything runs in your browser.

1. Unzip the packet anywhere and open its folder.
2. Open the script preview's page (the link and password are in
   `START-HERE.txt`) and drag `dialogue/ch01.dlg` onto it. It plays your
   scenes in the game's own text box, with the faces and the music.
3. Edit `dialogue/ch01.dlg` in any plain-text editor (Notepad or
   TextEdit in plain-text mode is fine; not Word), save, and drag it onto
   the page again. The **Problems** tab lists anything the game couldn't
   read, with its line.

**Optional, for VS Code users:** the packet's `vscode-dlg` folder colors
the script and indents a reply's reaction for you. Close VS Code, copy the
whole folder into VS Code's extensions folder
(`%USERPROFILE%\.vscode\extensions\` on Windows, `~/.vscode/extensions/`
on a Mac or Linux) and open VS Code again. To remove it, delete the
folder. Nothing to run: it's a few text files, and `vscode-dlg/README.md`
says more. Any plain-text editor still works.

**Your file is the only copy: the page keeps nothing.** The preview only
shows the file you drop on it. You can't edit there, and it saves nothing.
Reloading it, closing the tab, or an update to the page (it can be
updated any night, without warning) clears what you dropped. Write in
the file on your own computer, save often, and keep a backup copy
somewhere else, such as a cloud drive or a second folder. If the page is
reset, drop the file again.

`cheat-sheet.md` is the whole script format on a few pages. The demo
link in `START-HERE.txt` lets you play Chapter 1 as a player sees it.

## What to send back

- your `ch01.dlg`;
- a short note, as long as you like: what you changed and why, anything
  you'd do with more time, anything in the beats you'd change.

Reply to the message that brought you this packet, with the file
attached. The due date is in `START-HERE.txt`. Tell me if you need
longer: that's fine, just say so before then.

## What I'm looking at

- Does it sound like people talking, not robotic?
- Does each character sound like themselves, and different from the
  others?
- Is there warmth, without a joke every few lines?
- Does the lead stay someone the player can speak through (the lead's
  section of `story/4-characters.md`)?
- Does it read well in the text box (the preview shows how long a box
  can be)?

A few problems in the Problems tab won't count against you: I can fix a
format slip. The writing is what I'm judging.

## About the job

The game is a hobby project with a revenue share. The trial is unpaid,
and no pay is guaranteed before the game launches. After launch, the
writer I pick gets a share of the game's net revenue after Steam's cut,
probably tied to how it sells (for example 5 to 10% depending on the
game's sales). The exact terms will be in a written agreement before any
work beyond this trial. `terms.md` says what happens to your entry.
