---
id: "0076"
title: "Decide: Hedda's and the Hound lieutenant's faces and combat pictures"
type: design-decision
milestone: Design decisions
model: opus-5.5
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0076 — Decide: Hedda's and the Hound lieutenant's faces and combat pictures

## Context

Ticket 0070 picked the looks of Act 1's new cast (`docs/design/look-and-feel.md`,
*Act 1's new cast*) except two, both axe fighters, where nothing Nick saw
was right:

| Who | Story | Class / weapon | Chapter ticket |
| --- | ----- | -------------- | -------------- |
| Hedda Ravn (`shieldbearer`) | Ragna's Brennish bodyguard from a northern hill people, loaned to the lead "to make sure you get there" | Raider (axe) | 0851 (Chapter 5) |
| The Hound lieutenant (Chapter 2 boss) | Leads the Hounds disguised as Brennish raiders at Kell's Ford; carries the Houndmaster's Axe | axe | 0848 (Chapter 2) |

What Nick said (0070): Hedda's two faces were "ok", but he wants her
**in between** the Amazon Warrior hero ("scantily clad (she almost looks
naked)") and the closed-helm armoured axe fighter ("totally full body
armor"): mail or leather with fur, face visible. For the lieutenant he
turned down Shironejiya's armoured axe fighter in grey and in Hound
black, and Magitek Dynasty's halberdier ("Search more").

Three searches on 2026-10-27/28 found nothing in our style (chibi,
side-on, dark outlines, 35–80 px, licensed for a sold game, hand-made).
Already checked, so don't repeat them: Mega Tiles' whole catalogue
(owned; no new packs as of 2026-10-28), every Shironejiya battler, Sagak
Art's whole catalogue, Pixel Flag, MattWalkden, cogabushi (AI-assisted;
Nick "doesn't mind much" its red-armoured axe captain for a boss, 0040),
JAPANweb, AlexPixels, Elthen, LuizMelo, Helianthus, Apyryon, Fat Cat
Games, Time Fantasy, Holder, AssetSmithy, indieartifex, RPG Maker DLC,
CraftPix Amazon and Gladiators, Robert Pinero (Low), karsiori,
sanctumpixel, Dreamir, Retsuto, Sevarihk, falvey, Jitsu Koan, SolaarNoble
bases, P3X-774, bubblevski's Bandits (28 px, front-facing, no outline),
K.BOOTH 8-bit SV, Clockwork Raven, Beowulf, The Mighty Palm, somexone,
Pellaore, Hajileee, Pixel Moon, Antahonist, Bitwise, lyssir, Aekashics,
LimeZu, Ocean's Nostalgia, Toby Yasha, Gilignition. GameDev Market
refused automated fetches, so it was never really searched.

## Nick input

**Decision**, with the `ask-nick` skill (the question pop-up), on mockups
in the scratchpad (never committed: ADR-0040), drawn as 0070 drew them:
the face cut as the dialogue frame shows it, and the combat picture at
the game's size facing Harl (Hedda) or the lead (the lieutenant). **Say
who each character is in the story before asking** (Nick, 0070: "please
give me more insights into who each character is before asking me to
pick"). Three to five candidates each, with the pack and file name, plus
"describe your own".

## Questions to ask

1. Hedda's combat picture, and her face to match it (a bought hero face,
   or a generator face Nick builds from a parts list Claude gives; Nick,
   0070: "your simulated generator does not work as well as you think it
   does").
2. The lieutenant's combat picture. A face only if his script gives him
   lines outside the fight (Nick: "Only if they speak"); then the same
   parts-list way.

## Scope

**In:**
- A new search: new packs since 2026-10-28 (Mega Tiles first, then Sagak
  Art, Shironejiya, itch.io's newest side-view battlers), GameDev Market
  by hand in the browser pane, and anything the list above missed.
- Mockups, the questions, and recording the answers in
  `look-and-feel.md` (*Act 1's new cast*: their two rows) with Nick's
  words; a line in 0848's or 0851's Context naming the pick.

**Out (do not do):**
- Buying anything (Nick buys), commissions, drawing art. A small edit of
  a picture (a recolour) is fine; a new weapon, clothes or body is not.
- Importing the files (the chapter tickets do).
- Changing Hedda's class or weapon (0070 offered it; Nick chose to keep
  looking).

## Implementation steps

1. Read `look-and-feel.md` (*Portraits and battle art (bought)*, *Fighters
   with no Tiny Tales art*, *Act 1's new cast*) and ADR-0040.
2. Search, starting with the newest packs. Check each licence against
   ADR-0013 and the bought-art rules (commercial use, no engine lock,
   hand-made).
3. Mock up three to five candidates each; ask; record.

## Acceptance criteria

- [ ] Hedda has a recorded face and combat picture, and the lieutenant a
      recorded combat picture (and a face if he speaks), with Nick's words
      in `look-and-feel.md`.
- [ ] 0848 and 0851 name the picks in their Context.
- [ ] No bought file or mockup made from one is committed.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

- None (decision).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
