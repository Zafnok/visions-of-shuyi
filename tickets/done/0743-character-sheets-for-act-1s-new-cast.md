---
id: "0743"
title: "Character sheets for Act 1's new cast and named bosses"
type: content
milestone: M6 Story & dialogue
model: fable-5.1
effort: high
status: done
blocked_by: ["0736"]
nick_input: none
completed: 2026-10-28
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

- [x] A sheet for each of the five, with want, need, flaw, pressure,
      arc, voice notes and three sample lines. The voice notes are the
      skill's voice card (0736: age and experience, temperature,
      openness, what they do when scared, care signature, a fault the
      others put up with, punctuation habit, words from their life, won't
      say); anyone who can die also gets a small plan for after and an
      object seen in their hands. Sample lines show different situations
      (under fire, at camp, lying, grieving) and are never copied into a
      script.
- [x] Short sheets and registry entries for each named boss.
- [x] `cargo xtask check-script` and the content gates pass (the names
      table).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Content gates only (the names table); no Rust code.

## Completion notes

Done 2026-10-28 (Opus 5.5).

**Sheets** in `docs/story/characters/`, in the shape of `sergeant.md`:

- The five: `defector.md` (Joss), `prizefighter.md` (Gil),
  `shieldbearer.md` (Hedda), `envoy.md` (Ragna), `battlemage.md`
  (Oriel). Each has want, need, flaw, secret or pressure, an arc across
  Act 1's chapters, relationships with the whole cast, the 0736 voice
  card, four sample lines (under fire, at camp, lying, grieving), a
  *Planted for after* (a plan and an object in their hands), a portrait
  brief and the support partners 1073 can pick from.
- Short sheets for the named bosses: `hound_lieutenant.md` (Silas
  Hendry, Chapter 2), `greywater_prior.md` (Prior Edwin Sallow, Chapter
  3), `undercroft_warden.md` (Constance Hobb, Crane's champion, Chapter
  7); and `hound_sergeant.md` (Sergeant Ned Tolley, Chapter 3), a named
  Hound guarding the prior who isn't the boss. Each: want, a grievance a
  player could share, who shares his or her history (the 2–4 box
  exchange), voice, three lines, an object, a look.

**Registry:** the four new names and their short forms in `names.md` and
`assets/data/names.ron`, plus `defector.nickname` ("Needle", what Silas
and then Tamsin call Joss). `Ned` is added to `_typos.toml` like the
other invented names. The names test that counts two-word names
(`every_two_word_name_has_short_forms`) now expects 16 people and 13
surnames of their own, for the four new names.

**Looks:** 0070 merged while this PR was open, so the sheets' looks
were rewritten to its picks (`look-and-feel.md`, *Act 1's new cast*):
Joss, Gil, Oriel, Ragna, the prior and the champion. Hedda's and the
lieutenant's stay loose for 0076.

**Elsewhere:** `docs/story/README.md`'s row for the sheets; a one-line
note of the named boss in 0744, 0745, 0749, 0848, 0849 and 0853.

**Choices made inside the outline** (story detail, no beat changed; Nick
may veto any in the Act 1 playtest, 1064):

- **Chapter 3's boss is the prior** (0070, and Nick on this PR,
  2026-10-28: "I'm making the ch3 boss the prior"). When beaten he dies
  or surrenders ("can say dies or surrenders"): the sheet gives a line
  for each, and 0745/0849 pick. The Hound sergeant guarding him stays
  named ("the sergeant can still be named"): an old Veyne guardsman who
  served under Hollis, with an exchange with him.
- Joss sewed the false Brennish badges himself (his secret, told in his
  Chapter 5 testimony); Silas Hendry, his lieutenant, picked him and
  called him "Needle".
- Gil threw the seventh fair (Cairnford) for silver before the war; his
  code is penance for the cousin who died paying the debt. He joins by
  his own rule: a beaten man serves the one who beat him until a fair
  rematch.
- Hedda is Ragna's foster-sister, and openly raided the Thornmarch in
  the war. Ragna wrote most of the Ash Peace and has a son of eight.
- Oriel saw the prince's banner, sent to the king to hold the fire, and
  was told to burn. He commanded the Candles' line, Rue and Wren on it;
  he won't swear fire again (his Sorcerer class vow is force).
- Constance Hobb kept the Candles' barracks under Ossary and believes,
  because Crane told her, that death is a door and she'll see the burned
  ones again. Written sad, not creepy.
- The midpoint twist stays where the outline put it: no new clue was
  added (Oriel knows only the order and the missing body).

**Gameplay rules decided:** none (classes, stats, levels and whether
Ragna is on the Chapter 5 map stay with the chapter tickets).

**Follow-ups:** none new. 0070 picks the faces; 1073 picks the supports
from each sheet's candidates; the scripts (0744–0750) use the sheets.

Engine friction: none, no Rust code.
