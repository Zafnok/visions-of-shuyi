---
id: "0087"
title: "Decide: legendary beasts, their hunts, their parts and forging"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0766"]
nick_input: decision
completed:
---

# 0087 — Decide: legendary beasts, their hunts, their parts and forging

## Context

This is the ticket that ties the 2026-11-16 hook chain together. Nick's
storefront-hook research ran five rounds and about a hundred agents, and
ended on his own idea:

> "rather than making it like one per act it can be more like legendary
> pokemon, where they're hidden in a dungeon, and the reward for making
> it to the end, and then its an epic battle. kinda combines raid boss +
> this idea + mystery dungeon. And probably the enemy army is also
> trying to track these mythical beasts or smth. Granted if our story
> doesnt revolve around dragons or there's no room for it in the arcs or
> beats I dont wanna totally rewrite it but if there is room for
> something like this, maybe adjusted to something else then I think its
> ok"

The concept as Nick approved it on 2026-11-16 ("this all sounds pretty
good"):
- A rumor in town puts a "?" on the world map. Skirmishes nearby turn up
  tracks, and after enough tracks the dungeon entrance appears.
- A rival enemy column marches to the same place. It moves only when you
  finish a story battle, so random fights never cost you the beast.
- A party of 3–6 goes down through dark floors (0088). The beast waits
  on the last floor.
- **The beast is a third army, hostile to everyone, with its own Beast
  Phase.** It covers several tiles, and each body tile is a part (head,
  ice claw, tail). Each of its attacks belongs to a part, and breaking a
  part removes that attack.
- You aim at a part like any enemy. The forecast adds a part bar, for
  example: "Claw 14/20 → 5/20 · If broken: Claw Sweep gone · Drops Rime
  Shard". Nick: "sounds good".
- The parry happens in the combat frame, and a perfect one steals the
  turn (0086).
- A broken part falls as terrain (an ice claw's shards freeze the river)
  and drops a material. A unit can spend an action to carve it instead
  of using the terrain. A part broken during a stolen move gives the
  rare material.
- If a fight runs too long, the beast retreats deeper rather than wiping
  the party.
- Back in town, the blacksmith forges the material into a weapon that
  *does* something new, never stronger than the chapter allows.

What Nick said on crafting and balance:

> "we shouldnt go overboard with [traits] or else we turn from SRPG to
> looter"

> "I do wanna be aware of the reward vs balancing. if we require a minmax
> build the game becomes grindy which stalls the SRPG flow. If we don't
> require a minmax build then the game gets bloated numbers if a player
> does grind. So we need gated progression gates like you cannot get
> tier 3 weps super early or smth. I think monhun gates this behind
> hunter rank we can prob gate based on act/chapter like we're doing w
> steel or plan to"

> "yes we would need a grinder bot"

> On the six questions below: "these could be in a story ticket tying
> the change together".

**What exists:**
- The Other phase and `Faction::is_hostile_to` (`turn-structure.md`,
  ticket 0302).
- Elemental classes with Weak, Resist and Absorb (`magic.md`).
- Tiers by chapter: Iron from Chapter 2, Steel from Chapter 5
  (`weapons-and-items.md`, 0044, 0073, 0074).
- The blacksmith repairs, and "materials and forging" is deferred
  (`weapons-and-items.md`).
- The playtest bots and autobalance (`playtest-bots.md`).
- Acts lock behind you on the world map (`world-structure.md`).

**The story.** A research pass found room in the existing bible with one
new line and no beats moved. Its framings are in the private
repository's story documents, because this repository is public. 0772
writes what Nick picks here into the story documents, with no dialogue.

This waits for the writer audition (0766), as Nick asked. Element
reactions (0085), timing (0086) and delves (0088) are decided next to it,
in any order. This ticket names their answers where they meet.

## Nick input

**Decision** (`ask-nick`): at most three questions per message, as
player situations with numbers, with options from real games plus
"describe your own". The story options are shown from the private
`story/` documents, never pasted into this repository.

**The six questions Nick asked for:**

1. **What are the beasts in the story?**
   - A. Remnants of the world's dead gods. This fits a magic rule the
     private bible already has, and needs one new bible line. Shadow of
     the Colossus did the same with fragments of a sealed god.
   - B. A, plus a reason the main villain needs them, which gives the
     rival column real stakes. A writer checks it against the midpoint.
   - C. Plain beasts with no lore, like Monster Hunter's monsters.
   - D. Describe your own.
2. **How do you aim at a part?**
   - A. Each body tile is a part, as with Three Houses' demonic beasts
     (recommended).
   - B. You pick from a part list in the forecast, as in Monster Hunter
     Stories 2.
   - C. The side you attack from decides the part, like flanking.
   - D. Describe your own.
3. **If the rival column reaches a beast first:**
   - A. It's bound, and you meet it chained in a later story battle.
     Break the chains and it attacks everyone.
   - B. It flees to another region, like Pokémon's roaming beasts.
   - C. It's gone for good, as in Gold/Silver.
   - D. Describe your own.
   - Also: what happens to a beast you never reached once its act locks
     behind you?
4. **Can forging raise a weapon's numbers?**
   - A. Not at all. The trait is the whole reward.
   - B. A tiny bit (Mt +1 or Hit +5), never past the chapter's tier.
   - C. Fire Emblem-sized, with caps, as in Path of Radiance.
   - D. Describe your own.
5. **How many traits can one weapon hold?**
   - A. One.
   - B. One, rising to two at the third tier.
   - C. Three, as in Atelier.
   - D. Describe your own.
6. **Can you farm beasts?**
   - A. Rare parts once only. Rematches give common parts, like Echoes'
     repeatable boss.
   - B. Free but capped: extra parts buy more *kinds* of weapon, never
     bigger numbers.
   - C. Both.
   - D. Describe your own.

**The rest of the system** (asked in later rounds, with the same
format):

7. **The Beast Phase:** where it falls in the order (after Enemy? after
   Other?), whether the rival column fights the beast too, and whether
   the beast shows only its next attacking part glowing. The glow shows
   only the beast's next move, not the whole enemy phase, which Nick
   rejected.
8. **Parts:** whether damage to a part also comes off the beast's total
   HP, whether a perfect parry chips the attacking part, whether the part
   you hit strikes back, and the turn limit before it retreats deeper.
9. **Carve or use:** a choice for each broken part, or both
   automatically, like Monster Hunter's end-of-hunt rewards.
10. **The rare carve:** only from a part broken during a stolen move, or
    from any break.
11. **The gates:** the third tier's chapter. Whether a beast's part sets
    the *kind* of weapon while the chapter sets its strength. The trait
    list, about 10–12 traits, each one a thing the weapon does (for
    example, "a hit leaves frost on the target's tile").
12. **The grinder bot's margin:** how much easier a fully farmed and
    crafted army may find a story battle than a shop-only army. For
    example, at most 10 points more wins for the Casual bot, or at most
    one turn faster.
13. **The world-map marks:** the random skirmish already uses a "?" on a
    grey plate (`look-and-feel.md`, *The world map*), so the hunt's mark,
    its tracks and the rival column need marks of their own. Ask with a
    mockup in the picture look.
14. **How many beasts,** where (one per region?), and the party size (a
    fixed 4, or the player's pick of 3–6).

## Scope

**In:**
- Running the questions and recording the answers, with every number
  *tunable*:
  - beasts, hunts, the Beast Phase and parts in a new
    `docs/design/beasts.md`;
  - forging and the gates in `weapons-and-items.md`, *Forging*;
  - the grinder bot in `playtest-bots.md`, *The grinder bot*;
  - the story answer in the private repository's
    `story/design/beasts.md` (the `content-change` skill: a pull request
    there, and the pin moved here). Or leave the story part to 0772,
    named in the completion notes.
- The `docs/design/README.md` rows. Bring 0772, 1095–1099 and 1275–1281
  in line with the answers.

**Out (do not do):**
- Building anything.
- Dungeon floors and light (0088).
- Element reactions (0085).
- Writing dialogue or bible prose (0772, after the writer is hired).

## Implementation steps

1. Read `turn-structure.md`, `magic.md`, `weapons-and-items.md`,
   `progression.md`, `playtest-bots.md`, `world-structure.md`, and the
   private `story/` bible, outline and beats.
2. Run the questions with `ask-nick` in rounds. Use mockups in the
   picture look (the bought Tiny Tales "epic monsters" battlers are in
   the private library; never commit a picture pasted from them).
3. Write the docs and follow-up changes, and move this ticket to `done/`
   in the same PR.

## Acceptance criteria

- [ ] `docs/design/beasts.md` exists, `weapons-and-items.md` has
      *Forging* and `playtest-bots.md` has *The grinder bot*, each with
      Nick's words verbatim and the derived rules.
- [ ] No story detail beyond what the public docs already say is in this
      repository. The story answer is in the private repository or handed
      to 0772.
- [ ] 0772, 1095–1099 and 1275–1281 match the answers.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

- None (a design decision).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
