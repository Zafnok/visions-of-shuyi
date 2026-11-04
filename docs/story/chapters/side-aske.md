# Side quest: Letters Home (Aske's; beat sheet)

Source: ticket 1072 (the outline's Chapter 5 entry, `../outline.md`;
Aske's sheet, `../characters/poacher.md`; the hook Chapter 5 planted on
the quay, `ch05.md` and `assets/dialogue/ch05.dlg`: the refugee boat
with the hill people's letters and silver, taken off the point by League
privateers, "somewhere the League won't say", and Ragna's promise to
find out). Script: `assets/dialogue/side_letters.dlg`, written on a
blank page from this sheet's cards (the `story-writing` skill). The
staging below is the writer's, to be thrown away by any rewrite.

A **side quest** (`docs/design/world-structure.md`): its place is
*Letters Home*, on the coast east of the Envoy's House
(`assets/worldmaps/act1.ron`, `letters_home`); it opens once *The
Envoy's House* is won, while Aske is in the army, and closes with Act 1.
The battle is `assets/battles/side_letters_home.ron` on
`assets/maps/side_letters_home.map`, music `side_quest`. **The reward is
a personal weapon: Tor's Bow** (0069, 0018's special weapons; 0329's
`wielder`), her dead brother's hill bow, sent south by her mother, with
its own art **Longshot** (a shot at 2 to 3 tiles, hit −10, 3
durability). Only Aske can wield it.

## When it plays, and what it may not say

After Saltmere, so Aske's news-selling is known to everyone and Ragna's
word has come (that is how the place opens); maybe after the Ashfields
or Ossary. Nothing here may lean on Chapter 6: Rue never speaks to Aske
directly, and nothing of the fire line is said. Hedda is lent by then if
she lived, so she is in `@if shieldbearer` blocks, as Gil is in his.
Aske's mother is not named (her sheet names nobody at the hold but Tor).

## Cast on screen

| Id | Name | Slot |
| -- | ---- | ---- |
| `poacher` | Aske | Archer; **whose quest it is** (always in the army here) |
| `lead` | the lead (Veyne) | Exile |
| `retainer` | Hollis Marr | Guard |
| `sergeant` | Tamsin Rook | Rider |
| `keeper` | Maud | Cleric |
| `heretic` | Rue | Mage |
| `defector` | Joss Pellam | Swordsman |
| `prizefighter` | Gil Parrow | Brawler, if he joined (`@if`; a slot when 1071 makes him a unit) |
| `shieldbearer` | Hedda Ravn | Raider, if she lived (`@if`; a slot when 0851 makes her a unit). **The only other Brennish speaker**: she hears what Sten says |
| `privateer` | Jory Casker | A League privateer with a council paper: the boss, who falls |
| `boatmaster` | Sten | The hill people's boat master, a speaker: reads Aske's letters to her mother and writes her answers |

Generic enemies: the League's privateers (`privateer_*` templates):
sailors with cutlasses, axes, bows and fists, nobody's soldiers. The
hill people in the hold never speak; narration and Sten carry them.

### Voice cards (minor)

- **Jory Casker**, 38, a privateer captain of the League with a letter of
  marque. Tidy and pleased with himself; "lawful", "the council likes a
  list"; calls Aske "girl", "hill girl". Writes everything down. Won't
  say: pirate. Flat, then shrill at half strength; a `!` when he is
  losing.
