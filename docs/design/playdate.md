# The Playdate version (Chapter 1)

Decided: 2026-10-15
Source: ticket 0061 (build tickets: 1025; technical test: 1026); the
one-bit pictures: ticket 0062

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
- **No Dialogue log in the map menu** (Nick, 2026-10-17, ticket 1035):
  the log holds the conversation playing and opens only while one plays,
  when the map menu can't open, so on the map it would never work. Asked
  whether to leave it out, show it greyed out, or open the last
  conversation's log from the map, he picked **"Leave it out"**. During a
  conversation the system menu's Dialogue log opens it. So the Playdate's
  map menu gains Danger zone, Auto-end and Unit info, each only while it
  has no button.
- *Claude's starting rules (ticket 1035, Nick can veto):* the extra
  entries sit after Objective, in that order. Danger zone and Auto-end
  close the menu and do what their keys do (the danger zone then shows on
  the map). Unit info opens the info screen of the unit under the cursor
  and is greyed out on an empty tile. Winding the crank while "Use 1 of N
  charges?" is up does nothing; with nothing to rewind or outside the
  player's turn it does what the Rewind key does there. One notch is 30°
  (*tunable*, `keymap.ron`'s `crank`) wound one way; turning the other
  way starts a fresh notch, so after a step back the next step forwards
  takes a whole notch, and a hand resting on the crank never steps.

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

## One-bit pictures

Decided: 2026-10-15
Source: ticket 0062 (build tickets: 1040 converter, 1041 combat picture,
1042 portraits, 1044 title, 1052 launcher)

Nick judged one-bit versions of Chapter 1's real faces (the male lead,
Harl, Rue, the female lead), battlers (the female lead, the Brigand) and
title backdrop at 400 × 240, shown at ×2. In the second round they were
also on his Playdate, as a slideshow he sideloaded. Pictures made from
the bought art are not in this repository (ADR-0040). The committed
ones use only the game's own logo:

| Picture | Shows |
| ------- | ----- |
| [`playdate/0062-icon.png`](playdate/0062-icon.png), [`×4`](playdate/0062-icon-x4.png) | **The launcher icon** (32 × 32): the block S of `SHUYI` |
| [`playdate/0062-card-logo.png`](playdate/0062-card-logo.png) | **The logo as on the launcher card**: white letters, a black shadow and rim. The checker here stands in for the dithered backdrop |

### Nick's words

> **Q1, the portraits' style** (A Bayer 4×4, B Floyd–Steinberg, C
> Atkinson, D threshold, E three tones, F Bayer with five flat greys):
> "I like A, B, and F for faces and hair, but seems none of the options
> capture mouth expressions very well. E seems best for the hat of the
> witch. I don't like C or D." … [round 2: the art's own dark lines kept
> solid (A+, B+, F+ lines, G: four flat greys plus lines), the skin
> lighter:] "if there is a way, without bloating ROM size, to include
> this as a toggle between a few rendering styles in playdate, we can do
> that. Otherwise I am leaning towards the original A now that mouth and
> eyes kept... not even A+lines" … [told it costs about 120 KB a style;
> which A, and which styles go in the toggle:] "A1 round 1" … "G as
> well,F+ lines,A+ lines"
>
> **Q1, the portraits' size** (A 3× speaker only, B 2× both faces, C 1×
> in a text box over the map): "both A and B seem workable, I would have
> to see how readable it would look on the real (small) playdate screen.
> Could be that we have to go with A for readable text" … [after the
> device:] "B. 2x, both faces"
>
> **Q2, the combat picture** (A dithered fighters, on a plain ground or
> the dithered backdrop; B silhouettes; C outlines; D the map tiles
> large): "A is more readable but the background is plain, I like A2
> background but with the dithering on both character and foreground they
> blend which is not good. B, C, D look bad." … [round 2, four backdrops:
> faded, outlined fighters, horizontal lines, only above the horizon:]
> "horizon looks good (D) but the brigand looks horrible here, his face is
> totally fucked up. The female model looks fine." … [the Brigand fixed,
> fighters in style A:] "Yes, this one"
>
> **Q3, the launcher card** (A the logo on white, B the dithered title
> backdrop with the logo, C a face beside the logo): "B. Dithered
> backdrop" … **the icon:** "where is the launcher icon shown in the
> playdate OS? I don't see it used anywhere" … [told: only in the
> launcher's list view, and every game must have one:] "A. The S from
> SHUYI"

### Rules

Every picture is made one-bit **ahead of time** by the converter (1040),
never on the device. Transparent pixels are white (the ground).
**Dithering is done at the screen's pixels**: the picture is enlarged
first, then dithered, so each dot is one Playdate pixel. In the first
round, dithering the art's own pixels and then enlarging gave blocky 2×2
dots and was not offered.

**The styles** (the names are the ones Nick saw):

