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
All numbers are FE-sized; ticket 0013 kept that scale.

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
- A special weapon may also have a **passive** that costs nothing (the
  Moonblade, *Special weapons* below).
- Chapter 1 has **no special weapons** (no loot in Chapter 1, `chapter-1.md`).
  The first ones come in Chapter 2 (*Special weapons*, ticket 0018).

## Using an art (exact rules)

1. **When:** only when the unit **attacks on its own phase**, chosen in the
   `Arts` menu before the target (Nick, 2026-10-12; see *The arts menu and
   the forecast*). Never picked on counters or during another side's phase
   (Nick). One exception: **Overwatch** (bow, rank B) is used as an action
   of its own, without attacking (Nick, ticket 0018).
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

## Ranks C–S (ticket 0018, 2026-10-26)

### Nick's words

Shown the 20 proposed arts on a page
(<https://claude.ai/artifact/U13aJ3WFR1FWALZTiZonUn>), with a forecast
mockup:

> "I think sword and spear are good enough, maybe we change them super
> later but for now they are ok. Not super happy about axe though... feels
> too technical for haft break, and wide swing imo should not have a normal
> combat phase but just be a cleave 270 degree rng 1. dread swing too
> technical. felling blow is probably ok. Bow having break shot in mechanic
> and name is too on the nose right? is that even legal? overwatch is ok, I
> mean that's a military term we can use it. but maybe it should be a free
> action rather than forcing an attack. still cost dur. others are ok for
> bow. for gauntlet maybe deflect should be stronger to also parry. rest
> are ok I guess."

Follow-ups: for the axe's B and A he picked **Hurl** and **Skull Splitter**
(over Shoulder Charge and Earthsplitter); Deflect's parry is "Miss, then
strike back (Recommended)". Break Shot is renamed **Rending Arrow** with
the same effect: a mechanic can't be owned, but the ticket asked for our
own names and the name was Three Houses' (*Claude's choice*, Nick may
rename it).

### The list (*tunable* costs and numbers; the list is Nick's)

C arts are for tier-1 classes (they top out at rank C), B and A for tier 2,
S for tier 3. None is a plain "might +N" or "+1 strike": those are what
class actives do.

| Art | Kind | Rank | Cost | Range | Effect | Drawn from |
| --- | ---- | ---- | ---- | ----- | ------ | ---------- |
| **Riposte** | Sword | C | 4 | weapon | Stance: when an enemy attacks the user, the user **strikes first**. | FE's Vantage |
| **Disarm** | Sword | B | 5 | weapon | On the first hit, the target is **disarmed** until the end of its next phase: it can't attack or counter with a weapon. | FFT's Rend Weapon, for one turn |
| **Cascade** | Sword | A | 6 | weapon | Each **hit** after the first deals **+3 more than the one before** (2nd hit +3, 3rd +6, 4th +9); misses don't count. | Our own, on the sword trait |
| **Unbroken Stride** | Sword | S | 8 | weapon | If the target falls this combat, the user **acts again** (full move and action). Once per phase. | Engage's / Fates' Galeforce |
| **Long Thrust** | Spear | C | 3 | **1–2** | This attack, the spear reaches 2. Hit **−10**. | Three Houses' javelins |
| **Spear Wall** | Spear | B | 4 | weapon | Stance: the user **counters at range 1–2**, and its counters get hit **+15**. | Fates' Distant Counter |
| **Skypiercer** | Spear | A | 5 | weapon | This combat, effective against **Flying ×2**. | Three Houses' Knightkneeler, at the sky |
| **Comet Lance** | Spear | S | 8 | **1–3, straight** | One strike at **every enemy on a straight line** of 3 tiles, nearest first. No counters. | Into the Breach's Laser Mech |
| **Wide Swing** | Axe | C | 4 | 1 | A **cleave** instead of a combat: one strike at each enemy on the tiles **in front of, left of and right of** the user. No counters. | Battle Brothers' Round Swing |
| **Hurl** | Axe | B | 4 | **1–2** | The user throws the axe: this attack reaches 2. Hit **−10**. | FE's Hand Axe |
| **Skull Splitter** | Axe | A | 5 | weapon | This combat: crit **+30**. | Nick's pick |
| **Felling Blow** | Axe | S | 8 | weapon | This combat, the axe's minimum damage is **a third of the target's max HP** (rounded down) when that is more than 5. | Our own, the axe trait taken to its end |
| **Rending Arrow** | Bow | C | 4 | weapon | On the first hit: the target gets **Def −5** until the end of its next phase. | Three Houses' Break Shot (renamed) |
| **Overwatch** | Bow | B | 5 | weapon | **An action of its own, no attack** (Nick: "a free action rather than forcing an attack. still cost dur"). Stance: the first enemy that **ends a move within the bow's range** is shot once, before it acts. | XCOM's Overwatch |
| **Marked Shot** | Bow | A | 6 | weapon | On the first hit: the target is **marked** until the end of its next phase: every attack on it by its enemies gets hit **+20** and crit **+10**. | XCOM 2's Holo Targeting |
| **Arrow Rain** | Bow | S | 8 | **2–3** | One strike at the target **and every enemy next to it**. No counters. | FFT's area skills; XCOM's grenades |
| **Off Balance** | Gauntlet | C | 3 | weapon | On the first hit: the target gets **avoid −20** until the end of its next phase. | Our own; it lets axes land |
| **Deflect** | Gauntlet | B | 5 | weapon | Stance: the first enemy strike that would hit the user **misses**, and the user **strikes back** at once. | FE's Pavise, made certain but once |
| **Chain Strike** | Gauntlet | A | 6 | weapon | After the combat, if the target stands, **every ally next to it strikes it once**. | Engage's Chain Attack |
| **Vital Point** | Gauntlet | S | 8 | weapon | On the first hit: the target gets **Str, Mag, Def, Res and Spd −4** until the end of its next phase. | Pressure Point taken all the way |

