---
id: "0809"
title: "Play the victory sting, defeat sting and Game Over music"
type: feature
milestone: M7 Chapter 1 & game flow
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["0022", "0023", "0024", "0214", "0801", "0807"]
nick_input: sign-off
completed: 2026-10-12
---

# 0809 — Victory sting, defeat sting, Game Over music

## Context

Tickets 0022, 0023 and 0024 pick the music for three moments at the end of a
battle ([`docs/design/audio.md`](../../docs/design/audio.md), *Music cues*):

| Cue | Decided in | Plays |
| --- | ---------- | ----- |
| `victory` (a pool) | 0022 | Once, while the `VICTORY` banner is shown: one of 13 versions of Fupi's Win Jingle, picked at random each win |
| `defeat` | 0023 | From the `DEFEAT` banner on, through the Game Over screen: the whole of The Fallen (Jonathan Shaw, CC-BY 3.0, 77 s). No separate sting |
| (`game_over`) | 0024 | No track of its own (0023). 0024 decides whether `defeat` loops or plays once |

The banners exist already (0405, `crates/ui/src/screens/battle/banner.rs`,
`BannerKind::Outcome`). The Game Over screen comes from 0801. Battle music
comes from 0807, which today makes Game Over **stop** the music; this ticket
replaces that with `game_over`. Audio plumbing: 0212 (ADR-0026), `ui` emits
cues through `ctx.audio` (`crates/ui/src/audio.rs`: `play_music`,
`stop_music`), only `app` plays them (ADR-0004). Tracks are imported the way
0214 did it (`assets-src/audio/import.py`, ADR-0027).

0022's answer: `victory` is a **pool** like `skirmish` (`pools` in
`assets/audio/audio.ron`, 0807): 13 of the 17 `.wav` files in `winjingle.zip` (not Saw, Square, Banjo or
Harpsichord: Nick cut them)
(CC0, one credit entry), each a `looped: false` track. 0022 also reserved
`victory_grand` and `ceremony_intro`; nothing plays them yet, so do **not**
import them here.

0023's answer: the Game Over screen must **not** stop or replace the music;
`defeat` keeps playing there. Wherever this ticket says
`play_music("game_over")`, read: the screen emits nothing, and 0807's
`stop_music` is removed. `scene_loss` (Bittersweet) is reserved; do **not**
import it here.

If one of 0022–0024 recorded "no music here" or "reuse `<cue>`", follow that
answer for that moment and say so in the PR.

## Nick input

**Sign-off:** win and lose a battle (Quick Battle or Chapter 1) and say
whether the stings and the Game Over music fit and are timed right.

## Scope

**In:**
- Import the chosen files with `import.py` (loudness-matched like the
  others), add them to `assets/audio/audio.ron` (`looped: false` for the two
  stings, and for `game_over` if 0024 chose "once"), `assets/audio/licenses`
  if needed, `THIRD_PARTY_ASSETS.md`, and the credits data.
- On `BattleEnded`, when the outcome banner appears: `play_music("victory")`
  or `play_music("defeat")`. The battle track fades out as usual (0.5 s,
  `music_fade_ms`).
- The Game Over screen emits `play_music("game_over")` when shown, replacing
  0807's `stop_music`.
- *the agent's starting rules* (tunable; list them in the PR for Nick):
  - After the victory sting ends, nothing plays until the next scene's
    `@music` (0710) or the next screen's cue. The sting is not cut short by
    the banner closing; a new cue fades it out as usual.
  - Rewinding (0307) can't happen after `BattleEnded`, so no rewind rule is
    needed. `Retry chapter` from Game Over starts the battle's own track again
    (0807).

