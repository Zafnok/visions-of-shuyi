---
id: "0743"
title: "Character sheets for Act 1's new cast and named bosses"
type: content
milestone: M6 Story & dialogue
model: fable-5.1
effort: high
status: todo
blocked_by: ["0736"]
nick_input: none
completed:
---

# 0743 — Character sheets for Act 1's new cast and named bosses

## Context

The story pipeline (ADR-0011, the `story-writing` skill) gives every
named character a sheet in `docs/story/characters/` before a script
uses them: want, need, flaw, a secret or pressure, an arc, voice notes
and three sample lines. Chapter 1's cast has sheets. Act 1's new people
(`docs/story/outline.md`; ids in `docs/story/names.md`) don't:

| Id | Name | Role |
| -- | ---- | ---- |
| `defector` | Joss Pellam | Young Hound who defects at Kell's Ford (Chapter 2, Swordsman) |
| `prizefighter` | Gil Parrow | "Champion of Six Fairs", a Yakuza-style oddball with an absurd code of honour (Chapter 4, Brawler) |
| `shieldbearer` | Hedda Ravn | Ragna's Brennish bodyguard, loaned to the lead (Chapter 5, Raider) |
| `envoy` | Ragna Holt | The Brennish envoy in Saltmere (Chapter 5, NPC) |
| `battlemage` | Oriel Mast | The crown battle-mage who gave the Candles the order (Chapter 6, Sorcerer, tier 2) |

Plus **the named bosses** the outline calls for: the Hound lieutenant
at Kell's Ford (Chapter 2), whoever holds Greywater Abbey (Chapter 3:
the prior burning records, or the Hound captain guarding him), and
Crane's champion in the Undercroft (Chapter 7); Dace and Crane have
sheets already. Name a boss only if the chapter needs one; a boss
sheet can be short (want, voice, three lines).

**Looks picked by 0070** (`docs/design/look-and-feel.md`, *Act 1's new cast*): Joss as the Thief hero (brown spiky hair, dark headband, vest; in combat sword and shield, dark Hound cloth); Gil slicked-back black hair, a scar over one eye, a goatee, bare-chested with gold fist wraps and a red sash in combat; Ragna dark-blonde hair in a bun, a traveller's coat; Oriel greying hair and stubble, a grey-and-gold court robe; Crane now has **long white hair and a full white beard** (his sheet is updated); Greywater's boss is **the prior** (a dark-robed priest), not the Hound captain; Crane's champion is **a woman** in blue armour with sword and shield. Hedda and the Hound lieutenant are 0076's: describe them loosely.

Every Act 1 script (0744–0750, 1072, 1073) waits for this ticket, and
all of them for 0736 (the craft research that governs the scripts).

## Nick input

None. Claude writes the sheets. A story decision goes to Nick with
`ask-nick` before the sheet that needs it: a new named character the
cast doesn't have, or anything that changes a beat in `beats.md` or the
outline. New names are placeholders Nick may rename
(`docs/story/names.md`).

## Scope

**In:**
- A sheet per person in the table, in the shape of the existing sheets
  (`docs/story/characters/sergeant.md` is a good model), with the
  personal arc across Act 1's chapters (the outline's *Personal arcs by
  chapter (Act 1)*), how they speak, what they want from the lead and
  the others, and the supports they could have (1073 picks the pairs).
- The named bosses' short sheets and their names in `names.md` and
  `assets/data/names.ron`.
- **Looks:** if 0070 has picked their faces, describe the look to match;
  otherwise describe it from the outline and tell 0070 (a line in its
  Context) what each sheet says they look like.
- Update `docs/story/README.md`'s list of sheets if it has one.

**Out (do not do):**
- Scripts (0744–0750), supports (1073), side quests (1072).
- Classes, stats and levels (the chapter tickets, 0848–0854).

## Implementation steps

1. Read the `story-writing` skill (with 0736's craft section),
   ADR-0011, `beats.md`, `bible.md`, the outline and the existing
   sheets.
2. Write the five main sheets, then the bosses'.
3. Check each against the canon order and the others' sheets (wants
   that collide make the personal stories).
4. Names in the registry and the names table; content gates.

## Acceptance criteria

- [ ] A sheet for each of the five, with want, need, flaw, pressure,
      arc, voice notes and three sample lines. The voice notes are the
      skill's voice card (0736: age and experience, temperature,
      openness, what they do when scared, care signature, a fault the
      others put up with, punctuation habit, words from their life, won't
      say); anyone who can die also gets a small plan for after and an
      object seen in their hands. Sample lines show different situations
      (under fire, at camp, lying, grieving) and are never copied into a
      script.
- [ ] Short sheets and registry entries for each named boss.
- [ ] `cargo xtask check-script` and the content gates pass (the names
      table).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Content gates only (the names table); no Rust code.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
