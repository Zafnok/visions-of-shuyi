---
id: "1069"
title: "Playdate: one-bit faces and combat pictures for Act 1's new cast"
type: content
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0070", "1042", "1041", "0076", "0858", "0861", "0862", "0866", "0852", "0853", "0854"]
nick_input: none
completed:
---

# 1069 — Playdate: one-bit faces and combat pictures for Act 1's new cast

## Context

1042 made one-bit versions of Chapter 1's faces and 1041 the one-bit
combat picture, by 0062's method (`docs/design/playdate.md`, *One-bit
pictures*: busts at 2×, the portrait style setting with its styles, the
simple combat picture). 0070 picks the faces and combat pictures of
Act 1's new cast (Joss, Gil, Hedda, Ragna, Oriel, Dace in person, Crane,
the bosses) from the bought art. Nick plays Act 1 on his Playdate in
1064, so they need one-bit versions too.

The chapter tickets (0848–0851) left their art to follow-ups: 0858
(Joss, Hendry), 0861 (the prior, Ned), 0862 (Gil, Dace), 0866 (Hedda,
Ragna), each waiting for Nick to build faces in the generator and buy
or download pictures. Crane and the champion come with Chapters 6–8
(0852–0854). Until those land there is nothing in the private
repository to convert (checked 2026-11-10 at `assets-private.rev`
`2df02b8f`), so they block this ticket.

Since 1042 the Playdate's bundle converts every PNG under
`assets-private/game/` itself when it is built, and 1042's test
(`crates/playdate/src/pictures.rs`, every face a portrait sidecar names)
already covers new faces. What is left here once the art lands: look at
each conversion in every style, touch up where it loses the face, extend
the test to the combat pictures (`battlers/`), and the screenshots.

## Nick input

None. No check after merge (Nick, 2026-10-21): screenshots of the
running game go in the PR (never the files or a sheet made from them,
ADR-0040), and faces that need another pass go on 1064's checklist.

## Scope

**In:**
- Convert every face, expression and combat picture 0070 picked (the
  list from `look-and-feel.md` and the scripts, `cargo xtask lines` or
  the portrait sidecars) with 1040's tool and 0062's method, one set per
  portrait style, as 1042 did.
- Hand touch-ups where the conversion loses the face, by the
  `ascii-art` skill's rules for small edits on bought portraits; touched
  files win over converted ones, as in 1042.
- Push the private repository's change, then `cargo shuyi
  private-assets --pin` and commit `assets-private.rev`; the new paths
  go in the credits' `private` lists (ADR-0051).
- The Playdate bundle lists them (1036, or 1068's per-chapter lists if
  it built them).

**Out (do not do):**
- The PC import of the same art (each chapter ticket, 0848–0854).
- Faces 0070 didn't pick; any new expression.
- Committing any one-bit face or a picture made from them other than a
  screenshot of the running game.

## Implementation steps

1. Read 0062's answer, 1042's and 1041's completion notes, 0070's
   picks.
2. If a chapter ticket hasn't imported a face yet (0848–0854), convert
   it from the private repository's `library/` path 0070 recorded, and
   name the file as that chapter ticket will.
3. Convert, touch up, push, pin.
4. Screenshots in the Simulator for the PR: each new face in a dialogue
   box and each new combat picture.

## Acceptance criteria

- [ ] Every face and combat picture 0070 picked has a one-bit version
      in each portrait style.
- [ ] `assets-private.rev` moved; credits list the new paths.
- [ ] Screenshots of the running game in the PR; no bought file
      committed.
- [ ] Starting rules and anything left to judge by playing are in the
      Completion notes (they go on 1064's checklist).
- [ ] All gates in the `run-gates` skill pass, and the CI `playdate` job.

## Tests required

- Unit: the Playdate bundle has a one-bit file for every face the
  scripts use (the test 1042 added, extended).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
