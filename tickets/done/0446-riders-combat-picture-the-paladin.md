---
id: "0446"
title: Riders' combat picture (Pixel Flag's Paladin)
type: content
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: ["0413"]
blocks: ["1064"]
nick_input: setup
completed: 2026-11-16
---

# 0446 — Riders' combat picture (Pixel Flag's Paladin)

## Context

Ticket 0413 built the combat stage (ADR-0056): each fighter is one bought
still picture, imported by `cargo xtask battler-import` from the table in
`crates/xtask/src/battler_import.rs`. In ticket 0040 Nick chose **Pixel
Flag's Paladin** (an armoured lancer on a white horse) for every mounted
class, at **1.5×**, recoloured per character
(`docs/design/look-and-feel.md`, *Fighters with no Tiny Tales art*). The
pack was not bought when 0413 was done, so the Rider class (Tamsin, the
`sergeant`, and the Red Company's `red_rider` in Chapter 1) shows the
public placeholder figure in a fight. Nick, 2026-10-11: "is the paladin
needed before ch1 playtest?"; it isn't, so it is this ticket.

Chapter 1 playtest round 3 (0804, 2026-10-16), on the intro cinematic's
combat scenes: "we need the battler for tamsin". So 0804's round 4 now
waits for this ticket, and like the rest of round 3's tickets it waits
for the engine carve (1118) and the Playdate port (1050), Nick's
priority.

**Re-gated 2026-10-21 (0129):** the carve is done and 1050 was folded
into the Act 1 playtest (1064), so neither 1118 nor 1050 is in
`blocked_by` any more.

## Nick input

**Setup:** buy *ドット絵TRPGのコマ「クラスと モブ」全64種* from Pixel Flag
(¥500, <https://booth.pm/ja/items/7851483>) and say where the download is
(or drop the zip in `D:\tactical-rpg\assets-private\originals\pixel-flag\`).

Then a short **decision** (`ask-nick`, on renders of the real game as in
0413): Tamsin's colours against the Red Company rider's (0413's rule: the
class's picture is the enemy's look, in red; the named character gets her
own recolour).

## Scope

**In:**
- The pack sorted into `assets-private/library/pixel-flag/` with its
  licence text (ADR-0032, ADR-0040), and its row in
  `THIRD_PARTY_ASSETS.md`.
- Two rows in `BATTLERS`: `rider` (the Red Company's colours) and
  `sergeant` (Tamsin's), scale `(3, 2)`, the right `faces`, tints as Nick
  picks; `keyed` if the file has a flat background.
- A new credit in `assets/data/credits.ron` with `private:
  ["battlers/rider.", "battlers/sergeant."]` (ADR-0051), and the picture
  named in `look-and-feel.md`.
- Commit and push in `assets-private/`, then `cargo xtask private-assets
  --pin`.

**Out (do not do):**
- The riders' map sprites (MattWalkden's, another ticket's).
- Other classes' pictures; changes to the stage or the motions.
- A picture taller than the stage's ground line allows (138 px once
  scaled): if the Paladin at 1.5× is, say so and ask Nick for a size.

## Implementation steps

1. `cargo xtask private-assets --library`; unzip the pack into
   `assets-private/library/pixel-flag/`, keep its terms as a text file
   there.
2. Add the rows to `BATTLERS` (and a `const fn` like `shironejiya` for the
   pack's defaults). Check `MAX_SCALED_PX` in `trpg_content::battler`.
3. `cargo xtask battler-import`, then render a rider's fight with
   `cargo run -p xtask --features private-assets -- frame-png …`
   (`assets/battlers/README.md`) and send it to Nick with the recolour
   options.
4. Credits, `THIRD_PARTY_ASSETS.md`, `look-and-feel.md`; push and pin.

## Acceptance criteria

- [x] In a `private-assets` build a Rider's fight shows the Paladin on
      both sides, in different colours, facing each other.
- [x] `cargo test -p trpg-content --features private-assets --test it
      private_assets::` passes (every bought file has a credit).
- [x] Nick was sent a render and chose the colours.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `the_games_table_names_real_ids_once_each…` in `battler_import.rs`
  covers the new rows; update the credits tests that list the bought
  works.

## Completion notes

Nick bought the pack on 2026-11-16 (the zip `icons_trpg001.zip`).

- **The pack** is in the private repository: `library/pixel-flag/` (the
  64 pictures at 64×64, their 8× copies, the sheet), `terms.txt` (the
  item page's terms as read today: commercial use and edits allowed, no
  redistribution, no credit needed) and `INDEX.md`, **a catalogue of
  all 64 pieces** with the shop's name, what each shows, its size and
  whether it faces the viewer or turns to the side. Nick asked for it so
  a later picture comes from what's bought before buying another.
  `look-and-feel.md` (*The sources and their terms*) sums it up and
  points there. The zip as bought is in `originals/pixel-flag/`.
- **The rows**: `rider` (the class: the Red Company's riders) and
  `sergeant` (Tamsin), the Paladin (no. 24) at 1.5×, facing left, not
  keyed (a clear background). 96 px tall once scaled, under the 138 px
  ground line.
- **Colours (Nick, on renders of the intro's fight shot pointed at the
  Chapter 1 fort road's rider):** the enemy's Paladin has its gold trim,
  lance and cloth in red, like the Red Company's guard. Tamsin keeps the
  gold trim and lance; her tabard and saddle cloth are a clear red. Three
  options first (as bought, Veyne blue, faded brick red); Nick: "prob a
  bit more red than A or C"; then the reddest of three. Recorded in
  `look-and-feel.md` (*Who is recoloured in Chapter 1*).
- Credit `pixel_flag` in `credits.ron` (the terms ask for none; every
  bought work has one anyway), its row in `THIRD_PARTY_ASSETS.md`, and
  the two credits tests that list the bought works.
- **Deviation:** the enemy's recolour was not named in the ticket (it
  said "the Red Company's colours"). Recolouring only the teal cloth left
  the two riders almost the same on screen, so the trim and lance went
  red too, the Church Knight's own recolour (`GOLD_TO_RED`).
- The render was made by pointing the intro cinematic's fight shot at
  the rider for one local run; that edit was undone and is in no commit.

No starting rules. Nothing for the Act 1 playtest beyond the look.
