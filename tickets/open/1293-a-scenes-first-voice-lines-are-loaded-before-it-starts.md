---
id: "1293"
title: "A scene's first voice lines are loaded before it starts, from the title and each save"
type: bug
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1293 — A scene's first voice lines are loaded before it starts, from the title and each save

## Context

Nick, 2026-11-17, playing the itch.io build: "some of the ai voices,
consistently, the first one (thornmarch something) dont load on time".

Why (`docs/voice.md`, *How the game plays them*): the dialogue screen
calls `ctx.preload_voices` with a scene's lines only when the scene
starts (`crates/ui/src/screens/dialogue.rs`), and says the first line on
that same frame. The engine drops a clip that isn't loaded 0.3 s after
it was asked for (`LATE_SECS` in the engine's
`crates/platform/src/audio/player/voice.rs`: "a voice starting in the
middle of a line is worse than none"). On the desktop the clip is read
from disk in time. On the web it is fetched and decoded, which takes
longer on itch.io, so the **first line of a scene** is skipped. Later
lines are fine: they were preloaded while the player read the earlier
ones. Ticket 0494 found and fixed the same thing for Vix's tips in
battle (its completion notes describe the reproduction).

Nick's direction (2026-11-17): "pre-load at least a few lines of intro +
voice lines that might be played from each save... cancel loads of
irrelevant ones after loading a save and queue up other lines in that
path".

The engine already has what this needs: `Ctx::keep_voices` (engine#3213,
used by the battle since 0496) keeps a set of clips loaded until another
set replaces it, and a clip that is both kept and in a scene's preload
list is loaded once and plays at once. No engine change.

## Nick input

None. Whether the opening sounds right goes on the Act 1 playtest's
checklist (1064).

## Scope

**In:**
- While the title screen (and the screens before a game starts: the
  voices question, Load Game, lead select) is up, keep loaded the first
  3 voiced lines of each scene that may play first: New Game's opening
  scene, and for each save on the Load Game list, the scene its game
  starts with when loaded (if it starts with one).
- When the player picks New Game or a save, narrow the set to that
  path's lines (letting the others go), so they are loaded before the
  scene's screen asks for them.
- More generally, before any screen change that starts a scene (the
  flow moving to a chapter's opening, a battle's closing scene), the
  scene's first lines are kept a moment ahead where the flow already
  knows what comes next (`crates/ui/src/flow.rs`). If this turns out to
  be more than a small change, do the title and saves only and file
  the rest as a ticket.
- `docs/voice.md`: say who preloads what before a scene starts.

**Out (do not do):**
- Changing `LATE_SECS` or `AHEAD`, or letting a late clip play (the
  engine's rule stays).
- Holding a scene's first box until its clip loads. If preloading
  can't cover a case (a save loaded straight into a scene with no
  screen before it), say so in the completion notes and file it.
- The battle's guide lines (0494, 0496): they keep their own set. The
  battle replaces the kept set on its first frame and empties it when it
  leaves; check that doesn't let a scene's kept lines go before the
  scene preloads them.
- Any line's text or clip (dialogue writing is paused, 0766).
- The out-of-battle tips' lines (1291 keeps them loaded on the title).
  Either order works, but the engine keeps one set: if 1291 is done,
  the title's kept set is its tips' lines plus the scenes' first lines,
  from one owner; if this lands first, 1291 adds its tips' lines to this
  ticket's set instead of sending its own.

## Implementation steps

1. Find what plays first: for New Game, the opening scene's id; for a
   save, the scene its loaded game starts with (read how `flow.rs` and
   the save screens resume a game). Write one function that answers
   "the scene (if any) this start leads into" for both.
2. Find a scene's first voiced lines the way the dialogue screen does
   (`coming_lines` in `dialogue.rs`), the first 3 (the engine's
   `AHEAD`).
3. Add a small owner of the pre-game kept set (like `GuideVoice` in
   `screens/battle/guide_voice.rs`): it sends `ctx.keep_voices` only
   when the set changes. The title screen sets it to New Game's lines;
   the Load Game list adds each listed save's; picking one narrows it.
4. Make sure no screen between the pick and the scene sends an empty
   `keep_voices` first.
5. Tests, then the web check (below), then `docs/voice.md`.

Expected size: about 80 production lines (one lookup, one owner of the
kept set, three call sites), judged by `guide_voice.rs`.

## Acceptance criteria

- [ ] A test shows the title screen hands `keep_voices` the opening
      scene's first 3 voiced lines (read from the test `Ctx`).
- [ ] A test shows the Load Game list keeps each save's first lines and
      picking one narrows the set to that save's.
- [ ] A test shows the opening scene's first line is in the kept set when
      the dialogue screen first asks for it.
- [ ] In a web build served with every clip held 0.5 s (as 0494 did:
      a local server that delays each `.ogg`, in the browser pane), New
      Game's first line and a loaded save's first scene line are fetched
      before their box shows, and play. Use the non-release web build
      (`cargo shuyi web`): a local `--release` build stalls on wasm-opt.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: which lines are kept, for New Game and for a save.
- Integration: the title → opening scene path through the harness,
  reading the voice requests.
- Manual: the delayed-clip web check above.

## Completion notes

*(Filled in by the session that completes the ticket.)*
