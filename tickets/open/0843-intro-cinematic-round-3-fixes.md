---
id: "0843"
title: "Intro cinematic: longer talk, shorter fight hold, hit sounds, logo on the chord, a cleaner start"
type: feature
milestone: M7 Chapter 1 & game flow
model: sonnet-5
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
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
   blow about **0.5 s** (*Claude's starting rule*), by starting the
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
   title backdrop go to black (cut, or a 0.2 s fade, *Claude's starting
   rule*), then the cinematic fades in from black over 0.5 s (*tunable*).
   The menu no longer dissolves over the cinematic's first frames. The
   way back (the menu fading in on the logo) is unchanged.
- Update `docs/design/title-screen.md` (*As built* table and *Claude's
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

*(Filled in by the session that completes the ticket.)*
