---
id: "0747"
title: "Chapter 5 script: Salt and Silver, beat sheet and scenes"
type: content
milestone: M6 Story & dialogue
model: fable-5.1
effort: high
status: done
blocked_by: ["0736", "0743", "0746"]
nick_input: none
completed: 2026-10-29
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

- [x] `docs/story/chapters/ch05.md` and `assets/dialogue/ch05.dlg` exist and
      cover every beat above.
- [x] The critique pass checklist is ticked in the PR body.
- [x] The ledger records what changed for each character.
- [x] `cargo xtask check-script` passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The dialogue validator (content gates); no Rust code.

## Completion notes

Done 2026-10-29 (Fable 5.1, high).

**What was done.** The beat sheet `docs/story/chapters/ch05.md` (cards
written first in a scratch file, never in the script) and the script
`assets/dialogue/ch05.dlg`, written on a blank page: the road south
(`ch05_intro`), the two fixed skirmishes' scenes (*The Salt Road*, where
Hollis sees the sea; *Smugglers' Cove*), the three scenes before the
hold (the envoy's house, the sea steps with Hollis's confession, the
attack), the battle's lines (a nameless Hound officer as boss, his
exchange with Joss, turn 1, the roofs, death and retreat lines for the
seven, Gil and Hedda), and four scenes after (the morning with the
magistrate and Joss's third telling, the quay where Aske's secret comes
out and her boat is named, the boat where Ragna says "war" and lends
Hedda, and the cut to Dace). Three reply choices (to Hollis, to Joss, to
Aske). Every companion line outside a trigger is in an `@if` block, and
every scene reads through on every path; the cut to Dace changes if
Hollis or Joss is dead. New registry entries: `hound_officer`
("Officer") and `magistrate` ("Magistrate"), nameless speakers like the
herald, in `names.md` and `names.ron`; `envoy`, `shieldbearer`,
`hound_officer` and `magistrate` added to `characters.ron`'s speakers.
The ledger has an *After Chapter 5* section with the every-path notes.

**Critique pass.** A fresh subagent cold-read the player view (names
resolved) twice: after the first draft and after the redraft. The first
read's hits: a pre-battle scene where everyone was the same calm
commander and the doors blurred (redrafted from its card: Tamsin angry
at Ragna, Joss doing drill and asking to be put in a door, Maud unable
to get the shutter bar across, "the street door" named), death and
retreat lines on one template (every one reshaped), two boxes that could
not be followed in the opening (Joss now has a prompt and says "the
ford"; Aske's "that is why" cut and Tamsin points at her contradiction),
a speech on the quay ("two things, the first, the second" gone), Ragna
and Hedda blurring without their names (Ragna now speaks in conditions
and counts; Hedda is slower, warmer and touches people), a "not X but Y"
reply, and Britishisms ("downs", "verge", "mind", "you'll not", "he'll
have said"). The second read is in the PR body.

**Deviations from the sheet's first draft.** Aske's secret comes out on
the quay after the hold, not in the envoy's house: the three scenes
before the first move are capped at 28 boxes together (the
`story-writing` budget), and the house needed its room for two new faces
and Joss's first telling. The sheet records the choice and that a
rewrite may move it back. Joss's second and third tellings both fall on
the morning after (the expulsion made "the day after" the same morning).

**For 0851 (the battle).** The scene ids and trigger shapes are in the
script's header. `ch05_eve` plays `scene_sad` as a stand-in: Nick picked
`scene_confide` (0071); import it with `battle_night_siege`, drop both
from `RESERVED` in `crates/content/src/audio.rs`, and change that one
`@music` line. Hedda is sent to the street door in `ch05_prebattle` and
should fight the hold as a player unit (then her death and retreat lines
can fire); Ragna lends her in `ch05_tide`, which is when she joins the
roster (if Hedda fell in the hold on Classic she never joins; every line
of hers after the battle is already in an `@if shieldbearer`). Whether
Ragna stands on the map (green) is 0851's; she must survive. The
`@if` blocks in the death quotes, retreat lines and the officer's
exchange with Joss go as each trigger is wired. The house, the quay and
the boat are three distinct places (the house's room, the quay at the
sea steps, the same quay with the boat); the Hound post cut needs a
fourth. Tell 1071: the Salt Road's post is Hounds at a march stone; the
cove's men are smugglers, nobody's soldiers, who run before they fight
(Hollis says so).

**For 1072 (*Letters Home*).** The hook is planted on the quay: the
refugee boat with the hill people's letters and silver (Aske's among
them) was taken off the point by League privateers three days before and
is held somewhere the League won't say; Ragna will find out and send
word. The quest unlocks after this battle with Aske alive.

**Claude's starting rules** (story staging, not gameplay): the boss is a
nameless Hound officer (no new named character, so nothing went to
Nick); the magistrate is nameless too; Hollis sees the sea on the Salt
Road ridge and says "It's a lot of water", which Dace quotes in the cut;
Hedda knows Aske's name from Ragna's papers but doesn't say so in this
chapter. Nothing gameplay-affecting was decided.

**Engine friction:** none, no Rust code.

**Follow-ups:** none filed.
