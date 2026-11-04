# Side quest: The Company Ledger (Tamsin's; beat sheet)

Source: ticket 1072 (the outline's Chapter 4 entry, `../outline.md`;
Tamsin's sheet, `../characters/sergeant.md`; the hook Chapter 4 planted
at the toll-chest, `ch04.md` and `assets/dialogue/ch04.dlg`). Script:
`assets/dialogue/side_ledger.dlg`, written on a blank page from this
sheet's cards (the `story-writing` skill). The staging below is the
writer's, to be thrown away by any rewrite. A line this sheet quotes
says what a moment gets across, never the words to use.

A **side quest** (`docs/design/world-structure.md`): an optional battle
on the Act 1 map, tied to Tamsin's story, with scenes before and after
and one unique reward. Its place is *The Company Ledger*, a road north
of Millhaven (`assets/worldmaps/act1.ron`, `company_ledger`); it opens
once *Aldwater Toll Bridge* is won, while Tamsin is in the army, and
closes for good with Act 1. The battle is `assets/battles/side_company_ledger.ron`
on `assets/maps/side_company_ledger.map`, music `side_quest`. **The
reward is gold: 4,212 crowns** (0069's "a purse of gold"), the number
her book has carried for three years, which she hands to the lead.

## When it plays, and what it may not say

The quest opens after the toll bridge and stays open to the end of the
act, so a player may fight it before Veyne Hall or after Saltmere.
Nothing in it may lean on what comes after the bridge: no "you knew",
no Dace at the gate, no sea, no Hedda. Joss talks about the Hounds'
pay-day and never about his captain. Coldwell is not said; that is her
supports' (1073). The paymaster says her number once (he has her
pay-notes); she says it once, in the strongroom, counting.

## Cast on screen

| Id | Name | Slot |
| -- | ---- | ---- |
| `sergeant` | Tamsin Rook | Rider; **whose quest it is** (always in the army here: the place opens only while she is) |
| `lead` | the lead (Veyne) | Exile |
| `retainer` | Hollis Marr | Guard |
| `poacher` | Aske | Archer |
| `keeper` | Maud | Cleric |
| `heretic` | Rue | Mage |
| `defector` | Joss Pellam | Swordsman |
| `prizefighter` | Gil Parrow | Brawler, if he joined (a speaker until 1071 makes him a unit: his lines here are in `@if` blocks, and his battle lines wait for 1071 to give him a slot) |
| `paymaster` | Ambrose Tull | The crown paymaster: the boss, who **retreats** at 0 HP (`turn-structure.md`, *A boss who retreats*) |
| `red_veteran` | Denny Rudd | A Red Company rider who rode with Tamsin; a named enemy on the barn step, who falls |

Generic enemies: Hounds in black at the gate and in the barn (the
paymaster's escort), and men of the Unpaid in faded red in the yard, on
his shilling. The validator doesn't know a side quest's scenes can count
on their character, so Tamsin's lines in the scenes before and after the
battle sit in `@if sergeant` blocks like everyone else's; in the battle
her triggers name her.

### Voice cards (minor; the skill's *Voices*)

- **Ambrose Tull**, 54, the crown paymaster of the southern counties.
  A clerk's sentences: numbers, warrants, dates; "I don't make the money,
  I count it." Precise and dry, and frightened in the body (he dabs his
  face, he stands behind the guard). He believes a warrant is a reason
  and is honest about what it says. Won't say: that he stole. Flat
  periods; one `--` where she cuts him off; `!` only when he runs.
- **Denny Rudd**, 41, Red Company, rode in Tamsin's troop; a wife, Bet,
  in Millhaven with the cough. Slow, tired, Thornmarch plain; calls her
  "Rook"; says the price of things ("a shilling a day"). Won't say that
  she was wrong to leave; says that she left. Periods, and one `...`
  looking at her.

## Budget

The `story-writing` skill's scene-length budget: the barn gate 5–10
(cap 12, plus 1 for the paymaster, met for the first time); battle
triggers 1–2; Tull and Denny with Tamsin 2–4 each (cap 5); the
strongroom 6–12 (cap 16). **One reply choice**, in the strongroom.

## What the quest must do

- **The outline:** "she finds the crown paymaster who pocketed the Red
  Company's wages, and has to decide what the money is worth."
- **Her sheet:** everything is a transaction; she jokes her way out of
  what might cost her something to feel; the grief goes in the book, and
  the book comes out when she can't say a thing; she needs to find
  something nobody can pay her for (Act 3 tears the book up). This quest
  moves her one notch: she takes exactly what the book says, not a penny
  over, gives it away, and keeps the names uncrossed.
- **What the player learns:** there was never "no funds". A warrant
  struck the Ash Peace companies from the rolls after three years, and
  the money went to the Hounds' pay. The paymaster pays Unpaid men a
  shilling a day, out of it, to guard it.

## Scenes

### 1. The barn gate (`side_ledger_intro`)

- **Where:** Kingsmead at the end of an afternoon's ride, the barn's
  shadow across the yard, crown wagons by the doors, men in black at the
  gate and men in faded red in the yard. Tamsin in the saddle, hands raw,
  nothing eaten since she fed the mare. Music: `talk_calm`; the paymaster
  brings `talk_antagonist`.
