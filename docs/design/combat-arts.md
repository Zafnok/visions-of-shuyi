# Combat Arts (and what class actives cost)

Decided: 2026-09-25
Source: ticket 0014

## Nick's words

> **Q1. Who gets which arts?** "Mix: rank + weapons (Recommended)" (reaching a
> weapon rank teaches that weapon kind's arts, usable with any weapon of that
> kind; a few special weapons carry their own art).
>
> **Q2. Limits besides durability?** "Durability only" (the offered option:
> no extra restrictions, the art boosts the whole combat).
>
> **Q3. Arts vs class actives?** "I think class actives and weapon arts should
> both draw durability but have no overlap in their function. We'll have to
> see how that can hold but they should do different things flavor wise"

Follow-ups:

> **Actives that don't swing a weapon** (Brace, War Cry, Sanctuary, Shove,
> spell actives): "Weapon dur + spell uses" (weapon actives cost durability,
> spell actives cost spell uses, non-attack actives cost the equipped weapon's
> durability).
>
> **Arts when attacked:** "same as class actives, the effect might hold for
> just your turn or for both your and enemy turn. Nothing to pick on enemy turn
> but not a counter either"
>
> **Where the line between arts and actives is:** "I don't think there's
> necessarily a line just something lore flavored... like take Dark Souls MLGS
> it uses magic damage so a unique weapon like that might have all damage hit
> against Res instead of Prt/Def. But there might also be a Spellsword class
> that can buff itself to make for that turn its weapon do the same effect.
> But these overlaps should make sense and be rare. Basically, both sides
> should be flavorful."
>
> **Starter list:** "Approve as is (Recommended)".
>
> **Ranks of the starter arts:** "E and D (Recommended)".
>
> **Do enemies use arts and actives?** "Bosses only".

After the PR was opened, Nick added:

> **Weapon skill gain:** "I think skill gain should be a formula of base +
> amount of damage. So we can say sure base 2/4 skill gain but if you do a
> lot of dmg you get extra and if you do only a little you get just a tiny
> extra amount."
>
> **Spells:** "per battle actives should only exist on spells otherwise yes
> cost durability" … "spells are like Fire or Blizzard and each of those will
> have some set amount per battle maybe it's 10 per battle for Fire and 8 per
> battle for Blizzard and these are the one exception to using weapon
> durability is that these do not use durability whatsoever they simply tap
> the spell counter."

Reviewing ticket 0312 (2026-09-27), Nick added:

> **Line Pierce on a diagonal:** "for line pierce I am ok with it except for
> diagonal attack. Since diagonal should be Rng 2 like FE handles it. So if
> you happen to have a spear with Rng 2 then I think line pierce should work
> diagonally"
>
> **Green units:** "I think green units can use arts/actives so long as
> they're combat green units... not like villagers you have to protect or
> wild beasts you slay for materials and nothing more..."

Reviewing ticket 0503 (2026-10-01), Nick changed what **non-attack actives**
cost (this replaces "Weapon dur + spell uses" for them, and "per battle
actives should only exist on spells"):

> **Non-attack actives:** "now that I look at them, I think we should rework
> these actives. Flavor-wise I don't see why we give any of these dur costs.
> They should all be per-battle uses. We can for example give things like
> Shove a higher use count per battle and something strong like Rally or
> Inspire or Sanctuary a lower per-battle-use allowance...... this should
> count for both the player and the boss."
>
> **Actives that swing a weapon** (Keen Edge, Lance Rush, Long Shot…): "the
> ones that swing a weapon, yes, should use durability. I am ok with this."
>
> **How many uses:** "I think we can have more extremes between tiers here.
> Like Shove is so minor it may as well have 8 or 10 uses. But Sanctuary 2 is
> very impactful and not the only healing option - it should probably stay 1.
> Sanctuary 1 can probably be 2 or 3 per battle."
>
> **Sanctuary 2:** "the way I thought of class actives getting replaced is by
> higher numbers, not fundamentals like range. So when I see sanctuary 1
> heals all within 1 and sanctuary 2 heals all within 2, that represents a
> gameplay shift more significant than a simple replacement on numbers alone
> imo... in this case, the promotion could probably give a new skill that
> heals all within 2 and keep sanctuary."
>
> **Bosses and non-attack actives:** "I am ok with boss using non-combat
> active if it is at least proximally close to some enemies (i.e. it thinks
> the player will reach it, or it the player, within next turn)"

Playing Quick Battle (2026-10-03, while answering ticket 0430), Nick wanted
the arts out of the forecast:

> "like three houses, but up/down should not pick an art. the arts should not
> be in the same menu. we should have an arts menu. that was something i didnt
> like about the current quick battle setup anyway"

and for ticket 0442 (2026-10-12) he chose, from rendered mockups:

> **How do you get to the arts menu?** "A. Action menu only" (the offered
> option: like Three Houses, `Arts` beside `Attack`; after pointing at an
> enemy: Cancel, then `Arts`).
>
> **Where do the class actives go (Vault, Keen Edge, Overcast)?** "C. Spell
> actives in spell list" (the offered option: weapon actives in Arts; the
> spell list gets `Fire + Overcast` lines, no `Arts` for mages).
>
> **What do the keys do in the forecast once the list is gone?** "A. Same
> everywhere (Recommended)" (the offered option: left/right swap the weapon
> in every weapon forecast, up/down change the target, no weapon list after
> `Attack`).

