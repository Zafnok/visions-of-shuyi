---
id: "0930"
title: "The desktop demo tells the player where its exported save went"
type: feature
milestone: M8 Release
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0930 — The desktop demo tells the player where its exported save went

## Context

The Chapter 1 demo (0914, ADR-0119, `docs/design/demo.md`) ends with "Thanks for playing, {lead}", then "Export save?" Yes / No. *Yes* hands the player the campaign as a file the full game imports (0873) to go on to Chapter 2. The demo keeps no save of its own, so that file is the only copy.

0914 left a deviation: "A desktop demo's *Yes* writes the file to the exports folder without a line saying so (the title doesn't hear the platform's answer). The web, the demo's target, downloads it." In the code:

- `crates/ui/src/flow/demo.rs`, `FlowScreen::demo_export`: "Nothing is shown after it, so the flow ends."
- `crates/ui/src/save_files.rs`, `hear`: a `SystemEvent::FileOffered` goes only to `SlotPickerScreen` or `ProfileScreen`, so the demo's answer is dropped.
- A failed write (`Exported::Failed`) is silent too: the player may think they have a file they don't.

Load Game's export already tells the player, with the game's own words and keys (`assets/lang/en/ui.ron`):

- `save_files.written`: "Exported {file} to the exports folder"
- `save_files.failed`: "The file couldn't be exported."
- `hint.open_folder`: "{WorldFind} open folder" (`Exported::folder` picks the exports folder)

This matters for the Windows demo (`ITCH_DEMO_DESKTOP_PROJECT`, `docs/releasing.md`, *The demo's itch.io pages*, ticket 0915). The writer trial (0766) uses the web demo only (Nick, 2026-11-14), so this doesn't block the trial. It should land before a Windows demo key is handed out. Found by the demo audit of 2026-11-19.

## Nick input

**Decision**, asked before building with the `ask-nick` skill. Show each option as a `cargo shuyi frame-png` mockup in both looks (glyphs and pictures). No real map: reuse the title. Then record the answer in `docs/design/demo.md` with his words.

*In the Windows demo, after you pick* Yes *on "Export save?", how do you learn where the file went, or that it couldn't be written?*

1. **A line under the title's menu.** You land on the title as now, and Load Game's line shows under its menu ("Exported {file} to the exports folder", or "The file couldn't be exported."). With it comes the open-folder key, until you move the cursor. Like Minecraft (Java Edition), which prints "Saved screenshot as {name}" in the chat where you already are, and you go on playing.
2. **The question waits.** "Export save?" stays up after *Yes*, with that line and the open-folder key under its panel. Confirm then goes to the title. That is how Load Game's own export works today: the line shows on the list you exported from.
3. **A notice that fades.** You land on the title, and a short notice in a corner shows the same words for a few seconds, then fades by itself (no key). Like RetroArch's on-screen "Saved state to slot #0" notices.
4. **Describe your own.**

Whichever he picks, the line shows on the desktop when the file was written or failed. On the web, a successful download shows nothing new, because the browser's download bar already names the file (part of the question, so he can say otherwise). Never a check after merge: the PR is verified by the tests and the pictures below.

## Scope

**In:**
- The `ask-nick` question above with mockups, and its answer in `docs/design/demo.md`.
- The demo's *Yes* keeps the file it offered (an `ExportNotice`, as `ProfileScreen` and `SlotPickerScreen` do). `save_files::hear` hands the platform's answer to the screen of Nick's option.
- That screen's view carries the notice. Its skin paints it in the look Nick picked, with Load Game's existing text keys. For options 1 and 2 the open-folder hint and action work as on Load Game.
- A Harness test and `frame-png` pictures in the PR.

**Out (do not do):**
- New words for the line. Reuse `save_files.written` and `save_files.failed`, unless Nick's own option (4) asks for others.
- Changing the full game's end of Chapter 1, Load Game's export or the import (0872, 0873).
- The web demo's download behavior beyond what Nick's answer says.
- The package, README and voice checks: 0929.
- Hosting or pushing the demo (0915's workflow).

## Implementation steps

1. Run the `ask-nick` skill with the mockups.
   - Render each option with a throwaway edit that starts `frame-png` on the demo's bundle, as 0914's pictures were made (`docs/img/0914-*.png`). The edit is reverted before the PR, and no committed picture shows a real map (ADR-0118).
   - Record the answer in `docs/design/demo.md`.
2. **Size first** (rule 9): about 100 production lines for options 1 and 2.
   - An `ExportNotice` on the chosen screen: 15.
   - Routing in `save_files::hear`: 10.
   - The flow handing the offered name to that screen: 20.
   - The view field and its skin line, judged by `ProfileScreen`'s notice: 40.
   - The open-folder action: 15.
   - Option 3 adds a timer to the view, about 40 more.
   - Measure with `cargo xtask code-lines`. At 3×, run `engine-friction`.
3. **Option 1:** give the title screen (`crates/ui/src/screens/title.rs`) an `ExportNotice` and a `heard` method.
   - `FlowScreen::demo_export` (`crates/ui/src/flow/demo.rs`) records the offered file name so the title made after it starts with `offered(file)`.
   - `save_files::hear` adds the title to the screens it hands `FileOffered` to.
   - The title clears the notice when the cursor moves.
   - The open-folder key opens `Exported::folder(...)` as Load Game's does.
4. **Option 2:** give `SavePromptScreen` (`crates/ui/src/screens/save.rs`, `export_demo`) the notice.
   - After *Yes* it stays up and shows the line and the hint.
   - Confirm then pops to the flow, which goes to the title (`Stage::DemoExport`).
   - `hear` routes to it.
5. **Option 3:** a notice in the title's view with a time it was shown, painted by the skin and faded by the time alone (`shuyi-animate`). The rules never hold the time.
6. Follow ADR-0054: the screen's logic builds the view, the skin paints it, and tests read the view. Keys only through the player's keymap (`keyboard-input` skill): the hint names `{WorldFind}`, never a key.
7. Pictures with `cargo shuyi frame-png` of the chosen screen after a written export and after a failed one, in both looks. Crop them to the panel and put them in the PR.

## Acceptance criteria

- [ ] `docs/design/demo.md` records Nick's answer with his words and the date.
- [ ] Harness test (in `crates/ui/tests/it/demo.rs`): playing the demo to "Export save?", *Yes*, then a platform answer of `Written` gives the screen of Nick's option a view holding `save_files.written`'s text with the offered file name.
- [ ] The same test with a `Failed` answer: the view holds `save_files.failed`'s text.
- [ ] On the web (`Downloaded`), the view shows what Nick's answer says (by default, no new line).
- [ ] For options 1 and 2, the view's help names the open-folder action, and taking it asks the platform to open the exports folder (the Harness records the request, as Load Game's tests do).
- [ ] 0914's existing demo tests still pass: *No* returns to the title with no file and no line; the full game still asks "Save your progress?" after Chapter 1.
- [ ] `frame-png` pictures of the written and failed cases, in both looks, are in the PR.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `ExportNotice` on the chosen screen (offered, heard, cleared), as `save_files/tests.rs` covers it.
- Integration: the Harness play of the demo's end with `Written`, `Failed` and `Downloaded` answers (`crates/ui/tests/it/demo.rs`).
- Pictures: `cargo shuyi frame-png` of the line in both looks, in the PR. A text snapshot of the look only if the screen already has snapshot tests.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations, follow-up tickets created, notes for Nick.)*
