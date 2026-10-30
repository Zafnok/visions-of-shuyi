---
id: "0750"
title: "Chapter 8 script: The Last Watch, beat sheet and scenes"
type: content
milestone: M6 Story & dialogue
model: fable-5.1
effort: high
status: done
blocked_by: ["0736", "0743", "0749", "0069"]
nick_input: none
completed: 2026-10-30
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
  (1072, 1083, 1084).
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

Done 2026-10-30.

- **Written:** the beat sheet `docs/story/chapters/ch08.md` (in
  `ch07.md`'s shape) and the script `assets/dialogue/ch08.dlg`, on a
  blank page from scene cards: the crossroads bill (the king's
  declaration; east); the night before on the hill above Saltmere
  (Hollis and the lead, the sword and the stone, "my girl's on this side
  of the water"); the quay at first light (Ragna's ship; Hedda chooses
  east; the mare); three streets (Dace's offer, refused by silence); the
  battle's lines (turn 1, the second street, the last turn from the
  deck, Dace to anyone, to the lead, to Hollis, to Joss, to Rue, at half
  strength, down; the lead's fall; nine death quotes and nine retreat
  lines); the gangway (Hollis holds it and is shot; the version where
  the lead tells Dace); the deck (the goodbye); Dace alone on the quay;
  the king's closing scene. Two reply choices (to Hollis the night
  before; on the deck). All talk before the first move is 28 boxes on
  the longest path, the cap.
- **Both versions of Hollis's death** (0069, `death-and-difficulty.md`):
  `ch08_victory`'s `@if retainer` is the gangway with him; its `@else`
  has Dace read his father's death off the plank and the lead tell him
  about Wren (Rue adds what she saw if she is there). **The `@dies
  retainer` is in the next scene**, `ch08_sail`: Hollis is still in the
  army through the gangway, so the deck's `@if retainer` can hold the
  grief of people who watched him die an hour ago (and the `@dies`), and
  its `@else` the deck when he died chapters back. After that nobody is
  "there" for him on any path, so the dock and the king's scene never
  mention him. 0715's blocks were enough; no new way to branch.
- **The critique pass** ran three rounds: two fresh readers on the
  player view (names shown; names hidden, asked who speaks each box),
  then a third on the second draft and a fourth on the reworked scenes,
  plus the editor's list each time. Round 1 found Hollis trailing off
  twenty-two times in twenty boxes (his card allows about one), seven
  speakers sharing "I'm not X, I'm Y", nine deaths all handing someone
  an object and seven retreats on "I'll go to the plank", thirty-two
  "somebody/nobody", Dace's British "I've no orders", and the deck
  assuming Hollis died today on every path (the `@dies` move above).
  Round 2 found Dace closing seven boxes by repeating his own order,
  "the old man" meaning both Crane and Hollis, "last" shown three times
  on the gangway, the book passage echoing the chapter (a storm and a
  ship; then steps and a sleeping knight), and that Dace was at Veyne
  Hall, so he may already know his father is dead (he now reads it off
  the plank; the lead's "He's dead." is gone). Round 3 found three
  retreats on "knee gone, I'll sit on a crate" and five one-word
  openers. All fixed; what the last reader still flags is in the PR.
- **Names:** no new names. `king` added to `characters.ron`'s
  `speakers` (he speaks in the closing scene and never fights in Act 1).
  The king has no face yet (0070 did not pick one); the script uses the
  five expressions every character has.
- **Music:** 0071's picks are `battle_finale` (the battle file's),
  `scene_loss` (Hollis dies), `scene_departure` (the ship sails) and
  `scene_lamplight` (the king). None is imported, so the script marks
  each place with a comment; **0854 imports them and adds the `@music`
  lines** (its step 3 already says so). The scenes 0071 left open use
  the existing mood cues as *Claude's picks* (`talk_calm`,
  `dungeon_tense`, `scene_sad`).
- **For 0854:** the scene ids and triggers are listed at the top of the
  script; the `@if` blocks around companion lines in battle scenes go as
  each trigger is wired; Ragna speaks once mid-battle from the deck
  without being a unit; the turn count is the scenes' "an hour". **For
  1065:** the closing scene is `ch08_closing`, after the victory scenes.
- **Deviations:** none from the outline. Hollis's last words to Dace and
  the king's "Your hands are warm tonight" are the outline's words
  (Nick approved the outline at gate 2); the rest is new.
- **For the Act 1 playtest (1064):** nothing to judge by playing beyond
  the words. No starting rules were decided (no Rust, no numbers).
- **Follow-ups:** none filed.
