# Tutorial: the guide, lessons and Guides

Decided: 2026-10-14
Source: ticket 0057 (after Chapter 1 playtest round 2, 0804)

## Nick's words

The request, playtest round 2 (2026-10-14):

> "we should have some kind of basic tutorial, I am almost thinking of the
> annoying fairy from oot or clippy or something... some tutorial mascot
> thing idk."

Shown mockups of the first turn of Chapter 1 (a guide with a face in the tip
box, Hollis in the conversation box, a *Guides* list, a practice battle
prompt, scripted steps), he answered:

> "Let's make the very basic controls optional, and anywhere our game
> differs from traditional strategy rpgs (even something like rewind)
> forced. Let's let player pick familiarity. Not familiar --> full
> tutorial, familiar with other games --> small tutorial, familiar with
> this game in particular --> no tutorial whatsoever. Additionally, I do
> want a guide character, a new mascot or perhaps a Shuyi character
> herself... a cameo of my wife in her preferred game state... or if you
> can present cute mascots I might pick those instead. Additionally, I do
> like the guides as a menu option, to refresh a player's memory via a
> quick look. As well, taken from other games, I like the implementation
> of tutorials forcing usage of buttons like use W to move up, use A to
> move left, without forcing the specific order (a checklist)"

Shown a shortlist of faces from the bought packs and the checklist in the
guide's box, asked who the guide is, whether the guide is in the story,
where the lessons happen and which topics count as different:

> "1. a) fox miko b) outside for now, with decision much later if we want
> to tie it in (think - Angel from borderlands 2)
> 2. A
> 3. your recommended ones look correct to me"

(2A: the lessons happen inside Chapter 1's battle, each the first time it
matters, like Fire Emblem Awakening's prologue, not in a separate practice
battle. 3: the five topics under *The lessons* below.)

Everything marked *tunable* or *Claude's starting rule* is a default Claude
chose where Nick's words didn't say; Nick may veto it. Unmarked rules are
Nick's.

## Rules

### How familiar are you? (the tutorial level)

New Game asks the player how familiar they are, with three answers:

| Answer | Level | What teaches them |
| ------ | ----- | ----------------- |
| Not familiar | **Full** | The basic-controls lessons, the *different here* lessons, and every tip |
| Familiar with other strategy RPGs | **Short** | Only the *different here* lessons |
| Familiar with this game | **None** | Nothing: no lessons, no tips |

- The question comes **after the game mode** (Classic / Casual), before the
  lead screen. *Claude's starting rule.*
- The answer belongs to the campaign (kept in its saves, like the game
  mode). *Claude's starting rule.* It **can be changed later in Options**
  while that campaign is loaded (Nick, 0058; `options.md`, *Where options
  are kept*).
- At **Short**, today's standard tips (first unit selected, enemy in reach,
  forecast, low HP, level up, enemy phase, danger zone) don't show; they
  are in *Guides*. *Claude's starting rule.*
- There is no separate "Tips: On/Off" option; the level replaces it
  (Nick: "answered above"). *Reset tips* in Options stays.

### The guide

- A **new mascot**, outside the story: the **Fox Miko** face from the
  bought Tiny Tales *Heroes 2: Rebellious Souls* pack (its 8 expressions).
  Not the Shuyi cameo; not Hollis.
- **Outside the story for now**: the guide talks only to the player, never
  appears in a scene, isn't in the party and isn't touched by deaths or
  plot. Whether the guide is later tied into the story (Nick's example:
  Angel in Borderlands 2, the voice guiding the player who turns out to be
  part of the plot) is decided **much later**.
- The guide **speaks every lesson and every tip**, in a box with the face
  beside the text (the mockup "A1": the tip box over the map, the face at
  twice its size on the left, the guide's name as the box's title). The
  conversation box with the big portrait is not used for tips.
- The guide's **name** and voice: **Vix**, the dry insider (ticket 0731,
  Nick's words in `docs/story/characters/guide.md`). The box's title is
  her name, from the names table. Claude writes every line
  (`docs/story/writers-guide.md`).

### The lessons (forced, as a checklist)

- A lesson is a box from the guide with a **checklist**: each item is
  something to do ("move the cursor up", "move it left"…), ticked as the
  player does it, **in any order**. The lesson closes itself when every
  item is ticked.
- Lessons happen **inside Chapter 1's battle** (and later battles), each
  **the first time it matters**, not in a practice battle and not all at
  the start.
- **Forced**: while a lesson is open the game works as usual under it (the
  items are real actions), but **the turn can't end until the checklist is
  done**. *Claude's starting rule* for how "forced" works.
- Each lesson shows once per campaign. *Claude's starting rule.*

**Basic controls** (Full only):

- Moving the cursor: up, down, left, right.
- Choosing a unit, moving it, and its action menu.
- Attacking: the forecast, then confirming.
- The next unit and a unit's details.
- Ending the turn.

**Different here** (Full and Short; Nick agreed to Claude's list):

- **The keyboard controls**: the cursor on keys, and that every key can be
  changed in Options.
- **Rewind**: undoing turns, with a limited number of charges per map.
  The lesson opens and closes the rewind screen; it doesn't spend a charge.
  *Claude's starting rule.*
- **Threat arrows**: which enemies can reach the tile you're moving to.
- **Combat Arts**: their own menu, and that some cost uses per battle.
- **Terrain magic**: a spell can change a tile (Fire can burn a forest).
  Chapter 1 doesn't script it (`chapter-1.md`, *Extras*), so this lesson
  waits for the first time it can happen, in whichever battle.

The exact items of each lesson, and when each starts, are written by the
build tickets (0456, 0457) and listed there as *Claude's starting rules*
for Nick to veto.

### Tips

- Today's tips stay (`assets/data/tips.ron`), spoken by the guide, at
  Full.
- Nothing explains Combat Arts, skills, magic, terrain, rewind or threat
  arrows today. Rewind, threat arrows, Combat Arts and terrain magic get
  lessons (above). The rest get **new tips at Full**: **skills**, **magic**
  (casting and healing) and **terrain** (movement costs and bonuses). A
  topic with a lesson doesn't also get a tip. *Claude's starting rule.*

### Guides (in the menu)

- A **Guides** entry in the battle's menu opens a list of every lesson
  and tip; picking one shows its text, to "refresh a player's memory via a
  quick look." Like Fire Emblem Engage's *Guides* (System menu).
- It works at every level, None included. Every topic can be read, seen
  or not. *Claude's starting rule* (the mockup greyed out unseen ones;
  dropped so a player at None can read them all).

## How other games do it (checked 2026-10-14)

- *The Legend of Zelda: Ocarina of Time*: Navi, the fairy who calls out
  hints. *Microsoft Office*: Clippy.
- *Advance Wars* (2001): Field Training, 14 lesson maps with Nell; you can
  skip ahead.
- *Fire Emblem: The Blazing Blade*: Lyn's chapters are the tutorial,
  required on a first playthrough, skippable after.
- *Fire Emblem Awakening*: the prologue is a story map with tutorial
  pop-ups on a first playthrough.
- *Fire Emblem Engage*: tutorials can be read again any time under System
  → *Guides*.
- *Borderlands 2*: Angel guides the player over the radio and is later
  revealed as part of the plot.

## Open sub-questions (deferred)

- Tying the guide into the story (Angel-style): much later (Nick).
- The guide's name: 0731.

## Built by

- **0455** the guide's face in the tip box.
- **0835** the familiarity question at New Game.
- **0456** checklist lessons (the box, ticking, the turn waits).
- **0457** the lessons themselves.
- **0458** the new tips and the *Guides* list.
- **0731** the guide's name and lines.
