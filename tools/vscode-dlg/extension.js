// The .dlg formatter (Format Document, and on save): it fixes only what
// the game would refuse, and leaves a script the game reads as it is.
//
// - Curly quotes, the ellipsis and dashes become the plain ' " ... -- -
//   the game wants; a no-break space or a tab inside a line becomes a space.
// - A tab at the start of a line becomes two spaces.
// - Spaces at the end of a line, and empty lines at the end of the file,
//   go; the file ends with one line break.
// - Lines starting with @ go to the first column, except inside a reply's
//   reaction, where they go two spaces in.
// - Inside a @choice, a reaction line one space in, or at the first column
//   under a reply, goes two spaces in; a reply line ("* tone: text") or
//   @endchoice that was indented goes to the first column.
//
// Notes (lines starting with #) keep their text. It reads only the file
// it formats, writes nothing and connects to nothing. The rules follow the
// game's reader (Shuyi Engine's crates/assets/src/dialogue/parse.rs).

"use strict";

const PLAIN = [
  [/[‘’]/g, "'"],
  [/[“”]/g, '"'],
  [/…/g, "..."],
  [/—/g, "--"],
  [/–/g, "-"],
  [/[ \t]/g, " "],
];

/** The first word of `body` (a directive's name, such as `@endchoice`). */
function word(body) {
  return body.split(" ")[0];
}

/** Whether `lines[from..]` reaches `@endchoice` before the scene ends. */
function closes(lines, from) {
  for (let i = from; i < lines.length; i++) {
    const name = word(lines[i].trim());
    if (name === "@endchoice") return true;
    if (name === "@end" || name === "@scene" || name === "@choice") return false;
  }
  return false;
}

/** `text`, a whole .dlg file, formatted. */
function format(text) {
  const eol = text.includes("\r\n") ? "\r\n" : "\n";
  const lines = text.split(/\r?\n/).map((line) => {
    const tabs = line.match(/^[ \t]*/)[0].replace(/\t/g, "  ");
    return (tabs + line.replace(/^[ \t]*/, "")).replace(/\s+$/, "");
  });
  const out = [];
  let inChoice = false;
  let underReply = false;
  for (let i = 0; i < lines.length; i++) {
    let line = lines[i];
    let body = line.replace(/^ +/, "");
    let indent = line.length - body.length;
    if (body === "" || body.startsWith("#")) {
      out.push(line);
      continue;
    }
    for (const [from, to] of PLAIN) body = body.replace(from, to);
    const name = word(body);
    if (inChoice) {
      if (body.startsWith("*") && indent <= 2) {
        indent = 0;
      } else if (name === "@endchoice") {
        indent = 0;
      } else if (indent === 1) {
        indent = 2;
      } else if (indent === 0 && underReply) {
        const ends = ["@endchoice", "@choice", "@end", "@scene"].includes(name);
        if (!ends) indent = 2;
      }
    } else if (body.startsWith("@")) {
      indent = 0;
    }
    if (indent === 0) {
      if (inChoice && ["@endchoice", "@end", "@scene"].includes(name)) {
        inChoice = false;
        underReply = false;
      } else if (!inChoice && name === "@choice" && closes(lines, i + 1)) {
        inChoice = true;
        underReply = false;
      } else if (inChoice && body.startsWith("*")) {
        underReply = true;
      }
    }
    out.push(" ".repeat(indent) + body);
  }
  while (out.length > 0 && out[out.length - 1] === "") out.pop();
  return out.join(eol) + eol;
}

/** VS Code starts it when a .dlg file opens: Format Document formats it. */
function activate(context) {
  const vscode = require("vscode");
  const provider = {
    provideDocumentFormattingEdits(document) {
      const text = document.getText();
      const formatted = format(text);
      if (formatted === text) return [];
      const all = new vscode.Range(
        document.positionAt(0),
        document.positionAt(text.length),
      );
      return [vscode.TextEdit.replace(all, formatted)];
    },
  };
  context.subscriptions.push(
    vscode.languages.registerDocumentFormattingEditProvider("dlg", provider),
  );
}

function deactivate() {}

module.exports = { activate, deactivate, format };
