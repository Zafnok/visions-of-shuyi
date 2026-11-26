---
id: "0882"
title: "Volumes are a straight percentage of full: 20 plays at 20%"
type: tuning
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
blocks: ["1064"]
nick_input: none
completed:
---

# 0882 — Volumes are a straight percentage of full: 20 plays at 20%

## Context

On 2026-11-26 Nick reported that the music had stopped on the itch.io
playtest page, from the title on and in every chapter. It hadn't
stopped: his profile had **Music volume 20**. Since engine #318 (moved
in by PR #624) each step of a volume row is **0.6 dB** (the engine's
`shuyi_ui::audio::loudness`, `STEP_DB_TENTHS = 6`), so 20 is 48 dB under
full: **about 0.4% of full volume**, silent on any speaker. A new
profile, or a private window, played the music at full.

Nick, 2026-11-26, deciding the fix: "it shouldnt be .4% of full volume,
it should be more linear idk?? this seems insane its so much quieter. 1
should be 1% and 20 should be 20%".

So a volume row's number is now **that percentage of full volume**:
the gain is `n / 100`, a straight line, for every volume row (music,
sound, ambience, voice, and the engine's master volume if a row shows
it). 0 is silent and 100 is full, as before.

What exists today:

- The curve is the engine's: `shuyi_ui::audio::loudness` (`TABLE`, 101
  entries of a 4096 unit, `loudness(position) -> Gain`,
  `position_of_linear`), used by `audio/groups.rs` (each group's gain
  from its volume) and `audio/duck.rs` (the dip under a voice). The
  game's settings (`crates/ui/src/settings.rs`) store each group's
  number in `Volumes`, version 4.
- `docs/design/options.md`, *Volumes*, and `docs/design/audio.md`,
  *Ambience*, say "0.6 dB each" and "6 dB quieter".
- 0879 made every volume start at 100; 1322 and 1332 start the ambience
  "10 under" the music (Nick: "10 ticks under"), 90.

## Nick input

None: Nick decided the rule in the session that found this (above).
Whether music, beds and voices sit well together at the new numbers is
judged by playing: it goes on the Act 1 playtest's checklist (1064).

## Scope

**In:**
- A volume number `n` (0–100) plays at gain `n / 100` of full, for every
  sound group the volume rows set, on every target (desktop, web,
  Playdate).
- The change in the engine, by the `engine-change` skill (engine PR,
  then the pin move in this ticket's PR), **with Nick's OK before the
  engine ticket and PR are made** (his standing rule). The engine
  never names this game: if another engine game may want the dB curve,
  the engine offers both and this game picks the straight line; the
  session decides which and records it in the engine's ticket.
- Saved settings: **a saved number keeps its number** (a profile at
  music 20 plays at 20% after this, which is Nick's example). No
  conversion of version 3 or 4 saves, and no settings version bump
  unless the reading changes. A version 1 or 2 save's old linear
  percent (`OLD_DEFAULT_VOLUME`, `read_old_volumes`, `set_from_linear`)
  now reads as that same number.
- The design documents: `options.md` *Volumes* and `audio.md`
  *Ambience* say the numbers are percentages of full volume, not steps
  of 0.6 dB. Keep the ambience's start at **10 under the music's** (90),
  as Nick worded it, and say that on the straight line this is 90% of
  full (about 1 dB quieter, where the curve made it 6 dB): put a line on
  1064's checklist to listen to the beds under the music.
- The tests and snapshots that name a gain from a volume.

**Out (do not do):**
- The dip under a voiced line (`duck.rs`): its amount is its own (in dB
  or a factor), not a volume row. Keep how loud the dip is unless the
  engine change forces it, and then keep the same dip in dB.
- The starting numbers (100, ambience 90), the slider's step of 5 and
  the number box: unchanged (0879, `options.md`).
- Per-cue volumes in `assets/audio/audio.ron` (e.g. the
  cursor's 60%): unchanged.
- A different curve, a hybrid or "perceptual" mapping: Nick asked for a
  straight percentage.

## Implementation steps

1. Ask Nick (one line) whether to go ahead with the engine change; the
   `engine-change` skill then files the engine ticket and makes the
   engine PR.
2. In the engine, `shuyi_ui::audio::loudness`: the gain of a position is
   `position / 100` of `Gain::FULL` (4096 units, rounded), or a choice
   between that and the current table that a game sets, as the session
   decides (step 1's ticket says which). Update `position_of_linear` to
   the identity on the straight line, and the engine's tests and docs
   that state 0.6 dB a step.
3. Move the engine pin (`cargo xtask engine-pin`, or the `engine-change`
   skill's §7) and, if the engine made it a choice, pick the straight
   line in the game where `GameSettings` / the app's config is set.
4. `crates/ui/src/settings.rs`: check that a version 3 and 4 save's
   numbers are read as they are, and that the version 1–2 path gives the
   same number it was saved with. Fix the module and function docs that
   say "where it sounds the same".
5. Update `docs/design/options.md` *Volumes* and `docs/design/audio.md`
   *Ambience* (step size, the 10-under gap's loudness), and add the
   listening line to `tickets/open/1064-nick-plays-act-1-on-web-and-playdate.md`.
6. Update the tests and snapshots that read a gain; run `run-gates`.

Expected size: about 100–200 lines in the game (settings docs and tests,
design docs, the pin), and about as much in the engine.

## Acceptance criteria

- [ ] A group at volume 20 plays at a gain of 0.20 of full (±1 unit of
  4096), at 1 at 0.01, at 100 at full, at 0 silent: an engine unit test
  over every position 0–100 and a game test through `Ctx`/the audio
  queue for the music group.
- [ ] A saved version 4 profile with `volumes: { "music": 20 }` loads
  with music at 20 and plays it at 20% (a `settings.rs` test).
- [ ] A version 1–2 save's linear volume loads as the same number.
- [ ] `options.md` and `audio.md` say volumes are percentages of full;
  no design document says 0.6 dB a step.
- [ ] 1064's checklist has the line to listen to the beds under the
  music at the new numbers.
- [ ] The engine PR is merged and the pin points at it.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the engine's position → gain for all 101 positions; the game's
  settings reading (versions 1–4).
- Property: gain is non-decreasing in the position and equals
  `position / 100` within rounding.
- Snapshot / integration: any snapshot that prints a gain or a dB level
  (the audio player's, the harness's `gain(group)`), updated.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