- **What happens:** Tamsin sees the red coats before the black ones and
  names Denny on the step, loud, and what he's getting for it. Joss
  knows the barn: the Hounds' pay comes out of here on the first of the
  month with six riders; he rode with it twice, and is ashamed to know
  it. Hollis reads the yard: the cheap men where the arrows land, and has she eaten. The paymaster
  comes out between two guards with a purse already counted, because a
  clerk settles accounts: he knows her, three years of pay-notes, the
  number, and he says it; she cuts him off. His case, said once in two boxes: there
  was never "no funds"; there was a warrant, the Ash Peace companies
  struck off after three years, and the pay to the Hounds; he pays the
  men in red a shilling a day out of it, more than the crown did. Maud sees Tamsin's hands on the
  reins. Aske: take it; it is silver. Tamsin's turn: there's a book full
  of names and he has the one purse; if she takes it off his hand it's
  paid, and it isn't. Rue has been looking at the barn for food. Gil (if
  there) has put himself between the lead and the gate. The lead: "We
  take the barn." Tull goes in; the bar drops; Denny hasn't moved.
- **Changes:** *information* (the warrant; the back door), *stakes*
  (she refused her number), *plan* (the barn).

### 2. The battle (defeat Tull; he retreats; the paid men run)

- **Map:** the barn at the top with its doors open onto the yard and the
  strongroom at its west end; the yard wall's gate; the stream with its
  bridge (foot can wade, horses can't); the lane along the bottom.
  Hounds hold the gate and the barn; the Unpaid hold the yard; Denny on
  the barn step, holding it and coming no further; Tull in the
  strongroom with one guard. Riders come up the Millhaven road on turn 4.
  Objective: defeat Tull; at 0 HP he retreats out the back with the
  purse, which wins, and the men he pays run (`leftovers: Flee`).
- **Triggers:** `side_ledger_first_turn` (Tamsin: the gate, the mare,
  Denny's hers; else Hollis); `side_ledger_barn` (a player unit in the
  barn: Tamsin hears him counting); `side_ledger_veteran_engage`
  (Denny, anyone: paid to stand here); `side_ledger_veteran_engage_sergeant`
  (2–4: she left in the night without her pay-notes; he's had them back
  one a season; a shilling a day and Bet's cough; "off the step");
  `side_ledger_veteran_fell` (the shilling in his left boot; Rook knows
  where Bet lives); `side_ledger_boss_engage` (Tull, anyone: he's a
  clerk, there's a warrant); `side_ledger_boss_engage_sergeant` (2–4:
  the purse is on the desk, let him out the back / did he count the
  dead / he doesn't keep the dead, the rolls do / she keeps them);
  `side_ledger_boss_half` (shouts for Rudd and the back door);
  `side_ledger_boss_retreat` (keep the room; the purse is his, he
  counted it; scared, not tidy); `side_ledger_death_lead`; death quotes
  (Classic) and retreat lines (Casual) for the six, each a different
  shape and about the living: Hollis an order, Tamsin a verb about the
  book, Aske a count, Maud a request, Rue a question, Joss a question.
  Gil's and Hedda's wait for their slots (1071, 0851).

### 3. The strongroom (`side_ledger_victory`)

- **Where:** dusk in the stone room at the barn's end: chests, a desk,
  the warrant under a weight, a lamp, the purse gone with him. Tamsin on
  her knees at a chest, counting under her breath. Music: `scene_sad`.
- **What happens:** she reaches the number and it's coin, heavier and
  lighter than she thought; there's more in the chest, the Hounds'
  month, and she leaves it ("I'm not him"). Joss looks at the Hounds'
  pay and is glad and hates it. Maud has the book in her lap and does
  the sums, because Tamsin has put a man under the wrong column again.
  Tamsin counts out four thousand two hundred and twelve, exactly, and
  holds it out to the lead: the company's, and the company's her, so
  she gets to say where it goes; the war chest; "I'm paying you."
  **Choice:** the lead's reply (earnest: it's hers, three years / wry:
  paying for what / an act: give the book back); each way she gives a
  little and the gold changes hands. Then the one true thing, flat: the
  names aren't crossed out, because he paid *her*. Aske: he ran with the
  purse he counted; clerks spend. Rue has found the larder. Hollis:
  horses; the county's awake, a day's ride between them and the barn by
  dark; the lead's shoulder.
- **Changes:** *the army* (4,212 crowns), *relationship* (she paid the
  lead, the only way she gives), *information* (the names stay owed).

## Continuity notes

- Kingsmead is a crown tithe barn a half day north of Millhaven on a
  lane, with a stream and a bridge before its yard wall.
- Tull is alive, gone out the back with the purse he counted for her,
  with a Hound escort. Denny is dead if the player killed him.
- The Hounds' pay for the month is still in the barn: the party took
  the company's number and no more.
- Tamsin's book: the lead's name under the owed column (Chapter 4), and
  now a line through nothing. She said the number once here.
- Nothing here changes what the main chapters say happened.
