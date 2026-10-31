---
id: "0819"
title: "Title screen plays the intro cinematic in time with the title music"
type: feature
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0036", "0226", "0227", "0811", "0817"]
nick_input: answer-first
completed: 2026-10-13
---

# 0819 — Title screen plays the intro cinematic with the music

## Context

Nick (ticket 0036): the title's intro cinematic "should match the length of
the title song, and loop when the song does (i.e. take a brief pause same
as the song currently does) and then it can show our logo whatever it might
be and pause there during the brief pause before looping the song/cinematic
again".

The parts exist by now: the cinematic player draws any moment `t` of a
cinematic file (0817); `ctx.music_clock` says how far into its track the
music is (0227); the title has its logo (0811) and the `Press any key or
button` prompt on every build (0226). This ticket joins them in
`crates/ui/src/screens/title.rs`.

How the cinematic and the menu share the screen was decided in 0036
(2026-10-03). The rules are in `docs/design/title-screen.md`, *Intro
cinematic*: **How the title behaves** (rules 1–7, Nick's) and **the agent's
starting rules**. Read them first; in short:

- The first press shows the logo and menu and starts the song (as today).
- 15 s on the title with no press: the menu fades out (about 1 s) and the
  cinematic shows, at whatever point the song has reached.
- A press during the cinematic: the skip sound, the song jumps to its
  silent ending (117 s), the logo and menu show. That press does nothing
  else.
- The logo shot (117 s to the loop) always has the menu on it. Left alone,
  the menu fades when the song loops and the cinematic plays from 0.
- A press while the menu is up restarts the 15 s; after a press in the
  silence the menu follows the 15 s rule instead of fading at the loop.
- Back from the game: logo, menu, song from 0, same rules.

The title song (`title`, `music/new_sunrise_v1.ogg`) is 133.7 s; its last
note ends at about 117 s and the rest is silence until it loops.

**Added 2026-10-12 (0811 is done):** the title now paints its logo with
`title::glyph::paint_logo` (a backdrop that changes each launch, drifting
sparks, the logo picture; ADR-0059) and the menu over it. The sparks only
move while `Ctx::idle_motion` is on (`app` sets it; tests leave it off
unless they test the motion). The menu's fade must also fade the dark
band behind `Press any key or button` and behind a notice.

## Nick input

**Answer first:** ticket 0036 (answered 2026-10-03).

## Scope

**In:**
- `TitleScreen` plays the cinematic named by a constant (`"title"`),
  with `t` taken from the music clock, so picture and song stay together
  and loop together.
- The menu, the prompt and key presses behave as 0036 decided: the idle
  timer, the menu's fade out and in, the skip press and its sound.
- "Jumping the song to the silence" without a seek (step 3).
- A fallback so the cinematic still runs when there is no music clock.
- Until 0820 lands, `assets/cinematics/title.ron` is a stand-in: one map
  pan over the Chapter 1 battle (a story file may not name a test battle,
  ADR-0057), then the logo from 117 s. Its `length` is `Music("title")`
  (ADR-0060).

**Out (do not do):**
- The real shots and their timing (0820).
- New shot kinds (0818, 1010).
- An Options setting to turn the cinematic off: Nick said "later"
  (`title-screen.md`, *Open sub-questions*). Don't add one.
- Picking the skip sound (0041). Use the cue `hit_sword` behind a constant
  `TITLE_SKIP_SOUND` until 0041 names another; if 0041 is done, use its
  cue.
- Changing how the music starts, fades or loops (ADR-0026).

## Implementation steps

1. `TitleScreen` owns a `CinematicPlayer` for `title` (built on first
   update, since it needs `Ctx`). `draw` calls `player.draw(ctx, t, brightness, buf)`
   and then draws the menu / prompt over it as 0036 decided.
2. **Where `t` comes from**, each update:
   - `ctx.music_clock` is `Some` and its cue is `TITLE_MUSIC`: `t =
     position`. Never smooth or predict it; a one-frame lag is fine.
   - Otherwise (the track is still loading, its file is missing, or
     another track is fading out): hold `t = 0` for up to 3 s after the
     title asked for its music, then run `t` on the frame clock (`dt`),
     wrapping at the cue's `length_ms`. If the music clock appears later,
     switch to it.
   - Before the first press (the prompt is showing, 0226) there is no
     music yet: the logo and the prompt as today, no cinematic.
3. **The states** (a small enum in `TitleScreen`; all timing from `dt` and
   the music clock, never a wall clock):
   - `Menu { idle: f32 }`: logo, menu and help line as today. `idle` grows
     by `dt` and is set to 0 by any press. At 15.0 s (a constant,
     *tunable*) → `FadingOut`. If the menu came from the logo shot or a
     skip and no press has happened since, it instead waits for the song
     to loop (`t` wraps), then → `FadingOut`.
   - `FadingOut { f }`: about 1 s. Draw the cinematic at `t` with
     `brightness = f` (0817) and the title's own cells dimmed by `1 - f`
     over it; then → `Cinematic`. A press → back to `Menu`.
   - `Cinematic`: only the cinematic, no menu, no help line. When `t`
     reaches the logo shot (the last shot's `at`) → `Menu` (the menu fades
     in over about 0.5 s; mark it "from the logo"). A press → **skip**.
   - **Skip:** play `TITLE_SKIP_SOUND`; stop the title music at once (not
     the usual fade: ADR-0026's `Stop`, or a very short fade if a click is
     heard); from then on `t` runs on `dt` from the logo shot's `at`
     (117 s) until the cue's `length_ms`, then ask for the title music
     again, which starts from 0, and follow the clock as in step 2.
     quad-snd can't seek (ADR-0037), and it doesn't need to: the song's
     ending is silence, so "jump to the silence" is "stop, wait out the
     silence, start again". → `Menu`, marked "from a skip".
   - The timer and the states don't advance while another screen is open
     over the title.
4. Coming back to the title (a screen above it popped): the existing
   `music_on` flag asks for the title music again; it restarts from 0, and
   the state is `Menu { idle: 0 }`.
5. Input: follow the `keyboard-input` skill. The press that ends the
   prompt and the press that skips the cinematic do nothing else
   (`title-screen.md`): they don't move the menu cursor or choose an item.
   Controller buttons count as presses. No new text names a key.
6. The cinematic's lead (0818): pass `save::latest_lead(ctx.storage)` to
   the player, and build the player again when the title is shown again
   (`stale`), since a new game or a load may have changed it.
7. The logo shot must be the same picture as the title's own logo (0817
   shares the drawing), so when the menu appears on it nothing jumps.
