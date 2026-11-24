---
id: "0879"
title: "Decide and set the number the volumes start at on Options (now 97)"
type: tuning
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0879 — Decide and set the number the volumes start at on Options (now 97)

## Context

A new player who opens Options sees **Music volume 97, Sound volume 97,
Voice volume 97** (snapshots
`crates/ui/tests/it/snapshots/it__options__the_title_opens_options_and_cancel_closes_it.snap`
and `it__playdate__playdate_options.snap`). Before the writer trial (0766),
a test audit of the Chapter 1 web demo (0914, 0915) flagged that 97 may
read as a bug: an odd number just under the top, which nobody chose.

How it got there:

- 0805 started both volumes at "8 of 10" (later 80 of 100), marked
  *tunable*, and Nick did not choose that number either.
- PR #624 moved the engine pin past engine #318, which put the volumes on a
  loudness curve (ADR-0091 in the engine: each step is 0.6 dB, 100 is the
  sound as it is). The engine's `BUILT_IN_DEFAULT` (`crates/ui/src/audio/groups.rs`
  at the pin) is 97, the step nearest the old 80 % gain. So the game sounds
  exactly as loud as before but now *shows* 97. `docs/design/options.md`
  (*Volumes*) records this as the agent's note, not as Nick's answer.
- The slider moves 5 at a time (engine `VOLUME_STEP`), so from 97 the
  numbers go 100, or 92, 87, 82… and then 95, 90… once the player has been
  to the top.

The engine already lets a game choose each group's starting volume in its
audio manifest (`groups: [(id: "music", volume: 90)]`, shuyi_assets
`audio.rs` docs and `GroupDef::volume`), so any whole number Nick picks is
a data change in this game. It needs no engine work.

How loud each candidate is, against today's start (97, about −1.8 dB from
full):

| Start at | Gain | Compared with today |
| -------- | ---- | ------------------- |
| 100 | full (0 dB) | about 1.8 dB louder: hardly heard |
| 97 (today) | about −1.8 dB | the same |
| 95 | about −3 dB | about 1.2 dB quieter: hardly heard |
| 90 | about −6 dB | about 4 dB quieter: a little quieter |
| 80 | about −12 dB | about 10 dB quieter: roughly half as loud |

(The gains are 0.6 dB per step below 100, from the engine's
`crates/ui/src/audio/loudness.rs`. The session checks them against
`loudness()` before asking.)

The Ambience volume (1322, built by 1332) starts **10 under the
music's start** (Nick, 2026-11-23): whatever start Nick picks here, the
ambience's moves with it (87 while the music is at 97), and the question
shows it.

Related: `docs/design/options.md` (*Volumes*), 0805 (the Options menu),
0826 (voice volume), 0865 (the music dips under a voice: its `ducks`
entry is a separate number and stays), 0183 (which notes #624's 80 → 97
as the same loudness), 0766 (the writer trial that sends the demo).

This ticket does not block the writer trial (0766): at 97 the game sounds
as it always has, and the number only looks odd. The Act 1 playtest (1064) needs it, and it was added to
1064's `blocked_by` when this ticket was filed.

## Nick input

**Decision**, asked with the `ask-nick` skill before anything is built: what
number Music, Sound and Voice volume start at on a new player's Options
screen. Ask it with the question pop-up, in plain words and with the table
above reworded as "how loud the game starts". Options:

- **A. Keep 97.** It sounds the same as the game always has. The number
  just looks odd.
- **B. Start at 100 (the top).** Hardly louder than now. The slider then
  shows round numbers (95, 90…). Examples to confirm before asking, in
  the game or its wiki: Minecraft's sound sliders start at 100 %, and
  Celeste's Music and Sounds start at 10 of 10.
- **C. Start at 90.** A round number with headroom to turn up, and a
  little quieter than now (about 4 dB). The session finds and confirms one
  real game that starts a volume below the top before naming it, or offers
  this option without a comparison.
- **D. Start at 80**, the number `options.md` first wrote down. Noticeably
  quieter than now (about half as loud), with plenty of room to turn up.
- **E. Describe your own**, for example a different start for music than
  for sounds and voices (Nick picks each number).

If Nick wants the *number* 80 to keep today's loudness, that changes the
engine's curve (`engine-change`). Say that it is a separate engine ticket
and offer B–D for now. Don't build it here.

No check after merge. The result is verified by the tests and snapshots
below. How loud the start feels in play goes on the Act 1 playtest's
checklist (1064) through the Completion notes.

## Scope

**In:**
- Asking Nick (above) and recording his answer, with his words and the
  date, in `docs/design/options.md` (*Volumes*). Replace the agent's
  "starting at 80 (*tunable*)… the same starting loudness is **97**"
  paragraph with the decided number. Keep the sentence that an older save
  is read where it sounds the same.
