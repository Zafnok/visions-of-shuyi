# Turn structure

Decided: 2026-09-25
Source: ticket 0002

## Nick's words

> **Q1. Who acts when during a battle?** "Phases, like FE (Recommended)"
>
> **Q2. Moving and acting: how does a unit's turn work?** "fire emblem standard
> ONLY however certain combat skills can enable movement such as that one bow
> skill from FE that lets you move 1 tile away... no in-built Canto"
>
> **Q3. Should maps be able to spawn enemy reinforcements on set turns?**
> "Yes, never act on arrival (Recommended)"

Follow-ups:

> **How should your phase end?** "auto-end as option but have it enabled by
> default and it should have a shortcut to toggle auto-end"
>
> **Should there be a third side: allied/neutral NPC units that move on their
> own (green units in FE)?** "Yes, Other phase (Recommended)"
>
> **Turn limits: how should turn counts interact with map objectives?**
> "Per-map, optional (Recommended)"

So: Fire Emblem phases (Player → Enemy → Other), FE move-then-act with no
built-in Canto (post-action movement exists only as specific skills),
reinforcements that never act on the turn they arrive, auto-end on by default
with a toggle shortcut, and optional per-map turn limits.

> **Changed 2026-09-28 (ticket 0420):** Nick: "turn auto-end off by default".
> Auto-end is now **OFF** by default; the toggle shortcut stays.

Everything marked *tunable* / *Claude's starting rule* is a default Claude
chose where Nick's answer didn't say; Nick may veto it and balance tickets may
change it without asking. Unmarked rules are Nick's choices.

## Sides and phases

| Phase | Units that act | Controlled by | Banner |
| ----- | -------------- | ------------- | ------ |
| Player phase | `Player` faction | the player | `PLAYER PHASE` |
| Enemy phase | `Enemy` faction | AI | `ENEMY PHASE` |
| Other phase | `Ally` and `Neutral` factions ("green" units) | AI | `OTHER PHASE` |

- Other-phase units are AI-controlled. `Ally` units fight the enemy alongside
  the player; `Neutral` units (e.g. villagers) fight no one. Hostility between
  factions is the `Faction::is_hostile_to` table from ticket 0302.
- The player can never command Other-phase units.

## Exact sequence

```
battle start: turn = 1
loop:
    Player phase  (turn)
    Enemy phase   (turn)
    Other phase   (turn)
    turn += 1
```

Each phase:

1. **Start of phase:** every unit of the phase's factions becomes *ready*
   (`acted = false`). Then that phase's reinforcements for this turn arrive
   (see below) already *done*. Then turn-limit checks that happen at phase
   start (see Objectives). Then the banner shows.
2. **Actions:** ready units act one at a time, in any order the controller
   chooses (player: any order; AI: its own ordering, ticket 0501).
3. **End of phase:** see "Ending a phase" below.

A phase whose factions have **no living units** (and no reinforcements
arriving this phase) is skipped entirely: no banner, no pause. The turn
counter still advances after the Other phase slot, whether or not it ran.

The battle can end in the middle of any phase: objectives and defeat are
checked after every action and at every phase boundary; the moment an outcome
is decided, nothing further happens.

## A unit's action

FE standard, and **only** FE standard. There is **no built-in Canto** and no
split movement for anyone.

1. **Select** a ready unit.
2. **Move** it to any tile it can stop on (0 tiles = stay put). Moving is a
   preview: the player can cancel the move freely until they choose an action.
3. **Choose one action** from the action menu: `Attack`, `Item`, `Wait`, plus
   any action the map or other design docs add (e.g. `Seize` on a seize tile,
   `Talk`, `Trade`, `Heal`/magic per `magic.md`, `Items` per
   `weapons-and-items.md`, `Arts` per `combat-arts.md`).
4. Choosing the action **commits** the move and the action together. They
   can't be undone (except by rewind, per `death-and-difficulty.md`).
5. The unit is now **done** (`acted = true`, drawn dimmed) until the start of
   its side's next phase.

