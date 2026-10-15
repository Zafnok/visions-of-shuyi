---
id: "1042"
title: "Playdate: one-bit portraits of Chapter 1's faces, made from the bought art"
type: content
milestone: Post–Chapter 1
model: opus-5.5
effort: medium
status: todo
blocked_by: ["1040", "0062"]
nick_input: sign-off
completed:
---

# 1042 — One-bit portraits

## Context

Nick kept **portraits** in dialogue on the Playdate as **one-bit
(dithered) versions of the bought faces** (`docs/design/playdate.md`,
Q5); 0062 decided the style and size (*One-bit pictures*). 1040 built
the converter. "Making them is an edit of the bought art: check
ADR-0040's license notes first" (the design doc): the bought licence
allows modification for the game (ADR-0032: "Assets can be modified
freely to fit the needs of your game"; redistributing the files or
derivatives as they are is not allowed). So the one-bit faces are made
and kept **in the private repository** (ADR-0040), like the colour
faces, and never committed here.

Portraits today: the leads' faces are in `assets/portraits/lead_f`,
`lead_m` (public); the cast's are bought, in
`assets-private/game/portraits/` (see `cargo xtask portrait-import`).

## Nick input

**Sign-off:** Nick looks at every Chapter 1 face in one-bit, at the
device's size, in the Simulator or on his Playdate (screenshots of the
running game may be shared; the files may not), and names any face that
needs another pass.

## Scope

**0062's answer** (`playdate.md`, *One-bit pictures*): the busts at
**2×** in today's dialogue layout, listener faded by a 50% checker;
style **A** by default and a Playdate-only **portrait style** setting in
Options with A, A+ lines, F+ lines and G (one converted set per style
from 1040). The setting is in scope here: an Options entry the Playdate
build shows and the PC doesn't, saved with the other options.

**In:**
- Convert every face (and expression) Chapter 1's dialogue uses with
  1040's tool and 0062's method; the list from `assets/dialogue/` and
  `cargo xtask lines` or the portrait sidecars.
- Hand touch-ups where the conversion loses the face (eyes, mouth,
  outline), following the `ascii-art` skill's rules for small edits on
  bought portraits. Touched-up files go in the private repository with
  the converter's settings recorded so a re-run doesn't undo them (e.g.
  touched-up files win over converted ones).
- Commit and push the private repository's change, then
  `cargo xtask private-assets --pin` and commit `assets-private.rev`
  here (CLAUDE.md, *Bought art*). Credits: the new paths go in the
  existing credits' `private` lists (`assets/data/credits.ron`, ADR-0051)
  so `--pin` accepts them.
- The Playdate bundle (1036) lists them.

**Out (do not do):**
- The dialogue screen's Playdate look (1043).
- New faces or expressions the PC doesn't have.
- Committing any one-bit face to this repository, or a picture made
  from them other than a screenshot of the running game.

**Carve:** none.

## Implementation steps

1. Read 0062's answers, ADR-0032, ADR-0040, ADR-0051, the `ascii-art`
   skill.
2. List the faces; convert; review each at 1× and 2×; touch up.
3. Private repository commit and push; pin; credits.
4. Show Nick (Simulator or device).

## Acceptance criteria

- [ ] Every face Chapter 1 shows has a one-bit version in the private
      repository; `assets-private.rev` is pinned to it.
- [ ] `cargo xtask private-assets --pin` passes (credits list the paths).
- [ ] Nothing derived from the bought art is committed here.
- [ ] Nick's sign-off comment is in the completion notes (or "no comment
      yet").
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None beyond the all-assets test (each sidecar names a one-bit file that
  exists, when the private assets are present).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
