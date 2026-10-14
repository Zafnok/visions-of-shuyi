---
id: "0730"
title: Give Maud softer eyes
type: content
milestone: M6 Story & dialogue
model: opus-5.5
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0730 — Give Maud softer eyes

## Context

Nick, Chapter 1 playtest round 2 (0804, 2026-10-14): "Maud's eyes look too
scary like they are cat-like like too skinny, too focused, they stare into
my soul... maybe she needs different eyes".

Maud (`keeper`) has a **generator face**: Nick made it in Mega Tiles'
Character Generator EX after picking mockup C (ticket 0706;
`docs/design/look-and-feel.md`, *The Chapter 1 cast's faces*;
`docs/story/characters/keeper.md`). The art lives in the private
repository (ADR-0040): `assets-private/game/portraits/keeper/`, credited
in `assets/data/credits.ron`. The generator itself is in Nick's full copy
(`D:\tactical-rpg\assets-private\`), not in the game's checkout. The same
face may be her map sprite and combat picture: check
`docs/design/look-and-feel.md` and `credits.ron` for where else it is used.

Never commit the art or a picture made from it (renders go to Nick, not
into the repo); a screenshot of the running game may be committed.

## Nick input

**Decision**, with the `ask-nick` skill: show Maud's 8 expressions today
next to at least three versions with other eyes, at the game's size and
2× (in the dialogue screen, beside Rue and Hollis so the styles compare):

- **A.** Another eye shape from the generator (rounder, larger pupils);
  Nick re-makes her in the generator with the chosen eyes, since he made
  her (give him the exact parts to pick).
- **B.** Claude's small edit of today's eyes (wider pupils, a softer
  lid; the `ascii-art` skill allows small edits on bought portraits).
- **C.** The eyes of another bought face that fits her.
- **D.** "Describe your own."

Every expression must keep matching (8 faces). **Sign-off:** play the
Chapter 1 opening; does Maud still look like Maud, and not scary?

## Scope

**In:**
- The renders, Nick's choice and his words in `look-and-feel.md`.
- The new faces in the private repository (`assets-private/game/portraits/
  keeper/`, through `cargo xtask portrait-import` if they come from the
  generator), `cargo xtask private-assets --pin`, and `assets-private.rev`
  committed. Credits updated if the source changes.
- Her map sprite / combat picture too, if they show the eyes at a size
  where it matters (ask in the same question).

**Out (do not do):**
- Other characters' faces.
- Drawing a new face from scratch.

## Implementation steps

1. Fetch the private assets; render today's faces and the options.
2. Ask Nick; if he re-makes her in the generator, wait for his files.
3. Import, pin, check in a build (`cargo run -p trpg-app --features
   private-assets`), run the private-assets test.

## Acceptance criteria

- [ ] `look-and-feel.md` records the choice and Nick's words.
- [ ] The new faces are in the private repository and `assets-private.rev`
      points at them.
- [ ] `cargo test -p trpg-content --features private-assets --test it
      private_assets::` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The private-assets test above.

## Completion notes

*(Filled in by the session that completes the ticket.)*
