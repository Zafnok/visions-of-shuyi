# The Playdate version (Chapter 1)

Decided: 2026-10-15
Source: ticket 0061 (build tickets: 1025; technical test: 1026)

Nick judged rendered 400 × 240 one-bit mockups of Chapter 1's first battle
(Harrowby Ford), drawn with the game's own font (Terminus 8×16) and, for the
smaller sizes, Terminus 6×12, unscii-8 and Spleen 5×8. Every option shown
is in [`playdate/`](playdate/) at the device's real pixel size. Names,
numbers and the forecast in them are placeholders.

| Picture | Shows |
| ------- | ----- |
| `q1-a-8x16.png` | **Chosen size:** today's 8×16 cells, the map fills the screen, info in corner pop-ups |
| `q1-b-8x8.png`, `q1-c-6x8.png`, `q1-d-6x12.png` | The other sizes offered |
| `q2-a-forecast.png` | **Chosen layout:** the forecast in a pop-up over one side of the map |
| `q2-b-*.png`, `q2-c-*.png` | A strip along the bottom; info on its own screen (not chosen) |
| `q3r2-e-grey.png` | **Chosen:** your units solid black, enemies grey, acted units outlined; the range marks |
| `q3-*.png`, `q3r2-*.png` | The other ways offered to tell sides apart without colour |

## Nick's words

> **Q1, how big the text and the map are** (8×16, 8×8, 6×8 or 6×12):
> "both A and D look good but since it's a small screen" … [asked again,
> A or D:] "A. Today's 8x16"
>
> **Q2, how the battle screen is cut down:** "A. Corner pop-ups"
>
> **Q3, telling things apart without colour** (first round: your units
> solid, enemies solid, enemies framed, a black ground): "none, present
> more options" … [second round: bold letters, grey enemies, horn marks,
> capitals, HP lines, blinking:] "E. Yours black, foes grey" … [the range
> marks:] "Yes, keep them"
>
> **Q4, buttons and the crank:** "B. Menu + chords"
>
> **Q5, what the first version keeps:** "Music and sounds, 1-bit portraits,
> Simple combat picture, Title cinematic" … [voices and the other
> languages:] "is audio quality and ROM size enough to keep voices? if so,
> we can keep, but I doubt we can fit voices tbh! and probably we sell the
> diff language options separately rather than pack all into one tight
> ROM" … "Japanese and Chinese" … [told they all fit in one build:] "we
> can probably sell a license too all version builds with one sale, but
> each build should still be separate... I just don't think it makes sense
> to pack all in one ROM it is a limited device on purpose."
>
> **Q6, where it is sold:** "C. Both, itch.io first"

## Rules

The Playdate screen is 400 × 240 pixels, one bit (black or white, no greys,
no colour). Its controls are the D-pad, A, B, the crank and the system
Menu button, whose menu can hold three of the game's own entries.

### Size: today's cells (Q1 A)

- The Playdate version uses **today's 8 × 16 cells and today's font**:
  50 × 15 cells. A map tile stays two cells wide (16 × 16 pixels), so the
  map shows **24 × 15 tiles** at once (Fire Emblem on the GBA showed
  15 × 10). Chapter 1's 24 × 16 map fits but for one row, which scrolls
  with the cursor.
- The glyph look (ADR-0049) is the Playdate's look. The map tiles are the
  same glyphs as on the PC (`..`, `♣♣`, `≈≈`, `^^`, `╦╦`, …), in black.
- **The ground is white** and the glyphs black (the Playdate's own look;
  its screen is reflective, with no backlight). *Claude's starting rule*:
  Nick was shown a black ground and didn't pick it.

### Layout: the map fills the screen, info pops up (Q2 A)

Like Fire Emblem on the GBA.

- The map takes the **whole screen**. There is no side panel and no status
  line.
- **Browsing:** a small window in a corner shows the unit under the cursor
  (name, HP), and another the terrain (name, Def, Avoid). They sit in the
  top corners, and move to the bottom ones when the cursor comes near
  (*placement tunable*).
- **The forecast** is a window over one side of the map (the side away from
  the two units), with the same contents as the PC forecast
  (`look-and-feel.md`, *Attack forecast*): both sides' HP, Hit, Crit, the
  strikes in order, the kill skull.
- Anything else the PC side panel shows (stats, weapons, the chapter, turn
  and rewinds) is on the Unit info screen and in the map menu
  (*Claude's starting rule*).

### Telling things apart without colour (Q3 E)

- **Your units:** the two letters **white in a solid black tile**.
- **Enemies:** the two letters on a **grey tile** (a 50% checker), each
  letter with a white edge so it reads over the checker.
- **A unit that has acted:** its tile becomes an **outline** with black
  letters (no fill).
- **HP:** the line along the bottom of the tile, as long as the HP left
  (as on the PC, `look-and-feel.md`, *Units on the map*).
- **Move range:** light dots over the tiles. **Attack range:** diagonal
  lines. **The danger zone:** a dashed line along its edge, so it shows
  over the ranges. Terrain glyphs on a marked tile keep a white edge.
- **The cursor:** the PC's corner marks, drawn by flipping the pixels they
  cover, so they show on white, grey and black tiles alike
  (*Claude's starting rule*).
- *Claude's starting rules (Nick can veto):* allies (no ally in Chapter 1)
  wait for a later chapter's decision; neutral units likewise. The threat
  arrows and their `!` marks are drawn in black with a white edge.

### Buttons and the crank (Q4 B)

| Control | Does |
| ------- | ---- |
| D-pad | Moves the cursor (held: repeats) |
| A | Confirm |
| B | Cancel. With nothing to cancel, opens the map menu |
| A on an empty tile | Opens the map menu (as on the PC) |
| **A + B together** | End turn |
| **Hold B + left / right** | Previous / next ready unit |
| **Crank, wound backwards** | Rewind: the battle steps back one action per notch; A keeps that point (one charge), B cancels |
| Map menu | As on the PC, plus every shortcut the Playdate has no button for (Danger zone, Auto-end, Unit info, Dialogue log) |

- *Claude's starting rules (Nick can veto):* winding forwards steps back
  towards the present while choosing. The notch size is *tunable*. The
  crank does nothing else on the map. The three system-menu entries are
  Danger zone, Dialogue log and Options. Help text names the Playdate's
  buttons and the crank, through the keymap as everywhere else
  (`controls.md`).

### What the first version keeps (Q5)

Everything Chapter 1 has on the PC, made for the device:

- **Music and sound effects**, converted to formats the Playdate plays.
- **Portraits** in dialogue: **one-bit (dithered) versions** of the bought
  faces. Making them is an edit of the bought art: check ADR-0040's
  license notes first.
- **The combat picture**: a simple one-bit version.
- **The title cinematic.**
- **Voices**: kept (they fit: about 11 KB a second of compressed speech
  against the Playdate's 4 GB of storage).
- **Japanese and Chinese**: kept, but **each language is its own build**
  (Nick: "it is a limited device on purpose"). One purchase gets every
  language's build (*how the store hands them out*: see *Selling it*).

### Selling it (Q6 C)

- **itch.io first**, beside the Windows build: players sideload it.
- **Then apply to Panic's Catalog** (curated, needs an application).
- One purchase gets all the language builds (Nick: "we can probably sell a
  license too all version builds with one sale"). On itch.io that is one
  page with a download per language.

## Open sub-questions

- Allies and neutral units on the Playdate (none in Chapter 1).
- Whether Catalog allows one purchase to cover several builds: checked
  when applying (a store question, not a design one).
