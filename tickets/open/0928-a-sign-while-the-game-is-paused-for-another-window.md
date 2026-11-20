---
id: "0928"
title: "A sign on the screen while the game is paused for another window"
type: feature
milestone: M8 Release
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0928 — A sign on the screen while the game is paused for another window

## Context

Nick decided in 1246 (`docs/design/options.md`, *Another window in front*) that when the game's window loses the focus, the game pauses and goes silent:

> the game **pauses**: the picture stays on its last frame, nothing moves, and keys and buttons do nothing. [...] It is **silent** meanwhile

Nothing on the screen says so. At the engine pin, `crates/platform/src/app.rs` (the `Phase::Running(game) if paused` branch: "Paused without focus: no update, its last frame again") presents the last frame again and draws nothing over it.

On the web this is easy to trigger by accident. The engine's `crates/cli/web/mq_js_bundle.js` counts the game as focused only while `document.hasFocus()` is true and the page is visible, and it reads keys on the canvas. So any click outside the game's frame on its itch.io page pauses it: the page text, or **itch's own Fullscreen button under the game**, which Nick turned on for the Chapter 1 demo (0915's notes). The game then sits frozen and silent until the player clicks it. The writer trial's test audit (0766) found that a tester reads this as the game having hung. For now 0766's START-HERE tells writers to click the game again, so this ticket does not block the writer trial. This ticket makes the game say it.

The pause itself (pause and mute, the same for every player, no Options row) stays as Nick decided. This ticket only adds what the player sees while it lasts.

What exists to build on:
- The engine's `SystemHook` (`shuyi_platform`, `app.rs`) is documented as "on `FocusLost` [...] it may do more (open a pause menu)". The game's hook is `crate::save_files::hear`, set in `crates/ui/src/harness.rs:188`.
- While paused, the engine doesn't update or draw the game. So whatever the hook pushes on `FocusLost` isn't on screen until focus returns. Getting the sign into the held frame may need the engine to paint one more frame, or let the game hand it an overlay. That is a technical choice, and if it needs the engine it follows the `engine-change` skill.
- Engine ticket engine#3043 (the web lifecycle) is related but not needed here.

## Nick input

**Decision**, asked before building with the `ask-nick` skill. The question: what the player sees while the game is paused because another window, or the page around the game, has the focus. Show `cargo shuyi frame-png` mockups of each option on the title screen and on a battle map, in both looks (glyphs and pictures). Options:

1. **Nothing (today):** the picture freezes on its last frame and the sound stops, with no sign.
2. **A small "Paused" word** over the frozen picture (a corner or the top edge), the picture otherwise unchanged.
3. **A dimmed picture with a short card** in the middle: "Paused", and on the web a second line saying to click the game to go on.
4. **A pause menu that opens by itself**, as Minecraft (Java Edition) opens its game menu when its window loses focus. The player closes it to go on. The game has no general pause menu today, so this one is bigger: say so when asking.
5. **Describe your own.**

If he picks 1, the ticket closes with his answer recorded and nothing built. Whatever he picks is verified by the tests below and `frame-png` pictures in the PR, never a check after merge (`tickets/README.md`). Anything left to judge by playing goes on the Act 1 playtest's checklist (1064).

## Scope

**In:**
- The mockups and Nick's answer, recorded with his words in `docs/design/options.md`, *Another window in front*.
- The sign he picks, on every screen of the desktop and web builds while the game is paused for lack of focus. Its text goes by key in `assets/lang/en/ui.ron` and the `test` language pack (ADR-0045). It is a view plus a skin (ADR-0054), painted by every skin that runs where focus can be lost.
- The engine change, if one is needed and Nick says yes to it, through the `engine-change` skill (an engine ticket and PR first, then this PR moves the pin).

**Out (do not do):**
- Changing the pause and mute policy itself (1246, Nick's).
- Saving a battle when a tab or window closes (0092, the decision filed beside this ticket, and engine#3043).
- The Playdate, which has its own pause when its system menu opens (`options.md`).
- 0766's packet text.

## Implementation steps

1. Read `docs/design/options.md` (*Another window in front*), the engine's ADR-0077 at the pin, the paused branch of `App::frame` in `crates/platform/src/app.rs`, and `crates/ui/tests/it/focus.rs`.
2. Stage the mockups for each option with `frame-png`: the title and a battle map, in both looks. Use a throwaway `zz_` battle file or an env-var mock in the skin, then revert (no real map in committed pictures, ADR-0118). The mockups need no engine change, so asking never waits on the engine. Ask Nick, and expect more than one round. If the sign he picks may need the engine (step 4), say so when asking: an engine ticket or PR is opened only after he says yes to it.
3. Record the answer in `options.md`. If he picks option 1, stop here.
4. Find how the held frame can carry the sign. First try the game side (the hook on `FocusLost`, or the frame painted just before the pause). If the engine has no way to do it, ask Nick before opening an engine ticket or PR. With his yes, write the engine ticket per `engine-change`: for example, `App` paints one frame with the game's background overlay before it holds. The engine must not name the game's concept. The game never waits on the engine (AGENTS.md rule 9): if the engine change can't land in this ticket's session, ship the best game-side sign with a marked workaround, as `engine-change` describes. If Nick declines the engine path, build what the game side allows and record his answer and what was left out in the Completion notes.
5. Build the sign as a view (plain data: shown or not, its lines by key) and paint it in each skin. On the web build only, add the line about clicking the game. It names the mouse, not a key, but still follow the `keyboard-input` skill for any text that names input.
6. Expected size: about 100–150 production lines game-side (one small view, its paint in the glyph and picture skins, two or three text keys), judged against the demo's end card in `crates/ui/src/flow/demo.rs`. Measure with `cargo xtask code-lines` (ADR-0092).

## Acceptance criteria

- [ ] `docs/design/options.md` records Nick's choice with his words.
- [ ] If a sign was chosen: a Harness test in `crates/ui/tests/it/focus.rs` sends `FocusLost` on the title and on a battle, and the view (or the frame's text snapshot) shows the sign. Keys still do nothing and the three volumes read 0, as the existing tests check. After `FocusGained`, the sign is gone and the screen is what it was before.
- [ ] If a web-only line was chosen: a test shows it present with the web platform's facts and absent on the desktop's.
- [ ] The PR has `cargo shuyi frame-png` pictures of the sign on the title and on a battle map, in both looks.
- [ ] Any engine change Nick agreed to is merged in Zafnok/shuyi-engine first, and this PR moves the pin to it. Otherwise the Completion notes say why there is none (not needed, declined, or a marked workaround with its engine ticket).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: Harness tests of focus lost and gained with the sign (`crates/ui/tests/it/focus.rs`).
- Snapshot: text snapshots of the paused title and battle with the sign.
- Unit: the sign's view (its lines on the web and on the desktop).
- Engine (if changed): an `App` test that the held frame carries what the game handed it.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations, follow-up tickets created, notes for Nick.)*