- If the answer is not A: setting the starting volume of the `music`,
  `sfx` and `voice` groups to the decided number(s) in the game's audio
  manifest, in **both** `assets/audio/audio.ron` (public) and
  `assets-private/content/audio/audio.ron` (the private one laid over it,
  changed with the `content-change` skill), as a `groups:` list.
- Updating every test and snapshot that pins 97 as *the default* to the
  new number (not tests that set 97 on purpose).
- `Restore defaults` puts the volumes back to the new number. It reads
  the group defaults, so check it in a test rather than change it.

**Out (do not do):**
- Changing the engine's `BUILT_IN_DEFAULT`, loudness curve or
  `VOLUME_STEP` (engine work, `engine-change`; ask Nick first, per
  memory).
- Changing the Master or UI groups (they start at 100 in the engine and
  aren't rows of the game's Options), each cue's own `volume:` in the
  manifests, or the voice duck (`ducks: … to: 90`).
- Re-reading saves in a new way. A version 1 or 2 save at the old 80 is
  already read as "the group's default" (`OLD_DEFAULT_VOLUME` in
  `crates/ui/src/settings.rs`), so it follows the new default. Leave
  that as it is.
- Pushing the demo to itch.io (the Completion notes tell Nick to run the
  workflow; that is his call under 0766).

## Implementation steps

1. Run `cargo shuyi private-assets` in the worktree before the first build.
2. Ask Nick (Nick input above). Record the answer in
   `docs/design/options.md`, and update `docs/design/README.md`'s Options
   row if its decided-date note needs the new date.
3. If the answer is **A**: stop here. The ticket's change is the design
   doc alone. Mark the starting rule as decided and go to step 7.
4. Otherwise, add to both manifests, before `ducks:`:
   ```ron
   groups: [
       (id: "music", volume: N),
       (id: "sfx", volume: N),
       (id: "voice", volume: N),
   ],
   ```
   with each group's decided number. The private file goes through the
   private repository's PR and the pin (`content-change`), and the public
   one through this PR. Document the list in `assets/audio/README.md` (one
   short paragraph: what `groups` sets and that it is Nick's number from
   `options.md`).
5. Find every test that treats 97 as the default:
   `git grep -n -E "\b97\b" -- crates/ui crates/playdate crates/app`
   (known: `crates/ui/src/screen.rs` ~l.664-670,
   `crates/ui/src/screens/options/tests.rs` ~l.319-322 and 638,
   `crates/ui/tests/it/options.rs` ~l.315-343 and 913,
   `crates/ui/tests/it/voice.rs` ~l.215). Update the expected default to
   the new number, keep each test's meaning, and fix comments that say
   "97, where the old 80 was". Re-accept the two Options snapshots with
   `cargo insta` after checking that only the volume numbers and bars
   changed. Check whether the tests load the real manifest or a test one,
   and make sure at least one test reads the real one (step 6).
6. Add a test in `trpg-content` (beside its audio manifest tests) that
   loads the game's audio manifest and asserts `music`, `sfx` and `voice`
   start at the decided numbers. It must pass with the public files alone,
   and in CI with private content checked out, because the private
   manifest is laid over the public one.
7. Run the `run-gates` skill. Expected size: about 10 lines of data and
   docs, plus about 30 lines of test changes.
8. Completion notes: the decided number, and a line for Nick: "If the
   Chapter 1 demo is already on itch.io, run Actions → Demo → Run workflow
   before sending the writer packets (0766)." Add the starting loudness to
   the Act 1 playtest's checklist items (1064) as something to judge by
   playing.

## Acceptance criteria

- [ ] `docs/design/options.md` (*Volumes*) states the starting number(s)
      as Nick's decision, with his words and the date, and no longer
      calls 97 or 80 *tunable* or the agent's.
- [ ] If the number changed: `assets/audio/audio.ron` and the private
      `content/audio/audio.ron` each have a `groups:` list setting
      `music`, `sfx` and `voice` to it, and `assets-private.rev` points
      at the private commit that has it.
- [ ] The new `trpg-content` test asserts the three groups' starting
      volumes from the real manifest and passes.
- [ ] Snapshot `it__options__the_title_opens_options_and_cancel_closes_it`
      shows the decided number on Music, Sound and Voice volume, and
      `it__playdate__playdate_options` on its rows. Nothing else in
      either snapshot changes.
- [ ] A test shows that `Restore defaults` puts each volume back to the
      decided number. An existing one, updated, counts.
- [ ] `git grep -n -E "\b97\b" -- crates` has no test left that names
      97 as the default (each remaining 97 is a value set on purpose).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the `trpg-content` test on the manifest's group starting volumes,
  and the settings tests in `crates/ui/src/settings.rs` (a version 2 save
  at 80 still reads as the default) passing unchanged.
- Property: none (one constant per group).
- Snapshot / integration: the two Options snapshots; the Harness tests in
  `crates/ui/tests/it/options.rs` and `voice.rs` that check the gains
  heard at the defaults, updated to `loudness(N)`.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
