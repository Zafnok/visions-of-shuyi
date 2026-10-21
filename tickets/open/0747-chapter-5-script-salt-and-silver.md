---
id: "0747"
title: "Chapter 5 script: Salt and Silver, beat sheet and scenes"
type: content
milestone: M6 Story & dialogue
model: fable-5.1
effort: high
status: todo
blocked_by: ["0736", "0743", "0746"]
nick_input: none
completed:
---

# 0747 — Chapter 5 script: Salt and Silver

## Context

Act 1, Chapter 5 (world map), from `docs/story/outline.md`:

- **Story battle:** "The Envoy's House": holding a harbour-front house at night against Hounds from the docks and the rooftops, for a number of turns.
- **Turn:** The envoy gets Joss's testimony, but the League expels everyone involved; Ragna: both armies are massing at the Ashfields. The night before the battle, Hollis tells the lead everything (the purse, the list, the stamped pass).
- **Recruits:** Hedda Ravn (`shieldbearer`, Raider), loaned by Ragna "to make sure you get there".
- **Arcs advanced:** Aske (her secret), Hollis (confession), the lead and Hollis (mending).
- **New in this chapter:** nothing new.

The story pipeline (ADR-0011, the `story-writing` skill) goes beat sheet
→ script, and `docs/story/ledger.md` records what each chapter changed,
so the chapters are written in order: this ticket waits for Chapter
4's script (0746). It also waits for 0736 (the craft research,
which governs every Act 1 script: a rewrite starts on a blank page, the
length budget) and 0743 (the new cast's character sheets).

## Nick input

None. Claude writes all scripts; only story decisions go to Nick (a
change to a beat in `beats.md` or the outline, a new named character the
cast doesn't have), asked with `ask-nick` before writing the scene that
needs it. Nick reads the chapter in the Act 1 playtest (1064).

## Scope

**In:**
- **The beat sheet** `docs/story/chapters/ch05.md`, in
  `ch01.md`'s shape: cast on screen (ids from `docs/story/names.md`),
  each scene with its place, who speaks, what each speaker wants in it,
  what changes; the battle's lines (the boss, the new units' Classic
  death and Casual retreat lines, talk conversations).
- **The script** `assets/dialogue/ch05.dlg`: the chapter's intro
  and pre-battle scenes, the battle's scenes, the victory scenes, by the
  rules in `assets/dialogue/README.md` (role ids, name tokens, `{lead}`,
  no line that depends on the lead's gender).
- **Fixed skirmishes:** *The Salt Road* (guards the path) and *Smugglers' Cove* (optional, with loot). Write their short scenes here (1071 builds the battles and wires the scenes).
- **Town:** Saltmere (no script needed unless the beat sheet puts a scene there).
- **Side quests:** *Letters Home* (Aske's) and *The Captain's Oath* (Hollis's; 1072 writes both).
- New names in `docs/story/names.md` first, then `assets/data/names.ron`.
- The critique pass (`story-writing` skill), then `docs/story/ledger.md`.
- **Either order with the battle ticket (0851):** the beat sheet
  names each scene's place and mood; the `place` directives, their maps
  and the `@music` lines are 0851's, which comes after this ticket
  (the validator checks that a place exists). If 0071 has picked this
  chapter's scene music, add the `@music` lines here and tell 0851.

**Out (do not do):**
- The battle file, map, units and triggers (0851).
- Support conversations and camp events (1073); side quests' scenes
  (1072).
- Changing a story beat without Nick.

## Implementation steps

1. Read the `story-writing` skill (with 0736's craft section), ADR-0011,
   `beats.md`, `bible.md`, the outline, the ledger, every character
   sheet on screen, Chapter 4's beat sheet and script.
2. Write the beat sheet; check it against the canon order.
3. Write the script on a blank page (0736's rule), within the length
   budget.
4. Critique pass; fix; update the ledger.
5. `cargo xtask check-script` and the content gates.

## Acceptance criteria

- [ ] `docs/story/chapters/ch05.md` and `assets/dialogue/ch05.dlg` exist and
      cover every beat above.
- [ ] The critique pass checklist is ticked in the PR body.
- [ ] The ledger records what changed for each character.
- [ ] `cargo xtask check-script` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The dialogue validator (content gates); no Rust code.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
