---
id: "0750"
title: "Chapter 8 script: The Last Watch, beat sheet and scenes"
type: content
milestone: M6 Story & dialogue
model: fable-5.1
effort: high
status: todo
blocked_by: ["0736", "0743", "0749", "0069"]
nick_input: none
completed:
---

# 0750 — Chapter 8 script: The Last Watch

## Context

Act 1, Chapter 8 (world map, Act 1 finale), from `docs/story/outline.md`:

- **Story battle:** "Saltmere Docks": hold the dock for a number of turns while the ship loads, against Hounds from three streets; Dace is the boss and can't be killed; he retreats.
- **Turn:** Hollis dies holding the gangway; Dace orders the archers to stop, too late; Hollis's last words to Dace: "Your sister's alive. Rue saw her walk out of the fire. Go and get her, boy." The ship sails. In Classic, if Hollis already fell, the lead tells Dace instead. Closing scene: Varenhall, the king reads to someone the camera never shows ("Your hands are warm tonight"); *Act 1 ends*.
- **Recruits:** none.
- **Arcs advanced:** Hollis (ends), Dace (his turn begins), the lead (the mentor is gone), everyone (the goodbye).
- **New in this chapter:** nothing new.

The story pipeline (ADR-0011, the `story-writing` skill) goes beat sheet
→ script, and `docs/story/ledger.md` records what each chapter changed,
so the chapters are written in order: this ticket waits for Chapter
7's script (0749). It also waits for 0736 (the craft research,
which governs every Act 1 script: a rewrite starts on a blank page, the
length budget) and 0743 (the new cast's character sheets). It waits for 0069 too: how Hollis's death happens in play shapes the scenes.

## Nick input

None. Claude writes all scripts; only story decisions go to Nick (a
change to a beat in `beats.md` or the outline, a new named character the
cast doesn't have), asked with `ask-nick` before writing the scene that
needs it. Nick reads the chapter in the Act 1 playtest (1064).

## Scope

**In:**
- **The beat sheet** `docs/story/chapters/ch08.md`, in
  `ch01.md`'s shape: cast on screen (ids from `docs/story/names.md`),
  each scene with its place, who speaks, what each speaker wants in it,
  what changes; the battle's lines (the boss, the new units' Classic
  death and Casual retreat lines, talk conversations).
- **The script** `assets/dialogue/ch08.dlg`: the chapter's intro
  and pre-battle scenes, the battle's scenes, the victory scenes, by the
  rules in `assets/dialogue/README.md` (role ids, name tokens, `{lead}`,
  no line that depends on the lead's gender).
- **Town:** Saltmere (one last shop) (no script needed unless the beat sheet puts a scene there).
- **Both versions of Hollis's death** (`docs/story/outline.md`,
  *Classic permadeath*): Hollis alive (he holds the gangway and dies,
  however 0069 decided it happens in play) and Hollis already fallen in
  Classic (the lead tells Dace). Branch with 0715's lines by who is
  still in the army; if 0319 built another way to branch, 0854 switches
  the script to it.
- **The closing scene** (Varenhall, the king's private rooms), played by
  1065 after the victory scenes. The king's sheet is
  `docs/story/characters/king.md`.
- New names in `docs/story/names.md` first, then `assets/data/names.ron`.
- The critique pass (`story-writing` skill), then `docs/story/ledger.md`.
- **Either order with the battle ticket (0854):** the beat sheet
  names each scene's place and mood; the `place` directives, their maps
  and the `@music` lines are 0854's, which comes after this ticket
  (the validator checks that a place exists). If 0071 has picked this
  chapter's scene music, add the `@music` lines here and tell 0854.

**Out (do not do):**
- The battle file, map, units and triggers (0854).
- Support conversations and camp events (1073); side quests' scenes
  (1072).
- Changing a story beat without Nick.

## Implementation steps

1. Read the `story-writing` skill (with 0736's craft section), ADR-0011,
   `beats.md`, `bible.md`, the outline, the ledger, every character
   sheet on screen, Chapter 7's beat sheet and script.
2. Write the beat sheet; check it against the canon order.
3. Write the script on a blank page (0736's rule), within the length
   budget.
4. Critique pass; fix; update the ledger.
5. `cargo xtask check-script` and the content gates.

## Acceptance criteria

- [ ] `docs/story/chapters/ch08.md` and `assets/dialogue/ch08.dlg` exist and
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
