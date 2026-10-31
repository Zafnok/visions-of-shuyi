---
id: "0820"
title: "The title cinematic itself: shots and snippets timed to the title song"
type: content
milestone: M7 Chapter 1 & game flow
model: reasoning
effort: high
status: done
blocked_by: ["0036", "0706", "0707", "0803", "0818", "0819", "0827"]
nick_input: sign-off
completed: 2026-10-13
---

# 0820 — The title cinematic itself

## Context

Everything needed to play a cinematic on the title exists (0817, 0818,
0819), but `assets/cinematics/title.ron` is a stand-in. This ticket makes
the real one from the storyboard Nick approved in 0036
(`docs/design/title-screen.md`, *Intro cinematic*, **Storyboard**): nine
shots with hard cuts. A pan over the party, the opening lines of
`ch01_intro` in the dialogue screen, six faces sliding past, the opening
lines of `ch01_prebattle`, a march, two close-up fights, the whole field,
and the logo during the song's silence. Its first shot (0:00–0:15) is
hidden by the menu on launch and only seen on later loops, so nothing a
first-time viewer needs goes there.

The title song (`title`, New Sunrise V1) is 133.7 s:

| Time | What the music does |
| ---- | ------------------- |
| 0:00–0:30 | Very quiet opening, slowly rising |
| 0:31–1:24 | Middle section, moderate |
| 1:25–1:56 | Full and loudest |
| 1:57–2:14 | The last note ends, then silence until the loop: **the logo** |

Real content it draws on: the Chapter 1 battle and map (0803), the cast's
bought faces (0706), the Chapter 1 script (0707) and the marching and
fighting shots (0827).

