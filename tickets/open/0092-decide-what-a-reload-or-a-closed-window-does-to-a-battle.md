---
id: "0092"
title: "Decide: what a reload or a closed window does to a battle that isn't suspended"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0092 — Decide: what a reload or a closed window does to a battle that isn't suspended

## Context

Nick picked Fire Emblem's saving rules (`docs/design/death-and-difficulty.md`, *Saving*): chapter saves, plus one `Suspend` in the map menu that saves the whole battle and is deleted when the player continues it, then "No other saving mid-battle."

The writer trial's test audit (0766) found what that means in the Chapter 1 web demo:
- **A battle that isn't suspended is lost to anything that ends the page:** F5, Ctrl+W, a browser crash, or a background tab the browser discards to save memory.
- **In the demo the loss is the whole run.** It has no Load Game and makes no chapter save (`crates/ui/src/screens/title.rs:324-332`; `docs/design/demo.md`, rule 2), so after a reload the player starts again from New Game, Chapter 1's opening scenes included.
- **Closing the window on Windows loses the battle the same way.** The game's system hook (`crates/ui/src/save_files.rs`, `hear`) saves nothing on `Suspend` or `QuitRequested`.

On the web the game doesn't even hear the page going. Engine ticket engine#3043 (open) says the engine's web build never sends `Suspend` or `QuitRequested`. That ticket adds a save inside the browser's own event when the tab is hidden or the page closes.

A suspend save made by `Suspend` already survives a reload (it is kept in the browser's storage, and the title then shows `Continue`). 0766's START-HERE tells writers to suspend before they stop, which covers the trial. This ticket asks whether the game should protect the player itself. That would change a rule Nick decided, so it is his call.

Not a blocker for 0766: START-HERE already tells writers to Suspend before a reload.

## Nick input

**Decision** (`ask-nick`). The question: when the page reloads or closes, or the game's window is closed, in the middle of a battle the player didn't suspend, what happens? Options, from real games:

1. **As now (Fire Emblem on the Game Boy Advance and 3DS):** only `Suspend` keeps a battle, and anything else loses it. In the demo, that means back to New Game.
2. **A warning first.** On the web this is the browser's own "Leave site? Changes you made may not be saved" dialog, which web apps like Google Docs show. On Windows it is a yes/no question before the window closes. Either one shows only while a battle is running and isn't suspended. It catches F5, Ctrl+W and the close button, but not a crash or a tab the browser discards. The browser writes the dialog's words, not the game.
3. **The game suspends by itself** when the page is hidden or closes, or the window closes, and the next start shows `Continue`. XCOM 2's Ironman mode keeps saving as the battle goes, and Into the Breach saves at the start of each turn, so quitting there loses little or nothing. The save is the battle exactly as it is when the page goes, and `Continue` still deletes it, so it can't be used to undo a turn. It also catches a discarded tab, but not a crash while the game is in front.
4. **Both 2 and 3.**
5. **Describe your own.**

Ask also whether the answer is the same for the demo and the full game. The full game has chapter saves and Load Game, so it loses only the current battle.

## Scope

**In:** the question, Nick's words recorded in `docs/design/death-and-difficulty.md` (*Saving*, which loses or qualifies "No other saving mid-battle") and, if it differs for the demo, in `docs/design/demo.md`. The build ticket(s) for his answer, in the block of the system they touch:
- For option 3, a game ticket that saves the suspend on the engine's `Suspend` and `QuitRequested` events in the game's system hook, `blocked_by` engine#3043 for the web.
- For option 2, a game ticket for the question on Windows, plus an engine ticket for the web's `beforeunload` warning through the `engine-change` skill, since the web page shell is the engine's. The session asks Nick before filing any engine ticket.

**Out (do not do):**
- Building any of it.
- The pause while the game has no focus (1246, and 0928, the pause sign).
- What the player is told when the browser refuses storage (0932). Either order: if storage is refused, option 3's automatic suspend saves nothing either, and 0932 covers telling the player. If 0932 is done, put its answer into the question.
- 0766's packet text.
- The Playdate, which has its own suspend.

## Implementation steps

1. Read `death-and-difficulty.md` (*Saving*), `demo.md`, `title.rs`'s `build`, `save_files.rs`'s `hear`, and engine#3043 in Zafnok/shuyi-engine.
2. Ask Nick with the `ask-nick` skill, in plain player situations ("you press F5 on turn 6 of the bandit fight: what happens?"), with each option's consequence spelled out.
3. Record his words and the rules in the design docs.
4. Write the build ticket(s) for his answer, with their `blocked_by` (engine#3043 for the web save), and add them to 1064's `blocked_by` if they touch the full game's rules. Ask Nick before filing any engine ticket (option 2's web warning), and file it through the `engine-change` skill only after he says yes.

## Acceptance criteria

- [ ] `docs/design/death-and-difficulty.md` (and `demo.md` if the demo differs) records Nick's answer with his words.
- [ ] Each part of his answer that needs building has an open ticket naming its `blocked_by`, or the completion notes say nothing is to be built (option 1).
- [ ] `cargo xtask ticket-lint` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None here: a design decision. The build tickets list theirs (Harness tests that the suspend save exists after `Suspend` or `QuitRequested` mid-battle, and that `Continue` restores and deletes it).

## Completion notes

*(Filled in by the session that completes the ticket: Nick's answer, the tickets filed, notes for Nick.)*
