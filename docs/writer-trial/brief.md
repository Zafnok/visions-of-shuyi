# The trial: rewrite Chapter 1's scenes

Thank you for trying out. This page says what to do, how to send it back
and what I'll be looking at.

## What the game is

A tactics RPG in the manner of Fire Emblem: a small party, turn-based
battles on a grid, and a story told in text boxes with portraits between
the battles. Act 1's first five chapters are playable. The demo you have the
link to is the title screen, the opening and Chapter 1.

## Why I'm looking for a writer

The battles, the story's arcs and the beats are there. The dialogue isn't:
every script so far is an AI draft, and it reads robotic. I want someone
who can make these people sound like people talking.

## The voices are made with AI

The characters' spoken lines are made with AI voice synthesis; you'll
hear them in the demo. If I hire you, every line you write for the game
will be voiced that way: its text is given to a voice model, which makes
the clip. Your trial entry won't be, unless I hire you and we agree to
use it.

If that's a problem for you, for any reason, please tell me right away,
before you spend time on the trial. It's a fair concern, and I'd rather
know now.

## What to do

Rewrite Chapter 1's scenes in `dialogue/ch01.dlg`. These are the five
Chapter 1 scenes, in the order they play:

- `ch01_intro`: the opening captions, then the yard at dawn;
- `ch01_prebattle`: the letter and the Red Company; the party gathers;
- `ch01_first_turn`: a line at the start of the battle;
- `ch01_victory`: after the fight;
- `ch01_tbc`: the scene that ends the chapter.

What's in the file now is the **AI draft**: rewrite it, or throw it away
and start from the beats. Keep each scene's `@scene` id and its `@end`, so
the game can still find it. Inside a scene, everything is yours: the
lines, who speaks, how much, the order. Keep the lead's reply choices:
one in `ch01_intro`, one in `ch01_prebattle`, and one in each version of
`ch01_victory` (with Rue and without her). Their wording is yours too.

`ch01_victory` has `@if` blocks: a companion who fell in the battle is
gone, so the scene has to work without them, or with only the lead left.
The cheat sheet's *Conditions* section shows how, and the preview's
Switches tab plays the scene with someone missing. However the battle
went, the scene has to get three things across:

- Harl was carrying a paper sealed with the Veyne family's seal;
- only whoever holds the family's hall can seal with it;
- so the party's road now leads to Veyne Hall.

The story pages in `story/` are there to help, not to bind you. If a beat
or a character's voice card gets in the way of a good scene, change it and
say why in your note. I'd rather read that than a scene forced to fit.

Plan on about **five or six hours**. If you run long, send what you have:
two scenes that sing beat five that are merely fine.

## How to see it

The demo and the preview run in your browser: nothing to install. Use a
computer, not a phone or tablet: they are played with the **keyboard**
only. Click the game first, then press any key: the first key asks you to
pick a layout (right- or left-handed), and after that the line at the
bottom of the screen names the keys. The **Fullscreen** button under the
game makes the text bigger.

In the demo, a battle in progress is kept only if you **Suspend** it
(from the map menu) before you leave. Reloading the page, closing
the tab, or the browser putting a background tab to sleep loses an
unsuspended battle. Use a normal browser window, not a private or incognito
one: a private window may not keep your settings or a suspended battle.

1. Open the script preview's page (the link and password are in
   `START-HERE.txt`) and drag `dialogue/ch01.dlg` onto the game itself.
   It plays your scenes in the game's own text box, with the faces and
   the music.
2. Edit `dialogue/ch01.dlg` in a plain-text editor, save, and drag it
   onto the game again. **Keep the name `ch01.dlg`**: a file with another
   name (`ch01 v2.dlg`, or `ch01 (1).dlg` from a browser download) plays
   beside the first one instead of replacing it, and one that ends in
   `.txt` is skipped without a message. Save over the same file each time,
   and rename a copy back to `ch01.dlg` before you drop it. To start
   clean, reload the page and drop the file again. The **Problems** tab
   lists anything the game couldn't read, with its line. If your change
   doesn't show, look there: a file with a mistake keeps playing its last
   good version.

### Which editor

**VS Code is the one I recommend.** It's free, for Windows, Mac and Linux
([code.visualstudio.com](https://code.visualstudio.com/)). With the
packet's `vscode-dlg.vsix` it colors the script (who speaks, the
narration and reply marks, stage lines, names) and keeps a reply's reaction lines indented for you: press
Enter after a reply line such as `* earnest: We go now.` and the next
line starts two spaces in; press Enter on one of those indented lines and
the next one stays two spaces in. That indenting is the easiest thing to
get wrong by hand. Each time you save, it also tidies the file (below).

If you'd rather not install it, a plain-text editor works too, with the
preview's **Problems** tab to catch mistakes:

- **Windows:** Notepad. Right-click the file, **Open with**, **Notepad**.
- **Mac:** TextEdit, set up once so it doesn't change your quotes: in
  TextEdit's **Settings**, choose **Plain text**, and under **Edit >
  Substitutions** turn off **Smart Quotes** and **Smart Dashes**. When you
  save, keep the name `ch01.dlg`; if TextEdit offers to add `.txt`, say no.

Not Word or Google Docs: they change quotes and dashes and save a
different kind of file.

### Adding the colors to VS Code

The packet's `vscode-dlg.vsix` is the extension, in one file. To add it:

1. Open VS Code.
2. Open the Extensions panel: **Ctrl+Shift+X** on Windows,
   **Cmd+Shift+X** on a Mac (or click the four-squares icon on the left
   edge).
3. At the top of that panel, click **…** (More Actions) and choose
   **Install from VSIX…**.
4. Pick `vscode-dlg.vsix` from the packet folder. If VS Code asks whether
   to trust or install it, say yes.
5. Open `dialogue/ch01.dlg` (**File > Open File…**).

**How to tell it worked:** the script is in colors, not all one color,
and the bottom-right corner of the window reads **Dialogue script**.

**Tidying when you save.** Each time you save, it fixes what the game
would refuse and changes nothing else, so your words stay as you wrote
them: curly quotes and apostrophes become straight ones, `…` becomes
`...`, long dashes become `--`, a reply's reaction lines get their two
spaces, and `@` lines move to the start of the line (two spaces in inside
a reaction). To tidy without saving: right-click in the script and choose
**Format Document**. It reads only the file you have open, and sends
nothing anywhere.

To remove it: in the Extensions panel, find **Visions of Shuyi scripts
(.dlg)**, click it and choose **Uninstall**.

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

Email them to the address in `START-HERE.txt`, with the file attached.
That address is for everything official: your entry, and later any
agreement. Questions while you work can go to me on Discord, in the
conversation where you applied. The due date is in `START-HERE.txt`. Tell
me if you need longer: that's fine, just say so before then.

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
writer I pick gets a share of the game's net revenue. The exact terms are
in the compensation PDF I sent with my first email, and they'll be signed
as part of a written agreement before any work beyond this trial.
`terms.md` says what happens to your entry.
