---
id: "0085"
title: "Decide: element states on tiles and how spells react with them"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0766"]
nick_input: decision
completed:
---

# 0085 — Decide: element states on tiles and how spells react with them

## Context

On 2026-11-16 Nick ran five rounds of research on the game's storefront
hook. Several directions came out of it. Nick agreed to this one as a
foundation the rest stands on: element states on tiles, with spells
reacting to a tile's current state. The chain it starts is 0085–0088,
0772, 1091–1099 and 1275–1281. This ticket decides the element rules.
1091 builds them.

Nick's words:

> "I rather have innate elements, or interaction between casted spells
> and the terrain's current affected element"

> "elemental combos [like Genshin], or chain combos from DmC [Devil May
> Cry], stuff like that... idk how it can work in an SRPG tho"

> On Triangle Strategy's terrain reactions (fire melts ice into puddles,
> ice puts out fire, lightning spreads on water): "yea we should have
> this I guess"

> On powder kegs whose fuse counts the enemy's actions: "its ok but we
> shouldnt have on every map just some"

**What is decided today** (`docs/design/magic.md`):
- *Elements and affinities:* Fire, Ice and None. There's no triangle.
  Lightning was left out because it wouldn't read in ASCII. The picture
  look is now the main look, so that reason is weaker.
- *Terrain magic:* Fire on `forest` makes `burning` for one round, then
  `burnt`. Ice on `water` or `sea` makes `ice` for good. Casting at a
  unit never changes terrain. "Other interactions… **don't exist yet**.
  Adding one is a design change."
- *Push:* a unit pushed into `burning` takes 5 collision damage.

Elemental classes exist (Fire Elemental, Frost Elemental), with Weak,
Resist and Absorb.

Dialogue writing is paused (0766). Battle notes are chapter data, and new
ones wait for the writer. Nothing here needs new dialogue.

This waits for the writer audition (0766), as Nick asked on 2026-11-16:
"This is all work to be done after writer audition".

## Nick input

**Decision** (`ask-nick`): at most three questions per message, each
option shown as a player situation with numbers, and with options from
real games plus "describe your own". Check every game claim before
showing it. Several rounds are likely. The answers go into
`docs/design/magic.md` under a new section, *Element states on tiles*.

**1. What can a tile hold?**
- A. **Three states:** wet, burning, frozen. Rivers and sea always count
  as wet. This is a small, readable set, like Octopath's weaknesses.
- B. **Five:** add smoke (blocks ranged attacks) and mud (costs 1 more to
  move). This is closer to Divinity: Original Sin 2's surfaces.
- C. Describe your own.

**2. Which reactions are there?** Each option is shown with a Chapter 6
example, and the rows say what changes from today.
- Fire on frozen: melts it back to water. A rider standing on it is
  pushed off by the push rule (5 damage).
- Ice on wet: freezes the tile, and the unit on it is rooted (it can't
  move next phase).
- Fire on wet: makes steam, which blocks attacks from range for one round.
- Ice on burning: puts the fire out a round early.

Options:
- A. Exactly these four.
- B. These four plus lightning on wet, which spreads to every connected
  wet tile, as in Triangle Strategy. This would undo the old "no
  lightning" call.
- C. Describe your own.

**3. Does casting at a unit react with the tile under it?** Today only a
cast on an empty tile changes terrain.
- A. Yes. An elemental hit reacts with the tile under the target, like
  Genshin's "the second element triggers the first".
- B. Only combat arts with an element do, and spells stay as today.
- C. Describe your own.

**4. Do units carry an innate element?**
- A. Yes, one per class or character. Its *arts* strike with that
  element, and plain attacks stay neutral so the map doesn't churn every
  turn.
- B. No. Only spells and forged weapons (0087) carry elements.
- C. Describe your own.

**5. Does the forecast show the reaction before you commit?**
- A. Yes, always ("Melts ice · rider pushed · 5 dmg"), like Into the
  Breach. Only hit and crit stay as odds.
- B. Only once the player has seen that reaction once.

**6. Powder kegs on some maps** (Nick: "just some"):
- A. Yes. A keg's blast is a cross that stops at walls, breaks fences,
  burns forest, cracks ice, and sets off other kegs, Bomberman-style.
  Its fuse counts enemy actions, so it goes off during their turn. A
  unit next to a keg can kick it, and on ice it slides the whole lane.
  2–4 kegs on a map at most.
- B. Not now. Park it as a later map feature.
- C. Describe your own.

Also ask whether the Act 1 maps that are already built keep today's
rules, with the new reactions starting from a named chapter.

## Scope

**In:**
- Running the questions with Nick and recording his answers, with every
  number marked *tunable*, in `docs/design/magic.md`, *Element states on
  tiles*.
- Updating 1091's text if an answer changes what it builds (for example,
  kegs become a ticket of their own).
- The `docs/design/README.md` row.

**Out (do not do):**
- Building anything (1091).
- Dungeon light as an element (0088 decides that).
- Forged weapons' element traits (0087).

## Implementation steps

1. Read `docs/design/magic.md` (*Elements and affinities*, *Terrain
   magic*), `terrain.md` and `turn-structure.md` (phases) so every option
   is shown with real numbers.
2. Run the questions with `ask-nick`, using mockups in the picture look.
   Nick, 2026-11-16: "the game has mostly moved past glyphs beyond
   legacy compat".
3. Write the section, the README row and the follow-up changes, and move
   this ticket to `done/` in the same PR.

## Acceptance criteria

- [ ] `docs/design/magic.md` has *Element states on tiles* with Nick's
      words verbatim and the derived rules.
- [ ] 1091's text matches the answers.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

- None (a design decision).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