Every art doubles the weapon-EXP base, as in rule 8. "Stance" is rule 7's
stance: from this combat (or action) until the start of the user's next
phase. "Until the end of its next phase" is rule 7's debuff: it doesn't
stack with itself, and applying it again refreshes it. "Enemy" is a unit
hostile to the user; allies (green units included) are never struck.

### Exact rules (*Claude's starting rules* unless quoted; Nick may veto)

- **Riposte.** While the stance lasts, in a combat where the user is the
  defender and can counter (its weapon reaches the attacker), the user's
  first counter strike comes **before** the attacker's first strike. The
  rest of the order is as usual. A user who can't counter (out of reach,
  disarmed) gets nothing from it. It does nothing in the combat it is used
  in, where the user attacks.
- **Disarm.** From the first hit on, the target's remaining counters in this
  combat are cancelled. Until the end of its next phase it **can't counter**
  any combat and **can't attack with a weapon**. It can still move, cast
  spells and use non-attack actives. The forecast shows `no counter` once
  the first strike is sure to hit, else `disarms on hit`. The boss AI
  doesn't value it yet (as 0511 says for every debuff).
- **Cascade.** Only **hits** count (Nick, 2026-10-26: "cascade shouldnt
  count missed strikes"): the user's *n*-th hit this combat (n = 2, 3, 4)
  deals `+3 × (n − 1)` more. Example: hit, miss, hit, hit gives +0, —, +3,
  +6. It is added after the sword's follow-up bonus and before crit:
  base → sword follow-up ×6/5 → **+3(n−1)** → crit ×3. With one strike it
  does nothing. The forecast shows the damage if every strike hits.
- **Unbroken Stride.** If the target's HP reaches 0 from the user's strikes
  in this combat, the user may **move its full Mov again from where it
  stands and take another action**, as if its turn had just begun. It can
  use other arts in that action, but not Unbroken Stride again in the same
  phase. The forecast line reads `if it falls: act again`.
- **Long Thrust / Hurl.** The weapon's range is **1–2** for this attack
  (like Close Shot sets 1–2 for a bow). A target at 2 counters only if its
  own weapon reaches 2. At distance 2 a diagonal target counts, as in Line
  Pierce.
- **Spear Wall.** While the stance lasts, the user can counter an attacker
  at distance 1 **or 2** with its spear, whatever the spear's own range, and
  each counter strike gets hit +15. Only counters get the bonus.
- **Skypiercer** adds `(Flying, 2)` to the weapon's effective list for this
  combat; the largest matching multiplier wins, as for Armor Cleave. A
  flying unit isn't Mounted (`weapons-and-items.md`), so Unhorse never
  applies to it.
- **Comet Lance.** The player picks an enemy in the same row or column
  within 3 tiles (no diagonals). The line is the 3 tiles from the user in
  that direction. Every enemy on it takes **one strike**, nearest first;
  allies on the line are passed over, and walls don't stop it. No counters,
  no extra strikes. Each struck enemy counts as its own combat for unit EXP
  and class points (as Line Pierce's strike does); weapon EXP is one award
  with all the damage in `dealt`.
- **Wide Swing** (Nick: "should not have a normal combat phase but just be
  a cleave 270 degree rng 1"). The player picks an adjacent enemy, which
  sets the user's facing. The cleave hits the **front** tile (that enemy),
  then the **left** and **right** tiles: range 1 is the four tiles next to
  the user, so 270° is three of them, and the tile **behind** is never hit.
  Each enemy there takes **one strike**. No counters, no extra strikes, the
  axe minimum applies. EXP as for Comet Lance.

  ```
        .  X  .        @ = the user, facing east (the picked enemy)
        .  @  X        X = one strike each
        .  X  .        the tile behind (west) is safe
  ```
- **Skull Splitter** adds 30 to every strike's crit this combat (clamped
  to 100). It is the closest an art comes to Keen Edge (crit +10) and
  Deadly Blow (crit ×2); Nick picked it.
- **Felling Blow.** `axe minimum = max(5, target.max_hp / 3)` for this
  combat, in place of 5 (or Crushing Swing's 8), applied on a hit before
  crit. Against a 60-HP boss every hit deals at least 20.
- **Rending Arrow** works like Pinning Shot: once per combat, on the first
  strike that hits, `Def −5` (not below 0).
- **Overwatch** (an exception to rule 1: an art used **without attacking**).
  - It is a line of the `Arts` menu like any other. The unit may move first;
    choosing Overwatch (with a bow that isn't broken) pays 5 durability,
    ends the unit's action like `Wait`, and needs no target. The menu dims
    it only when the bow is broken.
  - Stance until the start of the user's next phase: the **first** enemy
    that **ends a move** on a tile within the bow's range of the user
    (distance 2 for a standard bow) is shot **once**, after its move and
    before its action. One strike, the normal formulas, no counter, no
    extra strikes. If it falls it doesn't act. Then the stance ends.
  - An enemy that doesn't move (it attacks from where it stands) doesn't
    trigger it.
  - The shot gives weapon EXP as an art (base 4 / 2) and unit EXP as a
    combat. On the map the user shows its stance, as Brace does.
- **Marked Shot.** On the first hit, the target is marked until the end of
  its next phase. Every attack on it (weapon or spell) by a unit hostile to
  it gets hit +20 and crit +10, in that unit's forecast too. The mark isn't
  a stat change; the forecast shows `marks: hit +20, crit +10`.
- **Arrow Rain.** The target is an enemy at distance 2 or 3 (whatever the
  bow's own range). It and every enemy on the four tiles next to it take
  **one strike** each, target first, then the others clockwise from the
  north. No counters, no extra strikes. EXP as for Comet Lance.
- **Off Balance** is a debuff on `avoid`: `avoid −20` (not below 0) until
  the end of the target's next phase, from the first hit.
- **Deflect** (Nick: "should be stronger to also parry"). While the stance
  lasts, the first enemy strike on the user that **would hit** (after the
  hit roll) misses instead, and at once the user makes **one strike** back
  at that attacker, if its equipped weapon reaches it. The strike back uses
  the normal formulas, can crit, can't be countered, and doesn't use up one
  of the user's own counters. Then the combat goes on. It works once per
  stance.
- **Chain Strike.** After the combat, if the target and the user are still
  standing, each **ally of the user next to the target** whose equipped
  weapon reaches it (distance 1) makes **one strike** at it with its own
  weapon and stats, clockwise from the north. No counters, no durability,
  no EXP for those allies; the user's weapon EXP counts their damage too.
  A strike that fells the target ends the chain. Green combat allies join
  it; non-combat ones don't.
- **Vital Point** applies its five changes at once, on the first hit, each
  not below 0.
- **Strikes at several units** (Wide Swing, Comet Lance, Arrow Rain, Chain
  Strike) use the 2RN hit roll and the crit roll in strike order. Their
  forecast lists one row per unit struck (*Forecast mockups* below).

### Forecast mockups

A Duelist at sword B (the menu grows by rank; Keen Edge and Blade Flurry
are its class actives):

```
 ┌─ Steel Sword 25/25 ───────────┐
 │ Flowing Cut     −2 dur  E     │
 │ Guard Break     −4 dur  D     │
 │ Riposte         −4 dur  C     │
 │▸Disarm          −5 dur  B     │
 │ Keen Edge       −3 dur  active│
 │ Blade Flurry    −5 dur  active│
 └───────────────────────────────┘
```

A debuff that changes what the target can do, and an art that strikes
several units (the numbers only show the layout):

```
 ╔════════ Forecast ══════════╗     ╔════════ Forecast ══════════╗
 ║ Disarm (25 → 20)           ║     ║ Arrow Rain (25 → 17)       ║
 ║ Joss          Hound Capt.  ║     ║ Aske          3 targets    ║
 ║ Steel Sword   Steel Axe    ║     ║ Steel Bow     no counters  ║
 ║ Hit    84     Hit   --     ║     ║ Hound    Hit 82  11 → 4    ║
 ║ 10 dmg  → 22   no counter  ║     ║ Hound    Hit 76  11 → 9    ║
 ║ 12 dmg  → 10               ║     ║ Warden   Hit 71   6 → 18   ║
 ║ disarms: no weapon attacks ║     ╚════════════════════════════╝
 ║   until end of its phase   ║
 ╚════════════════════════════╝
```

A stance art (this combat is normal; the line says what lasts):

```
 ╔════════ Forecast ══════════╗
 ║ Riposte (25 → 21)          ║
 ║ Tamsin        Brigand      ║
 ║ Steel Sword   Iron Axe     ║
 ║ Hit    90     Hit   61     ║
 ║  9 dmg  → 11   12 dmg → 8  ║
 ║ stance: strikes first      ║
 ║   until your next phase    ║
 ╚════════════════════════════╝
```

Overwatch has no forecast (no target): its line in the `Arts` menu reads
`Overwatch  −5 dur  B  ends action`.

### Overlaps with class actives (Nick: "rare and make sense")

- **Cascade** and **Blade Flurry** (Duelist, +1 strike): Cascade makes the
  strikes a fast swordsman already has hit harder; Blade Flurry adds one.
  One per attack, so a Duelist picks.
- **Spear Wall** and **Brace / Fortify**: all stances, but one is reach and
  the others are armour.
- **Unbroken Stride** and **Vault / Swoop**: one step after an attack
  against a whole turn after a kill.
- **Wide Swing, Comet Lance, Arrow Rain** and **Flurry / Volley**: more
  strikes, but at many units instead of one.
- **Skull Splitter** and **Keen Edge / Deadly Blow**: all crit; Nick chose
  it knowing.
- **Felling Blow** and **Heavy Blow / Rampage**: all big hits, but Felling
  Blow only matters against bosses and tanks.

## Special weapons (ticket 0018)

### Nick's words

> "so I think MLGS should be the end of a very long optional questline like
> they did in Elden Ring... as for its arts, I think it can have both. Str
> against Res as a passive (never use dur, it's innate) and the wave effect
> as an active. other weps are ok I guess I mean they are pretty early game
> rewards don't need to be too special."

and for the wave's power: "whichever is higher for the wielder".

### Rules for every special weapon

- A special weapon is an item with its **own art** in its `arts` list
  (*Learning arts*, above): anyone who can wield it may use the art while
  attacking with it, at any rank.
- It may also have a **passive**: an effect built into the weapon that
  applies to every strike made with it (attacks and counters) and costs
  nothing (Nick: "never use dur, it's innate"). Only the Moonblade has one
  so far.
- **Personal weapons** (side-quest rewards, `world-structure.md`) are
  special weapons that only one character can wield (like Lyn's Mani
  Katti). Ticket 1072 designs each with its quest.
- **Selling** (Nick, 2026-10-26: "special weapons can be sold but should
  warn the player"): a special weapon sells for half its price like any
  other, but selling it asks first. Example: `Sell Moonblade for 1500?
  No shop sells it, so it can't be bought back.` with **No** / Yes, the
  cursor on No (*Claude's starting rule*: the wording and the default).
  No shop ever stocks a special weapon, so a sold one is gone.
- *Claude's starting rule:* special weapons repair at a Blacksmith or the
  Preparations shop for gold, by the usual formula on their listed price.

### The first ones (*tunable* stats)

| Weapon | Kind | Rank | Mt | Hit | Crit | Wt | Rng | Dur | Price | Where |
| ------ | ---- | ---- | -- | --- | ---- | -- | --- | --- | ----- | ----- |
| **Houndmaster's Axe** | Axe | D | 10 | 70 | 5 | 7 | 1 | 25 | 900 | Chapter 2: the Hound lieutenant's axe, given when the battle is won |
| **Fordwarden's Spear** | Spear | E | 6 | 80 | 0 | 5 | 1–2 | 20 | 800 | Chapter 2: a reward for saving houses past the minimum |
| **Moonblade** *(working name)* | Sword | C | 10 | 85 | 0 | 4 | 1 | 30 | 3000 | The end of a long optional questline (Nick; not Chapters 2–3) |

- **Houndmaster's Axe: Pack Hunt** (3 dur): this combat, might **+2 for
  each other unit of the user's side next to the target**, up to +6. The
  lieutenant uses it against the player first (he is a boss: bosses use
  arts), so the player sees that a weapon carries an art before winning
  it. 0848 makes him an axe user.
- **Fordwarden's Spear** is a throwing spear, the first spear with range 2,
  so Line Pierce goes diagonal with it (Nick's 0312 review). **Hold the
  Ford** (3 dur): stance: an enemy that steps onto a tile **next to the
  user** must **stop there** (its move ends). One that starts its move next
  to the user may move away freely. Fliers are stopped too. Like Battle
  Brothers' zone of control.
- **Moonblade** (Nick's Moonlight Greatsword):
  - **Passive, Moonlit:** every strike with it hits **Res instead of Def**
    (`mitigation = B.Res + terrain`), still powered by **Str** + might.
    No cost.
  - **Art, Moonwave** (4 dur): this attack reaches **1–2**, and its power is
    **`max(Str, Mag)` + might** against Res (Nick: "whichever is higher for
    the wielder"). A target at 2 counters only if it reaches 2.
  - Its stats and rank are placeholders until its questline is placed
    (ticket 0075); it is the late-game blade, so they are high.
- **Chapter 3** gets no special weapon from this decision: the Moonblade
  moved to its questline, and Greywater's reliquary chests are Chapter 3's
  ticket's to fill.

### Forecast of a passive and a wave

```
 ╔════════ Forecast ══════════╗
 ║ Moonwave (30 → 26)         ║
 ║ Mystic        Hound Guard  ║
 ║ Moonblade     Iron Spear   ║
 ║ Hit    88     Hit   --     ║
 ║ 13 dmg  → 13   no counter  ║
 ║ moonlit: hits Res          ║
 ╚════════════════════════════╝
```

The `moonlit: hits Res` line shows on every forecast with the Moonblade,
plain attacks included.

## Class actives after the playtest (ticket 0018)

Nick, asked whether Lance Rush and Dive should get their own flavour:

> "dive is good to add but lance rush with rider's charge sounds
> broken...? … I just dont want overlap between 2 skills when they will
> definitely be used together it will just turn a lance mount class into a
> one shot machine with no strategy"

- **Dive** (Sky Warden, 5 dur): the user attacks an enemy **up to 3 tiles
  away in a straight line** (row or column), landing on the tile **next to
  it** on that line, which must be empty and one the user can stand on;
  might **+4** (was +6). The flight there doesn't count as movement for
  Charge. Tier 4 isn't built yet; this is the rule it is built with.
- **Lance Rush stays might +5.** It never depends on how far the Rider
  moved, so it doesn't double up with the Charge passive.
- **Heavy Blow** stays: "one strike only" already sets it apart.

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

- **Arts for ranks C–S, and special weapons with their own arts:** decided
  by ticket 0018 (above); built by 0325–0329.
- **The Moonblade's questline** (where it starts, how long, which acts):
  ticket 0075.
- **Tier-3 actives** must be spell actives for 0-weapon classes: ticket 1001.
- **Reflavouring tier 1–2 actives:** decided by 0018: Dive changes, Lance
  Rush and Heavy Blow stay.
- **Number scale:** decided by ticket 0013: FE-sized, so no rescale.