- **Sten**, 60, master of the hill people's boat, who runs it both ways
  and carries the letters; a hill man who knew Aske as a child and
  reads her letters to her mother in the hold's kitchen. **Speaks
  Brennish** (narration says so once; his lines are what he says, and
  Aske says them again in Ardevali, or doesn't). Slow, blunt, warm;
  says what her mother said the way she said it. Two days in a nailed
  hold: stiff, coughing. Won't say that the girl should come home;
  says what her mother said. Periods; a `...` once, at the bow.

## Budget

The cliff top 5–10 (cap 12); battle triggers 1–2; Casker and Aske 2–4
(cap 5); the shingle 6–12 (cap 16, plus 1 for Sten, met for the first
time). **One reply choice**, on the shingle.

## What the quest must do

- **The outline:** "a refugee boat carrying her letters and silver home
  has been seized by League privateers."
- **Her sheet:** she wants to go home with enough silver that her mother
  never has to beg; her need is to see people instead of flags, and to
  decide who she fights for; quick total hatred, and pride that won't
  let her take back a word; she reads badly and Maud is teaching her to
  write. Here the letters go the other way for once: her mother, through
  Sten, says stop sending it and come home, and sends Tor's bow when she
  won't. Aske sends the silver anyway, and one line she wrote herself.
- **People, not flags:** League privateers with a lawful paper; Brennish
  hill people in a nailed hold; an Ardevali exile's people getting them
  out; and the only one who understands the old man is Ragna's Brennish
  bodyguard, if she is there.

## Scenes

### 1. The cliff top (`side_letters_intro`)

- **Where:** evening, wind off the sea, the creek below: the hold's
  boat beached with its mast down, a fire on the shingle, a sloop at the
  creek mouth with the League's pennant. Aske on her belly in the grass,
  counting. Music: `talk_calm`.
- **What happens:** Aske counts them and knows the boat by the patch on
  its sail; Ragna's word said where, not how many (Maud asked). Joss
  counts too and gets a different number and takes it back. Tamsin: the
  mare won't take that path; she'll bring her down last, don't wait.
  Hollis: nobody runs for the boat; the fire first; the lead in the
  middle, same as always. Aske, still and literal: there are people in it;
  privateers nail the hatch so the cargo doesn't walk; she has read it in
  a letter, which Maud read to her. Rue, to the others, not to her, as a
  question: does anybody think she'd burn a boat with people in it; she
  is saying she knows; she won't. Hedda (if there): there'll be a council paper on that sloop
  with wax on it, and she washed a floor for that council for three
  years. Gil (if there) has taken the cliff side of the path. Maud is
  afraid of the drop and says so in a sentence that qualifies itself.
  The lead: "Down. Quietly." Aske: she goes first; she knows the path.
- **Changes:** *stakes* (people in the hold; the hatch nailed), *plan*
  (the camp first, the boat after).

### 2. The battle (defeat Casker; the rest run for the sloop)

- **Map:** the cliff path down from the top left, the lead last, three
  privateers at its foot; the shingle with the privateers' two fires;
  rocks for a bowman; the boat on the creek's
  near bank with its gangplank, bowmen on the deck and Casker on it by
  the nailed hatch; the sloop at the creek mouth out of reach. Two more come up
  the beach from the sloop's boat on turn 3. Objective: defeat Casker
  (he falls), and the rest run (`leftovers: Flee`).
- **Triggers:** `side_letters_first_turn` (Aske: count them, so she is
  not the only one counting; else Hollis); `side_letters_boat` (a player
  unit on the deck: the hatch, nailed; she has a knife; else narration);
  `side_letters_boss_engage` (Casker, anyone: letter of marque, council
  wax; you're the unlawful ones; he'll be paid for you too);
  `side_letters_boss_engage_poacher` (2–4: hill girl; her silver's in
  his sea chest with a tag; he's tidy / her mother's name is on that tag,
  he wrote her name / he writes everything down / then write hers);
  `side_letters_boss_half` (back to the sloop; leave the chest; no, bring
  it); `side_letters_boss_fell` (the paper's in his coat; it was lawful;
  somebody tell the council); `side_letters_death_lead`; death quotes
  and retreat lines for the six, different shapes, about the living:
  Hollis (an order: the hatch), Tamsin (the mare on the cliff top),
  Aske (her letters, the bad writing, send them anyway), Maud (the long
  letter at the end of "come"), Rue (a question: the hatch; eat), Joss
  (the sail wants a patch). Gil's and Hedda's wait for their slots.

### 3. The shingle (`side_letters_victory`)

- **Where:** dark coming on; the privateers' fire; the hatch up and hill
  people coming into the air one at a time; the chest open on the
  stones. Sten last out, with a bundle in oilcloth. Music: `scene_sad`.
- **What happens:** narration says it once: he speaks in Brennish, and
  Aske says it again for the rest, or doesn't. He knows her ("your mor's
  girl; you're thinner"), and he has carried something south for her.
  Her mother's words, as she said them: stop sending it; the hold eats;
  if you won't come home you'll have your brother's bow and not die with
  a bad Ardevali one. Aske, to the others: he says the hold is well; he
  says thank you. Hedda (if there): that is not what he said; easy; say it
  or don't, but don't say he said that. Else Sten finds one of his few
  Ardevali words, "mother". Aske, hot: she
  knows what he said. **Choice:** the lead's reply (earnest: send the
  silver anyway, and a letter in her own hand / blunt: your mother's
  right / an act: ask to see the bow); each way she stands by the word,
  and each way it stings or steadies her. Then people: Maud, the long
  letter at the end of "come", shown again, and "that's four comes";
  Tamsin counting the silver out loud and stopping herself; Rue at the
  far fire feeding hill people out of the privateers' pot, not looking
  this way; Hollis: the tide, get the old man his tide. Sten: he'll read
  it to her by the kitchen fire, the bad writing and all. The boat off
  the shingle; Aske with the bow in her hand, not strung, not watching.
- **Changes:** *the army* (Tor's Bow), *information* (what her mother
  said), *relationship* (the lead's reply; Hedda heard everything).

## Continuity notes

- Gull Creek: a half day east of Saltmere along the coast; a tidal creek
  with a shingle beach under cliffs, where League privateers lie up.
- Casker is dead if the player killed him; his council paper is in his
  coat. The sloop sailed with the rest.
- The hill people's boat sailed north on the evening tide with its
  silver and letters, Aske's among them, and one line in her own hand.
- Aske has Tor's bow. Her mother has asked her to come home; she hasn't
  answered yes or no.
- Hedda (if alive) heard every word of what the mother said. Nobody else
  did but the player.
- Nothing here changes what the main chapters say happened.