**The snippets are the script itself** (Nick, 0036: "we'll load the same
script file anyway and just play whatever is there for first few lines").
The `Talk` shots point at `ch01_intro` and `ch01_prebattle` in
`assets/dialogue/ch01.dlg`; there is no separate cinematic script and no
list of approved lines. Nick finds the current script too full of quips;
ticket 0724 rewrites it. Either order works: the cinematic shows whatever
the file says. If 0724 lands after this ticket and changes how many boxes
those scenes open with, 0724's session re-checks the counts here.

**No overworld shot yet** (0036, Q4: leave it out). 1010 adds it once the
world map exists.

## Nick input

**Sign-off** after merge, on the Pages build and the Windows download:
watch the title through one whole loop with sound. Does each shot land
where the music changes, is the logo up for the whole pause, do the
snippets give anything away, does anything look off? Comments become
follow-up tickets. Also ask him again, now that he has seen it, whether
Options should get a setting to turn the cinematic off (he said "later"
in 0036; `title-screen.md`, *Open sub-questions*), and record the answer
there (a yes becomes a ticket that adds the row to the Options menu).

## Scope

**In:**
- `assets/cinematics/title.ron`: the storyboard's shots at their times.
- Its two `Talk` shots read `ch01_intro` and `ch01_prebattle` from
  `assets/dialogue/ch01.dlg`, from each scene's first box. No new
  dialogue file.
- The march and the fights as 0827 actions on the Chapter 1 battle.
- Small timing changes so cuts land on the music (keep within a second or
  two of the storyboard; anything bigger goes back to Nick).

**Out (do not do):**
- New shots or characters that aren't in the storyboard, and any change
  to the script's lines. Ask Nick (`ask-nick`) if the storyboard can't be
  built as written.
- Code changes to the player or the title screen. If a shot needs
  something they can't do, write a ticket (`write-ticket`).
- Re-cutting the song.

## Implementation steps

1. Read the storyboard and its rules in `title-screen.md`. Listen to the
   song and note the exact second of each change you want a cut on.
2. Snippets: for each `Talk` shot choose `count`, the number of opening
   boxes of its scene, so that every box can be read twice in the shot's
   time (0818's timing) and the range stops before the scene's first
   `@choice`. Don't pick boxes from the middle of a scene and don't edit
   the script. Check the range against `docs/story/outline.md`: it must
   give away nothing past the opening premise (the openings of these two
   scenes don't today). If a rewrite has made an opening too long to show
   any of it, ask Nick.
3. Write `title.ron`: each shot's `at`, the map pans' start and end tiles
   and zoom on the Chapter 1 battle (`assets/battles/ch01.ron`), the six
   characters in the storyboard's order, the snippets, and `Logo` at the
   song's last note (about 117 s; set it by ear). For the march and the
   fights, write one 0827 action list shared by the four shots: choose
   units and paths that fit the real map and each unit's movement type,
   and fights whose weapons make sense (an archer strikes from two tiles
   away). Nobody in the party falls. Sizes as the storyboard says (3×, 4×,
   4×, 2×).
4. Watch it with the sound on in the native build and the web build, all
   the way round the loop, at least twice. Fix overlaps, cut-off text and
   shots that feel too short to read (a snippet must stay up long enough
   to read twice).
5. Snapshot each shot at one moment.

## Acceptance criteria

- [x] Content validation loads `title.ron`.
- [x] Every shot in 0036's storyboard is there, in order; the completion
      notes list each shot's storyboard time and final time.
- [x] The logo is on screen from the song's last note to the loop.
- [x] Both snippets start at their scene's first box, stop before its
      first `@choice`, and the notes give each one's scene and box count.
- [x] Snapshots: one per shot.
- [ ] Watched with sound on native and web for two full loops; the notes
      say so. (**Not met:** the agent can't hear. See the notes for what was
      checked instead; Nick's sign-off is the check by ear.)
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot / integration: one Harness snapshot per shot at a fixed music
  position; the 0819 loop test still passes with the real file.

## Completion notes

**Done 2026-10-13.** `assets/cinematics/title.ron` is the storyboard's nine
shots. No code changed: the player and the title screen already did
everything the storyboard needs.

**Shots and times** (the table is also in `title-screen.md`, *As built*):

| # | Shot | Storyboard | Built |
| - | ---- | ---------- | ----- |
| 1 | The party, 3×, the camera drifting to them | 0:00 | 0:00 |
| 2 | `ch01_intro`, 3 boxes | 0:15 | 0:15 |
| 3 | Six faces: the lead, Hollis, Tamsin, Aske, Maud, Rue | 0:31 | 0:30.75 |
| 4 | `ch01_prebattle`, 1 box | 0:50 | 0:50 |
| 5 | The march, 3× | 1:08 | 1:08.5 |
| 6 | The lead and a brigand on the road, 4× | 1:25 | 1:25.25 |
| 7 | Tamsin and Aske at the footbridge, 4× | 1:35 | 1:35.5 |
| 8 | The whole field, 2× | 1:45 | 1:44.5 |
| 9 | The logo | 1:57 | 1:57 |

No cut moved more than half a second.

**How the cuts were set.** the agent can't hear, so "by ear" became a
measurement: the song file's loudness in quarter-second steps (a
throwaway program, not in the repo). It swells at 30.75 s, phrases start
at 50.0 and 68.5, the loud part comes in at 85.25 and lifts at 95.5 and
104.5; the last chord is struck at 115.8 and has died away by 117.

**The snippets** (the script's own lines, from each scene's first box,
both before the scene's first `@choice`):

- `ch01_intro`, **3 boxes**: the three opening captions ("Three years
  after the Ash Peace." / "The war is won. The soldiers who won it were
  never paid." / "In Thornmarch, a long way from any court, an exile chops
  wood."), 5.25 s each.
- `ch01_prebattle`, **1 box**: Tamsin's news that Red Harl took the
  toll-fort, three lines, up for 18.5 s. Two boxes would get 9 s each,
  not enough to read three lines twice.

Neither gives away more than the opening premise (`outline.md`).

**The march and the fights** are one action list, `field`, shared by
shots 5 to 8 (its clock is the song's time less 68.5 s):

- The party leaves Harrowby east along the road. Tamsin (mounted) rides
  round to the footbridge and Aske follows to the bank below it.
- The brigand south of the road comes down to meet the lead. They trade
  blows; the brigand falls; the lead is left at 55% HP. Hollis comes up.
- At the footbridge Tamsin crosses and fights the brigand who guards it
  (she is left at 75%); Aske shoots him over the river from two tiles
  away; Tamsin fells him.
- Whole field: the enemy archer by the road falls back to the road
  bridge, the lead follows up the road, the others advance. The fort is
  at the top right.
- Two enemies fall, nobody of the party. Every walk is on tiles the
  unit's movement type can enter (the validator checks it), and no two
  units pass through each other.

**Other changes**

- `assets/battles/ch01.ron`: three enemies got an `id` (`road_brigand`,
  `road_archer`, `footbridge_brigand`) so the cinematic can name them. An
  id changes nothing in the battle.
- Tests: `the_storyboard_plays_shot_by_shot` in
  `crates/ui/tests/it/title_cinematic.rs` walks the nine shots, reads who
  is where from the scene, and takes one snapshot per shot. The 0819 loop
  test now expects the real shots; one content test expected Chapter 1 to
  name no enemies.
- `title-screen.md` (*As built*; the stand-in rule removed) and
  `assets/cinematics/README.md`.

**Deviations**

- **Not watched with sound.** Every shot was looked at as rendered frames
  (`cargo xtask frame-png`), in the glyph look and with the bought art,
  and the Harness test plays the whole loop. The web build compiles but
  was not watched. Whether the cuts feel right on the music is for Nick's
  sign-off.
- **Shot 1 can't pan "across the party"** at 3×: they stand in the map's
  bottom left corner, where the camera stops at the edges. It drifts from
  the fords down and left until it rests on them instead.
- **The first two blows of the road fight land at the end of the march
  shot**, so the close-up cuts in mid-fight.
- The Context paragraph about 0724 is out of date: 0724 landed first, and
  the snippets show its rewritten lines.

**the agent's starting rules** (Nick can veto): which units march where, who
fights whom, the HP each is left with, and the two box counts, all as
above.

**Follow-up tickets:** none.

**For Nick when he plays** (Pages build and the Windows download, sound
on, one whole loop):

1. Does each cut land where the music changes? The logo comes when the
   last chord has faded (1:57); it could instead cut on the chord itself
   (1:55.8), say if you'd like that.
2. Is one text box for 18 seconds in the second snippet too long? Two
   boxes are possible but each would be up 9 seconds.
3. Do the snippets give anything away? Does anything look off?
4. Now that you have seen it: should Options get a setting to turn the
   cinematic off? (Your answer goes in `title-screen.md`, *Open
   sub-questions*; a yes becomes a ticket.)
