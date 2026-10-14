---
id: "0729"
title: A dialogue log to read back, see and replay earlier lines
type: feature
milestone: M6 Story & dialogue
model: opus-5.5
effort: high
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0729 — Dialogue log

## Context

Nick, Chapter 1 playtest round 2 (0804, 2026-10-14):

> "we should have a chat log, accessible via key shortcut, optional
> binding. when a conversation plays, press chat log button and able to
> scroll thru messages, hover to see the portraits and that dialogue how it
> was, click to skip back (with option to skip forward again to latest seen
> point). Also if we add VA then it will play the voice again."

The dialogue screen (`crates/ui/src/screens/dialogue.rs`, ADR-0011's
`.dlg` scripts) shows one text box at a time with up to two portraits;
lines have stable line ids (`cargo xtask lines`, ADR-0046) that voice clips
are keyed by. Voices are played by 0720 (open; either order works: if 0720
is done, the log replays a line's clip; if not, add a line to 0720 saying
the log must play it). Screens follow ADR-0054 (logic builds a view, a skin
paints it). Keys follow the `keyboard-input` skill: a new **optional**
action (`docs/design/controls.md`, the required/optional lists), unbound
by default in both layouts and the controller unless Nick picks a key.

## Nick input

**Decision first** (ask with the `ask-nick` skill, mockups of the log over
the Chapter 1 opening with the bought portraits, real games for each):

1. What the log shows per line: speaker name and text (Fire Emblem: Three
   Houses' and Persona 5's backlogs), plus a small face?
2. "Hover to see the portraits and that dialogue how it was": the screen
   behind the log shows the scene as it was at the highlighted line
   (portraits, expressions, caption), and the log dims over it?
3. "Click to skip back": the scene goes back to that line and plays on
   from there; "skip forward again to latest seen point" jumps back to
   where the player was. What happens to a `@choice` passed on the way back
   (its answer is kept and shown, or asked again)? Claude's suggestion:
   kept, shown greyed, not asked again.
4. Does the log hold the current scene only, or every scene of the
   chapter or the save (and the battle's lines: boss lines, death quotes)?
5. Key: unbound by default (Nick: "optional binding"), or a default key?
6. In the title cinematic's `Talk` shots: no log (Claude's suggestion).

Record the answers in `docs/design/look-and-feel.md` (*Dialogue log*) and
`controls.md`. **Sign-off:** in the Chapter 1 opening, open the log, scroll,
jump back three lines, jump forward to where you were.

## Scope

**In:**
- The decision above, recorded.
- The log screen (view + skin), opened by the new action while a dialogue
  scene plays, scrolled with the cursor keys, closed with Cancel.
- Showing the scene as it was at the highlighted line; jumping back; jumping
  forward to the furthest line seen.
- Replaying the line's voice clip when voices exist (see *Context*).
- The action in `keymap.ron` (optional, unbound unless Nick picks a key), on
  the Key bindings screen, and in the help bar while dialogue plays.
- Text through the language files (ADR-0045), not literals.

**Out (do not do):**
- A log of battle events (attacks, level-ups).
- Changing how dialogue scripts are written.
- Anything Nick's answers rule out.

## Implementation steps

1. Ask Nick; record. Don't build before the answer.
2. Keep a history in the dialogue screen: for each box shown, the line id,
   speaker, text and the portraits/caption state at that point (enough to
   redraw it). Jumping back restores that state and the script position;
   jumping forward replays to the furthest box without re-asking choices.
3. `Action::DialogueLog` (name it to fit `Action`'s naming), optional, in
   `keymap.ron` for both layouts and the pad (`[]` if unbound).
4. The log view and skin (`screens/dialogue/log.rs` or similar).
5. Voice replay through the same path the dialogue screen uses (0720).

## Acceptance criteria

- [ ] Nick's answers recorded in `look-and-feel.md` and `controls.md`.
- [ ] Harness: with a key bound to the log action, three boxes into
      `ch01_intro`, open the log: it lists the three lines in order (read
      the view).
- [ ] Harness: highlight the first line and jump back: the dialogue shows
      the first box with the portraits it had then; jump forward: the third
      box again.
- [ ] Harness: a `@choice` passed on the way back behaves as Nick decided.
- [ ] The action is unbound by default (or Nick's key) and listed on the Key
      bindings screen; `cargo xtask check-keys` passes.
- [ ] Snapshot of the log in both layouts.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the history (push, jump back, jump forward, choices).
- Snapshot / integration: the Harness tests and snapshots above.

## Completion notes

*(Filled in by the session that completes the ticket.)*
