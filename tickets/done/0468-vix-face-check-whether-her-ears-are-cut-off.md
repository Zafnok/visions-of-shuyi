---
id: "0468"
title: "Vix's face: check whether the tip box cuts off her ears"
type: bug
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-08
---

# 0468 — Vix's face: are her ears cut off?

## Context

The guide's face (the Fox Miko from the bought Tiny Tales packs, 0455)
shows at 2× (128×128) in the tip and lesson boxes. Chapter 1 playtest
round 3 (0804, 2026-10-16):

> "it generally looks quite good but I am wondering if her ears are cut
> off or that's just how her portrait is"

The bought files are in the private repository (ADR-0040; on Nick's
machine `D:\tactical-rpg\assets-private\`). Never commit the files or a
picture cut from them; a screenshot of the running game is fine.

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

> A screenshot of the tip box before and after (if changed).

## Scope

**In:**
1. Compare the face the game draws with the original portrait file
   (`cargo xtask private-assets --library` for the bundle): is the top
   cut by the import's crop, by the box's size, or are the ears cut in
   the art itself?
2. If the game cuts them: fix the crop or the box so the whole head
   shows (in the private repository's `game/` for the art, then
   `cargo xtask private-assets --pin` and commit `assets-private.rev`).
3. If the art itself is cut: say so in the completion notes and to Nick,
   with the original's frame; change nothing.

**Out (do not do):** other faces; editing the art.

## Acceptance criteria

- [x] The cause is in the completion notes, with game screenshots.
- [x] If fixed: the whole head shows; the box still fits beside the side
      panel (0459's rule); `private_assets::` tests pass.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- `cargo test -p trpg-content --features private-assets --test it private_assets::`
  if the art changed.

## Completion notes

**Cause: the import's cut, not the art and not the box.** The bought
bust (80×80, `Heroes 2: Rebellious Souls`, Fox Miko) has her whole
head: the ear tips start at row 6. `cargo xtask portrait-import` keeps
the bottom 64 rows (16..80) for every bust, so it dropped rows 6..15,
the top of both ears; the tip box draws the 64×64 cut at 2× as it is.

**Fix: her ears are shortened in the art (Nick's call), not the cut.**
A first try moved the cut up 10 rows to show the whole ears; Nick
didn't like it ("her whole ears are so long and that's the problem")
and asked for the ears to be shortened to fit the frame instead. This
goes past the ticket's "Out: editing the art" at his request; it is a
small pixel edit of the bought bust (`ascii-art`'s allowed edits), not
redrawing. From several rendered options he picked "A2": on each of the
eight 80×80 busts, the ear on the right of the picture (it leans out
and read as longer) is pulled in towards her head, up to 6 px at the
tip and none at row 30; then both ears are squashed in height between
their tip and row 32 so the tips sit at rows 17 and 20, just inside the
usual cut (rows 16..80). The cut and the box are as before: still
64×64, the face and shoulders where they were, the box the same size
beside the side panel (0459's rule). The busts are in the private
repository (`db3e808`, its sidecar describes the edit) and
`assets-private.rev` points at it. That commit also brings 0476's voice
clips, pushed to the private main meanwhile (voice files only).

Screenshots of the game (bought art, allowed as game screenshots):
before `docs/img/0468-tip-before.png` (ears sliced flat), after
`docs/img/0468-tip-after.png` (shorter ears, whole).

- Other characters' busts may have the same cut; out of scope here
  ("other faces"), not checked.
- No code: the `--shift-y` import option of the first try was taken out.
- No gameplay rules decided. No follow-up tickets.
- For the Act 1 playtest (1064): the tip box before/after above.
