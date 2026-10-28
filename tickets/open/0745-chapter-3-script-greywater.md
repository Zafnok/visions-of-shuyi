---
id: "0745"
title: "Chapter 3 script: Greywater, beat sheet and scenes"
type: content
milestone: M6 Story & dialogue
model: fable-5.1
effort: high
status: todo
blocked_by: ["0736", "0743", "0744"]
nick_input: none
completed:
---

# 0745 — Chapter 3 script: Greywater

## Context

Act 1, Chapter 3 (linear), from `docs/story/outline.md`:

- **Story battle:** Greywater Abbey: a causeway to an island abbey, wardens and Hounds on the walls, the scriptorium as the Seize point, chests in the reliquary.
- **Turn:** The register: Rue's name twice, Wren's name ("Ashfields: lost"), Vosse's pay-book ("D. Marr, informer: eleven children"); Rue tells the lead about the mill, Maud confesses to Rue, Hollis says nothing.
- **Recruits:** none.
- **Named boss (0743):** Prior Edwin Sallow (`greywater_prior`), who dies or surrenders when beaten (Nick, 2026-10-28: "can say dies or surrenders"; this ticket picks). Sergeant Ned Tolley (`hound_sergeant`), an old Veyne guardsman, commands the Hounds guarding him: a named enemy, not the boss. Sheets in `docs/story/characters/`.
- **Arcs advanced:** Maud (breaks with the Vigil), Rue (tells), Hollis (his silence starts to show), the lead and Dace.
- **New in this chapter:** a Seize objective and chests; after this chapter the Act 1 world map opens.

The story pipeline (ADR-0011, the `story-writing` skill) goes beat sheet
→ script, and `docs/story/ledger.md` records what each chapter changed,
so the chapters are written in order: this ticket waits for Chapter
2's script (0744). It also waits for 0736 (the craft research,
which governs every Act 1 script: a rewrite starts on a blank page, the
length budget) and 0743 (the new cast's character sheets).

## Nick input

None. Claude writes all scripts; only story decisions go to Nick (a
change to a beat in `beats.md` or the outline, a new named character the
cast doesn't have), asked with `ask-nick` before writing the scene that
needs it. Nick reads the chapter in the Act 1 playtest (1064).

## Scope

**In:**
- **The beat sheet** `docs/story/chapters/ch03.md`, in
  `ch01.md`'s shape: cast on screen (ids from `docs/story/names.md`),
  each scene with its place, who speaks, what each speaker wants in it,
  what changes; the battle's lines (the boss, the new units' Classic
  death and Casual retreat lines, talk conversations).
- **The script** `assets/dialogue/ch03.dlg`: the chapter's intro
  and pre-battle scenes, the battle's scenes, the victory scenes, by the
  rules in `assets/dialogue/README.md` (role ids, name tokens, `{lead}`,
  no line that depends on the lead's gender).
- New names in `docs/story/names.md` first, then `assets/data/names.ron`.
- The critique pass (`story-writing` skill), then `docs/story/ledger.md`.
- **Either order with the battle ticket (0849):** the beat sheet
  names each scene's place and mood; the `place` directives, their maps
  and the `@music` lines are 0849's, which comes after this ticket
  (the validator checks that a place exists). If 0071 has picked this
  chapter's scene music, add the `@music` lines here and tell 0849.

**Out (do not do):**
- The battle file, map, units and triggers (0849).
- Support conversations and camp events (1073); side quests' scenes
  (1072).
- Changing a story beat without Nick.

## Implementation steps

1. Read the `story-writing` skill (with 0736's craft section), ADR-0011,
   `beats.md`, `bible.md`, the outline, the ledger, every character
   sheet on screen, Chapter 2's beat sheet and script.
2. Write the beat sheet; check it against the canon order.
3. Write the script on a blank page (0736's rule), within the length
   budget.
4. Critique pass; fix; update the ledger.
5. `cargo xtask check-script` and the content gates.

## Acceptance criteria

- [ ] `docs/story/chapters/ch03.md` and `assets/dialogue/ch03.dlg` exist and
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
