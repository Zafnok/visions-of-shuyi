---
id: "0605"
title: "Play the EXP bar and level-up sounds"
type: feature
milestone: M5 Progression
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["0602", "0212"]
nick_input: setup
completed: 2026-11-27
---

# 0605 — Play the EXP bar and level-up sounds

## Context

Ticket 0602 built the EXP bar, the level-up page and the class-progress box
(`crates/ui/src/screens/battle/progress.rs`, driven from
`BattleScreen::update` in `crates/ui/src/screens/battle/mod.rs`) with no
sound. Nick is making the level-up and EXP sounds himself (recorded in
tickets 0022 and 0809: "Level-up or EXP sounds (Nick's own work)"). This
ticket plays them once he has supplied them. Ticket 0212 made the cue
system: screens call `ctx.audio.play_sound(cue)` (`crates/ui/src/audio.rs`),
`app` plays it, and every cue is listed in `assets/audio/audio.ron`
(ADR-0026).

The moments a sound can go with (see the 0602 snapshots in
`crates/ui/src/screens/battle/snapshots/*progress_tests*`):

1. The EXP bar filling (about 0.6 s).
2. The bar reaching 100 (a level up), just before the level-up page.
3. The level-up page opening (`LEVEL UP!`).
4. Each stat that grew appearing (one every 0.12 s).
5. `CLASS MASTERED!` / `Learned: <name>`.

## Nick input

**Setup:** Nick puts his sound files in the repo (or sends them) and says
which of the moments above each one is for; a moment without a file stays
silent. Blocked until then. Record his answer in `docs/design/audio.md`
(new cue rows; remove "level up" from the open sub-questions).

No check after merge (Nick, 2026-10-21: he checks only design and the
full Act 1 playtest, 1064). Verification is by tests and `cargo shuyi
frame-png` pictures in the PR; what is left to judge by playing goes
on 1064's checklist: too loud, too long, tiring after many level ups?

## Scope

**In:**
- The moments Nick gave a sound, and only those.
- Nick's files under `assets/audio/sfx/` with `audio.ron` entries. They're
  our own work (no `THIRD_PARTY_ASSETS.md` row); if any part isn't, check
  its license against ADR-0013 first.

**Out (do not do):**
- Picking or making sounds for moments Nick left silent. Combat sounds
  (0424), menu sounds (0425).
- Changing the pages' timings or layout (signed off in 0602).

## Implementation steps

1. Add Nick's files and their `audio.ron` entries (check the manifest test
   passes), and record the cues in `docs/design/audio.md`.
2. In `progress.rs`, give `Progress` a way to say which cues start this
   frame: e.g. `fn cues(&self, before: f32, after: f32) -> Vec<&'static str>`
   comparing the page clock before and after a `tick`, with one-shot cues on
   a page's first frame (page opened), per stat row revealed
   (`stats_shown()` rising) and when `exp_shown()` crosses a multiple of 100.
   A Confirm press that jumps to the end must not fire every skipped stat
   tick at once: play at most one per frame.
3. In `BattleScreen::update`, after `progress.tick` and `progress.confirm`,
   call `ctx.audio.play_sound` for each cue.
4. Harness tests (`crates/ui/src/screens/battle/progress_tests.rs` has the
   99-EXP level-up setup) using `Harness::audio_requests`.

## Acceptance criteria

- [x] A level up from 99 EXP requests exactly Nick's cues, in order,
      each once (a stat tick once per stat that grew).
- [x] An EXP gain without a level up requests only the bar's cue (if any).
- [x] Skipping a page with Confirm doesn't burst several cues in one frame.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the cue timeline in `progress.rs`.
- Snapshot / integration: harness tests of the audio requests.

## Completion notes
- **Nick's answer changed the setup.** He had no files and asked for
  candidates instead ("no I want you to find some"). A listening page of
  28 in-house candidates, five to six per moment, with a mock level up at
  the game's timings
  (https://claude.ai/artifact/JiRNwJiDg81CfNWq9vqVHZ,
  `assets-src/audio/levelup-audition.html`). He picked F6, H3, L6, S6
  and M1, one for each moment, so none stays silent. Recorded in
  `docs/design/audio.md` (cue rows, a note and his words in the appendix;
  "Level up is Nick's own work" left the open questions).
- **The page timings changed, at Nick's word** (the ticket's Out list
  said not to): "let L6 play thru before continuing the animation / sound
  unless player skips the animation and then it can skip the sound forward
  too". `ProgressTimings::fanfare` (1.1 s, where the fanfare's held note
  ends; its last 0.2 s is a tail 30 dB down) is the level-up page's wait
  before its first stat row, in place of 0.3 s. A promotion keeps 0.3 s.
- **The files:** `cargo xtask sfx` renders them (`crates/tools/src/sfx.rs`,
  the page's recipes ported): `exp_fill`, `exp_full`, `level_up`,
  `level_up_stat_1` … `_7` (S6 plays each grown stat a note higher, so
  one file per note) and `class_mastered`. Each page's set is now
  normalised on its own (`SETS`), so the five older files didn't change
  (one shared gain would have clipped the fanfare). The new set is 1.9 dB
  under the old, and its cues play at 50 where the others play at 40.
- **The cues:** `progress/sounds.rs`, `Progress::take_sounds` compares
  the pages' clock with the last frame's. The fill and the fanfare start
  with a handle (`start_sound`). A Confirm press, or Confirm held past
  their part of the page, stops them with a 60 ms fade. Stat rows that a
  press reveals at once play nothing. A frame that shows several rows
  (Confirm held) plays one tick, the newest row's. The bar's level-up
  sparkle still plays on a skip. Nothing sounds while a combat's playback
  still plays (the pages wait for it). A long sound that played to its
  end isn't stopped: its handle is forgotten and goes with the battle
  screen. Three tests that list a battle's exact audio now expect the
  bar's sound (`rewind_tests`, `game.rs`).
- **Engine friction:** estimate 250 production lines (0213's renderer
  recipes, about 25 lines a sound, plus a cue timeline like the combat
  playback's `sounds`), written after the code, not before as ADR-0092
  asks; measured 281 (1.1×).
- **Manifest:** both `assets/audio/audio.ron` and the private
  `content/audio/audio.ron` (the private pull request comes first).
- **The agent's starting rules** (the design docs were silent): a
  promotion page plays none of these sounds; `Learned:` alone plays the
  mastery sound (Nick's moment 5 joins them); a reclass or "To stock" box
  plays nothing; a skip still plays the bar's sparkle; held Confirm cuts
  the fill and fanfare too; the fade is 60 ms.
- **For the Act 1 playtest (1064):** too loud (volume 50, starting
  value), too long (the 1.1 s wait before the stats), tiring after many
  level ups?