So:

- **Combat Arts** are weapon techniques. A unit learns a weapon kind's arts by
  reaching **weapon ranks** in that kind (E, D, then C, B, A, S later), and a
  few **special weapons** carry their own art.
- Arts cost the weapon's **durability**, and so do the **class actives that
  swing a weapon**. Spell actives cost **spell uses** instead. **Non-attack
  actives** (Brace, Shove, Sanctuary…) have their own **uses per battle**
  (Nick, 2026-10-01).
- **Durability is the only limit.** An art boosts **every strike** of the
  combat. It's picked only when attacking on your own turn. Some arts leave an
  effect that lasts until your next phase, so it also helps on the enemy's
  turn, but nothing is ever picked then.
- Arts and actives have **no fixed dividing line**. Each should be flavourful
  for its weapon or class. Overlaps between them should be **rare and make
  sense** (Nick's example: a unique blade whose art hits Res, and a Spellsword
  class whose active does the same for a turn).
- **Only bosses** among enemies use arts and actives. **Combat green units**
  use them too; non-combat green units (villagers, wild beasts) never do
  (Nick, 0312 review).
- **Weapon EXP** is a base (2, or 4 with an art) **plus a bonus from the
  damage dealt** (Nick).
- **Per-battle resources:** spell uses and the uses of non-attack actives.
  Casting never touches durability; spells and spell actives only spend the
  spell's uses (Nick).

Everything marked *tunable* is a starting value Claude chose; balance tickets
may change it without asking. Items marked *Claude's starting rule* fill a gap
Nick's answers didn't cover; Nick may veto them. Unmarked rules are Nick's.
All numbers are FE-sized and rescale with ticket 0013.

## Learning arts

- **Rank arts.** Each weapon kind has a list of arts, each with a minimum
  rank. A unit **knows** a rank art once its rank in that kind is at least the
  art's rank. Ranks never drop, so a known art is never lost.
- A unit can **use** a rank art only while its **current class** can wield that
  kind and it attacks with a weapon of that kind. After a reclass to a class
  without that kind, the art is known but unusable (same as the ranks
  themselves, `progression.md`).
- **Weapon arts.** A weapon's data may list its own arts (`arts: [ArtId]`).
  Anyone who can wield that weapon can use its art while attacking with it,
  whatever their rank (*Claude's starting rule*: the rank to wield the weapon
  already gates it).
- Chapter 1 has **no special weapons** (no loot in Chapter 1, `chapter-1.md`).
  The first ones are designed with later chapters.

## Using an art (exact rules)

1. **When:** only when the unit **attacks on its own phase**, chosen in the
   `Arts` menu before the target (Nick, 2026-10-12; see *The arts menu and
   the forecast*). Never picked on counters or during another side's phase
   (Nick).
