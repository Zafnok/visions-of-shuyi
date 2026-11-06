---
id: "1042"
title: "Playdate: one-bit portraits of Chapter 1's faces, made from the bought art"
type: content
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["1040", "1061", "0062"]
nick_input: none
completed: 2026-11-05
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

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> Nick looks at every Chapter 1 face in one-bit, at the
> device's size, in the Simulator or on his Playdate (screenshots of the
> running game may be shared; the files may not), and names any face that
> needs another pass.

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
- [ ] Starting rules and anything left to judge by playing are in the
      Completion notes (they go on 1064's checklist).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None beyond the all-assets test (each sidecar names a one-bit file that
  exists, when the private assets are present).

## Completion notes

**What was done:**

- **The faces.** Chapter 1's script (`ch01.dlg`) shows ten characters:
  both leads, the retainer, the sergeant, the poacher, the keeper, the
  heretic, the messenger, the rival and the red captain. It uses 26 of
  their bought pictures: neutral and sad for the leads, angry, neutral
  and sad for most of the cast. 1040's tool converted every one of them
  with 0062's method, in all four styles (`cargo xtask
  playdate-images`, into the git-ignored
  `assets-private/generated/playdate-images/`). Each one was looked at
  at 1× and 2× beside its colour original.
- **No hand touch-ups.** No conversion loses a face. In every style the
  outline, hair, eyes and clothes read. Style A loses some mouth
  detail and makes sad and neutral look alike for some faces (the
  keeper, the sergeant). Nick knew this when he chose A ("none of the
  options capture mouth expressions very well"). The three line styles
  keep the mouths. So the private repository has nothing new, and
  `assets-private.rev` and the credits stay as they were.
- **The Portrait style setting.** `ShuyiSettings::portrait_style`
  (Dots, Dots and lines, Flat, Bold, which are styles A, A+ lines, F+
  lines and G). Options shows it only on the Playdate
  (`PLAYDATE_ONLY`, beside 1045's `PC_ONLY`), and it is saved with the
  other options only when it isn't the default, so older saves don't
  change. The Playdate front end draws its pictures in that style and
  redraws the whole frame when the style changes.
- **The test:** every face a portrait sidecar names is a one-bit picture
  in the Playdate's bundle, twice its PNG's size, in all four styles.
  The plain build checks the leads' faces. `cargo test -p trpg-playdate
  --features private-assets` checks the bought ones too (passed).

**Deviations:**

- The one-bit faces are **not committed to the private repository**.
  1040 and 1061 made the Playdate's bundle convert the bought PNGs
  itself when it is built. A `.1bit` file committed beside a PNG counts
  as a touch-up and wins over the conversion. Committing all 26 would
  freeze them, so later tuning of the converter's thresholds would
  never reach them. Only touched-up faces would go there, and none was
  needed. "Every face has a one-bit version" holds in the bundle, which
  is what the new test checks.

**the agent's starting rules** (for the Act 1 playtest's checklist,
1064):

- The setting's names: **Portrait style**, with **Dots**, **Dots and
  lines**, **Flat** and **Bold** (0062 left the wording *tunable*).
- It sits after Cursor, with the other looks.
- The combat picture's fighters stay in style A whatever the setting
  (0062's starting rule; the setting changes only the portraits).

**Left to judge by playing (1064's checklist):** Nick looks at every
Chapter 1 face in one-bit at the device's size, in the Simulator or on
his Playdate, in each portrait style, and names any face that needs
another pass. Keep an eye on the keeper's and the sergeant's sad faces
in Dots. The dialogue screen's Playdate layout is 1043's, so until it
lands the faces show in the PC's layout, cut off.

**Follow-up tickets:** none.