**Pointing at an enemy** (Nick, 2026-09-29, ticket 0428). With a unit
selected, moving the cursor onto an enemy it can attack after moving aims the
path at the tile it will attack from, and Confirm attacks straight away
instead of going through the action menu.

> "moving cursor from unit selected to an enemy in attack range should move it
> to an appropriate tile and open the menu to select an attack / art that is
> allowed w/in range."

- **Which tile:** keep the arrow the player drew if it already ends on a tile
  the unit can attack the enemy from; otherwise the arrow jumps to the closest
  such tile (cheapest move; *Claude's tie-break:* shortest path, then topmost,
  then leftmost).
- **What opens** (Nick, 2026-10-03, ticket 0430; before that a weapon list
  came first): after Confirm the unit walks there and the **attack forecast
  on that enemy opens at once**, with what the unit has **equipped**. No list.

  > "straight to forecast, for weapons too, with keys to quickly go thru
  > potential weapons/spells without backing out of forecast screen."
  >
  > A unit with a sword and a spell that both reach: "Whatever is equipped,
  > no list"
  >
  > Which keys swap: "like three houses, but up/down should not pick an art.
  > the arts should not be in the same menu. we should have an arts menu.
  > that was something i didnt like about the current quick battle setup
  > anyway"

  - **Spells count too.** A unit whose attack spell reaches the enemy can
    point at it the same way. Example: the Test Mage (Fire equipped, also
    knows Frost) points at the Frost Elemental: it walks to two tiles away
    and the forecast of Fire opens.
  - **Left / right swap** the weapon or spell, like Fire Emblem: Three
    Houses, through everything that reaches that enemy from that tile:
    weapons first (slot order), then attack spells, wrapping round. Example:
    `Iron Sword → Steel Sword → Fire → Frost → Iron Sword`.
  - **Up / down change the target** in this forecast, and so do the
    previous / next unit keys (Nick, 2026-10-12, ticket 0442: "Same
    everywhere", the offered option "left/right swap the weapon in every
    weapon forecast, up/down change the target, no weapon list after
    `Attack`").
  - **The arts are not in the forecast** (Nick, ticket 0442: "Action menu
    only"). To use a Combat Art on an enemy pointed at: Cancel, then `Arts`
    in the action menu (`combat-arts.md` → *The arts menu and the
    forecast*). A spell active is under `Magic` (`Fire + Overcast`).
  - *Claude's starting rules (Nick can veto):*
    - If the equipped weapon or spell can't reach the enemy from that tile,
      the forecast opens with the first one that does (weapons before
      spells). Example: a mage with a sword equipped points at an enemy two
      tiles away: the forecast opens with Fire.
    - Only a plain attack counts: an enemy that only an art reaches (Close
      Shot on the one beside an archer) can't be pointed at from that tile.
    - With only one weapon or spell that reaches, left / right change the
      target too.
    - After changing target, a swap keeps that target if the new weapon
      reaches it; if not, the cursor goes back to the enemy pointed at.
    - Cancel goes back to the action menu at that tile, on `Attack` (on
      `Magic` if the forecast showed a spell), and Cancel again to the path.
    - The help bar says `cast` instead of `attack` when a spell would open.
    - The forecast opened from `Attack` swaps the same way, between the
      unit's weapons that reach someone (ticket 0442: there is no weapon
      list any more). `Magic` keeps its spell list, and a spell's forecast
      opened from it doesn't swap.

**Pointing at an ally** (Nick, 2026-10-14, ticket 0451). With a healer
selected, moving the cursor onto a hurt ally it can heal after moving aims
the path at a tile a heal reaches them from, like pointing at an enemy.

> "healers should have the same hover over friendly unit, click, means
> heal, like units have for enemy attacks..."

Asked with Maud (Heal, range 1, +16; Vigil Light, range 1–2, +12) and
Hollis at 10/24 HP:

> **Where does she walk?** [A beside him, for Heal; B the closest tile any
> heal reaches:] "Closest tile any heal reaches"
>
> **What opens after Confirm?** [A the heal preview at once, left / right
> swap Heal and Vigil Light; B a list of her heals first:] "A list of her
> heals first"
>
> **Up / down in the preview change the target, as in the enemy
> forecast?** "Yes, same as enemies"
>
> **Only a hurt ally is pointed at; potions aren't used this way; a unit
> that can attack and heal attacks enemies and heals allies:** "Agree with
> all three"

- **Which tile:** the same rule as an enemy: keep the arrow the player drew
  if a heal reaches the ally from its end; otherwise the cheapest tile from
  which **any** heal reaches (same tie-break). Example: Maud two tiles from
  Hollis stays where she is, since Vigil Light reaches; to use Heal, draw
  the arrow next to him first.
- **What opens:** Confirm walks there and opens **the list of the unit's
  heals** (Heal, Vigil Light; no attack spells). Choosing one opens its
  preview on that ally (`Heal on Hollis: HP 10 → 24`); Confirm heals.
- **Up / down** (and the cursor keys, and the previous / next unit keys)
  step to the other hurt allies that heal reaches from there, as when a
  heal is cast from `Magic`.
- **Only a hurt ally** is pointed at. Over an ally at full HP the cursor
  does what it does today (a tile the unit can't stop on).
- **Heal spells only.** A potion is not used by pointing; a unit that only
  carries potions behaves as before.
- **Enemies are attacked, allies healed.** A unit that can do both does
  each as before.
- *Claude's starting rules (Nick can veto):*
  - In the list, a heal that can't reach the ally pointed at from that
    tile is dimmed (Heal, two tiles away), and the list opens on the first
    one that can.
  - The list opens even when only one heal is listed (the Test Mage, or a
    healer that knows only Heal), so it always looks the same.
  - Cancel in the preview goes back to the list; Cancel in the list to the
    action menu at that tile, on `Magic`; Cancel again to the path.
  - The help bar reads `heal` while the cursor is on a hurt ally the unit
    can heal (`arrows move · f heal · d cancel`).
  - An ally of the green side counts as an ally, as for `Magic`.

**Skill-granted movement.** Specific combat skills may give a unit movement
*after* its action, like Fire Emblem's bow skill that steps the archer 1 tile
away after attacking. These are not a general rule: each skill states exactly
what movement it grants (e.g. "after attacking, move 1 tile away from the
target"). Which skills exist and who gets them is decided by
`progression.md` (0005) and `weapons-and-items.md` (0003). The turn system
only guarantees that an action may end with a skill-defined extra move, after
which the unit is done.

## Threat arrows: who can attack this tile

Playtest round 3 (2026-10-16): when the arrows show is "Right", and the
danger zone is now used "rarely, this is great, danger zone should be for
advanced tacticians not mandatory for every playstyle". Nick asked to
"ensure the threat arrows are accurate to if the enemies have obstacles
(like our allied units moving to block them)": they are. An enemy's reach
is worked out with every unit of yours as a wall it can't walk through,
and the unit you are moving counted on the tile you're moving it to
(`threats_at`, `crates/core/src/movement.rs`). An enemy that never moves
uses only its weapon's range from where it stands.

Decided 2026-10-12, ticket 0055. Nick asked for it while playing Chapter 1
(ticket 0804):

> "when moving or proposing to move into an enemy's attack range, it should
> illuminate or draw slim arrows from enemy to the tile or unit you're
> moving to. Newer fire emblem games do this. So you don't need to rely on
> danger zone as much."

Asked with mockups on the Chapter 1 map (the look is in
`look-and-feel.md`, *Threat arrows*):

> **What marks the enemies?** [A curved lines, B straight arrows, C a mark
> over each enemy, D a red frame around each enemy, E curved lines and
> marks:] "B + C"
>
> **Which enemies get a line?** [A everyone who could attack that tile,
> guards included; B only those the enemy AI would send at that unit:] "A"
>
> **When does it show?** [A only while a unit is selected and the cursor is
> on a tile it can move to; B also whenever the cursor rests on one of your
> units:] "B"
>
> **When an enemy that reaches the tile is off the screen?** [A the arrow
> comes in from the screen's edge; B nothing; C the arrow and a count:] "A"

The rule:

- **Which enemies:** every enemy that **could attack the tile in the next
  enemy phase**: it can move to a tile its weapon or attack spell reaches
  from. It doesn't matter whether the enemy's AI would choose to. Example:
  on turn 1 of Chapter 1 the Sergeant proposes the road tile 5 east of
  him; the Brigand beside it, the Archer, the Brigand to the east and the
  Raider north of the road bridge all get an arrow.
- **Enemies that wait in place count** (Chapter 1's bridge and fort guards
  attack whoever comes within reach): the Raider above stands guard and
  still gets its arrow. An enemy that never moves (Harl in the keep)
  counts for the tiles it can attack from where it stands.
- **When it shows**, on the player's phase:
  1. **A unit is selected and the cursor is on a tile it can move to**
     (the tile it stands on included): the arrows point at that tile.
  2. **Nothing is selected and the cursor rests on one of your units:**
     the arrows point at that unit where it stands.
  A tile no enemy can attack shows nothing.
- **No setting and no new key.** Nick wasn't asked for one and didn't ask.
- The danger zone and picking an enemy to see its range are unchanged.
- *Claude's starting rules (Nick can veto):*
  - "Can attack the tile" is worked out as the danger zone works it out,
    with one difference: the selected unit is counted as standing on the
    proposed tile, not where it started. (Built in ticket 0448. A second
    difference turned up there: the danger zone tints the tiles an enemy
    that never moves could attack if it walked first, so it is wider
    round Harl than his arrows. Ticket 0450 fixes the danger zone.)
  - With a unit selected and the cursor on an enemy it can attack (the
    cursor aims the path at the tile it will attack from, see *Pointing
    at an enemy*), the arrows point at that tile.
  - The arrows stay while the action menu is open on the proposed tile,
    and go when the forecast opens. They also stay while the unit walks
    to the tile and in the lists the action menu opens (arts, spells,
    skills, items, equip); they go whenever a target is being picked
    (ticket 0448).
  - With a unit selected and the cursor on a tile it can't stop on (out
    of its range, or a tile another unit stands on), nothing shows
    (ticket 0448).
  - A unit of yours that has already acted shows them too (case 2).
  - Other-phase (green) units don't: only your own.
  - Only attacks count: an enemy that can only heal gets no arrow.

## Ending a phase

**Player phase:**

- **Manual:** the player can end the phase at any time from the map menu or
  the End Turn key (`Space`; `Space` again confirms, see `controls.md`). If any
  units are still ready, a confirmation asks first
  (`End turn with N units ready?`); units still ready simply don't act.
- **Auto-end:** when **auto-end is ON** and the last ready player unit becomes
  done, the player phase ends immediately (no confirmation).
- **Auto-end is OFF by default** (Nick changed this in ticket 0420). It's a player setting, saved with the other
  options, and has a **shortcut key to toggle it** during battle (default key
  `Shift+Space`, decided in `controls.md`; rebindable), which shows a brief `Auto-end: ON/OFF`
  message. The current state is also shown in the help bar.
- With auto-end OFF and every unit done, nothing happens until the player ends
  the turn manually (no confirmation needed then, since no units are ready).

**Enemy and Other phases:** always end automatically when the AI has no more
actions to take (every unit acted or chose to wait).

## Reinforcements

Maps may schedule reinforcements: a list of `(turn, faction, units at tiles)`.

- Reinforcements for turn `N` arrive at the **start of their own side's phase
  on turn `N`**, and arrive **already done**: they never act on the turn they
  appear. Their first action is in their side's phase on turn `N + 1`.
- So enemy reinforcements scheduled for turn `N` appear at the start of Enemy
  phase `N`, and the player always gets Player phase `N + 1` to react before
  they move.
- Any side can have reinforcements (enemy waves, allied militia arriving,
  player units joining), same rule for all.
- *Claude's starting rule:* if an arrival tile is occupied, that reinforcement
  waits and tries again at the same point on the next turn (blocking a
  reinforcement point is a legitimate tactic, as in FE). Other reinforcements
  in the same wave still arrive.
- Arriving reinforcements are visible immediately (map, danger zone) and the
  arrival is an event the UI can show (camera pans to them).

## Turn limits and objectives

The current turn number is always visible (map menu → Objective, phase
banner). Turn limits are **optional and per map**; most maps have none.

- **Survive N turns** (an objective by itself): the player **wins** when turn
  `N`'s last phase ends, i.e. at the moment `turn` would become `N + 1`, if
  the player hasn't lost by then.
- **Any other objective "within N turns"** (Rout, Defeat boss, Seize, ...):
  the player **loses** if the objective isn't complete when turn `N`'s last
  phase ends. Completing it at any point up to then wins.
- A map shows its limit in the objective text, e.g. `Seize the gate (Turn 3/8)`.
- Loss conditions from `death-and-difficulty.md` (0006) are checked the same
  way (after every action and at every phase boundary).

## Act 1's battle rules: escape, save the village, bosses who get away

Decided: 2026-10-22
Source: ticket 0069 (built by 0319; the village rule by 0313)

Act 1 (`docs/story/outline.md`) needs four battle rules Chapter 1 never
did. Nick's words (asked with the question pop-up, options from real games):

> **Chapter 6, escape:** "Everyone out, one by one (Recommended)"
>
> **Escape turn limit:** "No limit, rising pressure (Recommended)"
>
> **Chapter 4, Dace flees:** "Retreats at the killing blow (Recommended)"
>
> **Chapter 8, Dace can't be killed:** "probably goes to "0" but then
> becomes grayed out / takes no more actions / can't be targeted (by ally
> or enemy)"
>
> **Does his body still block his tile?** "Blocks his tile (Recommended)"
>
> **Villages in battle:** "if it has a story beat attached, it should be a
> real race not an optional one... like save the village with a minimum
> number houses allowed to pass, and past that get extra rewards at the
> end..."
>
> **How is a house saved?** "Both" (reach it first and evacuate it, or keep
> raiders off it)
>
> **Villages with no story attached:** "None (Recommended)"

### Escape (Chapter 6, the Ashfields)

Example: the party crosses the Ashfields to the Brennish lines. Aske, the
fastest, reaches the lines on turn 3, picks `Escape` and is gone, safe.
Tamsin follows on turn 5. Maud is slow and the echoes are closing in, so the
lead stays back to cover her. The battle is won on the turn the last unit
still standing steps out.

- The map marks an **exit area** (a set of tiles, e.g. the Brennish lines).
  The objective reads like `Get everyone to the Brennish lines (2/6 out)`.
- A player unit that **ends its move** in the exit area may choose
  **`Escape`** as its action. It leaves the map and is **safe** from then
  on: it can't act, can't be targeted, doesn't block tiles and never falls.
- **Win:** when no player unit is left on the map and at least one escaped,
  i.e. every unit still standing is out. A unit that falls on the way falls
  as usual (`death-and-difficulty.md`: Classic dies, Casual retreats); it
  doesn't have to escape. Nobody is "left behind".
- **No turn limit** (Nick). The pressure comes from reinforcements (echoes
  and Hounds arriving in waves, *the chapter ticket sets them*), so waiting
  gets dangerous but never loses on its own.
- *Claude's starting rules:* the lead may escape before the others (the
  battle goes on until everyone is out); once escaped the lead can't fall,
  so the lord rule can't lose the battle after that. `Escape` gives no EXP.
  Green (allied) units don't have to escape. Escaped units count as standing
  on the results screen.

### Save the village (Chapter 2, Kell's Ford)

Villages appear in a battle **only when the story is about them**, and then
as a real race, never an optional extra (Nick). There are **no** plain
"Visit for a gift" villages on ordinary maps (Nick: "None"); that rule in
`weapons-and-items.md` is replaced.

Example: Kell's Ford has five houses and raiders heading for them. The
objective is `Defeat the lieutenant` and `Save at least 3 of 5 houses`.
Aske runs to the nearest house and evacuates it (saved for
good). The lead kills the raider next to the second one before his turn
ends, so it still stands. A raider reaches the third and it burns (lost).
At the end, four houses stand: three meet the minimum, the fourth adds an
extra reward.

- The map marks **houses** (each one `house` tile, `terrain.md`, *Building
  tiles*; "the house's door" below is that tile) and a **minimum** to
  save. The objective line shows both, e.g. `Save at least 3 of 5 houses
  (1 saved, 1 lost)`, next to the battle's main objective (Defeat the boss,
  Rout, ...).
