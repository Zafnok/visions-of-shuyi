---
id: "0473"
title: Play the shop music on the shop screen
type: feature
milestone: M3 Battle UI
model: implementation
effort: low
status: todo
blocked_by: ["0028"]
nick_input: none
completed:
---

# 0473 — Play the shop music on the shop screen

## Context

Ticket 0409 built the shop screen (`crates/ui/src/screens/battle/shop.rs`,
`ShopScreen`, pushed over the battle from a unit's `Shop` action). Its
ticket also asked for the `shop` music cue, but the track is chosen by
0028 (`docs/design/audio.md`), which was still open when 0409 was
worked. This ticket does that part once 0028 is answered.

## Nick input

None. The track is 0028's decision. Verified by tests; how it sounds
goes on the Act 1 playtest's checklist (1064).

## Scope

**In:**
- If 0028 recorded a track: import it with `assets-src/audio/import.py`,
  add the `shop` cue to `assets/audio/audio.ron` with its credit, a row in
  `THIRD_PARTY_ASSETS.md` (and the license text if the source needs one).
- If 0028 recorded "reuse `<cue>`": add `shop` as that cue's file in
  `audio.ron` (or play that cue), no import.
- If 0028 recorded "no music here": close this ticket with that note and
  change nothing.
- The shop screen asks for `shop` when it opens; when it closes, the
  battle's own cue plays again (`BattleScreen`'s `music` field, as the
  rewind's undo of an ending does in `BattleScreen::rewound`).

**Out (do not do):**
- Choosing the track (0028).
- The Preparations shop tab (0443) and town shops (1007): they reuse the
  screen's lists; if they open `ShopScreen` itself they get the music
  with it, otherwise their own tickets play it.

## Implementation steps

1. Read 0028's row in `docs/design/audio.md` *Music cues*.
2. Import and register the cue as above (`assets/audio/README.md`).
   `crates/content/src/audio.rs`'s `every_design_cue_is_in_the_manifest`
   test then finds `shop` in the manifest.
3. In `BattleScreen::open` (`crates/ui/src/screens/battle/mod.rs`), on
   `Opened::Shop`, call `ctx.audio.play_music("shop")` before pushing the
   screen. In `uncovered`, on a `ShopVisit`, play `self.music` again if it
   is `Some`.
4. About 10 production lines, besides the data.

## Acceptance criteria

- [ ] Opening the shop screen asks for `shop`; closing it asks for the
      battle's cue again (a test reading `ctx.audio`'s pending requests,
      as `crates/ui/src/screens/battle/sound_tests.rs` does).
- [ ] `audio.ron`, `THIRD_PARTY_ASSETS.md` and the credits list the track
      (unless 0028 said "no music here" or reused a cue).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (ui): the two music requests above.

## Completion notes

*(Filled in by the session that completes the ticket.)*
