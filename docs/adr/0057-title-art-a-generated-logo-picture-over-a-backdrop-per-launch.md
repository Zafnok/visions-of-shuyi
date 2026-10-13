# ADR-0057: Title art: a generated logo picture over a backdrop per launch

- **Status:** Accepted
- **Date:** 2026-10-12
- **Related tickets:** 0811, 0036, 0817, 0819, 0830
- **Follows:** ADR-0038 (a picture is a sprite item), ADR-0040 (bought
  files live in the private repository), ADR-0054 (a view, painted by a
  skin), ADR-0056 (the backdrops' size and mirrored edges)

## Context

- Nick chose the title's look in ticket 0811
  (`docs/design/look-and-feel.md`, *Title screen*): big block lettering,
  over one of seven bought battle backdrops (a different one each launch),
  with sparks drifting up.
- The lettering is made of square blocks 8 screen pixels big, in several
  colours with a dark edge. The ticket (written before ADR-0038) asked for
  it as glyph art in a text file. Half-block glyphs can't sit on a picture:
  a cell whose two halves differ needs a background colour, and a cell's
  background is drawn under a backdrop sprite. And ADR-0038 says a picture
  is a sprite from an image file, never cells or rectangles.
- The same logo is the intro cinematic's last shot (0817) and will be on
  the Game Over screen (0830).
- The backdrops are bought art: not in this repository, and every gate
  must pass without them.

## Decision

1. **The logo is one PNG, `assets/title/logo.png`, generated** by `cargo
   xtask title-art` from a text file, `assets-src/title/logo.txt` (a
   character a pixel, each character a palette name), and the palette. So
   the art stays reviewable text and uses palette names only; a test fails
   if the committed PNG isn't what the tool makes. It is drawn as one
   sprite over the glyphs at 8 console pixels per logo pixel.
2. **`assets/title/title.ron` names the logo and the backdrops**, loaded
   by `trpg_content::title` into `Content::title` and checked with the
   other assets (sizes, missing files, duplicates).
3. **Backdrops share their names between the two repositories**, as the
   battle backdrops do: flat placeholders here, the bought pictures in
   `assets-private/game/title/` (`cargo xtask title-art --bought`). They
   are battle backdrops in size (336×248) and are drawn the same way: at
   2×, with a 32-pixel strip at each edge mirrored to fill the width.
4. **One backdrop per launch.** The title picks it when it is made, with
   `Ctx::music_seed` (the seed `app` already sets from the clock for picks
   that don't touch the simulation), never the one named under the storage
   key `title_backdrop`, which its first update then overwrites.
5. **The sparks are a function of time alone.** The view carries
   `sparks: Option<u32>` (milliseconds the title has been shown); the skin
   places each spark from its number and that time. No particle state.
6. **`Ctx::idle_motion`** says whether screens show such idle motion.
   It is off by default, like `tips_enabled`, so a screen left alone in a
   test looks the same a moment later; `app` (and `frame-png`) turn it on.
7. **`title::glyph::paint_logo`** paints the backdrop, the sparks and the
   logo and nothing else: the title paints its menu over it, and the
   cinematic's logo shot (0817) and Game Over (0830) call it or
   `logo_sprite`.

## Consequences

- A copy of the public repository shows the logo over a flat placeholder.
- A colour theme (0806) needs `cargo xtask title-art` run per palette, or
  the logo keeps palette D's colours. That ticket decides.
- The title's text keys `title.title` and `title.subtitle` are gone: the
  name is a picture and is not translated.
- A later "reduce motion" option only has to set `idle_motion`.

## Alternatives considered

- **Half-block glyph cells, as ADR-0018's first portraits.** Works on
  black only; on a backdrop every two-colour cell needs the sprite cut
  under it (hundreds of cuts a frame), and ADR-0038 rules it out.
- **Rectangles per block.** The same rule; and a snapshot would list
  hundreds of them.
- **A hand-made PNG.** Not reviewable, and free to drift from the palette.
- **The private `title.ron` listing its own backdrops.** The public list
  would then differ from what ships, and tests of the rotation would run
  on other data than the game.