**Out (do not do):**
- Level-up or EXP sounds (Nick's own work).
- World map, capital, camp and shop music (0025–0028, built by 1007, 1003,
  0409).
- A results screen (0810).

## Implementation steps

1. Import: add the tracks (13 for the `victory` pool, see `audio.md`) to `assets-src/audio/import.py`'s list with
   the source URLs and versions recorded in `audio.md`, run it, commit the
   `.ogg` files and the updated manifest entries.
2. In the battle screen, where `BannerKind::Outcome` is created on
   `BattleEnded`, emit the matching music cue once.
3. In 0801's `GameOverScreen`, replace the `stop_music` from 0807 with
   `play_music("game_over")`.
4. Update `docs/design/audio.md` only if a starting rule above was changed
   by Nick at sign-off.

## Acceptance criteria

- [x] Harness: winning a battle emits `play_music("victory")` exactly once,
      when the `VICTORY` banner appears. (It asks for the version picked
      from the pool, e.g. `victory_pizzicato`.)
- [x] `victory` is a pool of the 13 Win Jingle versions; a test shows two
      different seeds can pick two different versions, the way the
      `skirmish` pool's test does.
- [x] Harness: losing (the lord falls) emits `play_music("defeat")` once,
      and the Game Over screen emits no music request, so `defeat` keeps
      playing (0023).
- [x] Harness: `Retry chapter` from Game Over emits the battle's own cue.
- [x] The content validator loads the new cues; `looped` matches the rules
      above; each has a credit.
- [x] `THIRD_PARTY_ASSETS.md` lists the new tracks.
- [ ] Nick signed off.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: manifest entries (looped flags, credits) via the existing audio
  manifest tests.
- Snapshot / integration: the three Harness flows above, asserting on
  `AudioRequest`s.

## Completion notes

**Done (2026-10-12).**

- Imported with `import.py`: 13 Win Jingle versions (`music/win_jingle_*.ogg`,
  3.8 s each, not 5.7 s as `audio.md` said) and The Fallen
  (`music/the_fallen.ogg`). Manifest: 13 `victory_*` cues (`looped: false`),
  the `victory` pool, `defeat` (looped), two credits. `THIRD_PARTY_ASSETS.md`
  has both rows. The Fallen is the first CC BY 3.0 work, so
  `assets/audio/licenses/CC-BY-3.0.txt` is added (ADR-0027). The pack's own
  terms ask for the composer's site in the credit: the author reads
  "Jonathan Shaw (www.jshaw.co.uk)".
- The battle screen asks for the outcome's music once, when the `VICTORY`
  or `DEFEAT` banner comes on screen. The Game Over screen asks for nothing
  and 0807's `stop_music` is gone, so `defeat` plays on from where it was.
- **0024 was still open.** Asked here (Nick: "Loop it"), recorded in
  `audio.md`, and 0024 is archived by this PR. Nick then added: "ensure the
  loop has a 10-15s pause between loops for defeat/game over, and
  transition b/w defeat and game over should not reset music playhead".
  The pause is silence at the end of the file, so the loop carries it.

**Deviations.**

- `content`'s test that every cue in `audio.md` is in the manifest was red
  on `main` since 0022/0023 added rows. It now passes, with a list of the
  three reserved cues (`victory_grand`, `ceremony_intro`, `scene_loss`)
  that this ticket was told not to import; the test fails if one of them
  is imported and left in the list.
- `Rewind` from Game Over (0822) used to start the battle's music the
  moment it was chosen. Now it starts once a rewind point is picked (see
  the rules below), and it is the same track, not a new pick.

**the agent's starting rules** (*tunable*, also in `audio.md`):

- The pause between two rounds of the defeat music is 13.8 s (the piece's
  own quiet ending, 1.8 s, and 12 s added).
- After the victory jingle (3.8 s) nothing plays until the next scene's
  music or the next screen's: the results screen is silent after it. In
  Chapter 1 the victory scene then starts `scene_sad`.
- On Game Over, choosing `Rewind` keeps the defeat music while you look
  through the rewind points. Backing out returns to Game Over with the
  music still going. Picking a point starts the battle's track again from
  its beginning: the same track, also in a skirmish.
- `Retry Battle` starts the battle's music as a new battle does (a
  skirmish picks a track again).

**For Nick's sign-off:** win and lose a Quick Battle. Listen for the jingle
at `VICTORY` (a different instrument on different wins), the defeat piece
starting at `DEFEAT` and carrying on unbroken into Game Over, and, if you
wait 77 s, the pause before it starts again.

No follow-up tickets.