| Style | How |
| ----- | --- |
| **A** (A1) | The picture's brightness is stretched so its darkest 3% is black and its lightest 3% white, then a 4 × 4 Bayer ordered dither: a regular grid of dots (the family of *Return of the Obra Dinn*'s 1-bit look) |
| **A+ lines** | A on a lighter picture (brightness curve 0.8, so skin is lighter), then the art's **own dark lines** drawn solid black on top. A line is an art pixel clearly darker than the brightest pixel around it, or very dark: this keeps mouths, eyes and brows |
| **F+ lines** | As A+ lines, but each grey is first rounded to one of 5 levels, so an area is one even dot pattern |
| **G** | As F+ lines with 4 levels: bolder and flatter |

The thresholds (3%, 0.8, how dark a line is) are *tunable*. Nick saw
that A alone loses some of the mouths ("none of the options capture
mouth expressions very well") and still chose it as the default. The
line styles are there for players who want the expressions.

#### Portraits (Q1)

- **Style A by default.** *Options* has a **portrait style** setting with
  **A, A+ lines, F+ lines and G** (Nick: "a toggle between a few
  rendering styles"). Each style is a separate set of pictures, about
  2 KB a face, about 120 KB a style for Chapter 1's cast. The setting
  exists only on the Playdate.
- **Size: the bust at 2×** (128 × 128 pixels), today's dialogue layout
  at half size: the speaker on the left in a double frame, the listener
  on the right in a single frame, the names under them, and a 3-line
  text box across the bottom (47 letters a line). 4× does not fit the
  240-pixel screen.
- *Claude's starting rules (Nick can veto):*
  - **The listener is faded** by turning every other pixel of its
    picture white (a 50% checker), as Nick saw in the mockups. One-bit
    has no dimming.
  - A face that faces the wrong way is mirrored, as on the PC.
  - The setting is called *Portrait style* and lists its choices by
    what they look like (wording *tunable*).

#### The combat picture (Q2)

- **The fighters are dithered in style A**, at the PC's double size
  (`look-and-feel.md`, *Combat screen*), facing each other as on the PC.
  The portrait style setting does not change them: Nick saw the line
  styles on the Brigand and called his face "totally fucked up"
  (*Claude's starting rule*).
- **A fighter drawn at a size that isn't a whole number** (in
  Chapter 1 the Brigand, the Raider, Harl, the Cleric and the Mage, all
  at 1.25×) **is enlarged smoothly before it is dithered**, not by doubling pixels. Doubling
  some rows and not others broke the Brigand's face.
- **The backdrop shows only above the horizon:** the bought battle
  backdrop, lightened and dithered (Bayer 4 × 4), down to a black
  horizon line. Below it the ground is **plain white**, and the fighters
  stand on it, so they never sit on a pattern. A white rim (2 pixels)
  around each fighter keeps the tops of their heads apart from the
  backdrop.
- The plates (name, HP, damage, hit, crit) sit along the bottom and the
  result (`HIT -6`) pops up over the target, both in white boxes with a
  black edge. The striker's plate has the double edge. The motions are
  the PC's.
- *Tunable:* the horizon's height (110 pixels from the top in the
  mockup) and how much the backdrop is lightened. *Claude's starting
  rule:* the backdrop follows the PC's rules (terrain, time of day),
  each picture converted the same way.

#### The launcher card and icon (Q3)

The Playdate's launcher shows a game as a **card** (350 × 155, the
default "cards" view) or as an **icon** with the game's name (32 × 32,
"list" view). Every game needs both, and a **launch image** shown while
it loads.

- **Card: the dithered title backdrop with the logo** (Nick: "B"). The
  backdrop is dithered in Bayer 4 × 4. The logo is drawn 4 pixels a
  block, its letters white, its shadow black, with a black rim one
  block wide so it reads over the dots.
- **Icon: the block S of `SHUYI`**, 2 pixels a block, black with its
  shadow as a 50% checker, on white.
- *Claude's starting rules (Nick can veto):*
  - The card's backdrop is **Fields B at dusk** (the one Nick saw). The
    card is fixed; the title's rotation is for the game's own title
    screen.
  - The first card and icon don't animate. Nick wants an animated card
    later ("seems the cards view lets you use animated gif or something
    so we should do that too at some point"): ticket 1054.
  - The **launch image** (400 × 240) is the card's look at full screen:
    the same backdrop and logo.
  - No "wrapping paper" (the optional pattern on a newly downloaded,
    unopened game).
  - A build without the bought art has the logo over a 50% checker
    instead of the backdrop (as in `0062-card-logo.png`).

#### The title cinematic (1044)

*Claude's starting rule:* its backdrops are dithered like the card's
(lightened, Bayer 4 × 4), and its logo is the card's (white letters,
black shadow and rim). Its fight shots use the combat picture above.
Nick saw the card, not the cinematic. He signs off on it in 1044.

## Open sub-questions

- Allies and neutral units on the Playdate (none in Chapter 1).
- Whether Catalog allows one purchase to cover several builds: checked
  when applying (a store question, not a design one).
