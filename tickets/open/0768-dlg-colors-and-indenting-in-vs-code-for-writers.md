---
id: "0768"
title: ".dlg colors and indenting in VS Code for writers"
type: feature
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: in-progress
blocked_by: []
nick_input: none
completed:
---

# 0768 — .dlg colors and indenting in VS Code for writers

## Context

Nick, 2026-11-15, during 0766 (the writer trial): "if we have .dlg
highlighting and indenting settings for vscode I might suggest allowing
writers use them too". There are none. The nearest thing is Shuyi
Engine's `site/syntaxes/dlg.sublime-syntax` (the engine's website colors
`.dlg` code blocks with it; the engine is MIT OR Apache-2.0, and our own).

Asked where it should live, Nick picked **a folder in the packet**: a small
VS Code extension, as plain files, that a writer copies into VS Code's
extensions folder by hand. No `.vsix`, no Marketplace, no program to run
(0766 dropped the preview `.exe` from the packet for that reason), and
nothing published.

The format is `assets/dialogue/README.md`: `# comment`, `@directive`,
`> narration`, `speaker[expression] (direction): text`, a reply option
`* tone: text` inside `@choice` … `@endchoice` whose reaction lines are
indented by exactly two spaces, `@if` … `@else` … `@endif` blocks (`:350`:
written as far in as their `@if`, never indented in a scene), an indented
line continuing the line above (`:75`), and name tokens `{n:<id>}`,
`{N:<id>}`, `{lead}` inside text.

## Nick input

None. Verified by the tests below and a picture of VS Code with the
trial's script open, in the PR.

## Scope

**In:**
- `tools/vscode-dlg/`: `package.json` (a language `dlg` for `*.dlg`, its
  grammar and its language configuration; `engines.vscode`, a publisher
  of `visions-of-shuyi`, version `0.1.0`, no code, no `main`),
  `syntaxes/dlg.tmLanguage.json` (the engine's sublime syntax translated
  rule for rule, plus a scope for name tokens `{n:…}`, `{N:…}`, `{lead}`
  inside text), `language-configuration.json` (`#` as the line comment;
  `onEnterRules`: Enter after a `* tone:` option line or after a line
  indented two spaces inside a choice starts the next line at two spaces;
  no extra indent after `@if` or `@choice`, as the format says),
  `README.md` (plain words: what it does, how to install by copying the
  folder, how to remove it). A header comment or README line credits the
  engine's syntax file as the source.
- **The trial packet** (0766's `docs/writer-trial/packet.txt`): the four
  files under `vscode-dlg/`, one `packet.txt` line each (the packet copies
  a file whole when a line names no sections). `brief.md`'s *How to see
  it* gains an optional paragraph: VS Code users copy the `vscode-dlg`
  folder into their extensions folder (`%USERPROFILE%\.vscode\extensions\`
  on Windows, `~/.vscode/extensions/` on a Mac or Linux) and restart VS
  Code; any plain-text editor still works.
- **The writer's mirror**: `tools/story-mirror/files.txt` copies the
  folder as `editor/vscode-dlg/` (no `import`), and the mirror's
  `README.md` says the same paragraph.

**Out (do not do):**
- A `.vsix`, the Marketplace, any engine change (Nick chose the game-side
  folder; a public extension for every Shuyi game would be an engine
  ticket, asked first).
- Errors in the editor (a language server, diagnostics): the preview's
  Problems tab is the checker.
- Other editors (Sublime Text, Notepad++).
- Writing or rewording any script line (the writing pause, 0766).

## Implementation steps

1. Expected size: about 200 changed lines, no Rust (three JSON files of
   about 40–80 lines each, a README, the packet and mirror lines, the
   test). Comparable: the engine's sublime syntax (about 60 lines).
2. **If 0766's PR (#696) hasn't merged**, base the branch on
   `t0766-find-a-writer` and say in the PR that it is stacked (`packet.txt`
   and `brief.md` are there). Otherwise base it on `main`.
3. Write the three JSON files and the README in `tools/vscode-dlg/`.
4. Add the `packet.txt` lines, the `brief.md` paragraph, the `files.txt`
   line and the mirror README paragraph.
5. A test in `crates/xtask/src/trial.rs` (or a new module beside it):
   every JSON file in `tools/vscode-dlg/` parses (`serde_json`);
   `package.json` names the grammar and configuration paths that exist;
   every `packet.txt` line under `vscode-dlg/` names a file that exists,
   and every file in the folder has one.
6. Install it by hand (copy the folder into the extensions folder),
   open a trial script in VS Code, and take a screenshot for the PR: the
   comment, a directive, narration, a speaker with an expression and a
   direction, a choice with its indented reaction, a name token. Press
   Enter after a `* tone:` line and check the new line starts at two
   spaces. Remove the copy afterwards.
7. `cargo xtask trial-packet --for test --due x` (with a throwaway
   `dist/trial/links.txt`) and check the zip holds `vscode-dlg/`.

## Acceptance criteria

- [ ] `tools/vscode-dlg/` holds `package.json`,
      `language-configuration.json`, `syntaxes/dlg.tmLanguage.json` and
      `README.md`; the JSON parses and the paths agree (test).
- [ ] A trial packet holds `vscode-dlg/` with the four files (test over
      `packet.txt`, and the zip from step 7).
- [ ] The PR has a screenshot of VS Code coloring a `.dlg` script with the
      folder installed, and says Enter after an option indents two spaces.
- [ ] `brief.md` and the mirror's README say how to install it, in plain
      words, and that it's optional.
- [ ] No `.vsix`, no program, nothing published.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the folder's JSON parses and its paths agree; every file is in
  `packet.txt`.
- By hand: the screenshot (step 6).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