8. Update `docs/design/title-screen.md` only if a starting rule had to be
   added while building (mark it as the agent's, for Nick to veto).

## Acceptance criteria

- [x] Harness: after the first press the menu shows; with no further
      press, at music position 10 s the menu is still up, at 17 s the
      screen is the stand-in's pan with no menu, at 120 s it is the logo
      with the menu, and after `wait`ing past 133.7 s it is the pan from
      the start again (it looped with the song).
- [x] Harness with `music_load_delay(2.0)`: the picture holds at `t = 0`
      until the music starts, then follows it.
- [x] Harness with `without_music()`: after 3 s the cinematic runs anyway
      and still loops at the track's length.
- [x] Harness: each rule in *How the title behaves* and each of *the agent's
      starting rules* that this ticket builds has a test: a press at 14 s
      keeps the menu another 15 s; a press during the cinematic asks for
      the skip sound, stops the music, shows logo and menu, doesn't move
      the menu cursor, and the music is asked for again 16.7 s later; a
      press in the silence keeps the menu past the loop; the timer doesn't
      run under Load Game; returning from a battle shows the menu with the
      song from 0.
- [x] The existing title tests (menu choices, music requests, the prompt)
      still pass, changed only where 0036's rules require it.
- [x] `screens_cover_the_whole_buffer` still holds for the title.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the choice of `t` (music clock, hold, fallback, switch back).
- Snapshot / integration: the Harness tests above.

## Completion notes

**Done (2026-10-13).**

- `crates/ui/src/screens/title.rs`: the title has three states (`Phase`):
  the menu, the menu fading out over the cinematic, the cinematic alone.
  `TitleScreen::play` moves them each frame; `view` says what shows
  (`TitleView::cinematic`, `menu_opacity`) and `title/glyph.rs` paints it
  (ADR-0054): the cinematic at its brightness, the title's own pictures
  over it at the rest, the menu, notice and help line dimmed.
- `crates/ui/src/screens/title/song.rs` (`SongTime`): where the
  cinematic's time comes from: the music clock, the 3 s hold and the frame
  clock without it, and the silence counted out after a skip.
- `Timeline::last_start_ms` / `in_last_shot` (`cinematic.rs`): the logo
  shot is the cinematic's last shot.
- `assets/cinematics/title.ron`: the stand-in (a pan over the Chapter 1
  map at 3x from the party to the fort, the logo from 117 s).
- Tests: `title/song.rs` (the choice of `t`), `title/tests.rs` (the
  states, from the view), `title/glyph.rs` (the look of the fades),
  `tests/it/title_cinematic.rs` (the Harness acceptance tests).

**Deviations**

- **Step 3, "stop the music at once":** the skip asks for the usual
  `StopMusic`, so the song fades out over the game's half second
  (`music_fade_ms`) instead of cutting. An instant stop would be a new
  kind of audio request, and "changing how the music fades" is out of
  scope. If the player leaves the title and comes back inside that half
  second, the song plays on (ADR-0026 keeps a track that is asked for
  again while it fades) and the picture follows it.
- **Step 6 (the lead of the latest save):** done once 0818 merged (it
  landed while this PR was open): `TitleScreen::refresh` passes
  `save::latest_lead` to the player each time the title is shown.
- **Step 1:** the player is made by `refresh` (the first update, and
  every time the title is shown again), not only on the first update, and
  opening Options also marks the title stale, since a colour theme
  changes what the cinematic is painted in.
- **Step 3, `FadingOut`:** the title's pictures (backdrop, logo) fade by
  their opacity over the cinematic; the menu's box, which is made of
  cells, darkens to black and is gone when the fade ends. A cell can't be
  see-through by halves.
- `Harness::wait` runs at most about 33 s, so the tests wait in pieces.

**the agent's starting rules** (also in `docs/design/title-screen.md`; Nick
can veto):

- A skip fades the song out over half a second instead of cutting it.
  The logo and menu show at once.
- The logo always has the menu on it. If the 15 seconds run out while the
  logo is up, the menu stays until the song loops.
- A press while the menu is fading out brings the menu straight back and
  counts as a normal menu press (it moves the cursor or chooses).
- If the song hasn't started after 3 seconds (its file is slow or
  missing), the cinematic runs without it.
- The stand-in cinematic until 0820: one slow pan over the Chapter 1
  battlefield, then the logo.

**Follow-ups:** none new. 0820 (the real
shots; its tests of what shows at which second replace the stand-in's
line in `title_cinematic.rs`), 0041 (the skip sound: `TITLE_SKIP_SOUND`).

**For Nick, when playing:** leave the title alone for 15 seconds; press
any key during the cinematic; leave it again and watch it loop after the
logo. The fade of the menu box (it darkens, it doesn't dissolve) is the
part most worth a look.
