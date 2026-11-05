---
id: "0862"
title: "Chapter 4 art: Gil's face and combat picture, Dace's combat picture"
type: content
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0850"]
nick_input: setup
completed:
---

# 0862 — Chapter 4 art: Gil's face and combat picture, Dace's combat picture

## Context

0850 built Chapter 4's story battle at Veyne Hall and made Dace Marr
(`rival`) and Gil Parrow (`prizefighter`) units, but not their art, for
the reasons 0848 and 0849 split theirs off into 0858 and 0861: Gil's
face waits on Nick (he builds generator faces himself, 0070), Gil's
combat picture on a purchase, and all of it goes through the private
repository in one push (`CLAUDE.md`, *Bought art*; ADR-0040). Until this
lands, Gil speaks with an empty portrait frame, and both are drawn by
their class's map sprite and combat picture (Brawler; Shadowblade, which
has none of its own yet, so the default).

The picks (`docs/design/look-and-feel.md`, *Faces and combat pictures
for the Act 1 cast*, and the table under *Fighters with no Tiny Tales
art*; the sheets in `docs/story/characters/`):

- **Gil**: generator face **B** (slicked-back black hair, a scar over one
  eye, a goatee); combat picture Sagak Art's **"8-Bit Heroes 2:
  Fighter"** (<https://sagak-art-pururu.itch.io/8bitheroes-fighter>),
  bare-chested, gold fist wraps, red sash, drawn at **1.75×** (0070).
- **Dace**: his face is done (0706: `portraits/rival` in the private
  repository); his combat picture is Shironejiya's **dark knight**
  (`m_s_darkknight`, black horned armour, closed helm, red cape) at
  **1.25×** (0040). It is already in the private repository's
  `library/shironejiya/monster_s/m_s_darkknight.png`, with that site's
  credit text (`look-and-feel.md`, *Fighters with no Tiny Tales art*).
  His map picture is 0735's, not this ticket's.

0858 and 0861 go through the same tools; one session may do several of
them in one push of the private repository.

## Nick input

**Setup** (before the session starts):

1. **Buy** Sagak Art's "8-Bit Heroes 2: Fighter" ($3, the link above;
   0070: "Nick buys it") and put the download in the private
   repository's `library/` (any folder name; say which).
2. **Build Gil's face** in the Character Generator EX from this parts
   list, and save the eight expressions as the other generator faces
   were saved (`assets-private/game/portraits/keeper/` shows the set).
   His scenes use all five of `neutral`, `happy`, `angry`, `sad` and
   `surprised`.
   - **Gil** (34, `prizefighter.md`): a big adult man; slicked-back
     black hair; a scar over one eye; a black goatee; bare-chested, or
     the plainest sleeveless outfit; a red sash or belt if one fits; no
     hat (face B, 0070).

Verification after that is by tests and `cargo shuyi frame-png`
pictures (built with `--features private-assets`, screenshots only);
no check after merge (the Act 1 playtest, 1064, covers the look).

## Scope

**In:**
- Gil's generator face (`cargo xtask portrait-import --generator`).
- Gil's combat picture at 1.75× and Dace's at 1.25× (`cargo xtask
  battler-import`).
- Credits: each file's path in its credit's `private` list in
  `assets/data/credits.ron`, the packs and artists added (Sagak Art,
  Shironejiya) if 0858 or 0861 hasn't; `THIRD_PARTY_ASSETS.md` rows.
- Push the private repository, `cargo shuyi private-assets --pin`,
  commit `assets-private.rev`.

**Out (do not do):**
- Drawing any face or picture (recolours and small edits only, the
  `ascii-art` skill).
- Dace's map picture (0735); anyone else's art; any change to the
  battle, units or script (0850's, 1071's).

## Implementation steps

1. Read `look-and-feel.md` (*Faces and combat pictures for the Act 1
   cast*), the `ascii-art` skill, 0858 and 0861, and how 0706/0413
   imported Chapter 1's art (`tickets/done/`).
2. `cargo shuyi private-assets --library`.
3. Gil's face; the two combat pictures.
4. Credits and `THIRD_PARTY_ASSETS.md`; push; pin.
5. Render a fight with Dace at Veyne Hall and Gil's join scene
   (`ch04_millhaven_road_victory`, once 1071 has wired it; else his
   death line, `ch04_death_prizefighter`) with `cargo shuyi frame-png`
   built with `--features private-assets`, look at them, and fix what
   looks wrong.

## Acceptance criteria

- [ ] With the private assets, Gil has a face in every scene he speaks
      in, and Gil and Dace their own combat pictures.
- [ ] The private-assets tests resolve their character-specific
      portrait and battler ids; substituting a class default or omitting
      a portrait fails. Ordinary gates stay independent of the bought
      files.
- [ ] `cargo shuyi private-assets --pin` passes (every new file has a
      credit).
- [ ] All gates in the `run-gates` skill pass, the private-assets tests
      included.

## Tests required

- The private-assets tests (`crates/content/tests/it/private_assets.rs`)
  load the new portrait and battlers.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