- **Saving a house** (Nick: "Both"):
  - **Reach it first:** a player unit on an unsaved house's door picks
    **`Evacuate`** (its action). The house is saved for good (the villagers
    get out; a line may play).
  - **Keep raiders off it:** a house that is still standing when the battle
    is won counts as saved too.
- **Losing a house:** an enemy the map marks as a **raider** that ends its
  turn on or next to an unsaved house's door sets it burning. It is lost for
  good. (*Claude's starting rule:* "next to" means the four tiles beside the
  door, as for attacks.)
- **Loss:** the moment too many houses are lost for the minimum to be met,
  the battle is lost (game over: Retry or Rewind, `death-and-difficulty.md`).
- **Extra rewards:** each house standing at the end **past the minimum**
  adds a reward when the battle is won (Nick: "past that get extra rewards
  at the end"). The chapter ticket picks the rewards (items or gold).

### A boss who retreats (Chapter 4, Veyne Hall)

Example: the lead's sword brings Dace from 9 HP to 0. Instead of dying he
says "Not here. Not in this house.", and leaves the map. `Defeat Dace` is
complete, the battle is won, and the lead gets the EXP of a kill.

- The map marks a unit **"retreats instead of falling"**. When its HP
  reaches 0 its defeated line plays (`battle-scenes-and-recruitment.md`,
  *Boss lines*) and it leaves the map, alive.
- It counts as **defeated**: for the objective (`Defeat Dace` is won), for
  EXP (the striker gets the EXP of a kill) and on the results screen.
- The forecast is the normal one: it shows the hit that brings him to 0,
  like any boss, as in FE Three Houses' bosses who retreat when beaten.

### A boss who can't be killed (Chapter 8, Saltmere Docks)

Example: the party holds the dock for the ship. Tamsin's lance brings Dace
to 0. He goes down on one knee, grayed out, on his tile. From then on he
takes no actions, nobody (ally or enemy) can target him, and nobody can
walk through his tile. His Hounds keep coming until the hold is over. In the
closing scene he is still there to order the archers to stop.

- The map marks a unit **"can't be killed"**. When its HP reaches 0 its
  defeated line plays and it is **down**: it stays on its tile, drawn grayed
  out (the look is the map skins'), **takes no more actions** and **can't be
  targeted** by anyone, attacks, spells or healing (Nick).
- **It blocks its tile** (Nick): no unit can walk through it or stop on it.
- The battle goes on (Chapter 8's objective is to hold for a number of
  turns). The striker gets the EXP of a kill; a down unit counts as defeated
  for the results (*Claude's starting rule*).
- The forecast is the normal one: it shows his HP reaching 0 (Nick: "goes
  to 0").
- The scene after the battle can still use him (he gets up in the script).

## Open sub-questions (deferred)

- Which skills grant post-action movement, and exactly how much: tickets
  0003 / 0005.
- Whether losing an Ally/Neutral unit can lose a map ("protect the villagers"
  objectives): decided per chapter when such a map is designed (0009 and
  later chapter tickets).
