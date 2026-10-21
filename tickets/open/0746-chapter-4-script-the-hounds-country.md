---
id: "0746"
title: "Chapter 4 script: The Hound's Country, beat sheet and scenes"
type: content
milestone: M6 Story & dialogue
model: fable-5.1
effort: high
status: todo
blocked_by: ["0736", "0743", "0745"]
nick_input: none
completed:
---

# 0746 — Chapter 4 script: The Hound's Country

## Context

Act 1, Chapter 4 (world map), from `docs/story/outline.md`:

- **Story battle:** "Veyne Hall": storming the lead's childhood home; Hollis and the lead know the old servants' passages; Dace is the boss and escapes at low HP.
- **Turn:** Dace doesn't deny it ("I did what fed us. You had a house; I had a sister."), then to Hollis: "You knew. You always knew." Dace escapes.
- **Recruits:** Gil Parrow (`prizefighter`, Brawler), recruited in the *Millhaven Road* skirmish (1071 builds the battle; this script writes his scenes).
- **Arcs advanced:** the lead and Dace, Hollis (his secret is out), Tamsin.
- **New in this chapter:** the world map, a town (Millhaven), fixed skirmishes (*Aldwater Toll Bridge*, guarding the path; *Millhaven Road*, optional), the first side quest (Tamsin's, 1072).

The story pipeline (ADR-0011, the `story-writing` skill) goes beat sheet
→ script, and `docs/story/ledger.md` records what each chapter changed,
so the chapters are written in order: this ticket waits for Chapter
3's script (0745). It also waits for 0736 (the craft research,
which governs every Act 1 script: a rewrite starts on a blank page, the
length budget) and 0743 (the new cast's character sheets).

## Nick input

None. Claude writes all scripts; only story decisions go to Nick (a
change to a beat in `beats.md` or the outline, a new named character the
cast doesn't have), asked with `ask-nick` before writing the scene that
needs it. Nick reads the chapter in the Act 1 playtest (1064).

## Scope

**In:**
- **The beat sheet** `docs/story/chapters/ch04.md`, in
  `ch01.md`'s shape: cast on screen (ids from `docs/story/names.md`),
  each scene with its place, who speaks, what each speaker wants in it,
  what changes; the battle's lines (the boss, the new units' Classic
  death and Casual retreat lines, talk conversations).
- **The script** `assets/dialogue/ch04.dlg`: the chapter's intro
  and pre-battle scenes, the battle's scenes, the victory scenes, by the
  rules in `assets/dialogue/README.md` (role ids, name tokens, `{lead}`,
  no line that depends on the lead's gender).
- **Fixed skirmishes:** *Aldwater Toll Bridge* (guards the path) and *Millhaven Road* (optional; Gil's recruitment). Write their short scenes here (1071 builds the battles and wires the scenes).
- **Town:** Millhaven (no script needed unless the beat sheet puts a scene there).
- **Side quests:** *The Company Ledger* (Tamsin's; 1072 writes it).
- New names in `docs/story/names.md` first, then `assets/data/names.ron`.
- The critique pass (`story-writing` skill), then `docs/story/ledger.md`.
- **Either order with the battle ticket (0850):** the beat sheet
  names each scene's place and mood; the `place` directives, their maps
  and the `@music` lines are 0850's, which comes after this ticket
  (the validator checks that a place exists). If 0071 has picked this
  chapter's scene music, add the `@music` lines here and tell 0850.

**Out (do not do):**
- The battle file, map, units and triggers (0850).
- Support conversations and camp events (1073); side quests' scenes
  (1072).
- Changing a story beat without Nick.

## Implementation steps

1. Read the `story-writing` skill (with 0736's craft section), ADR-0011,
   `beats.md`, `bible.md`, the outline, the ledger, every character
   sheet on screen, Chapter 3's beat sheet and script.
2. Write the beat sheet; check it against the canon order.
3. Write the script on a blank page (0736's rule), within the length
   budget.
4. Critique pass; fix; update the ledger.
5. `cargo xtask check-script` and the content gates.

## Acceptance criteria

- [ ] `docs/story/chapters/ch04.md` and `assets/dialogue/ch04.dlg` exist and
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
