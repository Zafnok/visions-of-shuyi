---
id: "0865"
title: "The music dips a little under a voiced line and fades back after it"
type: feature
milestone: M7 Chapter 1 & game flow
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: ["engine#2343"]
nick_input: none
completed:
---

# 0865 — The music dips a little under a voiced line

## Context

A friend's playtest of the Act 1 build, passed on by Nick (2026-11-07):

> "During spoken dialogue (lines which have AI voices -- so it should
> ensure the voice exists and AI voice option is checked and voice volume
> is not 0) we should duck the music volume slightly and return it to
> full vol after a line ends -- prob a fade transition in volume"

`docs/design/audio.md` ~:39 left lowering the music under a voice to "a
tuning ticket if Nick asks" (from 0238). He has asked.

**This is the engine's job, and its ticket exists:** engine 2343,
"Ducking: a sound group lowered while a voice plays, as rules in the
audio manifest, stepped in whole milliseconds"
([ticket](https://github.com/Zafnok/shuyi-engine/blob/main/tickets/open/2343-ducking-a-group-lowered-while-a-voice-plays-rules-in-the-audio-manifest.md)).
It already does all three of Nick's conditions: it ducks only while the
device reports a voice clip playing (no clip, or voices off, plays
nothing), and "a voice the player turned down to 0 lowers nothing". The
dip and the return are straight-line fades (`attack_ms`, `release_ms`).
Once it is in, the game's part is one rule in `assets/audio/audio.ron`.

Today the game can't do it cleanly itself: the engine's `AudioRequest`
has no music-gain request, and `MusicState`'s own fades would overwrite a
gain the game set (engine `crates/ui/src/audio.rs`).

The engine's chain: 2511 → 2342 (sound groups, a volume per group) →
2343. They are "After MVP" in the engine; 2343 says "If Nick wants it
before the MVP, this ticket moves to the MVP with 2342". This ticket is
that want: the session that picks up the engine side (the
`engine-change` skill) moves them.

## Nick input

None. "Slightly" is *the agent's starting value*: about −6 dB (on
2342's loudness curve; 2343's own example, `to: 80`, is −12 dB, more than
"slightly"), `attack_ms: 150`, `release_ms: 600`; judged in the Act 1
playtest (1064's checklist).

## Scope

**In:**
- ~~The pin moved past engine 2343 (its first step, `engine-change` §7, or
  the open `engine-pin` PR that passes it).~~ (done by the pin move of 2026-11-10): engine
  #332, with sound groups (#318) and the groups in the manifest (#326).
- One rule in `assets/audio/audio.ron`'s `ducks`:
  `(when: "voice", lower: "music", to: <about −6 dB>, attack_ms: 150, release_ms: 600)`,
  `to` read off 2342's curve.
- A test with the game's Harness: a voiced line in a scene with music
  lowers the music's gain to `loudness(to)` while it plays and brings it
  back after `release_ms`; with voices off, or the voice volume at 0, or
  a line with no clip, the music's gain doesn't move.
- `audio.md`: the rule and Nick's request (2026-11-07).

**Out (do not do):**
- The engine work (2511, 2342, 2343): in the engine's repository.
- The Playdate (1048 voices it later; the rule is data, so it follows
  when the Playdate's backend reports a voice).
- Lowering sound effects; a player setting to turn ducking off.

## Implementation steps

1. ~~Check engine 2343 is done and the pin is past it.~~ (done by the pin move of 2026-11-10).
2. Add the rule to `audio.ron`.
3. The Harness tests above (the engine's `Harness::voice_length` sets how
   long the headless voice plays).
4. Docs.

## Acceptance criteria

- [ ] `audio.ron` has the rule, and the content checks load it.
- [ ] The Harness test: the music's gain is lowered while a voiced line
      plays and back after it ends.
- [ ] With voices off, voice volume 0, or no clip for the line, the
      music's gain never changes (three tests).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the Harness tests above.

## Completion notes

*(Filled in by the session that completes the ticket.)*