2. **One per attack** (*Claude's starting rule*): an attack uses at most one
   art **or** one combat active, not both.
3. **Can use** if: the art is usable (above) and the attacking weapon is
   **not broken**. With less durability left than the cost it can still be
   used: it spends what is left and the weapon breaks after the combat
   (Nick, 0414 review: "it has 3 dur left and the art costs 5, ok, art goes
   thru, then weapon is broken").
4. **Cost** is paid **once per combat**, when the attack is committed, from
   the attacking weapon. It is paid even if every strike misses
   (*Claude's starting rule*, Three Houses).
5. If paying brings the weapon to 0, the combat still uses the **unbroken**
   weapon. The weapon breaks (`ItemBroke`) after the combat
   (*Claude's starting rule*).
6. **Durability only** (Nick): the art's effect applies to **all** of the
   attacker's strikes this combat, including extra strikes from attack speed.
   The defender still counters normally (unless the art says otherwise), and
   its counters are normal attacks.
7. **Lasting effects** (Nick: "might hold for just your turn or for both your
   and enemy turn"): an art may apply a **timed effect**. Two durations exist:
   - **Stance** (on the user): from this combat until the **start of the
     user's next phase**. This is the same as a stance rider in
     `progression.md`, so it also applies when the user is attacked on the
     enemy's turn.
   - **Debuff** (on the target): until the **end of the target's next phase**.
     An enemy hit on the player's phase is slowed through its own coming
     phase. The same effect doesn't stack with itself; applying it again
     refreshes it.
8. **Weapon EXP** (Nick: "base 2/4 … if you do a lot of dmg you get
   extra"): a combat using an art **doubles the base** of the weapon-EXP
   formula in `weapons-and-items.md` (**4** if at least one strike hit, **2**
   if every strike missed). The damage bonus `dealt / 5` is added on top and
   isn't doubled (*tunable*).
9. Unit EXP and class points are the normal combat award (`progression.md`).

## Chapter 1 arts (*tunable* numbers; list approved by Nick)

Range is the weapon's own unless the art says otherwise. "This combat" means
all of the attacker's strikes.

| Art | Kind | Rank | Cost | Range | Effect | Weapon EXP base |
| --- | ---- | ---- | ---- | ----- | ------ | --------------- |
| **Flowing Cut** | Sword | E | 2 | weapon | This combat, the sword follow-up bonus is **×3/2** instead of ×6/5 (strikes 2..N: `damage * 3 / 2`). | ×2 |
| **Guard Break** | Sword | D | 4 | weapon | This combat: hit **+10**, and the defender **can't counter**. | ×2 |
| **Unhorse** | Spear | E | 2 | weapon | This combat: hit **+10**, and the spear's Mounted effectiveness is **×3** instead of ×2. | ×2 |
| **Line Pierce** | Spear | D | 4 | weapon | After the combat, if the attacker is still standing, it makes **one strike** at the unit **directly behind** the target (see below). | ×2 |
| **Crushing Swing** | Axe | E | 2 | weapon | This combat: hit **+20**, and the axe's minimum damage is **8** instead of 5. | ×2 |
| **Armor Cleave** | Axe | D | 4 | weapon | This combat, the axe is effective against **Armored ×2**. | ×2 |
| **Close Shot** | Bow | E | 2 | **1–2** | This attack, the bow's minimum range is 1 (it can shoot an adjacent enemy). Hit **−10**. | ×2 |
| **Pinning Shot** | Bow | D | 3 | weapon | On the first strike that hits: the target gets **Mov −3** (not below 0), a debuff until the end of its next phase. | ×2 |
| **Pressure Point** | Gauntlet | E | 2 | weapon | On the first strike that hits: the target gets **Spd −3** (not below 0), a debuff until the end of its next phase. | ×2 |
| **Sidestep** | Gauntlet | D | 3 | weapon | Stance: **avoid +20**, from this combat until the start of the user's next phase. | ×2 |

Details:

- **Flowing Cut** changes only the follow-up multiplier. The order of
  `weapons-and-items.md` stays: base damage → axe minimum → sword follow-up
  (now ×3/2) → crit ×3. With one strike it does nothing extra, and the
  forecast shows that.
- **Guard Break:** the defender's side of the forecast shows `—  no counter`.
- **Unhorse:** effectiveness still uses the largest matching multiplier
  (`weapons-and-items.md`), so against a unit that is Mounted it's ×3.
  Against anything else only the hit bonus applies.
- **Armor Cleave** adds `(Armored, 2)` to the weapon's effective list for
  this combat. It uses the largest-multiplier rule too.
- **Line Pierce, exact:**
  - "Directly behind" is the next tile past the target on the line from the
    attacker: one step further in the attack's direction. The line is
    either straight (same row or column) or **diagonal** (Nick: a diagonal
    attack is a range-2 attack, as in FE, and pierces diagonally). So from
    `(0,0)`: a target at `(1,0)` or `(2,0)` has `(2,0)` or `(3,0)` behind
    it, and a diagonal target at `(1,1)` has `(2,2)` behind it.
  - Any other angle (only possible with range 3 or more, e.g. a target at
    `(1,2)`) has no tile behind it and no pierce (*Claude's starting
    rule*; no spear has range 3 yet).
  - The pierce happens **even if the main target died**, as long as the
    attacker is still alive and the tile holds a unit **hostile** to the
    attacker.
  - The pierce is one strike using the normal hit/damage/crit formulas
    against that unit (its own Def, avoid and terrain; the attacker's weapon
    and effectiveness). It gets **no counter** and no extra strikes. It uses
    the 2RN hit roll and the crit roll in strike order, after the main
    combat's strikes.
  - For unit EXP and class points the pierce counts as a **second combat**
    against that unit (its own award, `progression.md`). Weapon EXP is one
    award for the whole combat: the pierce's damage adds to `dealt`.
- **Close Shot** can be used at distance 1 or 2. At distance 2 it just costs
  durability and hit, and the forecast shows that honestly.
- **Pinning Shot / Pressure Point** apply once per combat, on the first
  strike that hits. Mov and Spd changes feed movement ranges, the danger zone
  and attack speed at once (a slowed enemy may now be doubled by the next
  attacker).
- **Sidestep's** avoid is added to `avoid_B` whenever the user is the one
  being attacked, and to its avoid during the rest of this combat.

### Later ranks

Arts for ranks **C, B, A and S**, and the first **special weapons** with
their own arts, come in a later decision (ticket 0018). Until then, reaching
rank C teaches nothing new. Nick's example for a special weapon: a unique
blade (like Dark Souls' Moonlight Greatsword) whose art makes that combat
**Magical** (hitting Res instead of Def).

## What class actives cost

Nick: "class actives and weapon arts should both draw durability" (ticket
0014), then for the non-attack ones "They should all be per-battle uses"
(2026-10-01):

| Kind of active | Examples | Pays with |
| -------------- | -------- | --------- |
| **Combat active** (with a weapon attack) | Keen Edge, Flurry, Heavy Blow, Vault, Long Shot | The attacking weapon's durability |
| **Non-attack active** (an action of its own) | Brace, Fortify, War Cry, Sanctuary, Shove | **1 of its own uses per battle** |
| **Spell active** (with a spell) | Overcast, Siphon | **1 extra use** of the spell being cast |

- The rules above for arts apply to combat actives too: they can't be paid
  with a broken weapon, can be used with less durability left than the cost
  (spending what is left; Nick, 0414 review: "Yes, all of them"), cost is
  paid once when committed, and a weapon that hits 0 breaks after the
  action.
- A spell active needs the spell to have `uses_left ≥ 2` (the cast plus the
  extra use). It never costs durability.
- Spells (Fire, Frost, Heal…) never use durability at all; they only spend
  their own uses, which refill every battle (`magic.md`). Spell actives
  follow that.
- Actives don't double the weapon-EXP base (*Claude's starting rule*; only
  arts do). The damage bonus applies as in any combat.

### Non-attack actives: uses per battle (Nick, 2026-10-01)

Example: a Grappler starts every battle with 8 Shoves. Each Shove takes
one. At 0 the Shove is greyed out until the next battle.

- Each non-attack active has a number of **uses per battle**. Using it
  spends one. They **refill at the start of every battle**, like spell uses.
- It costs **no durability** and needs **no weapon**: a unit with nothing
  equipped, or a broken weapon, can still use it (*Claude's starting rule*,
  from "I don't see why we give any of these dur costs").
- The uses belong to the **unit and the skill**: two Guards each have their
  own 3 Braces.
- **The same for everyone** (Nick: "this should count for both the player
  and the boss"): bosses and combat green units get the same uses.
- The weaker the effect, the more uses (Nick: "more extremes between
  tiers"):

| Active | Class | Uses per battle | |
| ------ | ----- | --------------- | - |
| Shove | Grappler | 8 | Nick: "8 or 10" (*tunable*) |
| Brace | Guard | 3 | *tunable* |
| Sanctuary | Cleric | 3 | Nick: "2 or 3" (*tunable*) |
| Fortify | Bulwark | 2 | *tunable* |
| War Cry | Vanguard | 2 | *tunable* |
| Inspire | Exile (lord) | 2 | *tunable* |
| Rally | Commander (lord) | 1 | *tunable* |
| Benediction *(working name)* | Priest | 1 | Nick: "it should probably stay 1" |

- **Benediction replaces Sanctuary 2** (Nick: ranks replace "by higher
  numbers, not fundamentals like range"). It is a skill of its own: heals
  every ally within 2 tiles by `Mag + 5`. A Priest promoted from Cleric
  keeps **Sanctuary** (3 uses, adjacent allies) and also has Benediction
  (1 use). White Magic adds to both. The name is Claude's working name;
  Nick may rename it.
- Classes with **0 weapon slots** (tier-3 magic classes) can now have
  non-attack actives as well as spell actives (ticket 1001).
- **On screen:** the Skill menu and the unit's info screen show the uses
  left of the uses per battle, the way spell uses are shown (`Brace  2/3`).
  A skill with none left is dimmed, with `no uses left` after it. Picking
  who to Shove shows `Shove on Brigand (8 → 7 uses)`.
- A unit that comes to know a non-attack active in the middle of a battle
  has no uses of it until the next battle, like a spell learned in battle
  (*Claude's starting rule*; nothing in the game teaches an active
  mid-battle yet).

### Durability costs (*tunable*: 2/battle actives cost 3, 1/battle actives cost 5)

The combat actives. The non-attack actives (Brace, Sanctuary, Shove, War
Cry, Fortify, Inspire, Rally) cost uses per battle, above.

| Active | Class | Was | Now |
| ------ | ----- | --- | --- |
| Keen Edge | Swordsman | 2/battle | 3 dur |
| Flurry | Brawler | 1/battle | 5 dur |
| Heavy Blow | Raider | 2/battle | 3 dur |
| Vault | Archer | (was the Skirmish passive) | 1 dur (Nick, 0311's PR) |
| Lance Rush | Rider | 2/battle | 3 dur |
| Swoop | Flier, Sky Lancer | 2/battle | 3 dur |
| Overcast | Mage, Sorcerer | 1/battle | +1 spell use |
| Blade Flurry | Duelist | 1/battle | 5 dur |
| Deadly Blow | Shadowblade | 1/battle | 5 dur |
| Hundred Fists | Striker | 1/battle | 5 dur |
| Rampage | Berserker | 1/battle | 5 dur |
| Long Shot | Marksman (was Archer; range +2) | 2/battle | 3 dur |
| Volley | Outrider | 1/battle | 5 dur |
| Trample | Iron Rider | 2/battle | 3 dur |
| Piercing Lance | Lancer | 2/battle | 3 dur |
| Dive | Sky Warden | 1/battle | 5 dur |
| Siphon | Mystic | 1/battle | +1 spell use |

The Chapter 1 budget: iron weapons have 20 durability and Chapter 1 has no
blacksmith, so each weapon's 20 is shared by its arts and combat actives for
the whole battle.

### Overlap check (Nick: overlaps rare and sensible)

The starter arts were picked so they don't repeat the tier 1–2 actives'
functions. The closest pairs, all judged in the playtest:

- **Crushing Swing** (axe, hit +20) and **Keen Edge** (Swordsman, hit +30,
  crit +10): both raise accuracy, but Crushing Swing fixes the axe's
  weakness and Keen Edge is a swordsman's precision.
- **Sidestep** (gauntlet stance, avoid) and **Brace / Fortify** (Guard
  stances, Def/Res): both are stances, but they defend in different ways.
- **Close Shot** (bow min range 1) and **Long Shot** (Marksman, bow max
  range +2): opposite ends of the bow's range.

## Enemies: bosses only; combat green units too

- Among enemies, only units marked **boss** in chapter data use arts and
  actives. They follow the same rules and pay the same way: their own
  weapon's durability, and their own uses per battle for non-attack actives
  (Nick: "Bosses only"; "this should count for both the player and the
  boss").
- **Green (Other-phase) units** use arts and actives when they are **combat
  units** (allied soldiers, mercenaries…), with the same rules (Nick, 0312
  review). **Non-combat** green units, marked so in chapter data (villagers
  to protect, wild beasts that only give materials), never do.
- Ordinary enemies never use arts or actives. Their passives still apply
  (at class level 1, generic units have none).
- When a boss or a green unit attacks with an art or active, the forecast
  and playback show its name.
- How the boss and green AI choose (ticket 0503; all *Claude's starting
  rules*, numbers *tunable* in `assets/data/ai.ron`):
  - It weighs every attack it can make plain and with each art or combat
    active, by the forecast, and picks the best. Using one counts against
    the attack (5 points per durability, where 1 damage is worth 10), so it
    keeps them for attacks they really improve. Example: a boss facing a
    unit whose counter would hurt it uses Guard Break (no counter); against
    an archer next to it, who can't counter anyway, it attacks normally.
  - It doesn't yet value what the forecast's numbers don't show (pinning or
    slowing the target, a stance for the enemy's turn), so it doesn't pick
    an art for that alone: ticket 0511.
  - A boss or combat green unit that holds its tile (it guards or never
    moves) and has nothing to attack uses its non-attack active (Brace, War
    Cry…) instead of waiting, when a hostile unit could reach and attack it
    on its coming turn (Nick, 2026-10-01: fine "if it is at least proximally
    close to some enemies"). It needs a use left: a boss with Brace braces
    on 3 such turns of a battle, then waits.

## The arts menu and the forecast

Nick, ticket 0442 (his words are at the top). Before that the arts were a
list beside the forecast (ticket 0414).

Example: the lord stands beside a brigand. Its action menu reads `Attack`,
`Arts`, `Skill`, `Item`, `Equip`, `Wait`. `Arts` lists Flowing Cut and Guard
Break. The player picks Guard Break, then the brigand, sees the forecast
with `Guard Break (20 → 16)` and `no counter`, and confirms.

- **`Arts` is an entry of the action menu, under `Attack`** (Nick: "Action
  menu only", like Fire Emblem: Three Houses). It is the only way to an art:
  the forecast has no arts list and no key that opens one.
- **What the menu lists** (Nick: weapon actives in Arts): the Combat Arts of
  the weapons the unit carries, then its **class actives that swing a
  weapon** (Keen Edge, Vault, Long Shot). Each line shows name, cost and
  source (`E`/`D` rank, `weapon` art, `active`). There is no `Attack` line:
  a plain attack is `Attack` in the action menu.
- **Spell actives are lines of the spell list** (Nick), not of `Arts`. Under
  `Magic`, every attack spell is followed by a line for each spell active
  the unit has: `Fire`, then `Fire + Overcast  +1 use`. A mage has no `Arts`
  entry. With an active the spell is cast at enemies only (*Claude's
  starting rule*: Overcast and Siphon do nothing for a tile).
- **A line that can't be used is dimmed, with the reason after it**:
  `broken` (the weapon), `1 uses left` (a spell active that needs two),
  `out of range` (nobody it reaches from that tile). A dimmed line can't be
  chosen. An art that costs more durability than is left stays usable; the
  forecast shows it breaking the weapon (`Guard Break (3 → 0)`).
- **Then the target**, with the forecast: the numbers **with** the art
  applied, plus a line with the art's name and the durability change, e.g.
  `Guard Break (20 → 16)`. Effects that aren't numbers are spelled out:
  `no counter`, `pierces`, `pins: Mov −3`, `slows: Spd −3`,
  `stance: avoid +20`.
- **The forecast's keys are the same in every weapon forecast** (Nick: "Same
  everywhere"): opened from `Attack`, from `Arts` or by pointing at an enemy.
  - **Left / right swap the weapon.** `Attack` no longer shows a weapon
    list first: it opens the forecast with the equipped weapon.
  - **Up / down change the target**, and so do the previous / next unit
    keys.
- **Cancel** from the forecast of an art goes back to the arts menu, on that
  art; Cancel again goes to the action menu, on `Arts`.
- *Claude's starting rules (Nick can veto):*
  - `Arts` is listed for a unit that has an art or a weapon active for a
    weapon it carries, and dimmed when none of them can be used on anyone
    from that tile.
  - **`Attack` and pointing at an enemy are plain attacks only.** An enemy
    only an art reaches is attacked through `Arts`. Example: an archer with
    a brigand right beside it can't shoot it plainly (a bow reaches 2), so
    `Attack` is dimmed, the menu opens on `Arts`, and Close Shot hits it.
  - An art's forecast opens with the equipped weapon if it can use the art
    on someone, else with the first weapon that can. Left / right swap only
    between weapons that can, and the art stays chosen. The menu's top
    border names the weapon and its durability (`Iron Sword 20/20`).
  - From `Attack`, left / right swap between the unit's **weapons** that
    reach someone (spells are under `Magic`). After pointing at an enemy
    they swap between weapons and spells, as before
    (`turn-structure.md`).
  - With one weapon to attack with, left / right change the target too.
  - `Magic` keeps its spell list (a spell can also be cast on an ally or a
    tile, which a forecast can't show), and a spell's forecast opened from
    it doesn't swap.
  - A line with a spell active shows only `Fire + Overcast` and its cost;
    the spell's uses and numbers are on the spell's own line, above it.
- The arts menu (Quick Battle's lord):

  ```
   ┌─ Iron Sword 20/20 ─────────────┐
   │ Flowing Cut     −2 dur  E      │
   │▸Guard Break     −4 dur  D      │
   └────────────────────────────────┘
  ```

  and the forecast of the art chosen in it:

  ```
   ╔════════ Forecast ══════════╗
   ║ Guard Break (20 → 16)      ║
   ║ Test Lord     Brigand      ║
   ║ Iron Sword    Iron Axe     ║
   ║ Hit   100     Hit   --     ║
   ║  8 dmg  →  1   no counter  ║
   ║  9 dmg  →  2               ║
   ╚════════════════════════════╝
  ```

  The spell list of a Mage:

  ```
   ┌─────────────────────────────────┐
   │ Fire   10/10  Mt5 Hit90 Rng1-2  │
   │▸Fire + Overcast  +1 use         │
   │ Frost  10/10  Mt4 Hit95 Rng1-2  │
   │ Frost + Overcast  +1 use        │
   │ Heal    8/8   HP+10 Rng1        │
   └─────────────────────────────────┘
  ```

## Worked example (for 0312's tests)

Example W1 from `weapons-and-items.md` (Swordsman, Iron Sword 20/20, rank D,
vs Brigand), with each sword art:

- **Normal:** `dmg 9 (then 10) ×2, hit 94, crit 3` / Brigand `dmg 12, hit 63`.
- **Flowing Cut** (20 → 18): `dmg 9 (then 13) ×2, hit 94, crit 3`
  (`9 × 3 / 2 = 13`) / Brigand unchanged.
- **Guard Break** (20 → 16): `dmg 9 (then 10) ×2, hit 100, crit 3`
  (`94 + 10`, clamped to 100) / Brigand `—  no counter`.
- Weapon EXP if both strikes hit (base 4 with an art, plus `dealt / 5`):
  normal attack `2 + 19/5 = 5`; Flowing Cut `4 + 20/5 = 8` (9 + 13 = 22,
  but the Brigand only had 20 HP, so `dealt` is 20 and it falls); Guard Break
  `4 + 19/5 = 7`.

## Open sub-questions (deferred)

- **Arts for ranks C–S, and special weapons with their own arts:** ticket
  0018.
- **Tier-3 actives** must be spell actives for 0-weapon classes: ticket 1001.
- **Whether any tier 1–2 active should be reflavoured** to overlap arts less:
  judged in the Chapter 1 playtest (0804).
- **Number scale:** all numbers rescale with ticket 0013.
