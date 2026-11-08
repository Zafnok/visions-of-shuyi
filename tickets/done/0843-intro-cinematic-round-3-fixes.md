---
id: "0843"
title: "Intro cinematic: longer talk, shorter fight hold, hit sounds, logo on the chord, a cleaner start"
type: feature
milestone: M7 Chapter 1 & game flow
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-07
---

# 0843 — Intro cinematic: Nick's round 3 fixes

## Context

Chapter 1 playtest round 3 (0804, 2026-10-16), on the title's intro
cinematic as 0832 and 0833 left it (`assets/cinematics/title.ron`,
`docs/design/title-screen.md`, *Intro cinematic*):

| Asked | Nick |
| ----- | ---- |
| Tamsin's news is up for 6 s (was 18.5) | "just a bit too short if it was 7-8s it would probably be good." |
| The two combat scenes, ~2 s hold at the end | "hold at end too long, want hit sounds, and we need the battler for tamsin" |
| Logo at 1:57 or on the chord at 1:55.8 | "On the chord, 1:55.8" |
| After 15 s idle the menu fades out over 1 s into the cinematic | "it's a little obnoxious the way the fade in and fade out weave together if you press nothing immediately. So the menu box fades out slow but we line up the playhead to be where a dialogue box appears and it looks like a visual artifact. Probably should detect if it's a dialogue scene and switch a lot sooner, or just don't fade out the menu and instead go to blacck and fade in the cutscene intro only." |

Kept as they are (Nick, round 3): the cuts land on the music, units move
like the game, the intro gives nothing away.

Tamsin's combat picture is **0446** (the Pixel Flag Paladin, which Nick
still has to buy); not this ticket. The bigger rework of the
cinematography Nick asked for is **0063**; this ticket is only the fixes
above, and 0063's build may later replace shots this ticket touches.

**Waits for the engine carve and the Playdate port** (Nick, 2026-10-16),
so `blocked_by` has 1118 and 1050.

**Re-gated 2026-10-21 (0129):** the carve is done and 1050 was folded
into the Act 1 playtest (1064), so neither 1118 nor 1050 is in
`blocked_by` any more.

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> Watch one loop with sound on. Is the news long enough? Is
> the end of each fight still held too long? Do the hit sounds sit well
> over the song? Does the switch from menu to intro look clean?

## Scope

**In:**
1. **Tamsin's news: 7.5 s** (shot 4, `Talk(scene: "ch01_prebattle")`).
   The cuts were placed on the song by measuring its loudness (the file's
   header comment). Lengthening the shot moves either its start (50.0) or
   the next cut (56.0). Measure the song around both (the same way 0833
   did, in its completion notes) and move the one that keeps a cut on a
   note or beat; say which in the completion notes.
2. **Shorter hold after each fight** (shots 6 and 8, `Combat`). Today the
   fight takes about 4.1 s of a 6 s shot. Make the hold after the last
   blow about **0.5 s** (*the agent's starting rule*), by starting the
   combat shot later and letting the map shot before it run longer, so
   the cut **after** the fight (74.5, 91.25) stays where it is. Re-time
   the `field` action so nothing walks or strikes during the wrong shot.
3. **Hit sounds in combat shots.** The combat scene plays its own sounds
   in a battle (hit, miss, critical, a fall). Play the same sounds during
   a `Combat` shot, over the song, at the volume the effects option sets.
   0832's "no sound during the shot" rule is replaced; update
   `title-screen.md`.
4. **The logo on the chord:** shot 12 (`Logo`) at **115.8** instead of
   117.0, and shot 11 ends there.
5. **Menu to intro through black.** When the 15 s run out, the menu and
   title backdrop go to black (cut, or a 0.2 s fade, *the agent's starting
   rule*), then the cinematic fades in from black over 0.5 s (*tunable*).
   The menu no longer dissolves over the cinematic's first frames. The
   way back (the menu fading in on the logo) is unchanged.
- Update `docs/design/title-screen.md` (*As built* table and *the agent's
  starting rules*) with every change.

**Out (do not do):**
- Tamsin's picture (0446), the cinematography rework (0063), a setting
  to turn the intro off (0844).
- Moving any other cut.

## Implementation steps

1. Read `assets/cinematics/title.ron`, `crates/ui/src/screens/title.rs`
   (`IDLE_SECS`, `FADE_OUT_SECS`, `FADE_IN_SECS` and the fade state) and
   `crates/ui/src/cinematic.rs` (how `Combat` shots run, 0832).
2. Measure the song (step 1 of Scope) and set the new times in
   `title.ron`, keeping the header comment's list of cuts true.
3. Add the combat scene's sounds to `Combat` shots (where the battle's
   combat screen asks for them; reuse that, don't duplicate the table).
4. Change the title's idle-to-cinematic transition to go through black.
5. Update the title-screen tests and snapshots that pin times.

## Acceptance criteria

- [ ] Shot 4 lasts 7.5 s; the completion notes say which neighbouring cut
      moved and why it still sits on the music.
- [ ] Each combat shot ends about 0.5 s after the fight's last blow; the
      cuts at 74.5 and 91.25 are unchanged.
- [ ] A `Combat` shot plays hit/miss/critical sounds (a test asserts the
      sound requests the shot makes).
- [ ] The logo shot starts at 115.8.
- [ ] Going idle on the title: the frame goes to black before the
      cinematic's first frame shows; no frame shows the menu over a
      cinematic shot (a test steps the frames).
- [ ] `title-screen.md` matches the build.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the timeline's shot starts; the sound requests of a combat shot.
- Integration: the title's idle transition frame by frame (menu, black,
  cinematic), via the screen harness.
