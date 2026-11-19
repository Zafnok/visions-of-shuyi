---
id: "0086"
title: "Decide: timed presses in combat, and Steal the Turn"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0766"]
nick_input: decision
completed:
---

# 0086 — Decide: timed presses in combat, and Steal the Turn

## Context

On 2026-11-16 Nick's research into the storefront hook landed on a
principle: the slow half of a fight, the enemy phase in a Fire
Emblem-like, has to become the exciting part. Expedition 33 (about 8
million sold) and Undertale both made it so by giving the player
something to do on the enemy's turn. This ticket decides two systems
that do that. 1092–1094 build them, and the beast fights (0087,
1095–1098, 1280) lean on both.

Nick's words:

> "the part that has to be most compelling has to be what fills in the
> downtime... esp the enemy turn... as well as making battles as flashy
> as possible."

> On E33-style timing ("the hit roll stays as it is, and your press only
> changes what happens; prompts for big or killing blows; can be
> switched off"): "probably a good addition"

> "press timing for attacks, follow ups, and parries/dodges", with
> auto-resolve counting every press as "Good".

> "the parry should be inside the combat frame rather than overworld
> frame. And there should be some timing indicator."

> On a solo-made Fire Emblem-like with a parry (RuneCipher): "I don't
> think this disqualifies us from using the idea."

> On Steal the Turn (a perfect parry freezes the enemy phase and gives
> you one move): "this sounds more interesting". And on the grey freeze
> effect: "yea this sounds ok".

Nick also saw a rough animation on 2026-11-16, built only for him and
not in the repository:
1. A sleeping beast's phase.
2. The cut into the combat frame.
3. A ring closing on the defender.
4. A PARRY bar with "good" and "perfect" zones.
5. PERFECT, a white flash, the world draining to grey, and "STOLEN".
6. One unit's move.

He said it "works quite well".

**What exists today:**
- Hit and crit use FE true hit (`docs/design/stats-and-combat.md`, *RNG
  procedure*). Hit is two rolls averaged. Crit is one honest roll.
- The combat frame (the close-up battle scene) plays each attack.
- Phases are Player, Enemy and Other (`turn-structure.md`).
- The archer's Overwatch art already acts on the enemy phase
  (`combat-arts.md`).
- Rewind charges undo actions (`death-and-difficulty.md`). Bots play
  every battle, and the engine is deterministic: a press has to be
  recorded as part of the command.

This waits for the writer audition (0766), as Nick asked.

## Nick input

**Decision** (`ask-nick`): at most three questions per message, as
player situations with numbers, with options from real games plus
"describe your own". Mockups in the picture look; the rough animation's
staging can be shown as stills. The answers go into a new
`docs/design/timing.md`.

**Timed presses**

1. **What a press changes.** The hit roll always happens first, and a
   press never turns a miss into a hit.
   - A. Attack: a perfect press adds a bonus, such as +2 damage or a
     guaranteed follow-up link. Defence: a dodge (wide window) halves the
     damage, and a parry (narrow window) blocks it and does something
     extra. Like Expedition 33.
   - B. Defence only. Attacks never need a press, like Paper Mario's
     guard without its attack commands.
   - C. Describe your own.
2. **Which blows prompt?** An enemy phase can hold 20 attacks.
   - A. Every blow on your units.
   - B. Only big ones: a blow that would kill, one over a share of max
     HP, and every beast or boss blow. The rest auto-resolve as Good.
   - C. Describe your own.
3. **The options:** Timing On, Assisted (wider windows) and Off (every
   press counts as Good, as Nick said). Should Off be the default for
   Casual? Should the bots always play Good? Is there a lag-calibration
   step for the web build and TVs?
4. **The indicator:** a ring closing on the defender, a bar with good and
   perfect zones, or both, as in the animation. Which key is the press,
   for example the same Action as Confirm? Any key can be rebound, and
   the text names the player's own key.
   - Does the ring show on blows that will miss? If it shows only on
     hits, it gives the roll away. The starting rule (1092) is that it
     shows either way and a grade on a miss changes nothing.
   - If the press is Confirm, holding Confirm can no longer speed up a
     fight that has presses. Is that acceptable?
   - On the Playdate, A is also a chord button. The press is timed from
     the frame the button went down (1093).

**Steal the Turn**

5. **When it happens:**
   - A. A perfect parry, at most once per enemy (or beast) phase.
   - B. Only against bosses and beasts.
   - C. Describe your own.
6. **Who gets the stolen move?**
   - A. The unit that parried, like Expedition 33's counter or Sekiro's.
   - B. Any one of your units, like Bayonetta's Witch Time, where the
     world stops for you.
   - C. The parrier or an ally next to them.
   - D. Describe your own.
7. **What the stolen move is:**
   - A. A full action: move and act.
   - B. Act without moving.
   - C. Move only.
   - Also: can it use an art, a spell or an item, and can it target the
     unit that was just parried?
8. **Rewind and permadeath:** a missed press is part of the recorded
   action, so rewinding past it costs a charge like any undo. Confirm
   this, or choose another rule.

Also confirm the look: one inverted frame on the hit, the world drained
to grey with only the acting unit in color, and a "their turn, your
move" banner. Ask about a reduced-flash option.

## Scope

**In:**
- Running the questions and recording the answers, with every number
  *tunable*, in `docs/design/timing.md`, and the `docs/design/README.md`
  row.
- Bringing 1092–1094's text in line with the answers.

**Out (do not do):**
- Building anything (1092–1094).
- Beast-specific rules, such as parrying a claw and breaking its part
  (0087).

## Implementation steps

1. Read `stats-and-combat.md` (*RNG procedure*), `turn-structure.md`,
   `combat-arts.md` (Overwatch and the defensive arts),
   `death-and-difficulty.md` (rewind), `options.md` and the
   `keyboard-input` skill (how a key is named in text).
2. Run the questions with `ask-nick`.
3. Write `timing.md`, the README row and the follow-up changes, and move
   this ticket to `done/` in the same PR.

## Acceptance criteria

- [ ] `docs/design/timing.md` exists with Nick's words verbatim and the
      derived rules for presses, the options and Steal the Turn.
- [ ] 1092–1094 match the answers.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

- None (a design decision).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
