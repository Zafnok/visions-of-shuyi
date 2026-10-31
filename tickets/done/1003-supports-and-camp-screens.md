---
id: "1003"
title: Supports and Camp lists at camp
type: feature
milestone: Post–Chapter 1
model: implementation
effort: medium
status: done
blocked_by: ["1002", "0008", "0408", "0704", "0801", "0027"]
nick_input: none
completed: 2026-10-29
---

# 1003 — Supports and Camp lists at camp

## Context

[`docs/design/supports.md`](../../docs/design/supports.md) (ticket 0010):
once unlocked, support conversations are read **at camp** (a screen between
battles, not only between chapters) from a **Supports** list, next
to a list of optional scripted **camp events** (extras, Triangle Strategy
style). The rules and state come from 1002, the dialogue player from
0702/0704, and the game flow from 0801/0408. Where camp appears (0008, `world-structure.md`): between linear chapters,
before each story battle's Preparations, and, from the world map on, as
`Camp` in the world map menu (hooked up by 1007).

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> Nick plays a few battles, opens Supports and Camp,
> reads a conversation of each kind, and says whether it's easy to see what's
> new.

## Scope

**In:**
- A **Supports** screen: every pair with an unlocked or seen conversation,
  grouped by unit, showing the rank (C/B/A) and "NEW" on unlocked, unviewed
  ones. Picking one plays it with the dialogue player and sends 1002's view
  command. Seen conversations can be re-read.
- A **Camp** screen: camp events available now (conditions: after a story flag or battle,
  listed units alive and recruited), "NEW" markers, played with the dialogue
  player. Seen events are remembered in the campaign state and saved.
- A camp event data format (id, conditions, script id) with content
  validation.
- A **Camp** screen in the game flow between battles that holds both lists.
  Offer it between linear chapters and before each story battle's
  Preparations, as a screen other menus can open (1007 adds it to the world
  map menu).
- The unit info screen lists the unit's support partners and ranks.
- One placeholder support conversation and one placeholder camp event in the
  data, so the screens can be tested.
- The Camp screen plays the `camp` pool chosen in 0027
  (`docs/design/audio.md`): three tracks (The Old Tower Inn, Woodland
  Fantasy, The Bard's Tale) that **rotate** while camp is open: one plays
  to its end, fades out, a short quiet, the next fades in (the timings and
  order are in audio.md's *Starting values*). Import them
  (`assets-src/audio/import.py`), add them to `assets/audio/audio.ron`,
  `THIRD_PARTY_ASSETS.md` and the credits.

**Out (do not do):**
- Writing real support or camp conversations (story pipeline tickets).
- Rules changes (1002), hub activities (1004), pair abilities (1005).

## Implementation steps

1. Read `supports.md`, 1002's code, and the 0408 and 0704 screens.
2. Add the camp event format and loader, and campaign state for seen events.
3. Build the two list screens with the existing menu widgets and help bar.
4. Wire them into the game flow between battles and the unit info screen.

## Acceptance criteria

- [x] The Supports list shows unlocked and seen conversations with rank and NEW.
- [x] Viewing a support conversation raises the rank (1002 rules).
- [x] The Camp list shows only events whose conditions are met; seen state is saved.
- [x] Unit info lists supports and ranks.
- [x] Starting rules and anything left to judge by playing are in the
      Completion notes (they go on 1064's checklist).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: camp event conditions; seen-state tracking.
- Snapshot: Supports list, Camp list, unit info support section.
- Integration: scripted keys open Supports, play a NEW conversation, then
  check that the rank went up and NEW is gone.

## Completion notes

Done 2026-10-29.

**What was built**

- `core`: `trpg_core::camp::CampEvent` and its conditions (every story
  flag set, every battle won, every character in the army).
  `Campaign` gains `battles_won` (a victory adds the battle) and
  `camp_seen` (`see_camp_event`); both are left out of a save while
  empty, so `SAVE_VERSION` 2 saves still load and the golden saves
  are unchanged.
- `content`: `assets/data/camp.ron` (empty: the story pipeline fills
  it, 1073) and `trpg_content::camp`, which checks ids, titles, scenes,
  battles and characters. The test files add the placeholder event
  `test_campfire` (`assets-test/data/camp.ron`,
  `dialogue/test_camp.dlg`); the placeholder support conversations were
  already there from 1002. Camp event scenes count as played away from a
  battle (`@place battle` checks) and with their characters there
  (presence checks). Camp event titles are translatable
  (`camp.<id>.title`).
- `ui`: the Camp screen (`screens/camp`: logic, view, glyph skin):
  sections Supports / Camp events / Break camp; Supports lists units,
  then a unit's conversations with rank and NEW; watching a NEW one
  calls 1002's `Campaign::view_support`. The flow opens it between
  chapters and before a battle's Preparations. The unit info screen
  lists support partners and ranks.
- Music: the three 0027 tracks imported (`import.py`, full versions),
  in `audio.ron` as the `camp` pool (not looped), credited in
  `audio.ron` and `THIRD_PARTY_ASSETS.md`. Camp rotates them.

**Deviations**

- The music rotation fades out over the engine's one fade (0.5 s) and
  starts the next track at full volume, not audio.md's 3 s fade-out /
  3 s fade-in: the engine has no fade-in or per-switch fade yet
  (engine#2341). The 5 s quiet and the order are as audio.md says.
  Marked `// engine-friction: engine#2341`; follow-up ticket **1080**.
- No Playdate (one-bit) skin for the Camp screen or the info screen's
  supports: that is 1067.
- No `frame-png` pictures: camp can't be reached by a key script from
  the title yet (no chapter after Chapter 1). The insta text snapshots
  (`screens/camp/snapshots/`, `battle/snapshots/...info_supports...`)
  show the screens.
- Engine friction: no estimate was written before coding (missed).
  Judged afterwards against the Preparations screen (~970 lines for a
  richer screen), about 600 production lines was a fair size; measured
  856 (1.4×), under 2×.

**the agent's starting rules (for the Act 1 playtest, 1064)**

1. Camp opens **once between two battles**: between two chapters
   (before the next chapter's scenes), or else before the Preparations
   of a battle that has them. Restart, Retry and Continue don't open it.
2. Camp is **skipped when it has nothing to list** (no support
   conversation unlocked or seen, no camp event available).
3. The Supports list is **by unit** (roster order); each unit's list has
   one row per conversation (seen ones and the unlocked one), so a pair
   shows under both of its units.
4. **NEW** shows on the conversation, on its units and on the Supports
   / Camp events sections, in the warning colour.
5. A partner who **left the army** (Classic death, a scripted death)
   keeps their seen conversations, which can still be watched from the
   other unit's list.
6. **Cancel never leaves camp**: it moves the focus to Break camp;
   Confirm there leaves, with no question.
7. The unit info screen lists **only partners with a rank gained**
   (nothing when there are none).

**For 1064's checklist** (Nick plays a few battles, opens Supports and
Camp, reads a conversation of each kind): is it easy to see what's new;
does the once-between-battles rule feel right; and the music fades
(after 1080).
