---
id: "1293"
title: "A scene's first voice lines are loaded before it starts, from the title and each save"
type: bug
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-18
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

- [x] A test shows the title screen hands `keep_voices` the opening
      scene's first 3 voiced lines (read from the test `Ctx`).
- [x] A test shows the Load Game list keeps each save's first lines and
      picking one narrows the set to that save's.
- [x] A test shows the opening scene's first line is in the kept set when
      the dialogue screen first asks for it.
- [ ] In a web build served with every clip held 0.5 s (as 0494 did:
      a local server that delays each `.ogg`, in the browser pane), New
      Game's first line and a loaded save's first scene line are fetched
      before their box shows, and play. Use the non-release web build
      (`cargo shuyi web`): a local `--release` build stalls on wasm-opt.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: which lines are kept, for New Game and for a save.
- Integration: the title → opening scene path through the harness,
  reading the voice requests.
- Manual: the delayed-clip web check above.

## Completion notes

**What changed.** `DialogueScreen::first_lines` gives a scene's first
3 lines from its first box (that box's, then `coming_lines`). The new
`crates/ui/src/flow/first_lines.rs` holds the lookup and the owner of
a kept set:

- `opening_scene(ctx, Start)` answers "the scene this start leads into"
  for New Game (its first chapter's first intro scene with anything to
  say for the army; a world-map chapter's opening scene) and for a save
  (a chapter save's next chapter's; a world-map save, or a save of the
  last chapter, none). It builds the scene as `FlowScreen::play` does
  (lead, those present), so a scene the army has nobody for is passed
  over as the flow passes it over.
- `KeptVoices` sends `ctx.keep_voices` only when its set changed, or
  on the first call after `covered()`. With voices off it sends
  nothing and is due again once they are on (New Game asks about voices
  after the title).

Who keeps what: the **title** keeps New Game's first lines while on
top (again after a screen over it); the **Load Game list** keeps
those plus each listed save's (again after an import). The **flow**,
each time it pushes a screen (`push_next`), keeps the first lines of
the scene it pushes, if it is one, plus those of the scene it knows
comes after it (`coming_lines`): the next scene waiting (the next
intro scene; the victory scenes while the results show), New Game's
opening during the mode, familiarity, voices and lead screens, the next
chapter's at the save prompt and the save picker, and the chapter camp
goes on to. Picking New Game or a save therefore narrows the set to
that path's lines. The flow always resends on a push: the battle may
have replaced the set in between.

**After merging main's 0812** (a chapter's title card before its
first scene, 3 s): the card is one more screen the flow pushes with the
intro queued, so the flow keeps the intro's first lines while it shows;
the two integration tests now pass the card, and the Load Game test
checks the set is the save's alone from the card on. The web check
below was made before that merge, without the card.

**Step 4 / the battle.** The engine drops a load the moment a new
kept set leaves it out, so a scene's own lines stay in the flow's set
when the flow pushes the scene (its first frame asks for line 1). No
screen between a pick and the scene sends an empty set; the battle
sends one when it leaves, and the flow's push of the results (or the
next screen) right after keeps the victory scenes' lines, which
`the_victory_scenes_first_lines_are_kept_while_the_results_show`
checks.

**Not covered** (filed as **1296**): scenes the battle plays over
itself from its triggers (`ch01_first_turn`, the boss and death
scenes), and a world-map battle's intro (the map pops straight into
it). Also not covered, and not filed: a victory scene when a battle
gives no results (never in the game's chapters), and a gendered line
among New Game's first 3, kept in the clip of the lead the title knows
(the last save's, or the default) until the lead is picked; Chapter
1's first gendered line is its 7th.

**Tests.** Unit (`flow/first_lines/tests.rs`): New Game's and a
chapter save's lines, a save with no scene, a scene passed over,
`KeptVoices` (sent once, changes, duplicates and unvoiced lines left
out, covered, an empty set only to let one go, voices off), and the
title's first frame read from the test `Ctx`. Integration
(`tests/it/voice.rs`): title → New Game → the opening scene, the last
set kept before the first `PlayVoice` holds its first 3 lines; the
Load Game list keeps New Game's and the save's, and loading narrows to
the save's before its box asks. `tests/it/flow.rs`: the victory scene's
lines kept while the results show.

**Web check.** `cargo shuyi web` (non-release), served by a Python
server holding every `.ogg` 0.5 s with `Cache-Control: no-store`,
logging each request, in the browser pane (hidden; frames driven by a
`MessageChannel` loop). With the private content, New Game is Chapter
1. The log, in seconds from the server's start:

- 16.9: `ch01_intro_bac90108`, `_6b303648`, `_706633d0` (the intro's
  first three lines, the Thornmarch line among them), asked for by the
  title, before any key.
- 71.1: the scene starts (New Game, Classic, not familiar, voices yes,
  a lead): `ch01_prebattle_91081df4`, `_81957f69`, `_1d315e4f` (the next
  scene's first three, kept by the flow), `ch01_intro_8061933e` (the
  dialogue's own preload of line 4) and the scene's music. No request
  for line 1: its clip was loaded 54 s before its box ("Three years
  after the Ash Peace...") showed, so it plays at once.
- 111.2: the prebattle scene starts (the intro skipped, its choice
  answered): only `ch01_prebattle_7072f12a` (its line 4). Its first
  line, Maud's box on screen, was loaded at 71 s.

A loaded save's first line wasn't checked on the web: Chapter 2, where
a save after Chapter 1 goes on, has no clips yet, so there is nothing
to fetch. The save path is checked by the harness test above; the last
acceptance box for the web stays unticked for that half. Whether the
opening sounds right goes on the Act 1 playtest's checklist (1064).

**Engine friction:** estimate 90 (`guide_voice.rs`'s keep logic, a
lookup and three call sites), measured 164 (1.8×), under 2×. Signals:
workaround no · third copy no · budget no.

**Follow-up tickets:** 1296 (the battle's scenes and a world-map
battle's intro).

No gameplay rule was decided.
