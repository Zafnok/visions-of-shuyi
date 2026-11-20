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
  Nick's words in `assets-private/story/characters/guide.md`). The box's title is
  her name, from the names table. Claude writes every line
  (`assets-private/story/writers-guide.md`).

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
- **A lesson is only up when it can be done that turn** (0464, Nick:
  "just make sure the tips are actually achievable on the turn they
  present themselves"). A lesson that asks a unit to do something opens
  only if some unit still to act this turn could do it; if none can when
  its turn to open comes, it doesn't open and waits until it starts again,
  that turn or a later one. An open one is put away, unfinished, the
  moment no unit left could do what it still asks, and comes back on a
  later turn. "Could do it": Attacking, some unit left could reach an
  enemy; Combat Arts, some unit left has an art it could use from a tile
  it can reach (for its attack item, Overwatch, which is no attack,
  doesn't count); Spells that change tiles, some unit left could aim a spell
  at a tile; Selecting a unit and Threat arrows, any unit left. For
  example: on turn 1 the Sergeant attacks the Brigand and nobody else can
  reach an enemy, so Combat Arts waits for a later turn and the turn can
  end. Lessons that need no unit (the cursor, the keys, the next unit,
  ending the turn, rewind) are unchanged. *Claude's starting rule.*

- **The end of the lessons** (0465, Nick: "you're on your own for the
  rest, if you need any help, here is where you find the Guide", and "ask
  if the player is OK with their position or wants to retry battle with no
  penalty, or retry tutorial with no penalty"). When the last lesson the
  tutorial level shows is done, the guide says so, points at Guides in the
  map menu, and asks: **Keep going** (the cursor starts here), **Restart
  the battle** (back to turn 1, the lessons stay done), or **Restart with
  the lessons** (back to turn 1, the lessons finished in this battle show
  again). Neither restart costs anything: a restart spends no rewind
  charge and counts nowhere. Never at None. *Claude's starting rules:* the
  box waits until the map is back to browsing, so it doesn't cut into what
  the last lesson had the player doing; it shows once per campaign, so
  finishing the lessons a second time doesn't ask again; it needs a
  campaign, so the debug Quick Battle has none. Since Spells that change
  tiles and Combat Arts wait for their first chance, it can come on a
  later turn or in a later battle.

**Basic controls** (Full only), in the order turn 1 shows them:

- Moving the cursor: up, down, left, right. It is the first box of the
  battle: no tip comes before it (1304, below).
- The next unit and a unit's details.
- Moving a unit: choosing it, sending it to a blue tile, and its action
  menu. Its line also says what the red tiles are, so an enemy in reach is
  taught here, before Attacking (1304).
- Attacking: the forecast, then attacking **or backing out**: either ticks
  the same item (1304).
- Swapping weapons: the first forecast that can swap, ticked by swapping.
- Zooming the map: zoom out and zoom in, each ticked.
- Ending the turn: on turn 1, once the first unit has acted.

**Turn 1 at Full, after Nick played the demo (1304).** Nick, 2026-11-20,
playing Chapter 1 as a writer applicant would (his words):

> 1. "I don't know why Vix's tutorials were collapsed they were better
>    before at least the earlier ones like when it asks to press one key
>    in each direction and tracks which have and have not been completed
>    (or I guess this is just a complaint for the very first Vix tip bc
>    she goes into it right after I think its wholly redundant)"
> 2. "Vix's attack tutorial doesn't track backing out as credit same as
>    attack, only attack counts as credit"
> 3. "Vix's zoom tutorial should force a zoom out and a zoom in"
> 4. "Vix when describing acronyms like DEF and AVO and EXP should say the
>    whole word not the shorted"
> 5. "Vix's moving tutorial should come before the attacking tutorial and
>    track movement as a checkmark"
> 6. "enemy in reach tutorial should play probably at same time as
>    movement tutorial or before attacking or before moving"
> 7. "swapping weapon vix tutorial should track progress"
> 8. "Vix's end turn tutorial should play earlier than exhausting all
>    units since the player might wanna be tactical and not use all units
>    every turn. Or it shouldn't play at all past turn 1 basically bc you
>    know player already knows the info."

So:

1. The *Your move* tip that opened the battle is gone: the cursor lesson
   is the first box.
2. The Attacking lesson's second item ticks when the forecast closes,
   whether the player attacks or backs out.
3. Zooming the map is a lesson (it was a tip): zoom out and zoom in.
4. What Vix **says** spells stats out (Defense, Avoid, Experience,
   Movement); the screen's short labels (DEF, AVO, EXP, Mov) stay.
5. and 6. The *Moving a unit* and *Enemy in reach* tips are folded into
   the Moving a unit lesson, which comes before Attacking: it ticks
   selecting, moving (sending it to a blue tile) and the menu, and its
   line says red tiles are what the unit can hit, an enemy on one in reach.
   Before, both tips missed their moment under the lessons and only came
   after Attacking.
7. Swapping weapons is a lesson (it was a tip), ticked by a swap.
8. Ending the turn opens on turn 1 back on the map once the first unit
   has acted, and never after turn 1. Ending the turn early still ticks
   it.

*Claude's starting rules* (1304), for the Act 1 playtest (1064):

- Point 2 is read as "backing out gets the same credit as attacking": one
  item, either ticks it (the ticket's draft had two items, both needed).
- The ending-the-turn lesson steps aside for any other lesson that comes
  up while it is open (Attacking, Swapping weapons, Zooming), and comes
  back once that one is done, so it never holds the rest of turn 1's
  lessons until the turn ends. Not done by the end of turn 1 (it can't
  be: ending the turn does it), it is put away for good.
- The zoom lesson opens the first time the player is back on the map
  with no lesson in play but ending the turn; with glyphs, which don't
  zoom, it waits until pictures are on, and is put away if the look
  changes to glyphs while it is open.
- Swapping weapons opens only if a unit still to act could attack an
  enemy with two weapons or spells from one tile.

**Different here** (Full and Short; Nick agreed to Claude's list):

- **The keyboard controls**: the cursor on keys, and that every key can be
  changed in Options.
- **Rewind**: undoing turns, with a limited number of charges per map.
  The lesson opens and closes the rewind screen; it doesn't spend a charge.
  *Claude's starting rule.*
- **Threat arrows**: which enemies can reach the tile you're moving to.
- **Combat Arts**: their own menu, and that an art is an attack, not a
  buff (Nick, 2026-11-08, after a playtest: "We should make it clear Arts
  are special attacks NOT just buffs that don't end your turn. My friend
  thought Arts are buffs that do nothing bc they end your turn. But they
  are buffed special attacks (attack tied to the art). Make it more clear
  probably Vix forced tutorial or smth"). Two items: open Arts in the
  unit's menu, then attack with an art (or a weapon active from that
  menu); a plain attack ticks nothing. The text says an art is the attack
  itself, every strike of that one fight boosted, paid in weapon wear
  (uses per battle are spells' and non-attack actives', so it no longer
  says them). *Claude's starting rules* (ticket 0490), for the Act 1
  playtest.
- **Terrain magic**: a spell can change a tile (Fire can burn a forest).
  Chapter 1 doesn't script it (`assets-private/story/design/chapter-1.md`, *Extras*), so this lesson
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
- **Zooming the map** (0466; Nick, playtest round 3: "ensure the guide has a
  tip for zoom on first turn"), at Full, when the map is drawn with pictures.
  A lesson since 1304 (above). *Claude's starting rule.*
- **Wading** (0483; a friend's Act 1 playtest, passed on by Nick
  2026-11-07: wading is "a bit unintuitive"), at Full, the first time a
  selected unit's move range reaches a river (`water`): a foot or flying
  unit, never a mounted or armored one. It says foot units can wade at 5
  Mov a tile, riders and armor can't, and the water gives +10 AVO and no
  DEF.
- **Weapon kinds** (0489; Nick's Act 1 playtest, 2026-11-08: "only I
  know that spears are good against horses"), at Full, the first time an
  attack's forecast has a strike effective by its weapon kind (a spear on
  a mounted unit, a bow on a flying one), the player's or the enemy's
  counter; not a spell on an element its target is Weak to. It says a
  spear's might counts double against riders and a bow's triple against
  fliers, for both sides, and that the forecast's `!` means it applies. At
  every level the forecast's note (`Spear ×2 vs Mounted`) and the unit
  info screen's weapon ranks say the same. *Claude's starting rule.*
- **Swapping weapons** (0463; Nick, playtest round 3: "swapping weapons on
  forecast menu feels quite hidden as a feature [...] the guide should
  probably mention it"; he picked a tip over a line in the Attacking
  lesson; a lesson of its own since 1304, Nick: "should track progress"), at Full, the first time a forecast opens where left / right
  swap the weapon (the unit has more than one weapon, or a weapon and a
  spell, to attack with there). She says the unit carries more than one
  weapon and each can be tried right in the forecast, the numbers changing
  to match; the instructions name the keys (`Swap weapon: {CursorLeft}
  {CursorRight}`). The forecast itself shows it at every level
  (`combat-arts.md`, *Showing that left / right swap the weapon*).
- **A tip or a lesson:** one fact with nothing to tick off is a tip; a
  lesson is for something the player does, step by step. *Claude's
  starting rule* (0483).

### How a tip box is put away (0495)

Nick, 2026-11-12, after playing the Chapter 1 demo: "Vix's tutorial steals
priority (D/F confirm / cancel) over other UI elements. So when she asks
you to go to options for example, several of her tutorials then play one
after another and D both closes options and her tutorial so you're trying
to go back out of options to do her tutorials and instead just skipping
them one after another."

Shown the box over the map menu after leaving Options (both looks) and
three choices (A. modal but safe, like *Fire Emblem: Three Houses*'
tutorial pop-ups; B. out of the way in a corner, the battle taking every
key; C. a key of its own), he picked **A. Modal, but safe**:

- A tip takes the keys until **Confirm or Cancel** closes it, as before,
  and only a press made while it is up closes it.
- A tip comes up **only in its moment** (selecting a unit, the forecast…).
  Nick, 2026-11-17, after playing ("why are Vix's tip triggers so random,
  they don't flow very well, like they pop up after a delay and stuff"),
  replacing the queue and the wait for a press that 0495 had: "how about
  just try again showing on its moment rather than randomly delaying in a
  queue waiting for a key press" (1292). So:
  - If a tip's moment passes while it can't show (a lesson, another tip
    or a screen over the battle is in the way), it isn't kept waiting: it
    stays unseen, and shows **the next time its trigger fires** with
    nothing in the way. For example: selecting a unit for the first time
    while a lesson is up brings up no tip then; the next time a unit is
    selected with no lesson up, *Moving a unit* comes up.
  - **No tip waits for a press on the map.** When one closes, another
    whose moment is still on comes up at once: selecting a unit that has a
    skill brings up *Moving a unit*; closed, *Skills* follows straight
    away.
  - Back from Options (or any screen over the battle), no tip comes up
    that fired before or under it; the D that closes the map menu closes
    the menu.
- *Claude's starting rules:* a lesson done goes straight to the next
  lesson, never to a tip between them. A combat's playback doesn't end a
  tip's moment: the level-up tip, fired by a combat, shows once the map
  is back after it. The box at the end of the lessons still waits for a
  press on the map after the last lesson (0465, 0495), so a key meant for
  the lesson never answers it.

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

- Tying the guide into the story (Angel-style): much later (Nick). In
  playtest round 3 (2026-10-16) he suggested a small role, map hints on
  unusual maps: ticket 0064.
- The guide's name: 0731 (Vix). Letting the player name her, and the lead,
  from suggestions or their own: ticket 0065.

Playtest round 3 (2026-10-16), Nick on what's built: the familiarity
question's wording and place are right and both short sets are right;
holding End turn while a lesson is open is right; the lesson box moving
out of the cursor's way: "this is a really great feature that I didn't
even ask for"; Vix's voice is right, and "she really ties ch1 together as
an intro to a quality game". To fix: a lesson that can't be done that turn
(0464), the end of the lessons pointing at Guides with a fresh start
(0465), a zoom tip (0466), tips lost before they show (0467), Tutorial
None not stopping what's queued (0841), the forecast's weapon swap (0463),
pictures in Guides (0469), a profile nudge on the title (0842), Vix's ears
(0468).

## Vix is voiced

Decided in 0043 (`voices-languages-and-script.md`, *Voices*). What she says and the
objectives are separate: her line is spoken and never names a key, and what names
a key or a button is silent text. She has lines all through the tutorial. Tickets
0475 and 0476.

How the split works (0475):

- **A lesson:** its text is her line; its checklist items are the objectives,
  and they name the keys.
- **A tip:** its text is her line; under it, after a blank row, its
  instructions (`how`, optional) name the keys, e.g. "Rewind: r". A tip with
  nothing to press has none.
- **Her line teaches on its own.** Heard without the box, it still says what
  to do ("Pick up a unit and its reach lights up"); the keys are only how.
  Every rule a line used to teach is still taught by the line or by the
  objectives.
- **Guides** shows the same: her line, then the items or the instructions.
- The content check fails a lesson or a tip whose text holds a key
  placeholder.

## Built by

- **0455** the guide's face in the tip box.
- **0835** the familiarity question at New Game.
- **0456** checklist lessons (the box, ticking, the turn waits).
- **0457** the lessons themselves.
- **0458** the new tips and the *Guides* list.
- **0731** the guide's name and lines.
- **0465** the guide's box at the end of the lessons.
- **0466** the zoom tip.
- **0495** how a tip box is put away, and one at a time.
- **1292** a tip that misses its moment waits for the next one.
- **1304** turn 1's lessons in Nick's order, ticking what the player does.
