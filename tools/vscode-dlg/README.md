# Colors for .dlg scripts in VS Code

Optional. If you write in [VS Code](https://code.visualstudio.com/), this
folder colors the game's `.dlg` scripts: comments, `@` lines, narration,
replies, speakers and their expressions, and name tokens such as
`{n:place.harrowby}`. It also helps with indenting: press Enter after a
reply line (`* earnest: ...`) and the next line starts two spaces in,
ready for that reply's reaction; press Enter on a reaction line and the
next one stays two spaces in. Tab puts in two spaces.

It only colors and indents. It doesn't check your script: the preview's
**Problems** tab does that. Any plain-text editor still works.

## Install

1. Close VS Code.
2. Copy this whole `vscode-dlg` folder into VS Code's extensions folder:
   - Windows: `%USERPROFILE%\.vscode\extensions\` (paste that into
     File Explorer's address bar)
   - Mac or Linux: `~/.vscode/extensions/` (in Finder: Go > Go to Folder)
3. Open VS Code and open a `.dlg` file. The language in the bottom-right
   corner reads "Dialogue script".

Nothing is downloaded and nothing runs: the folder is a few text files.

## Remove

Close VS Code and delete the `vscode-dlg` folder from the extensions
folder.

The colors follow Shuyi Engine's own syntax file for `.dlg`
(`site/syntaxes/dlg.sublime-syntax`, MIT OR Apache-2.0).