- Snapshot: updated title/cinematic snapshots, each reviewed.

## Completion notes

**Done 2026-11-07.**

**Estimate** (ADR-0092): about 80 production lines, judged by 0819's
fade phases and 0424's battle sound cues: two new title phases, a
`picture` field the two skins paint, and a `sounds(from, to)` over the
combat shot's existing `Playback` cues. PR size estimated at about 700
changed lines (tests and snapshots of 0833's size).

**What changed** (`assets/cinematics/title.ron`):

| # | Shot | Was | Now |
| - | ---- | --- | --- |
| 4 | Tamsin's news | 0:50 (6 s) | **0:48.5 (7.5 s)** |
| 6 | Combat: the brigand attacks the lead | 1:08.5 | **1:09.95** |
| 8 | Combat: Tamsin at the footbridge | 1:25.25 | **1:28.2** |
| 12 | The logo | 1:57 | **1:55.8** |

1. **Tamsin's news, 7.5 s: the start moved, not the cut after it.** The
   song was measured again (a throwaway loudness meter on the ogg in 50 ms
   windows, as 0833 did). 48.5 has a clear note (+7 dB onset, -37.6 to
   -30.5 dB), the bar before the phrase at 50.0; the cut at 56.0 is the
   strongest one around (a note after a second of hush) and the field
   action's clock hangs on it. Moving 56.0 to 57.5 would have put the cut
   between notes (the next ones are at 57.7 and 57.85). Shot 3, the
   faces, is 1.5 s shorter, so they cross a little faster.
2. **Half a second after the last blow.** The fights last 4.1 s (road) and
   2.55 s (footbridge), so the shots start at 69.95 (a note, +9.4 dB;
   0.45 s hold) and 88.2 (inside the sustained loud part, no onset near;
   0.5 s hold). The cuts after them (74.5, 91.25) are unchanged. The
   map shots before run longer; the field action was re-timed so nothing
   happens in the wrong shot: the road brigand now walks on the phrase at
   68.5 (was 67.0), Tamsin walks to the footbridge at 87.25 (was 83.75, on
   a note at 87.3), and the four `Strike`s that set the map's HP moved
   into their combat shots.
3. **Hit sounds.** A combat shot's `Duel` now builds its playback with
   `Playback::with_sounds` exactly as the battle does
   (`event_sounds::combat_attacks`, `heals`): the same hit, miss, block,
   critical and spell sounds at the same moments. `CinematicPlayer::sounds
   (from, to)` gives the cues a stretch of time crosses; the title plays
   them through `ctx.audio.play_sound` (the effects' volume, as a
   battle's) while the cinematic shows. The battle's sound table isn't
   duplicated. The scene cues an attack can set off stay out, as in 0832.
   The battle's own combat scene plays no separate "fall" sound, so
   neither does the shot.
4. **The logo at 1:55.8**, on the chord's onset (measured: it strikes at
   115.6 to 115.85). The skip's silence is now 17.9 s (the logo shot's
   length).
5. **Menu to intro through black.** New title phases `ToBlack` (menu and
   title picture to black, 0.2 s, `TO_BLACK_SECS`) and `FromBlack` (the
   cinematic alone from black, 0.5 s, `FROM_BLACK_SECS`); `FADE_OUT_SECS`
   is gone. `TitleView` gained `picture` (how solid the title's own
   picture is) instead of tying it to the cinematic's brightness; the
   glyph skin paints black under it, the Playdate skin shows black
   between the picture and the cinematic (one bit, cut at half). The same
   path runs when the song loops under the logo. The way back (the menu
   fading in on the logo) is unchanged.
- `title-screen.md` (*As built*, the song, rules 2 and 5, the starting
  rules) and `assets/cinematics/README.md` updated.

**Tests:** `the_titles_shots_start_on_round_threes_times`,
`the_titles_fights_end_half_a_second_before_their_cuts` (each hold 0.4 to
0.6 s), `the_shot_plays_the_battles_sounds_for_the_fight` (frame by frame
at 60 a second: every battle cue once, in order, at its moment), the
title unit tests for both new phases and presses during each,
`the_titles_picture_goes_to_black` (glyph skin), and in the harness
`going_idle_the_frames_go_through_black` (frame by frame: menu, to black,
from black, cinematic; the cinematic's first frame is black; no frame has
the menu or the title's picture over a shot) and the loop test now
asserting the sounds the title asked for are the two fights' cues.
Snapshots changed: `title_cinematic_3_faces` (faces further along in
the shorter shot), `_5_turn_one` and `_7_turn_two` (the camera's pan
point at the same second of a longer pan). The fight snapshots are
unchanged (same moment of each fight).

**Deviations**

- A press while the cinematic fades in from black brings the menu back
  and does nothing else (the menu wasn't showing). A press while going
  to black still acts on the menu, as during 0819's fade.
- Not watched with sound (the agent can't hear); the cuts and the logo
  were placed by measuring.

**the agent's starting rules** (Nick can veto): 0.2 s to black; the hold
of about half a second; the footbridge fight starting at 88.2 where the
song has no onset; the brigand's and Tamsin's walks re-timed onto notes;
a press during the fade-in from black only brings back the menu.

**Follow-up tickets:** none.

**For the Act 1 playtest (1064)**, sound on, one loop: is the news long
enough at 7.5 s? Is half a second after each fight right? Do the hit
sounds sit well over the song? Does the switch from menu to intro look
clean through black?
