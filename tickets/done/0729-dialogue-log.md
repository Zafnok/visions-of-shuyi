---
id: "0729"
title: A dialogue log to read back, see and replay earlier lines
type: feature
milestone: M6 Story & dialogue
model: reasoning
effort: high
status: done
blocked_by: []
nick_input: decision
completed: 2026-10-14
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
   (its answer is kept and shown, or asked again)? the agent's suggestion:
   kept, shown greyed, not asked again.
4. Does the log hold the current scene only, or every scene of the
   chapter or the save (and the battle's lines: boss lines, death quotes)?
5. Key: unbound by default (Nick: "optional binding"), or a default key?
6. In the title cinematic's `Talk` shots: no log (the agent's suggestion).

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

- [x] Nick's answers recorded in `look-and-feel.md` and `controls.md`.
- [x] Harness: with a key bound to the log action, three boxes into
      `ch01_intro`, open the log: it lists the three lines in order (read
      the view).
- [x] Harness: highlight the first line and jump back: the dialogue shows
      the first box with the portraits it had then; jump forward: the third
      box again.
- [x] Harness: a `@choice` passed on the way back behaves as Nick decided.
- [x] The action is unbound by default (or Nick's key) and listed on the Key
      bindings screen; `cargo xtask check-keys` passes.
- [x] Snapshot of the log in both layouts.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the history (push, jump back, jump forward, choices).
- Snapshot / integration: the Harness tests and snapshots above.

## Completion notes

**Nick's answers** (2026-10-14, three rounds of rendered mockups over the
Chapter 1 opening with the bought portraits; recorded in
`look-and-feel.md`, *Dialogue log*, `controls.md`, *Dialogue log*, and
`options.md`, *Voices in the dialogue log*):

1. Look **C**: the log fills the screen; each line has the speaker's face
   as it was on that line (the portrait at 1×, 8×4 cells), their name and
   the text.
2. A reply passed on the way back is **asked again**: the reply and every
   line after it are dropped; jumping forward stops at the line it answers.
3. **This scene only.**
4. Key: first "Up", then, told Up is Cursor up, a deliberate entry of its
   own that may overlap keys "in different context menus", with a system
   for it; then, told Persona 5 Royal opens its log with one key (`B`),
   **one key like Persona**: `b` right-handed, `n` left-handed, the left
   face button on a controller, in its own group *In the dialogue log*.
   If a player puts it on a key the scene uses, the scene wins.
5. Forward: **both** dim lines you can pick and a *Back to latest* row.
6. Voices: also **on hover, with an option to turn that off**.
7. No log in the title cinematic.

**What was built**

- `Action::DialogueLog` and **key groups** (ADR-0068): an action of one
  group may share a key or button with another group's. `keymap.ron`,
  `Keymap`, `InputState`, `LayoutBindings` / `PadBindings` and their
  load-time repair keep a key unique per group; one press gives each
  group's action. Defaults `b` / `n` / `West` (shared with Danger zone).
- The Key bindings screen's third group, *In the dialogue log*: Open the
  log. To fit it, the blank row under the keyboard / controller switch is
  gone and the panel runs down to the row over the message.
- `screens/dialogue/history.rs`: every line seen, as a copy of the
  `DialoguePlayer` there (its scene is now shared, so a copy is cheap) and
  the music playing; jump back, jump forward, a reply asked again.
- `screens/dialogue/log.rs` (view, ADR-0054) and `log/glyph.rs` (skin);
  `portrait::draw_portrait_in` draws a face in any area.
- The text box names the log key next to Confirm's (`b log   f ▼`).
- `DialoguePlayer::line`, `reply_line`, `pending_music`.

**Deviations**

- Voices: 0720 isn't done, so 0720 got step 7 (the log plays a line's
  voice when the cursor moves onto it, unless the option is off) and 0826
  the Options row. Each `LogRow` carries its line id for it.
- The acceptance criteria said "with a key bound": Nick gave the action a
  default key, so the tests use `b` (and one rebinds it to Up).

**the agent's starting rules** (also in `look-and-feel.md` and `controls.md`):

- A text box too long for one page is one row of the log, in full; the
  line still typing when the log opens is in it in full.
- Lines passed with *Skip scene?* aren't in the log.
- Jumping to a line types its text out again from the start; the music is
  the music playing at that line.
- *Back to latest* shows only after a jump back.
- The log makes the menu sounds; the cursor stops at the first and last
  rows (no wrap).
- The log key does nothing while *Skip scene?* is asked.
- The text box shows `b log` beside the Confirm key; nothing when the log
  has no key.
- What the scene uses wins: Confirm, End turn and Cancel always, the
  cursor keys while the replies are up.
- The controller button is the left face button (`X` / `□` / `Y`): Nick
  answered the controller question with "see earlier reply" (do as
  Persona), and a PS4 guide puts Persona 5's log on `□`.
- *Voice on hover in the log* starts On.
- The Key bindings screen lost the blank row under the switch (to fit the
  new group).

**Sign-off for Nick:** in the Chapter 1 opening, press `b`, scroll, jump
back three lines, then *Back to latest*.
