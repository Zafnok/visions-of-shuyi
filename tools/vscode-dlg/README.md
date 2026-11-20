# Colors for .dlg scripts in VS Code

Optional. Skip this unless you write in
[VS Code](https://code.visualstudio.com/): Notepad, TextEdit or any
plain-text editor works just as well, with the preview's **Problems** tab
to catch mistakes.

This folder makes VS Code color the game's `.dlg` scripts: comments, `@`
lines, narration, replies, who speaks and their expression, and names
such as `{n:place.harrowby}`. It also keeps a reply's reaction lines
indented for you: press Enter after a reply line such as
`* earnest: We go now.` and the next line starts two spaces in; press
Enter on one of those indented lines and the next one stays two spaces
in. Tab puts in two spaces.

When you save (or choose **Format Document**), it tidies the file: it
fixes only what the game would refuse (curly quotes, `…` and long dashes
become `'` `"` `...` `--`; a reaction's lines get their two spaces; `@`
lines move to the start of the line) and leaves your words as they are.
It doesn't check your script: the preview's
**Problems** tab does that. Nothing to run: the folder is four text
files.

## Add it

If you have `vscode-dlg.vsix` (a writer trial's packet has it): in VS
Code open the Extensions panel (Ctrl+Shift+X, Cmd+Shift+X on a Mac),
click **…** at its top, choose **Install from VSIX…** and pick the file.
Otherwise, copy this folder in by hand:

1. If you have never opened VS Code, open it once (that makes its
   extensions folder). Then close VS Code completely.
2. Open the extensions folder:
   - **Windows:** open File Explorer, click its address bar (the bar at
     the top showing the folder's path), paste
     `%USERPROFILE%\.vscode\extensions` and press Enter.
   - **Mac:** open Finder, choose **Go > Go to Folder…** in the menu bar,
     paste `~/.vscode/extensions` and press Return.
   - **Linux:** open your file manager, press Ctrl+L, type
     `~/.vscode/extensions` and press Enter.
3. Copy this whole `vscode-dlg` folder into it (drag it in, or copy and
   paste it). The result is a folder `vscode-dlg` inside `extensions`,
   with `package.json` directly in it.
4. Open VS Code again and open a `.dlg` script.

**How to tell it worked:** the script is in colors, not all one color,
and the bottom-right corner of the window reads **Dialogue script**. If
it doesn't, close VS Code and check step 3: `vscode-dlg` must sit
directly in `extensions`, not inside another folder.

## Remove it

Close VS Code and delete the `vscode-dlg` folder from the extensions
folder.

The colors follow Shuyi Engine's own syntax file for `.dlg`
(`site/syntaxes/dlg.sublime-syntax`, MIT OR Apache-2.0).
